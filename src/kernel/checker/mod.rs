use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

use super::{
    proof_term::ProofTerm,
    proof_tree::{ProofTree, ProofTreeConclusion},
};

pub mod check;
pub mod identifier;
pub mod identifier_context;
pub mod synthesize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TypeCheckerGoal {
    conclusion: ProofTreeConclusion,
    solution: Option<ProofTerm>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct TypeCheckerResult {
    pub proof_tree: ProofTree,
    pub goals: Vec<TypeCheckerGoal>,
}

impl TypeCheckerResult {
    pub fn is_closed(&self) -> bool {
        self.goals.len() == 0
    }

    pub fn create_with_alphq_eq_tree(&self, conclusion: ProofTreeConclusion) -> Self {
        Self {
            goals: self.goals.clone(),
            proof_tree: self.proof_tree.create_alphq_eq_tree(conclusion),
        }
    }
}

#[cfg(test)]
mod tests;
