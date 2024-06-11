use serde::{Deserialize, Serialize};

use super::{
    check::typify,
    proof_term::{ProofTerm, Type},
    prop::Prop,
};
use tsify::Tsify;

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTreeRule {
    AndIntro,
    AndElimFst,
    AndElimSnd,
    TrueIntro,
    ImplIntro(String),
    ImplElim,
    Ident(Option<String>),
    OrIntroFst,
    OrIntroSnd,
    OrElim(String, String),
    FalsumElim,
}

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ProofTree {
    pub hypotheses: Vec<ProofTree>,
    pub rule: ProofTreeRule,
    pub conclusion_type: Prop,
}
