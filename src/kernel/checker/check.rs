use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::kernel::{
    proof_term::{ProofTerm, ProofTermKind, ProofTermVisitor, Type},
    proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
    prop::{Prop, PropKind},
};

use super::{
    identifier_context::IdentifierContext,
    synthesize::{synthesize, SynthesizeError},
};

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum CheckError {
    #[error("An error happened while synthesizing")]
    SynthesizeError(#[from] SynthesizeError),

    #[error("Checkable type cannot include parameters")]
    PropHasFreeParameters(Prop),

    #[error("Proof Term does not match wich expected type")]
    IncompatibleProofTerm {
        expected_type: Type,
        proof_term: ProofTerm,
    },

    #[error("Expected different type kind")]
    UnexpectedProofTermKind {
        expected: Vec<ProofTermKind>,
        received: Type,
    },

    #[error("Synthesis yielded unexpected Proposition")]
    UnexpectedPropKind {
        expected: Vec<PropKind>,
        received: Type,
    },

    #[error("Cannot return datatype")]
    CannotReturnDatatype,

    #[error("Expected different type")]
    UnexpectedType { expected: Type, received: Type },

    #[error("Quantified object would escape it's scope")]
    QuantifiedObjectEscapesScope,
}

pub fn check(
    proof_term: &ProofTerm,
    expected_prop: &Prop,
    ctx: &IdentifierContext,
) -> Result<ProofTree, CheckError> {
    if expected_prop.has_free_parameters() {
        return Err(CheckError::PropHasFreeParameters(expected_prop.clone()));
    }

    check_allowing_free_params(proof_term, &Type::Prop(expected_prop.clone()), ctx)
}

pub(super) fn check_allowing_free_params(
    proof_term: &ProofTerm,
    expected_type: &Type,
    ctx: &IdentifierContext,
) -> Result<ProofTree, CheckError> {
    let mut visitor = CheckVisitor::new(expected_type.clone(), ctx);
    proof_term.visit(&mut visitor)
}

struct CheckVisitor<'a> {
    expected_type: Type,
    ctx: &'a IdentifierContext,
}

impl<'a> CheckVisitor<'a> {
    pub fn new(expected_type: Type, ctx: &'a IdentifierContext) -> Self {
        Self { expected_type, ctx }
    }
}

impl<'a> ProofTermVisitor<Result<ProofTree, CheckError>> for CheckVisitor<'a> {
    fn visit_ident(&mut self, ident: &String) -> Result<ProofTree, CheckError> {
        let (_type, proof_tree) = synthesize(&ProofTerm::Ident(ident.clone()), self.ctx)?;

        if !Type::alpha_eq(&_type, &self.expected_type) {
            Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: _type,
            })
        } else {
            Ok(proof_tree)
        }
    }

    fn visit_pair(
        &mut self,
        fst_term: &ProofTerm,
        snd_term: &ProofTerm,
    ) -> Result<ProofTree, CheckError> {
        let (fst_type, snd_type, rule, conclusion) = match self.expected_type {
            // And
            Type::Prop(ref prop @ Prop::And(ref fst, ref snd)) => (
                Type::Prop(*fst.clone()),
                Type::Prop(*snd.clone()),
                ProofTreeRule::AndIntro,
                ProofTreeConclusion::PropIsTrue(prop.clone()),
            ),

            // Exists
            Type::Prop(
                ref prop @ Prop::Exists {
                    ref object_ident,
                    ref object_type_ident,
                    ref body,
                },
            ) => {
                // instantiate body with parameter name of function to account for \alpha-Equivalence
                let substitution = match fst_term {
                    ProofTerm::Ident(ident) => ident.clone(),
                    _ => unreachable!(),
                };

                let mut substitued_body = *body.clone();
                substitued_body.substitue_free_parameter(&object_ident, &substitution);

                (
                    Type::Datatype(object_type_ident.clone()),
                    Type::Prop(substitued_body),
                    ProofTreeRule::ExistsIntro,
                    ProofTreeConclusion::PropIsTrue(prop.clone()),
                )
            }
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::Pair(fst_term.boxed(), snd_term.boxed()),
                })
            }
        };

        // check pair components
        let fst_proof_tree = check_allowing_free_params(fst_term, &fst_type, &self.ctx)?;
        let snd_proof_tree = check_allowing_free_params(snd_term, &snd_type, &self.ctx)?;

        Ok(ProofTree {
            premisses: vec![fst_proof_tree, snd_proof_tree],
            rule,
            conclusion,
        })
    }

    fn visit_project_fst(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let (body_type, body_proof_tree) = synthesize(body, &self.ctx)?;

        let (fst, conclusion) = match body_type {
            Type::Prop(Prop::And(ref fst, _)) => (
                Type::Prop(*fst.clone()),
                ProofTreeConclusion::PropIsTrue(*fst.clone()),
            ),
            _ => {
                return Err(CheckError::UnexpectedPropKind {
                    expected: vec![PropKind::And],
                    received: body_type,
                })
            }
        };

        if Type::alpha_eq(&self.expected_type, &fst) {
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
        let (body_type, body_proof_tree) = synthesize(body, &self.ctx)?;

        let (snd, conclusion) = match body_type {
            Type::Prop(Prop::And(_, ref snd)) => (
                Type::Prop(*snd.clone()),
                ProofTreeConclusion::PropIsTrue(*snd.clone()),
            ),
            _ => {
                return Err(CheckError::UnexpectedPropKind {
                    expected: vec![PropKind::And],
                    received: body_type,
                })
            }
        };

        if Type::alpha_eq(&self.expected_type, &snd) {
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
        param_type: &Option<Type>,
        body: &ProofTerm,
    ) -> Result<ProofTree, CheckError> {
        let (expected_param_type, expected_body_prop, rule, conclusion) = match self.expected_type {
            // Function
            Type::Prop(ref prop @ Prop::Impl(ref fst, ref snd)) => (
                Type::Prop(*fst.clone()),
                *snd.clone(),
                ProofTreeRule::ImplIntro(param_ident.clone()),
                ProofTreeConclusion::PropIsTrue(prop.clone()),
            ),

            // Allquant
            Type::Prop(
                ref prop @ Prop::ForAll {
                    ref object_ident,
                    ref object_type_ident,
                    ref body,
                },
            ) => {
                // instantiate body with parameter name of function to account for \alpha-Equivalence
                let mut expected_body_prop = *body.clone();
                expected_body_prop.substitue_free_parameter(object_ident, param_ident);

                (
                    Type::Datatype(object_type_ident.clone()),
                    expected_body_prop,
                    ProofTreeRule::ForAllIntro(param_ident.clone()),
                    ProofTreeConclusion::PropIsTrue(prop.clone()),
                )
            }
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::Function {
                        param_ident: param_ident.clone(),
                        param_type: param_type.clone(),
                        body: body.boxed(),
                    },
                })
            }
        };

        // fail if type annotation is not expected type
        if let Some(unboxed_param_type) = param_type {
            if !Type::alpha_eq(unboxed_param_type, &expected_param_type) {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::Function {
                        param_ident: param_ident.clone(),
                        param_type: param_type.clone(),
                        body: body.boxed(),
                    },
                });
            }
        }

        // check body of function
        let mut body_ctx = self.ctx.clone();
        body_ctx.insert(param_ident.clone(), expected_param_type.clone());
        let body_proof_tree =
            check_allowing_free_params(body, &Type::Prop(expected_body_prop), &body_ctx)?;

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
        // Try to synthesize function
        let function_type_synthesis = synthesize(
            &ProofTerm::Application {
                function: function.boxed(),
                applicant: applicant.boxed(),
            },
            &self.ctx,
        );

        if let Ok((_type, proof_tree)) = function_type_synthesis {
            return if Type::alpha_eq(&_type, &self.expected_type) {
                Ok(proof_tree)
            } else {
                Err(CheckError::UnexpectedType {
                    expected: self.expected_type.clone(),
                    received: _type,
                })
            };
        }

        // function cannot be synthesized
        // try to synthesize applicant instead
        let (applicant_type, applicant_proof_tree) = synthesize(applicant, &self.ctx)?;

        // determine required function body type
        let (required_function_body_type, conclusion) = match self.expected_type {
            Type::Prop(ref prop) => (prop.boxed(), ProofTreeConclusion::PropIsTrue(prop.clone())),
            _ => return Err(CheckError::CannotReturnDatatype),
        };

        // determine required function type
        let (required_function_type, rule) = match applicant_type {
            // Function
            Type::Prop(applicant_prop) => (
                Type::Prop(Prop::Impl(
                    applicant_prop.boxed(),
                    required_function_body_type.boxed(),
                )),
                ProofTreeRule::ImplElim,
            ),

            // Allquant
            Type::Datatype(object_type_ident) => {
                let object_ident = match applicant {
                    ProofTerm::Ident(ident) => ident.clone(),
                    _ => unreachable!(
                        "Datatype instances are only introduced through identifiers of quantors."
                    ),
                };

                (
                    Type::Prop(Prop::ForAll {
                        object_ident,
                        object_type_ident,
                        body: required_function_body_type,
                    }),
                    ProofTreeRule::ForAllElim,
                )
            }
        };

        // check if function has determined type
        let function_proof_tree =
            check_allowing_free_params(function, &required_function_type, &self.ctx)?;

        Ok(ProofTree {
            premisses: vec![function_proof_tree, applicant_proof_tree],
            rule,
            conclusion,
        })
    }

    fn visit_let_in(
        &mut self,
        fst_ident: &String,
        snd_ident: &String,
        pair_proof_term: &ProofTerm,
        body: &ProofTerm,
    ) -> Result<ProofTree, CheckError> {
        let (pair_proof_term_type, pair_proof_term_tree) = synthesize(pair_proof_term, &self.ctx)?;

        if let Type::Prop(Prop::Exists {
            object_ident,
            object_type_ident,
            body: mut exists_body,
        }) = pair_proof_term_type
        {
            // instantiate proof with given name for witness
            exists_body.substitue_free_parameter(&object_ident, &fst_ident);

            // check body
            let mut body_ctx = self.ctx.clone();
            body_ctx.insert(fst_ident.clone(), Type::Datatype(object_type_ident));
            body_ctx.insert(snd_ident.clone(), Type::Prop(*exists_body));
            let body_proof_tree = check_allowing_free_params(body, &self.expected_type, &body_ctx)?;

            if let Type::Prop(prop) = &self.expected_type {
                // check that quantified object does not escape it's scope
                if prop.get_free_parameters().contains(&fst_ident) {
                    return Err(CheckError::QuantifiedObjectEscapesScope);
                }

                Ok(ProofTree {
                    premisses: vec![pair_proof_term_tree, body_proof_tree],
                    rule: ProofTreeRule::ExistsElim(fst_ident.clone(), snd_ident.clone()),
                    conclusion: ProofTreeConclusion::PropIsTrue(prop.clone()),
                })
            } else {
                return Err(CheckError::CannotReturnDatatype);
            }
        } else {
            Err(CheckError::UnexpectedProofTermKind {
                expected: vec![ProofTermKind::ExistsPair],
                received: pair_proof_term_type,
            })
        }
    }

    fn visit_or_left(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let (expected, conclusion) = match self.expected_type {
            Type::Prop(ref prop @ Prop::Or(ref fst, _)) => (
                Type::Prop(*fst.clone()),
                ProofTreeConclusion::PropIsTrue(prop.clone()),
            ),
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::OrLeft(body.boxed()),
                })
            }
        };

        let body_proof_tree = check_allowing_free_params(body, &expected, &self.ctx)?;

        Ok(ProofTree {
            premisses: vec![body_proof_tree],
            rule: ProofTreeRule::OrIntroFst,
            conclusion,
        })
    }

    fn visit_or_right(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let (expected_body_type, conclusion) = match self.expected_type {
            Type::Prop(ref prop @ Prop::Or(_, ref snd)) => (
                Type::Prop(*snd.clone()),
                ProofTreeConclusion::PropIsTrue(prop.clone()),
            ),
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::OrLeft(body.boxed()),
                })
            }
        };

        let body_proof_tree = check_allowing_free_params(body, &expected_body_type, &self.ctx)?;

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
        let (proof_term_type, proof_term_tree) = synthesize(proof_term, &self.ctx)?;

        let (fst, snd) = match proof_term_type {
            Type::Prop(Prop::Or(fst, snd)) => (fst, snd),
            _ => {
                return Err(CheckError::UnexpectedPropKind {
                    expected: vec![PropKind::Or],
                    received: proof_term_type,
                })
            }
        };

        // check fst case arm
        let mut fst_ctx = self.ctx.clone();
        fst_ctx.insert(left_ident.clone(), Type::Prop(*fst));
        let fst_proof_tree = check_allowing_free_params(left_term, &self.expected_type, &fst_ctx)?;

        // check snd case arm
        let mut snd_ctx = self.ctx.clone();
        snd_ctx.insert(right_ident.clone(), Type::Prop(*snd));
        let snd_proof_tree = check_allowing_free_params(right_term, &self.expected_type, &snd_ctx)?;

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::CannotReturnDatatype),
        };

        Ok(ProofTree {
            premisses: vec![proof_term_tree, fst_proof_tree, snd_proof_tree],
            rule: ProofTreeRule::OrElim(left_ident.clone(), right_ident.clone()),
            conclusion,
        })
    }

    fn visit_abort(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let body_proof_tree =
            check_allowing_free_params(body, &Type::Prop(Prop::False), &self.ctx)?;

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::CannotReturnDatatype),
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
            Err(CheckError::IncompatibleProofTerm {
                expected_type: self.expected_type.clone(),
                proof_term: ProofTerm::Unit,
            })
        }
    }
}
