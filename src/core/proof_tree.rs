use serde::{Deserialize, Serialize};

use tsify::Tsify;

use super::prop::Prop;

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
    ForAllIntro(String),
    ForAllElim,
    ExistsIntro,
    ExistsElim(String, String),
}

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTreeConclusion {
    Prop(Prop),
    TypeJudgement(String, String),
}

#[derive(Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ProofTree {
    pub hypotheses: Vec<ProofTree>,
    pub rule: ProofTreeRule,
    pub conclusion: ProofTreeConclusion,
}
