use super::proof_term::ProofTerm;

#[derive(Debug)]
pub struct Proof {
    pub datatypes: Vec<String>,
    pub proof_term: ProofTerm,
}