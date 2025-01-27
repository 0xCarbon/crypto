//! Proofs of membership and non-membership with keyed-verification, i.e. the verifier needs to know the secret key to verify the proofs.
//! These are essentially keyed-verification proofs of knowledge of weak-BB signature.

use crate::{
    error::VBAccumulatorError,
    kb_universal_accumulator::witness::{
        KBUniversalAccumulatorMembershipWitness, KBUniversalAccumulatorNonMembershipWitness,
    },
    prelude::SecretKey,
    proofs_keyed_verification::{
        DelegatedMembershipProof, MembershipProof, MembershipProofProtocol,
    },
};
use ark_ec::AffineRepr;

use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{io::Write, rand::RngCore, vec::Vec};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use zeroize::{Zeroize, ZeroizeOnDrop};

#[derive(
    Clone, PartialEq, Eq, Debug, Zeroize, ZeroizeOnDrop, CanonicalSerialize, CanonicalDeserialize,
)]
pub struct KBUniversalAccumulatorMembershipProofProtocol<G: AffineRepr>(
    pub MembershipProofProtocol<G>,
);

#[serde_as]
#[derive(
    Clone, PartialEq, Eq, Debug, CanonicalSerialize, CanonicalDeserialize, Serialize, Deserialize,
)]
pub struct KBUniversalAccumulatorMembershipProof<G: AffineRepr>(pub MembershipProof<G>);

#[serde_as]
#[derive(
    Clone, PartialEq, Eq, Debug, CanonicalSerialize, CanonicalDeserialize, Serialize, Deserialize,
)]
pub struct KBUniversalAccumulatorDelegatedMembershipProof<G: AffineRepr>(
    pub DelegatedMembershipProof<G>,
);

#[derive(
    Clone, PartialEq, Eq, Debug, Zeroize, ZeroizeOnDrop, CanonicalSerialize, CanonicalDeserialize,
)]
pub struct KBUniversalAccumulatorNonMembershipProofProtocol<G: AffineRepr>(
    pub MembershipProofProtocol<G>,
);

#[serde_as]
#[derive(
    Clone, PartialEq, Eq, Debug, CanonicalSerialize, CanonicalDeserialize, Serialize, Deserialize,
)]
pub struct KBUniversalAccumulatorNonMembershipProof<G: AffineRepr>(pub MembershipProof<G>);

#[serde_as]
#[derive(
    Clone, PartialEq, Eq, Debug, CanonicalSerialize, CanonicalDeserialize, Serialize, Deserialize,
)]
pub struct KBUniversalAccumulatorDelegatedNonMembershipProof<G: AffineRepr>(
    pub DelegatedMembershipProof<G>,
);

impl<G: AffineRepr> KBUniversalAccumulatorMembershipProofProtocol<G> {
    /// Initialize a membership proof protocol.
    pub fn init<R: RngCore>(
        rng: &mut R,
        element: G::ScalarField,
        element_blinding: Option<G::ScalarField>,
        witness: &KBUniversalAccumulatorMembershipWitness<G>,
        accumulator: G,
    ) -> Self {
        Self(MembershipProofProtocol::init(
            rng,
            element,
            element_blinding,
            &witness.0,
            accumulator,
        ))
    }

    pub fn challenge_contribution<W: Write>(
        &self,
        accumulator_value: &G,
        writer: W,
    ) -> Result<(), VBAccumulatorError> {
        self.0.challenge_contribution(accumulator_value, writer)
    }

    pub fn gen_proof(
        self,
        challenge: &G::ScalarField,
    ) -> Result<KBUniversalAccumulatorMembershipProof<G>, VBAccumulatorError> {
        Ok(KBUniversalAccumulatorMembershipProof(
            self.0.clone().gen_proof(challenge)?,
        ))
    }
}

impl<G: AffineRepr> KBUniversalAccumulatorMembershipProof<G> {
    pub fn verify(
        &self,
        accumulator: G,
        secret_key: &SecretKey<G::ScalarField>,
        challenge: &G::ScalarField,
    ) -> Result<(), VBAccumulatorError> {
        self.0.verify(accumulator, secret_key, challenge)
    }

    pub fn challenge_contribution<W: Write>(
        &self,
        accumulator_value: &G,
        writer: W,
    ) -> Result<(), VBAccumulatorError> {
        self.0.challenge_contribution(accumulator_value, writer)
    }

    pub fn verify_schnorr_proof(
        &self,
        accumulator: G,
        challenge: &G::ScalarField,
    ) -> Result<(), VBAccumulatorError> {
        self.0.verify_schnorr_proof(accumulator, challenge)
    }

    pub fn to_delegated_proof(&self) -> KBUniversalAccumulatorDelegatedMembershipProof<G> {
        KBUniversalAccumulatorDelegatedMembershipProof(self.0.to_delegated_proof())
    }

    pub fn get_schnorr_response_for_element(&self) -> &G::ScalarField {
        self.0.get_schnorr_response_for_element()
    }
}

impl<G: AffineRepr> KBUniversalAccumulatorNonMembershipProofProtocol<G> {
    /// Initialize a membership proof protocol.
    pub fn init<R: RngCore>(
        rng: &mut R,
        element: G::ScalarField,
        element_blinding: Option<G::ScalarField>,
        witness: &KBUniversalAccumulatorNonMembershipWitness<G>,
        accumulator: G,
    ) -> Self {
        Self(MembershipProofProtocol::init(
            rng,
            element,
            element_blinding,
            &witness.0,
            accumulator,
        ))
    }

    pub fn challenge_contribution<W: Write>(
        &self,
        accumulator_value: &G,
        writer: W,
    ) -> Result<(), VBAccumulatorError> {
        self.0.challenge_contribution(accumulator_value, writer)
    }

    pub fn gen_proof(
        self,
        challenge: &G::ScalarField,
    ) -> Result<KBUniversalAccumulatorNonMembershipProof<G>, VBAccumulatorError> {
        Ok(KBUniversalAccumulatorNonMembershipProof(
            self.0.clone().gen_proof(challenge)?,
        ))
    }
}

impl<G: AffineRepr> KBUniversalAccumulatorNonMembershipProof<G> {
    pub fn verify(
        &self,
        accumulator: G,
        secret_key: &SecretKey<G::ScalarField>,
        challenge: &G::ScalarField,
    ) -> Result<(), VBAccumulatorError> {
        self.0.verify(accumulator, secret_key, challenge)
    }

    pub fn challenge_contribution<W: Write>(
        &self,
        accumulator_value: &G,
        writer: W,
    ) -> Result<(), VBAccumulatorError> {
        self.0.challenge_contribution(accumulator_value, writer)
    }

    pub fn verify_schnorr_proof(
        &self,
        accumulator: G,
        challenge: &G::ScalarField,
    ) -> Result<(), VBAccumulatorError> {
        self.0.verify_schnorr_proof(accumulator, challenge)
    }

    pub fn to_delegated_proof(&self) -> KBUniversalAccumulatorDelegatedNonMembershipProof<G> {
        KBUniversalAccumulatorDelegatedNonMembershipProof(self.0.to_delegated_proof())
    }

    pub fn get_schnorr_response_for_element(&self) -> &G::ScalarField {
        self.0.get_schnorr_response_for_element()
    }
}

impl<G: AffineRepr> KBUniversalAccumulatorDelegatedMembershipProof<G> {
    pub fn verify(&self, secret_key: &SecretKey<G::ScalarField>) -> Result<(), VBAccumulatorError> {
        self.0.verify(secret_key)
    }
}

impl<G: AffineRepr> KBUniversalAccumulatorDelegatedNonMembershipProof<G> {
    pub fn verify(&self, secret_key: &SecretKey<G::ScalarField>) -> Result<(), VBAccumulatorError> {
        self.0.verify(secret_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        kb_universal_accumulator::accumulator::KBUniversalAccumulator,
        persistence::test::InMemoryState,
        setup_keyed_verification::{PublicKey, SetupParams},
    };
    use ark_bls12_381::{Bls12_381, Fr, G1Affine};
    use ark_std::{
        rand::{rngs::StdRng, SeedableRng},
        UniformRand,
    };
    use blake2::Blake2b512;
    use schnorr_pok::compute_random_oracle_challenge;
    use std::time::{Duration, Instant};

    pub fn setup_uni_accum(
        rng: &mut StdRng,
        max: u64,
    ) -> (
        SetupParams<G1Affine>,
        SecretKey<Fr>,
        PublicKey<G1Affine>,
        KBUniversalAccumulator<Bls12_381>,
        Vec<Fr>,
        InMemoryState<Fr>,
        InMemoryState<Fr>,
    ) {
        let params = SetupParams::<G1Affine>::new::<Blake2b512>(b"test");
        let seed = [0, 1, 2, 10, 11];
        let secret_key = SecretKey::generate_using_seed::<Blake2b512>(&seed);
        let public_key = PublicKey::new_from_secret_key(&secret_key, &params);

        let domain = (0..max).map(|_| Fr::rand(rng)).collect::<Vec<Fr>>();
        let mem_state = InMemoryState::new();
        let mut non_mem_state = InMemoryState::new();
        let accumulator = KBUniversalAccumulator::initialize(
            &params,
            &secret_key,
            domain.clone(),
            &mut non_mem_state,
        )
        .unwrap();
        (
            params,
            secret_key,
            public_key,
            accumulator,
            domain,
            mem_state,
            non_mem_state,
        )
    }
    #[test]
    fn membership_non_membership_proof() {
        let max = 100;
        let mut rng = StdRng::seed_from_u64(0u64);

        let (_, secret_key, _, mut accumulator, domain, mut mem_state, mut non_mem_state) =
            setup_uni_accum(&mut rng, max);

        let mut members = vec![];
        let mut non_members = vec![];
        let mut mem_witnesses = vec![];
        let mut non_mem_witnesses = vec![];
        let count = 10;

        for i in 0..count {
            let elem = domain[i];
            accumulator = accumulator
                .add(elem, &secret_key, &mut mem_state, &mut non_mem_state)
                .unwrap();
            members.push(elem);
            non_members.push(domain[count + i])
        }

        for i in 0..count {
            let w = accumulator
                .get_membership_witness(&members[i], &secret_key, &mem_state)
                .unwrap();
            mem_witnesses.push(w);

            let w = accumulator
                .get_non_membership_witness(&non_members[i], &secret_key, &mut non_mem_state)
                .unwrap();
            non_mem_witnesses.push(w);
        }

        let mut mem_proof_create_duration = Duration::default();
        let mut mem_proof_verif_duration = Duration::default();
        let mut non_mem_proof_create_duration = Duration::default();
        let mut non_mem_proof_verif_duration = Duration::default();

        for i in 0..count {
            let start = Instant::now();
            let protocol = KBUniversalAccumulatorMembershipProofProtocol::init(
                &mut rng,
                members[i].clone(),
                None,
                &mem_witnesses[i],
                accumulator.mem_value().clone(),
            );
            mem_proof_create_duration += start.elapsed();

            let mut chal_bytes_prover = vec![];
            protocol
                .challenge_contribution(accumulator.mem_value(), &mut chal_bytes_prover)
                .unwrap();
            let challenge_prover =
                compute_random_oracle_challenge::<Fr, Blake2b512>(&chal_bytes_prover);
            let start = Instant::now();
            let proof = protocol.gen_proof(&challenge_prover).unwrap();
            mem_proof_create_duration += start.elapsed();

            let mut chal_bytes_verifier = vec![];
            proof
                .challenge_contribution(accumulator.mem_value(), &mut chal_bytes_verifier)
                .unwrap();
            let challenge_verifier =
                compute_random_oracle_challenge::<Fr, Blake2b512>(&chal_bytes_verifier);

            assert_eq!(challenge_prover, challenge_verifier);

            let start = Instant::now();
            proof
                .verify(
                    accumulator.mem_value().clone(),
                    &secret_key,
                    &challenge_verifier,
                )
                .unwrap();
            mem_proof_verif_duration += start.elapsed();

            proof
                .verify_schnorr_proof(accumulator.mem_value().clone(), &challenge_verifier)
                .unwrap();
            let delegated_proof = proof.to_delegated_proof();
            delegated_proof.verify(&secret_key).unwrap();

            let start = Instant::now();
            let protocol = KBUniversalAccumulatorNonMembershipProofProtocol::init(
                &mut rng,
                non_members[i].clone(),
                None,
                &non_mem_witnesses[i],
                accumulator.non_mem_value().clone(),
            );
            non_mem_proof_create_duration += start.elapsed();

            let mut chal_bytes_prover = vec![];
            protocol
                .challenge_contribution(accumulator.non_mem_value(), &mut chal_bytes_prover)
                .unwrap();
            let challenge_prover =
                compute_random_oracle_challenge::<Fr, Blake2b512>(&chal_bytes_prover);
            let start = Instant::now();
            let proof = protocol.gen_proof(&challenge_prover).unwrap();
            non_mem_proof_create_duration += start.elapsed();

            let mut chal_bytes_verifier = vec![];
            proof
                .challenge_contribution(accumulator.non_mem_value(), &mut chal_bytes_verifier)
                .unwrap();
            let challenge_verifier =
                compute_random_oracle_challenge::<Fr, Blake2b512>(&chal_bytes_verifier);

            assert_eq!(challenge_prover, challenge_verifier);

            let start = Instant::now();
            proof
                .verify(
                    accumulator.non_mem_value().clone(),
                    &secret_key,
                    &challenge_verifier,
                )
                .unwrap();
            non_mem_proof_verif_duration += start.elapsed();

            proof
                .verify_schnorr_proof(accumulator.non_mem_value().clone(), &challenge_verifier)
                .unwrap();
            let delegated_proof = proof.to_delegated_proof();
            delegated_proof.verify(&secret_key).unwrap();
        }

        println!(
            "Time to create {} membership proofs is {:?}",
            count, mem_proof_create_duration
        );
        println!(
            "Time to verify {} membership proofs is {:?}",
            count, mem_proof_verif_duration
        );
        println!(
            "Time to create {} non-membership proofs is {:?}",
            count, non_mem_proof_create_duration
        );
        println!(
            "Time to verify {} non-membership proofs is {:?}",
            count, non_mem_proof_verif_duration
        );
    }
}
