use serde::{Deserialize, Serialize};

use super::proof_term::ProofTerm;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Proof {
    pub datatypes: Vec<String>,
    pub proof_term: ProofTerm,
}