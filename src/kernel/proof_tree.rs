use serde::{Deserialize, Serialize};

use tsify_next::Tsify;

use super::prop::Prop;

#[derive(PartialEq, Eq, Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTreeRule {
    AndIntro,
    AndElimFst,
    AndElimSnd,
    TrueIntro,
    ImplIntro(String),
    ImplElim,
    Ident(String),
    OrIntroFst,
    OrIntroSnd,
    OrElim(String, String),
    FalsumElim,
    ForAllIntro(String),
    ForAllElim,
    ExistsIntro,
    ExistsElim(String, String),
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTreeConclusion {
    PropIsTrue(Prop),
    TypeJudgement(String, String),
}

#[derive(PartialEq, Eq, Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ProofTree {
    pub premisses: Vec<ProofTree>,
    pub rule: ProofTreeRule,
    pub conclusion: ProofTreeConclusion,
}
