use identifier_context::IdentifierContext;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tsify_next::Tsify;

use super::{
    proof_term::{ProofTerm, ProofTermKind, ProofTermVisitor, Type},
    proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
    prop::Prop,
};

#[cfg(test)]
mod tests;
pub mod identifier_context;

// Checkable terms:     M, N ::= (M, N) | (fn u => M) | inl M | inr N | (case R of inl u => M, inr v => N) | abort R | () | R // We can check for a GIVEN Prop A if the term has this type
// Synthesizing terms:  R    ::= fst R | snd R | u | R M    // We either can infer exactly one Prop A (not given before) that the term has as type or there is no such A.
// Questions:
// 1. Why is () not a synthesizing term? It clearly always has type True
// 2. (R, R) would also be a synthesizing term. Why is it not in the list?

// TODO: Schon erledigt? checken das der Typ einer quantifizierten Variable auch ein Datentyp und kein Prop ist.
// TODO: Schon erledigt? checken das paramitriserte Atomns A(...) nur identifier haben die auch eingeführt wurden. (besonders (aber nicht nur) bei Exists)

#[derive(Clone, PartialEq, Eq, Tsify, Serialize, Deserialize, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum TypeError {
    UnexpectedType {
        expected: Type,
        received: Type,
    },
    UnexpectedKind {
        expected: ProofTermKind,
        received: Type,
    },
    ExpectedProp {
        received: Type,
    },
    UnknownIdent {
        ident: String,
    },
    QuantifiedObjectEscapesScope,
    CaseArmsDifferent {
        fst_type: Type,
        snd_type: Type,
    },
}

#[derive(Debug, Error)]
pub enum CheckError {
    #[error("An error happened while synthesizing")]
    SynthesizeError(#[from] SynthesizeError),

    #[error("Pair has unexpected components")]
    UnexpectedPairComponents,

    #[error("Expected Proposition")]
    ExpectedProp,

    #[error("Expected different type kind")]
    UnexpectedKind {
        expected: Vec<ProofTermKind>,
        received: Type,
    },

    #[error("Expected different type")]
    UnexpectedType { expected: Type, received: Type },

    #[error("Quantified object would escape it's scope")]
    QuantifiedObjectEscapesScope,
}

pub fn check(
    proof_term: &ProofTerm,
    expected_type: &Type,
    ctx: IdentifierContext,
) -> Result<ProofTree, CheckError> {
    let mut visitor = CheckVisitor::new(expected_type.clone(), ctx);
    proof_term.visit(&mut visitor)
}

#[derive(Debug, Error)]
pub enum SynthesizeError {
    #[error("Checking type failed")]
    CheckError(#[from] Box<CheckError>),

    #[error("Unknown identifier")]
    UnknownIdentifier(String),

    #[error("The given proof term is not synthesizing")]
    NotSynthesizing(ProofTermKind),

    #[error("Expected different type kind")]
    UnexpectedKind {
        expected: ProofTermKind,
        received: Type,
    },

    #[error("Expected Proposition")]
    ExpectedProp,
}

pub fn synthesize(
    proof_term: &ProofTerm,
    ctx: IdentifierContext,
) -> Result<(Type, ProofTree), SynthesizeError> {
    let mut visitor = SynthesizeVisitor::new(ctx);

    proof_term.visit(&mut visitor)
}

struct CheckVisitor {
    expected_type: Type,
    ctx: IdentifierContext,
}

impl CheckVisitor {
    pub fn new(expected_type: Type, ctx: IdentifierContext) -> Self {
        Self { expected_type, ctx }
    }
}

impl ProofTermVisitor<Result<ProofTree, CheckError>> for CheckVisitor {
    fn visit_ident(&mut self, ident: &String) -> Result<ProofTree, CheckError> {
        let (_, proof_tree) = synthesize(&ProofTerm::Ident(ident.clone()), self.ctx.clone())?;

        Ok(proof_tree)
    }

    fn visit_pair(
        &mut self,
        fst_term: &ProofTerm,
        snd_term: &ProofTerm,
    ) -> Result<ProofTree, CheckError> {
        let (fst, snd) = match self.expected_type {
            Type::Prop(Prop::And(ref fst, ref snd)) => {
                (Type::Prop(*fst.clone()), Type::Prop(*snd.clone()))
            }
            Type::Prop(Prop::Exists {
                ref object_type_ident,
                ref body,
                ..
            }) => (
                Type::Datatype(object_type_ident.clone()),
                Type::Prop(*body.clone()),
            ),
            _ => return Err(CheckError::UnexpectedPairComponents),
        };

        let fst_proof_tree = check(fst_term, &fst, self.ctx.clone())?;
        let snd_proof_tree = check(snd_term, &snd, self.ctx.clone())?;

        let rule = if fst.is_datatype() {
            ProofTreeRule::ExistsIntro
        } else {
            ProofTreeRule::AndIntro
        };

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            Type::Datatype(_) => return Err(CheckError::ExpectedProp),
        };

        Ok(ProofTree {
            premisses: vec![fst_proof_tree, snd_proof_tree],
            rule,
            conclusion,
        })
    }

    fn visit_project_fst(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let (body_type, body_proof_tree) = synthesize(body, self.ctx.clone())?;

        let fst = match body_type {
            Type::Prop(Prop::And(ref fst, _)) => Type::Prop(*fst.clone()),
            _ => {
                return Err(CheckError::UnexpectedKind {
                    expected: vec![ProofTermKind::Pair],
                    received: body_type,
                })
            }
        };

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        if self.expected_type == fst {
            Ok(ProofTree {
                premisses: vec![body_proof_tree],
                rule: ProofTreeRule::AndElimFst,
                conclusion,
            })
        } else {
            Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: fst,
            })
        }
    }

    fn visit_project_snd(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let (body_type, body_proof_tree) = synthesize(body, self.ctx.clone())?;

        let snd = match body_type {
            Type::Prop(Prop::And(_, snd)) => Type::Prop(*snd.clone()),
            _ => {
                return Err(CheckError::UnexpectedKind {
                    expected: vec![ProofTermKind::Pair],
                    received: body_type,
                })
            }
        };

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        if self.expected_type == snd {
            Ok(ProofTree {
                premisses: vec![body_proof_tree],
                rule: ProofTreeRule::AndElimSnd,
                conclusion,
            })
        } else {
            Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: snd,
            })
        }
    }

    fn visit_function(
        &mut self,
        param_ident: &String,
        body: &ProofTerm,
    ) -> Result<ProofTree, CheckError> {
        let (assumption, body_goal) = match self.expected_type {
            Type::Prop(Prop::Impl(ref fst, ref snd)) => {
                (Type::Prop(*fst.clone()), Type::Prop(*snd.clone()))
            }
            Type::Prop(Prop::ForAll {
                ref object_type_ident,
                ref body,
                ..
            }) => (
                Type::Datatype(object_type_ident.clone()),
                Type::Prop(*body.clone()),
            ),
            _ => {
                return Err(CheckError::UnexpectedKind {
                    expected: vec![ProofTermKind::Function],
                    received: self.expected_type.clone(),
                })
            }
        };

        let mut body_ctx = self.ctx.clone();
        body_ctx.insert(param_ident.clone(), assumption.clone());
        let body_proof_tree = check(body, &body_goal, body_ctx)?;

        let rule = if assumption.is_datatype() {
            ProofTreeRule::ForAllIntro(param_ident.clone())
        } else {
            ProofTreeRule::ImplIntro(param_ident.clone())
        };

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        Ok(ProofTree {
            premisses: vec![body_proof_tree],
            rule,
            conclusion,
        })
    }

    fn visit_application(
        &mut self,
        function: &ProofTerm,
        applicant: &ProofTerm,
    ) -> Result<ProofTree, CheckError> {
        let (_type, proof_tree) = synthesize(
            &ProofTerm::Application {
                function: function.boxed(),
                applicant: applicant.boxed(),
            },
            self.ctx.clone(),
        )?;

        if _type == self.expected_type {
            Ok(proof_tree)
        } else {
            Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: _type,
            })
        }
    }

    fn visit_let_in(
        &mut self,
        fst_ident: &String,
        snd_ident: &String,
        pair_proof_term: &ProofTerm,
        body: &ProofTerm,
    ) -> Result<ProofTree, CheckError> {
        let (pair_proof_term_type, pair_proof_term_tree) =
            synthesize(pair_proof_term, self.ctx.clone())?;

        if let Type::Prop(Prop::Exists {
            object_ident,
            object_type_ident,
            body: mut exists_body,
        }) = pair_proof_term_type
        {
            exists_body.substitue_free_parameter(&object_ident, &fst_ident);

            let mut body_ctx = self.ctx.clone();
            body_ctx.insert(fst_ident.clone(), Type::Datatype(object_type_ident));
            body_ctx.insert(snd_ident.clone(), Type::Prop(*exists_body));
            let body_proof_tree = check(body, &self.expected_type, body_ctx)?;

            // check that quantified object does not escape it's scope
            if let Type::Prop(prop) = &self.expected_type {
                if prop.get_free_parameters().contains(&fst_ident) {
                    return Err(CheckError::QuantifiedObjectEscapesScope);
                }

                let conclusion = match self.expected_type {
                    Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
                    _ => return Err(CheckError::ExpectedProp),
                };

                Ok(ProofTree {
                    premisses: vec![pair_proof_term_tree, body_proof_tree],
                    rule: ProofTreeRule::ExistsElim(fst_ident.clone(), snd_ident.clone()),
                    conclusion,
                })
            } else {
                Err(CheckError::ExpectedProp)
            }
        } else {
            Err(CheckError::UnexpectedKind {
                expected: vec![ProofTermKind::ExistsPair],
                received: pair_proof_term_type,
            })
        }
    }

    fn visit_or_left(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let expected = match self.expected_type {
            Type::Prop(Prop::Or(ref fst, _)) => Type::Prop(*fst.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        let body_proof_tree = check(body, &expected, self.ctx.clone())?;

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        Ok(ProofTree {
            premisses: vec![body_proof_tree],
            rule: ProofTreeRule::OrIntroFst,
            conclusion,
        })
    }

    fn visit_or_right(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let expected = match self.expected_type {
            Type::Prop(Prop::Or(_, ref snd)) => Type::Prop(*snd.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        let body_proof_tree = check(body, &expected, self.ctx.clone())?;

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        Ok(ProofTree {
            premisses: vec![body_proof_tree],
            rule: ProofTreeRule::OrIntroSnd,
            conclusion,
        })
    }

    fn visit_case(
        &mut self,
        proof_term: &ProofTerm,
        left_ident: &String,
        left_term: &ProofTerm,
        right_ident: &String,
        right_term: &ProofTerm,
    ) -> Result<ProofTree, CheckError> {
        let (proof_term_type, proof_term_tree) = synthesize(proof_term, self.ctx.clone())?;

        let (fst, snd) = match proof_term_type {
            Type::Prop(Prop::Or(fst, snd)) => (fst, snd),
            _ => return Err(CheckError::ExpectedProp),
        };

        let mut fst_ctx = self.ctx.clone();
        fst_ctx.insert(left_ident.clone(), Type::Prop(*fst.clone()));
        let fst_proof_tree = check(left_term, &self.expected_type, fst_ctx)?;

        let mut snd_ctx = self.ctx.clone();
        snd_ctx.insert(right_ident.clone(), Type::Prop(*snd.clone()));
        let snd_proof_tree = check(right_term, &self.expected_type, snd_ctx)?;

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        Ok(ProofTree {
            premisses: vec![proof_term_tree, fst_proof_tree, snd_proof_tree],
            rule: ProofTreeRule::OrElim(left_ident.clone(), right_ident.clone()),
            conclusion,
        })
    }

    fn visit_abort(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let body_proof_tree = check(body, &Type::Prop(Prop::False), self.ctx.clone())?;

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::ExpectedProp),
        };

        Ok(ProofTree {
            premisses: vec![body_proof_tree],
            rule: ProofTreeRule::FalsumElim,
            conclusion,
        })
    }

    fn visit_unit(&mut self) -> Result<ProofTree, CheckError> {
        if self.expected_type == Type::Prop(Prop::True) {
            Ok(ProofTree {
                premisses: vec![],
                rule: ProofTreeRule::TrueIntro,
                conclusion: ProofTreeConclusion::PropIsTrue(Prop::True),
            })
        } else {
            Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: Type::Prop(Prop::True),
            })
        }
    }
}

struct SynthesizeVisitor {
    ctx: IdentifierContext,
}

impl SynthesizeVisitor {
    pub fn new(ctx: IdentifierContext) -> Self {
        Self { ctx }
    }
}

impl ProofTermVisitor<Result<(Type, ProofTree), SynthesizeError>> for SynthesizeVisitor {
    fn visit_ident(&mut self, ident: &String) -> Result<(Type, ProofTree), SynthesizeError> {
        let ident_type = self.ctx.get(ident);

        if let Some(ident_type) = ident_type {
            let conclusion = match ident_type {
                Type::Prop(prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
                Type::Datatype(datatype) => {
                    ProofTreeConclusion::TypeJudgement(ident.clone(), datatype.clone())
                }
            };

            Ok((
                ident_type.clone(),
                ProofTree {
                    premisses: vec![],
                    rule: ProofTreeRule::Ident(Some(ident.clone())),
                    conclusion,
                },
            ))
        } else {
            Err(SynthesizeError::UnknownIdentifier(ident.clone()))
        }
    }

    fn visit_pair(
        &mut self,
        fst: &ProofTerm,
        snd: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::Pair))
    }

    fn visit_project_fst(
        &mut self,
        body: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let (body_type, body_proof_tree) = synthesize(body, self.ctx.clone())?;

        let fst = match body_type {
            Type::Prop(Prop::And(fst, _)) => fst,
            _ => {
                return Err(SynthesizeError::UnexpectedKind {
                    expected: ProofTermKind::Pair,
                    received: body_type,
                })
            }
        };

        Ok((
            Type::Prop(*fst.clone()),
            ProofTree {
                premisses: vec![body_proof_tree],
                rule: ProofTreeRule::AndElimFst,
                conclusion: ProofTreeConclusion::PropIsTrue(*fst.clone()),
            },
        ))
    }

    fn visit_project_snd(
        &mut self,
        body: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let (body_type, body_proof_tree) = synthesize(body, self.ctx.clone())?;

        let snd = match body_type {
            Type::Prop(Prop::And(_, snd)) => snd,
            _ => {
                return Err(SynthesizeError::UnexpectedKind {
                    expected: ProofTermKind::Pair,
                    received: body_type,
                })
            }
        };

        Ok((
            Type::Prop(*snd.clone()),
            ProofTree {
                premisses: vec![body_proof_tree],
                rule: ProofTreeRule::AndElimFst,
                conclusion: ProofTreeConclusion::PropIsTrue(*snd.clone()),
            },
        ))
    }

    fn visit_function(
        &mut self,
        _param_ident: &String,
        _body: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::Function))
    }

    fn visit_application(
        &mut self,
        function: &ProofTerm,
        applicant: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let (function_type, function_proof_tree) = synthesize(function, self.ctx.clone())?;

        let (requested_applicant_type, return_type) = match function_type {
            Type::Prop(Prop::Impl(fst, snd)) => {
                (Type::Prop(*fst.clone()), Type::Prop(*snd.clone()))
            }
            Type::Prop(Prop::ForAll {
                object_type_ident,
                body,
                ..
            }) => (
                Type::Datatype(object_type_ident.clone()),
                Type::Prop(*body.clone()),
            ),
            _ => {
                return Err(SynthesizeError::UnexpectedKind {
                    expected: ProofTermKind::Function,
                    received: function_type,
                })
            }
        };

        let applicant_proof_tree = check(applicant, &requested_applicant_type, self.ctx.clone())
            .map_err(|err| Box::new(err))?;

        let conclusion = match return_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(SynthesizeError::ExpectedProp),
        };

        Ok((
            return_type.clone(),
            ProofTree {
                premisses: vec![function_proof_tree, applicant_proof_tree],
                rule: ProofTreeRule::ImplElim,
                conclusion,
            },
        ))
    }

    fn visit_let_in(
        &mut self,
        _fst_ident: &String,
        _snd_ident: &String,
        _pair_proof_term: &ProofTerm,
        _body: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::LetIn))
    }

    fn visit_or_left(&mut self, _body: &ProofTerm) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::OrLeft))
    }

    fn visit_or_right(&mut self, _body: &ProofTerm) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::OrRight))
    }

    fn visit_case(
        &mut self,
        _proof_term: &ProofTerm,
        _left_ident: &String,
        _left_term: &ProofTerm,
        _right_ident: &String,
        _right_term: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::Case))
    }

    fn visit_abort(&mut self, _body: &ProofTerm) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::Abort))
    }

    fn visit_unit(&mut self) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::Unit))
    }
}
