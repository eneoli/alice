use serde::{Deserialize, Serialize};

use tsify_next::Tsify;

use super::{checker::identifier::Identifier, prop::Prop};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Tsify, Debug)]
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
    Sorry,
    AlphaEquivalent,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTreeConclusion {
    PropIsTrue(Prop),
    TypeJudgement(Identifier, String),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub struct ProofTree {
    pub premisses: Vec<ProofTree>,
    pub rule: ProofTreeRule,
    pub conclusion: ProofTreeConclusion,
}

impl ProofTree {
    pub fn create_alphq_eq_tree(&self, conclusion: ProofTreeConclusion) -> ProofTree {
        let own_conclusion = &self.conclusion;

        match (own_conclusion, &conclusion) {
            (
                ProofTreeConclusion::PropIsTrue(ref own_prop),
                ProofTreeConclusion::PropIsTrue(ref prop),
            ) => {
                if !Prop::alpha_eq(own_prop, prop) {
                    panic!("Conclusions not alpha equivalent.");
                }
            }
            (
                ProofTreeConclusion::TypeJudgement(ref own_ident, ref own_datatype),
                ProofTreeConclusion::TypeJudgement(ref ident, ref datatype),
            ) => {
                if !Identifier::eq(own_ident, ident) || !String::eq(own_datatype, datatype) {
                    panic!("Conclusions not alpha equivalent.");
                }
            }
            _ => panic!("Conclusions not alpha equivalent."),
        }

        ProofTree {
            premisses: vec![self.to_owned()],
            rule: ProofTreeRule::AlphaEquivalent,
            conclusion,
        }
    }
}
