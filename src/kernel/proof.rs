use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use super::proof_term::ProofTerm;

#[derive(Clone, PartialEq, Eq, Tsify, Serialize, Deserialize, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Proof {
    pub datatypes: Vec<String>,
    pub proof_term: ProofTerm,
}