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

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CheckError {
    #[error("An error happened while synthesizing")]
    SynthesizeError(#[from] SynthesizeError),

    #[error("Checkable type cannot include parameters")]
    TypeHasFreeParameters(Type),

    #[error("Cannot check for datatypes")]
    CannotCheckForDatatypes,

    #[error("Proof Term does not match wich expected type")]
    IncompatibleProofTerm {
        expected_type: Type,
        proof_term: ProofTerm,
    },

    #[error("Cannot check type without further type annotations")]
    TypeAnnotationsNeeded(ProofTerm),

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
    expected_type: &Type,
    ctx: IdentifierContext,
) -> Result<ProofTree, CheckError> {
    if expected_type.has_free_parameters() {
        return Err(CheckError::TypeHasFreeParameters(expected_type.clone()));
    }

    if expected_type.is_datatype() {
        return Err(CheckError::CannotCheckForDatatypes);
    }

    check_comapre_free_parameters_structurally(proof_term, expected_type, ctx)
}

pub(super) fn check_comapre_free_parameters_structurally(
    proof_term: &ProofTerm,
    expected_type: &Type,
    ctx: IdentifierContext,
) -> Result<ProofTree, CheckError> {
    let mut visitor = CheckVisitor::new(expected_type.clone(), ctx);
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
        let (_type, proof_tree) = synthesize(&ProofTerm::Ident(ident.clone()), self.ctx.clone())?;

        if !Type::alpha_eq_compare_free_occurences_by_structure(&_type, &self.expected_type) {
            return Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: _type,
            });
        }

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
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::Pair(fst_term.boxed(), snd_term.boxed()),
                })
            }
        };

        let fst_proof_tree = check_comapre_free_parameters_structurally(fst_term, &fst, self.ctx.clone())?;
        let snd_proof_tree = check_comapre_free_parameters_structurally(snd_term, &snd, self.ctx.clone())?;

        let rule = if fst.is_datatype() {
            ProofTreeRule::ExistsIntro
        } else {
            ProofTreeRule::AndIntro
        };

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            Type::Datatype(_) => unreachable!(),
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
                return Err(CheckError::UnexpectedPropKind {
                    expected: vec![PropKind::And],
                    received: body_type,
                })
            }
        };

        if Type::alpha_eq_compare_free_occurences_by_structure(&self.expected_type, &fst) {
            let conclusion = match self.expected_type {
                Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
                _ => unreachable!(),
            };

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
                return Err(CheckError::UnexpectedPropKind {
                    expected: vec![PropKind::And],
                    received: body_type,
                })
            }
        };

        if Type::alpha_eq_compare_free_occurences_by_structure(&self.expected_type, &snd) {
            let conclusion = match self.expected_type {
                Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
                _ => unreachable!(),
            };

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
            if !Type::alpha_eq_compare_free_occurences_by_structure(unboxed_param_type, &assumption)
            {
                println!("{:#?}", unboxed_param_type);
                println!("{:#?}", assumption);
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

        let mut body_ctx = self.ctx.clone();
        body_ctx.insert(param_ident.clone(), assumption.clone());
        let body_proof_tree = check_comapre_free_parameters_structurally(body, &body_goal, body_ctx)?;

        let rule = if assumption.is_datatype() {
            ProofTreeRule::ForAllIntro(param_ident.clone())
        } else {
            ProofTreeRule::ImplIntro(param_ident.clone())
        };

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => unreachable!(),
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
        // Try to synthesize function
        let function_type_synthesis = synthesize(
            &ProofTerm::Application {
                function: function.boxed(),
                applicant: applicant.boxed(),
            },
            self.ctx.clone(),
        );

        if let Ok((_type, proof_tree)) = function_type_synthesis {
            return if Type::alpha_eq_compare_free_occurences_by_structure(
                &_type,
                &self.expected_type,
            ) {
                Ok(proof_tree)
            } else {
                Err(CheckError::UnexpectedType {
                    expected: self.expected_type.clone(),
                    received: _type,
                })
            };
        }

        // try to synthesize applicant
        let (applicant_type, applicant_proof_tree) = synthesize(applicant, self.ctx.clone())?;

        let required_function_type = match applicant_type {
            Type::Prop(applicant_prop) => Type::Prop(Prop::Impl(
                applicant_prop.boxed(),
                match self.expected_type {
                    Type::Prop(ref prop) => prop.boxed(),
                    Type::Datatype(_) => return Err(CheckError::CannotReturnDatatype),
                },
            )), // Function
            Type::Datatype(_) => return Err(CheckError::TypeAnnotationsNeeded(function.clone())), // Allquant
        };

        let function_proof_tree = check_comapre_free_parameters_structurally(function, &required_function_type, self.ctx.clone())?;

        let rule = match required_function_type {
            Type::Prop(Prop::Impl(_, _)) => ProofTreeRule::ImplElim,
            _ => unreachable!(),
        };

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => unreachable!(),
        };

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
            let body_proof_tree = check_comapre_free_parameters_structurally(body, &self.expected_type, body_ctx)?;

            if let Type::Prop(prop) = &self.expected_type {

                // check that quantified object does not escape it's scope
                if prop.get_free_parameters().contains(&fst_ident) {
                    return Err(CheckError::QuantifiedObjectEscapesScope);
                }

                let conclusion = match self.expected_type {
                    Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
                    _ => unreachable!(),
                };

                Ok(ProofTree {
                    premisses: vec![pair_proof_term_tree, body_proof_tree],
                    rule: ProofTreeRule::ExistsElim(fst_ident.clone(), snd_ident.clone()),
                    conclusion,
                })
            } else {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::LetIn {
                        fst_ident: fst_ident.clone(),
                        snd_ident: snd_ident.clone(),
                        pair_proof_term: pair_proof_term.boxed(),
                        body: body.boxed(),
                    },
                });
            }
        } else {
            Err(CheckError::UnexpectedProofTermKind {
                expected: vec![ProofTermKind::ExistsPair],
                received: pair_proof_term_type,
            })
        }
    }

    fn visit_or_left(&mut self, body: &ProofTerm) -> Result<ProofTree, CheckError> {
        let expected = match self.expected_type {
            Type::Prop(Prop::Or(ref fst, _)) => Type::Prop(*fst.clone()),
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::OrLeft(body.boxed()),
                })
            }
        };

        let body_proof_tree = check_comapre_free_parameters_structurally(body, &expected, self.ctx.clone())?;

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => unreachable!(),
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
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::OrLeft(body.boxed()),
                })
            }
        };

        let body_proof_tree = check_comapre_free_parameters_structurally(body, &expected, self.ctx.clone())?;

        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => unreachable!(),
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
            _ => {
                return Err(CheckError::UnexpectedPropKind {
                    expected: vec![PropKind::Or],
                    received: proof_term_type,
                })
            }
        };

        let mut fst_ctx = self.ctx.clone();
        fst_ctx.insert(left_ident.clone(), Type::Prop(*fst.clone()));
        let fst_proof_tree = check_comapre_free_parameters_structurally(left_term, &self.expected_type, fst_ctx)?;

        let mut snd_ctx = self.ctx.clone();
        snd_ctx.insert(right_ident.clone(), Type::Prop(*snd.clone()));
        let snd_proof_tree = check_comapre_free_parameters_structurally(right_term, &self.expected_type, snd_ctx)?;

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
        let body_proof_tree = check_comapre_free_parameters_structurally(body, &Type::Prop(Prop::False), self.ctx.clone())?;

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
