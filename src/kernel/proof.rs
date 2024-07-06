use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use super::proof_term::ProofTerm;

#[derive(Clone, PartialEq, Eq, Tsify, Serialize, Deserialize, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum ProofProcessingState {
    Parsed,
    TypesResolved,
}

#[derive(Clone, PartialEq, Eq, Tsify, Serialize, Deserialize, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct Proof {
    pub processing_state: ProofProcessingState,
    pub datatypes: Vec<String>,
    pub atoms: Vec<(String, usize)>,
    pub proof_term: ProofTerm,
}