use super::proof_term::ProofTerm;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Proof {
    pub datatypes: Vec<String>,
    pub proof_term: ProofTerm,
}