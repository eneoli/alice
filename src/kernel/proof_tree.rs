use serde::{Deserialize, Serialize};

use tsify_next::Tsify;

use crate::kernel::proof_term::{Application, Function, Type, TypeAscription};

use super::{
    checker::identifier::Identifier,
    proof::Proof,
    proof_term::{
        Abort, Case, Ident, LetIn, OrLeft, OrRight, Pair, ProjectFst, ProjectSnd, ProofTerm,
    },
    prop::Prop,
};

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Tsify, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTreeRule {
    AndIntro,
    AndElimFst,
    AndElimSnd,
    TrueIntro,
    ImplIntro(Identifier),
    ImplElim,
    Ident(Identifier),
    OrIntroFst,
    OrIntroSnd,
    OrElim(Identifier, Identifier),
    FalsumElim,
    ForAllIntro(Identifier),
    ForAllElim,
    ExistsIntro,
    ExistsElim(Identifier, Identifier),
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

    pub fn as_proof(&self) -> Proof {
        ProofTreeExporter::export_as_proof(self)
    }
}

#[derive(Clone, PartialEq, Eq)]
enum ReasoningMode {
    Check,
    Synthesize,
}

struct ProofTreeExporter {
    atoms: Vec<(String, usize)>,
    datatypes: Vec<String>,
}

impl ProofTreeExporter {
    pub fn new() -> Self {
        Self {
            atoms: vec![],
            datatypes: vec![],
        }
    }

    pub fn export_as_proof(proof_tree: &ProofTree) -> Proof {
        let mut exporter = Self::new();
        let proof_term = exporter.do_export_as_proof_term(proof_tree, &ReasoningMode::Check);

        Proof {
            atoms: exporter.atoms,
            datatypes: exporter.datatypes,
            processing_state: super::proof::ProofProcessingState::TypesResolved,
            proof_term,
        }
    }

    fn wrap_into_type_ascription(
        &mut self,
        mut proof_term: ProofTerm,
        conclusion: &ProofTreeConclusion,
        reasoning_mode: &ReasoningMode,
        expected_reasoning_mode: &ReasoningMode,
    ) -> ProofTerm {
        if *reasoning_mode == ReasoningMode::Synthesize
            && *expected_reasoning_mode == ReasoningMode::Check
        {
            let ProofTreeConclusion::PropIsTrue(prop) = conclusion else {
                panic!("Expected proposition.");
            };

            proof_term = ProofTerm::TypeAscription(TypeAscription {
                ascription: Type::Prop(prop.clone()),
                proof_term: proof_term.boxed(),
                span: None,
            });

            self.atoms.append(&mut prop.get_atoms());
        }

        proof_term
    }

    fn do_export_as_proof_term(
        &mut self,
        proof_tree: &ProofTree,
        reasoning_mode: &ReasoningMode,
    ) -> ProofTerm {
        let ProofTree {
            premisses,
            rule,
            conclusion,
        } = proof_tree;

        match rule {
            ProofTreeRule::TrueIntro => ProofTerm::Unit(None),
            ProofTreeRule::Ident(ident) => Ident::create(ident.name().clone()),
            ProofTreeRule::AlphaEquivalent => {
                self.do_export_as_proof_term(&premisses[0], reasoning_mode)
            }
            ProofTreeRule::Sorry => {
                let proof_term = ProofTerm::Sorry(None);

                self.wrap_into_type_ascription(
                    proof_term,
                    conclusion,
                    reasoning_mode,
                    &ReasoningMode::Check,
                )
            }
            ProofTreeRule::AndIntro => {
                let [ref fst, ref snd] = premisses[..] else {
                    panic!("Not enough premisses.");
                };

                let fst_reasoning_mode = Self::expected_premisse_mode(rule, &reasoning_mode, 0);
                let snd_reasoning_mode = Self::expected_premisse_mode(rule, &reasoning_mode, 1);

                let fst_proof_term = self.do_export_as_proof_term(fst, &fst_reasoning_mode);
                let snd_proof_term = self.do_export_as_proof_term(snd, &snd_reasoning_mode);

                Pair::create(fst_proof_term.boxed(), snd_proof_term.boxed(), None)
            }
            ProofTreeRule::AndElimFst | ProofTreeRule::AndElimSnd => {
                let expected_reasoning_mode = Self::expected_conclusion_mode(rule);

                let [ref body] = premisses[..] else {
                    panic!("Not enough premisses.");
                };

                let body_reasoning_mode =
                    Self::expected_premisse_mode(rule, &expected_reasoning_mode, 0);

                let body_proof_term = self.do_export_as_proof_term(body, &body_reasoning_mode);

                let proof_term = match rule {
                    ProofTreeRule::AndElimFst => ProjectFst::create(body_proof_term.boxed(), None),
                    ProofTreeRule::AndElimSnd => ProjectSnd::create(body_proof_term.boxed(), None),
                    _ => unreachable!(),
                };

                self.wrap_into_type_ascription(
                    proof_term,
                    conclusion,
                    reasoning_mode,
                    &expected_reasoning_mode,
                )
            }
            ProofTreeRule::ImplIntro(param_ident) => {
                let [ref body] = premisses[..] else {
                    panic!("No premisse given.");
                };

                let body_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 0);
                let body_proof_term = self.do_export_as_proof_term(body, &body_reasoning_mode);

                // check if type annotation needed
                let mut annotation = None;
                if *reasoning_mode == ReasoningMode::Synthesize {
                    let ProofTreeConclusion::PropIsTrue(Prop::Impl(fst, _)) = conclusion else {
                        panic!("Expected conclusion to be an implication.");
                    };

                    annotation = Some(Type::Prop(*fst.clone()));

                    self.atoms.append(&mut fst.get_atoms());
                }

                Function::create(
                    param_ident.name().clone(),
                    annotation,
                    body_proof_term.boxed(),
                    None,
                )
            }
            ProofTreeRule::ImplElim => {
                let [ref fst, ref snd] = premisses[..] else {
                    panic!("Not enough premisses given.");
                };

                let old_atoms = self.atoms.clone();
                let old_datatypes = self.datatypes.clone();

                let fst_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 0);
                let snd_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 1);

                let fst_proof_term_checking =
                    self.do_export_as_proof_term(fst, &fst_reasoning_mode);
                let snd_proof_term_checking =
                    self.do_export_as_proof_term(snd, &snd_reasoning_mode);

                let proof_term_checking = Application::create(
                    fst_proof_term_checking.boxed(),
                    snd_proof_term_checking.boxed(),
                    None,
                );

                let annotation_count = fst_proof_term_checking.annotation_count()
                    + snd_proof_term_checking.annotation_count();

                if *reasoning_mode == ReasoningMode::Synthesize || annotation_count == 0 {
                    return proof_term_checking;
                }

                // use =>
                //     <= rule

                let checking_atoms = self.atoms.clone();
                let checking_datatypes = self.datatypes.clone();

                self.atoms = old_atoms;
                self.datatypes = old_datatypes;

                let fst_proof_term_synth =
                    self.do_export_as_proof_term(fst, &ReasoningMode::Synthesize);
                let snd_proof_term_synth = self.do_export_as_proof_term(snd, &ReasoningMode::Check);

                let proof_term_synth = Application::create(
                    fst_proof_term_synth.boxed(),
                    snd_proof_term_synth.boxed(),
                    None,
                );

                if proof_term_checking.annotation_count() <= proof_term_synth.annotation_count() {
                    self.atoms = checking_atoms;
                    self.datatypes = checking_datatypes;

                    proof_term_checking
                } else {
                    proof_term_synth
                }
            }
            ProofTreeRule::OrIntroFst | ProofTreeRule::OrIntroSnd => {
                let expected_reasoning_mode = Self::expected_conclusion_mode(rule);

                let [ref body] = premisses[..] else {
                    panic!("Not enough premisses given.");
                };

                let body_reasoning_mode =
                    Self::expected_premisse_mode(rule, &expected_reasoning_mode, 0);
                let body_proof_term = self.do_export_as_proof_term(body, &body_reasoning_mode);

                let proof_term = match rule {
                    ProofTreeRule::OrIntroFst => OrLeft::create(body_proof_term.boxed(), None),
                    ProofTreeRule::OrIntroSnd => OrRight::create(body_proof_term.boxed(), None),
                    _ => unreachable!(),
                };

                self.wrap_into_type_ascription(
                    proof_term,
                    conclusion,
                    reasoning_mode,
                    &expected_reasoning_mode,
                )
            }
            ProofTreeRule::OrElim(fst_ident, snd_ident) => {
                let [ref head, ref fst, ref snd] = premisses[..] else {
                    panic!("Not enough premisses.");
                };

                let head_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 0);
                let fst_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 1);
                let snd_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 2);

                let head_proof_term = self.do_export_as_proof_term(head, &head_reasoning_mode);
                let fst_proof_term = self.do_export_as_proof_term(fst, &fst_reasoning_mode);
                let snd_proof_term = self.do_export_as_proof_term(snd, &snd_reasoning_mode);

                Case::create(
                    head_proof_term.boxed(),
                    fst_ident.name().clone(),
                    fst_proof_term.boxed(),
                    snd_ident.name().clone(),
                    snd_proof_term.boxed(),
                    None,
                )
            }
            ProofTreeRule::FalsumElim => {
                let expected_reasoning_mode = Self::expected_conclusion_mode(rule);

                let [ref body] = premisses[..] else {
                    panic!("Not enough premisses.");
                };

                let body_reasoning_mode =
                    Self::expected_premisse_mode(rule, &expected_reasoning_mode, 0);

                let body_proof_term = self.do_export_as_proof_term(body, &body_reasoning_mode);

                let proof_term = Abort::create(body_proof_term.boxed(), None);

                self.wrap_into_type_ascription(
                    proof_term,
                    conclusion,
                    reasoning_mode,
                    &expected_reasoning_mode,
                )
            }
            ProofTreeRule::ForAllIntro(param_ident) => {
                let [ref body] = premisses[..] else {
                    panic!("Not enough premisses.");
                };

                let body_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 0);
                let body_proof_term = self.do_export_as_proof_term(body, &body_reasoning_mode);

                let mut param_type = None;
                if *reasoning_mode == ReasoningMode::Synthesize {
                    let ProofTreeConclusion::PropIsTrue(Prop::ForAll {
                        object_type_ident,
                        ..
                    }) = conclusion
                    else {
                        panic!("Expected universal quantification.");
                    };

                    param_type = Some(Type::Datatype(object_type_ident.clone()));
                    self.datatypes.push(object_type_ident.clone());
                }

                Function::create(
                    param_ident.name().clone(),
                    param_type,
                    body_proof_term.boxed(),
                    None,
                )
            }
            ProofTreeRule::ForAllElim => {
                let expected_reasoning_mode = Self::expected_conclusion_mode(rule);

                let [ref fst, ref snd] = premisses[..] else {
                    panic!("Not enough premisses.");
                };

                let fst_reasoning_mode =
                    Self::expected_premisse_mode(rule, &expected_reasoning_mode, 0);
                let snd_reasoning_mode =
                    Self::expected_premisse_mode(rule, &expected_reasoning_mode, 1);

                let fst_proof_term = self.do_export_as_proof_term(fst, &fst_reasoning_mode);
                let snd_proof_term = self.do_export_as_proof_term(snd, &snd_reasoning_mode);

                let proof_term =
                    Application::create(fst_proof_term.boxed(), snd_proof_term.boxed(), None);

                self.wrap_into_type_ascription(
                    proof_term,
                    conclusion,
                    reasoning_mode,
                    &expected_reasoning_mode,
                )
            }
            ProofTreeRule::ExistsIntro => {
                let expected_reasoning_mode = Self::expected_conclusion_mode(rule);

                let [ref fst, ref snd] = premisses[..] else {
                    panic!("Not enough premisses");
                };

                let fst_reasoning_mode =
                    Self::expected_premisse_mode(rule, &expected_reasoning_mode, 0);
                let snd_reasoning_mode =
                    Self::expected_premisse_mode(rule, &expected_reasoning_mode, 1);

                let fst_proof_term = self.do_export_as_proof_term(fst, &fst_reasoning_mode);
                let snd_proof_term = self.do_export_as_proof_term(snd, &snd_reasoning_mode);

                let proof_term = Pair::create(fst_proof_term.boxed(), snd_proof_term.boxed(), None);

                self.wrap_into_type_ascription(
                    proof_term,
                    conclusion,
                    reasoning_mode,
                    &expected_reasoning_mode,
                )
            }
            ProofTreeRule::ExistsElim(fst_ident, snd_ident) => {
                let [ref fst, ref snd] = premisses[..] else {
                    panic!("Not enough premisses");
                };

                let fst_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 0);
                let snd_reasoning_mode = Self::expected_premisse_mode(rule, reasoning_mode, 1);

                let fst_proof_term = self.do_export_as_proof_term(fst, &fst_reasoning_mode);
                let snd_proof_term = self.do_export_as_proof_term(snd, &snd_reasoning_mode);

                ProofTerm::LetIn(LetIn {
                    head: fst_proof_term.boxed(),
                    fst_ident: fst_ident.name().clone(),
                    snd_ident: snd_ident.name().clone(),
                    body: snd_proof_term.boxed(),
                    span: None,
                })
            }
        }
    }

    fn expected_conclusion_mode(rule: &ProofTreeRule) -> ReasoningMode {
        match rule {
            ProofTreeRule::AndElimFst => ReasoningMode::Synthesize,
            ProofTreeRule::AndElimSnd => ReasoningMode::Synthesize,
            ProofTreeRule::TrueIntro => ReasoningMode::Synthesize,
            ProofTreeRule::Ident(_) => ReasoningMode::Synthesize,
            ProofTreeRule::OrIntroFst => ReasoningMode::Check,
            ProofTreeRule::OrIntroSnd => ReasoningMode::Check,
            ProofTreeRule::FalsumElim => ReasoningMode::Check,
            ProofTreeRule::Sorry => ReasoningMode::Check,
            ProofTreeRule::AlphaEquivalent => ReasoningMode::Check,
            ProofTreeRule::ForAllElim => ReasoningMode::Synthesize,
            ProofTreeRule::ExistsIntro => ReasoningMode::Check,
            _ => panic!("Both modes are possible."),
        }
    }

    fn expected_premisse_mode(
        rule: &ProofTreeRule,
        conclusion_mode: &ReasoningMode,
        premisse_pos: usize,
    ) -> ReasoningMode {
        match (premisse_pos, conclusion_mode, rule) {
            (0, _, ProofTreeRule::AndIntro) => conclusion_mode.clone(),
            (1, _, ProofTreeRule::AndIntro) => conclusion_mode.clone(),
            (0, ReasoningMode::Synthesize, ProofTreeRule::AndElimFst) => ReasoningMode::Synthesize,
            (0, ReasoningMode::Synthesize, ProofTreeRule::AndElimSnd) => ReasoningMode::Synthesize,
            (0, _, ProofTreeRule::ImplIntro(_)) => conclusion_mode.clone(),
            (0, ReasoningMode::Check, ProofTreeRule::ImplElim) => ReasoningMode::Check,
            (1, ReasoningMode::Check, ProofTreeRule::ImplElim) => ReasoningMode::Synthesize,
            (0, ReasoningMode::Synthesize, ProofTreeRule::ImplElim) => ReasoningMode::Synthesize,
            (1, ReasoningMode::Synthesize, ProofTreeRule::ImplElim) => ReasoningMode::Check,
            (0, ReasoningMode::Check, ProofTreeRule::OrIntroFst) => ReasoningMode::Check,
            (0, ReasoningMode::Check, ProofTreeRule::OrIntroSnd) => ReasoningMode::Check,
            (0, _, ProofTreeRule::OrElim(_, _)) => ReasoningMode::Synthesize,
            (1, _, ProofTreeRule::OrElim(_, _)) => conclusion_mode.clone(),
            (2, _, ProofTreeRule::OrElim(_, _)) => conclusion_mode.clone(),
            (0, ReasoningMode::Check, ProofTreeRule::FalsumElim) => ReasoningMode::Check,
            (0, _, ProofTreeRule::ForAllIntro(_)) => conclusion_mode.clone(),
            (0, ReasoningMode::Synthesize, ProofTreeRule::ForAllElim) => ReasoningMode::Synthesize,
            (1, ReasoningMode::Synthesize, ProofTreeRule::ForAllElim) => ReasoningMode::Check,
            (0, ReasoningMode::Check, ProofTreeRule::ExistsIntro) => ReasoningMode::Check,
            (1, ReasoningMode::Check, ProofTreeRule::ExistsIntro) => ReasoningMode::Check,
            (0, _, ProofTreeRule::ExistsElim(_, _)) => ReasoningMode::Synthesize,
            (1, _, ProofTreeRule::ExistsElim(_, _)) => conclusion_mode.clone(),
            (0, _, ProofTreeRule::AlphaEquivalent) => ReasoningMode::Synthesize,
            _ => panic!("Rule does not have that many premisses."),
        }
    }
}
