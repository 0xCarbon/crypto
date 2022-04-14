pub mod accumulator;
pub mod bbs_plus;
pub mod bound_check;
pub mod saver;
pub mod schnorr;

use crate::error::ProofSystemError;
use ark_ec::{AffineCurve, PairingEngine};
use ark_std::{boxed::Box, io::Write, pin::Pin, rc::Rc};

use crate::statement_proof::StatementProof;
use crate::sub_protocols::bound_check::BoundCheckProtocol;
use accumulator::{AccumulatorMembershipSubProtocol, AccumulatorNonMembershipSubProtocol};

/// Various sub-protocols that are executed to create a `StatementProof` which are then combined to
/// form a `Proof`
#[derive(Clone, Debug, PartialEq)]
pub enum SubProtocol<'a, E: PairingEngine, G: AffineCurve> {
    PoKBBSSignatureG1(self::bbs_plus::PoKBBSSigG1SubProtocol<'a, E>),
    AccumulatorMembership(AccumulatorMembershipSubProtocol<'a, E>),
    AccumulatorNonMembership(AccumulatorNonMembershipSubProtocol<'a, E>),
    PoKDiscreteLogs(self::schnorr::SchnorrProtocol<'a, G>),
    /// For verifiable encryption using SAVER
    Saver(self::saver::SaverProtocol<'a, E>),
    /// For range proof using LegoGroth16
    // BoundCheckProtocol(Pin<Box<BoundCheckProtocol<'a, E>>>),
    BoundCheckProtocol(BoundCheckProtocol<'a, E>),
}

pub trait ProofSubProtocol<E: PairingEngine, G: AffineCurve<ScalarField = E::Fr>> {
    fn challenge_contribution(&self, target: &mut [u8]) -> Result<(), ProofSystemError>;
    fn gen_proof_contribution(
        &mut self,
        challenge: &E::Fr,
    ) -> Result<StatementProof<E, G>, ProofSystemError>;
    fn verify_proof_contribution(
        &self,
        challenge: &E::Fr,
        proof: &StatementProof<E, G>,
    ) -> Result<(), ProofSystemError>;
}

impl<'a, E: PairingEngine, G: AffineCurve<ScalarField = E::Fr>> SubProtocol<'a, E, G> {
    pub fn challenge_contribution<W: Write>(&self, writer: W) -> Result<(), ProofSystemError> {
        match self {
            SubProtocol::PoKBBSSignatureG1(s) => s.challenge_contribution(writer),
            SubProtocol::AccumulatorMembership(s) => s.challenge_contribution(writer),
            SubProtocol::AccumulatorNonMembership(s) => s.challenge_contribution(writer),
            SubProtocol::PoKDiscreteLogs(s) => s.challenge_contribution(writer),
            SubProtocol::Saver(s) => s.challenge_contribution(writer),
            SubProtocol::BoundCheckProtocol(s) => s.challenge_contribution(writer),
        }
    }

    pub fn gen_proof_contribution(
        &mut self,
        challenge: &E::Fr,
    ) -> Result<StatementProof<E, G>, ProofSystemError> {
        match self {
            SubProtocol::PoKBBSSignatureG1(s) => s.gen_proof_contribution(challenge),
            SubProtocol::AccumulatorMembership(s) => s.gen_proof_contribution(challenge),
            SubProtocol::AccumulatorNonMembership(s) => s.gen_proof_contribution(challenge),
            SubProtocol::PoKDiscreteLogs(s) => s.gen_proof_contribution(challenge),
            SubProtocol::Saver(s) => s.gen_proof_contribution(challenge),
            SubProtocol::BoundCheckProtocol(s) => s.gen_proof_contribution(challenge),
        }
    }

    pub fn verify_proof_contribution(
        &self,
        challenge: &E::Fr,
        proof: &StatementProof<E, G>,
    ) -> Result<(), ProofSystemError> {
        match self {
            SubProtocol::PoKBBSSignatureG1(s) => s.verify_proof_contribution(challenge, proof),
            SubProtocol::AccumulatorMembership(s) => s.verify_proof_contribution(challenge, proof),
            SubProtocol::AccumulatorNonMembership(s) => {
                s.verify_proof_contribution(challenge, proof)
            }
            SubProtocol::PoKDiscreteLogs(s) => s.verify_proof_contribution(challenge, proof),
            SubProtocol::Saver(s) => s.verify_proof_contribution(challenge, proof),
            SubProtocol::BoundCheckProtocol(s) => s.verify_proof_contribution(challenge, proof),
        }
    }
}