use crate::error::DelegationError;
use ark_ec::{AffineCurve, PairingEngine, ProjectiveCurve};
use ark_ff::{One, PrimeField, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize, SerializationError};
use ark_std::collections::BTreeMap;
use ark_std::io::{Read, Write};
use ark_std::ops::{Add, Neg, Sub};
use ark_std::rand::RngCore;
use ark_std::{cfg_iter, UniformRand};
use ark_std::{vec, vec::Vec};
use dock_crypto_utils::ec::{
    batch_normalize_projective_into_affine, pairing_product_with_g2_prepared,
};
use dock_crypto_utils::msm::WindowTable;

#[cfg(feature = "parallel")]
use rayon::prelude::*;

/// SRS used for the 1-of-N proof
#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct OneOfNSrs<E: PairingEngine>(E::G1Affine);

/// Proof that 1 out of `N` public vectors of group elements when scaled (multiplied) by a scalar result in
/// a specific public group element. Based on NIZK argument in Section 7.2, Fig 6 of the
/// paper [Improved Constructions of Anonymous Credentials From SPS-EQ](https://eprint.iacr.org/2021/1680)
#[derive(Clone, Debug, CanonicalSerialize, CanonicalDeserialize)]
pub struct OneOfNProof<E: PairingEngine> {
    pub z: Vec<E::G1Affine>,
    pub d: Vec<E::G1Affine>,
    pub a: Vec<Vec<E::G2Affine>>,
}

impl<E: PairingEngine> OneOfNSrs<E> {
    /// Returns the SRS and trapdoor
    pub fn new<R: RngCore>(rng: &mut R, P1: &E::G1Affine) -> (Self, E::Fr) {
        let z = E::Fr::rand(rng);
        (Self(P1.mul(z.into_repr()).into_affine()), z)
    }
}

impl<E: PairingEngine> OneOfNProof<E> {
    /// `actual * witness = instance` but `actual` will be hidden among `decoys` and it will be proved that
    /// one of the members of this combined group is multiplied by `witness` to create `instance` without revealing
    /// which group member corresponds to `actual` and the `witness`. Note that the passed `decoys` don't
    /// contain `actual`. Expects length of all `decoys`, `instance` and `actual` to be same
    pub fn new<R: RngCore>(
        rng: &mut R,
        actual: &[E::G2Affine],
        decoys: Vec<&[E::G2Affine]>,
        instance: &[E::G2Affine],
        witness: &E::Fr,
        srs: &OneOfNSrs<E>,
        P1: &E::G1Affine,
    ) -> Result<Self, DelegationError> {
        if actual.len() != instance.len() {
            return Err(DelegationError::UnequalSizeOfSequence(
                actual.len(),
                instance.len(),
            ));
        }

        let m = actual.len();
        let n = decoys.len() + 1;
        let mut z = Vec::with_capacity(n);
        let mut a = Vec::with_capacity(n);
        let mut d = Vec::with_capacity(n);

        // The proof contains vectors `d`, `a` and `z` and each of these contain 1 item per `decoy` and
        // `actual`. To hide which item corresponds to the `actual`, the members of these 3 vectors need
        // to be sorted in certain order. Using a BtreeMap to order the members

        // Use BtreeMap to order the group of decoys + actual
        let mut all = BTreeMap::new();
        all.insert(Self::map_key(actual), (0, actual));
        for (i, pk) in decoys.into_iter().enumerate() {
            all.insert(Self::map_key(pk), (i + 1, pk));
        }

        let P1_table = WindowTable::new(4, P1.into_projective());

        let s = E::Fr::rand(rng);
        let s_repr = s.into_repr();

        // Generate `n - 1` random challenges which the paper calls `z_i`
        let random_challenges = (0..n - 1).map(|_| E::Fr::rand(rng)).collect::<Vec<_>>();
        let mut actual_at = 0;

        for (_, (i, pk)) in all.into_iter() {
            if i == 0 {
                // For `actual`
                actual_at = a.len();
                // `a_j = s * actual_j`
                a.push({
                    let a = cfg_iter!(pk).map(|p| p.mul(s_repr)).collect::<Vec<_>>();
                    batch_normalize_projective_into_affine(a)
                });
                // Temporary value for `d` and `z`, will be overwritten later
                d.push(E::G1Projective::zero());
                z.push(E::G1Projective::zero());
            } else {
                // For `decoys`
                if pk.len() != m {
                    return Err(DelegationError::UnequalSizeOfSequence(pk.len(), m));
                }
                let d_i = E::Fr::rand(rng);
                let d_i_repr = d_i.into_repr();
                let z_i = random_challenges[i - 1].into_repr();
                // `a_j = d_i * decoy_j - z_i * actual`
                a.push({
                    let a = cfg_iter!(pk)
                        .zip(cfg_iter!(instance))
                        .map(|(b, b_prime)| b.mul(d_i_repr).sub(b_prime.mul(z_i)))
                        .collect::<Vec<_>>();
                    batch_normalize_projective_into_affine(a)
                });
                z.push(P1_table.multiply(&random_challenges[i - 1]));
                d.push(P1_table.multiply(&d_i));
            }
        }

        // For `actual`, `z_i = z - (z_1 + z_2 + ....)` and `d_i = witness * z_i + s * P1`
        z[actual_at] = P1_table
            .multiply(&random_challenges.iter().sum::<E::Fr>())
            .neg()
            .add_mixed(&srs.0);
        d[actual_at] = z[actual_at]
            .mul(witness.into_repr())
            .add(P1_table.multiply(&s));
        Ok(Self {
            z: batch_normalize_projective_into_affine(z),
            d: batch_normalize_projective_into_affine(d),
            a,
        })
    }

    pub fn verify(
        &self,
        possible: Vec<&[E::G2Affine]>,
        instance: &[E::G2Affine],
        srs: &OneOfNSrs<E>,
        P1: &E::G1Affine,
    ) -> Result<(), DelegationError> {
        let n = possible.len();
        let m = instance.len();
        if self.a.len() != n {
            return Err(DelegationError::UnequalSizeOfSequence(self.a.len(), n));
        }
        if self.d.len() != n {
            return Err(DelegationError::UnequalSizeOfSequence(self.d.len(), n));
        }
        if self.z.len() != n {
            return Err(DelegationError::UnequalSizeOfSequence(self.z.len(), n));
        }

        // The sum of all `z` should match the one in SRS
        if self.z.iter().sum::<E::G1Affine>() != srs.0 {
            return Err(DelegationError::InvalidOneOfNProof);
        }

        // Use BtreeMap to order given inputs, similar to proof
        let mut all = BTreeMap::new();
        for pk in possible.into_iter() {
            all.insert(Self::map_key(pk), pk);
        }

        let prepared_instance = instance
            .iter()
            .map(|i| E::G2Prepared::from(*i))
            .collect::<Vec<_>>();

        // TODO: Optimize using randomized pairing check
        for (i, pk) in all.values().into_iter().enumerate() {
            if pk.len() != m {
                return Err(DelegationError::UnequalSizeOfSequence(pk.len(), m));
            }
            for j in 0..pk.len() {
                if !pairing_product_with_g2_prepared::<E>(
                    &[self.d[i].neg(), self.z[i], *P1],
                    &[
                        E::G2Prepared::from(pk[j]),
                        prepared_instance[j].clone(),
                        E::G2Prepared::from(self.a[i][j]),
                    ],
                )
                .is_one()
                {
                    return Err(DelegationError::InvalidOneOfNProof);
                }
            }
        }

        Ok(())
    }

    /// Create key for the BtreeMap
    fn map_key(pk: &[E::G2Affine]) -> Vec<u8> {
        let mut key = vec![];
        pk.serialize(&mut key).unwrap();
        key
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ark_bls12_381::Bls12_381;
    use ark_std::rand::{rngs::StdRng, SeedableRng};
    use std::time::Instant;

    type Fr = <Bls12_381 as PairingEngine>::Fr;
    type G2 = <Bls12_381 as PairingEngine>::G2Projective;

    #[test]
    fn one_of_n_proof() {
        let mut rng = StdRng::seed_from_u64(0u64);

        let P1 = <Bls12_381 as PairingEngine>::G1Projective::rand(&mut rng).into_affine();
        let (srs, _) = OneOfNSrs::<Bls12_381>::new(&mut rng, &P1);

        fn check(
            rng: &mut StdRng,
            size: usize,
            count_decoys: usize,
            P1: &<Bls12_381 as PairingEngine>::G1Affine,
            srs: &OneOfNSrs<Bls12_381>,
        ) {
            let actual = (0..size)
                .map(|_| G2::rand(rng).into_affine())
                .collect::<Vec<_>>();
            let decoys = (0..count_decoys)
                .map(|_| {
                    (0..size)
                        .map(|_| G2::rand(rng).into_affine())
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            let witness = Fr::rand(rng);
            let instance = actual
                .iter()
                .map(|b| b.mul(witness).into_affine())
                .collect::<Vec<_>>();

            let d = decoys.iter().map(|d| d.as_slice()).collect::<Vec<_>>();

            let start = Instant::now();
            let proof =
                OneOfNProof::new(rng, &actual, d.clone(), &instance, &witness, &srs, &P1).unwrap();
            let proving_time = start.elapsed();

            let start = Instant::now();
            for i in 0..count_decoys {
                let mut temp_d = d.clone();
                temp_d.insert(i, &actual);
                proof.verify(temp_d, &instance, &srs, &P1).unwrap();
            }
            let verifying_time = start.elapsed();

            println!("For {} decoys of size {} each, proving takes {:?} and verifying takes {:?} on average", count_decoys, size, proving_time, verifying_time / (count_decoys as u32))
        }

        for i in 10..20 {
            check(&mut rng, 5, i, &P1, &srs);
        }
    }
}