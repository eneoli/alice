use core::panic;

use thiserror::Error;

use crate::kernel::{
    proof_term::{ProofTerm, ProofTermKind, ProofTermVisitor, Type},
    proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
    prop::{Prop, PropKind},
};

use super::{
    check::{CheckError, check_comapre_free_parameters_structurally}, identifier_context::IdentifierContext
};

#[derive(Debug, Error, PartialEq, Eq)]
pub enum SynthesizeError {
    #[error("Checking type failed")]
    CheckError(#[from] Box<CheckError>),

    #[error("Unknown identifier")]
    UnknownIdentifier(String),

    #[error("Synthesis yielded unexpected Proposition")]
    UnexpectedPropKind {
        expected: Vec<PropKind>,
        received: Type,
    },

    #[error("Expected Proposition but go a datatype")]
    ExpectedProp { received_datatype: String },

    #[error("Expected different type kind")]
    UnexpectedProofTermKind {
        expected: Vec<ProofTermKind>,
        received: Type,
    },

    #[error("Cannot return datatype")]
    CannotReturnDatatype,

    #[error("The given proof term is not synthesizing")]
    NotSynthesizing(ProofTermKind),

    #[error("Both case arms must return the same type")]
    CaseArmsDifferent { fst_type: Type, snd_type: Type },

    #[error("Quantified object would escape it's scope")]
    QuantifiedObjectEscapesScope,
}

pub fn synthesize(
    proof_term: &ProofTerm,
    ctx: IdentifierContext,
) -> Result<(Type, ProofTree), SynthesizeError> {
    let mut visitor = SynthesizeVisitor::new(ctx);

    proof_term.visit(&mut visitor)
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
        let (fst_type, fst_proof_tree) = synthesize(fst, self.ctx.clone())?;
        let (snd_type, snd_proof_tree) = synthesize(snd, self.ctx.clone())?;

        match (&fst_type, &snd_type) {
            (Type::Datatype(type_ident), Type::Prop(snd_prop)) => {
                if let ProofTerm::Ident(ident) = fst {
                    let _type = Prop::Exists {
                        object_ident: ident.to_string(),
                        object_type_ident: type_ident.to_string(),
                        body: snd_prop.boxed(),
                    };

                    Ok((
                        _type.clone().into(),
                        ProofTree {
                            premisses: vec![fst_proof_tree, snd_proof_tree],
                            rule: ProofTreeRule::ExistsIntro,
                            conclusion: ProofTreeConclusion::PropIsTrue(_type),
                        },
                    ))
                } else {
                    panic!("Architecture error: Expected identifier. Are you implementing datatytpe functions?")
                }
            }
            (Type::Prop(fst_prop), Type::Prop(snd_prop)) => {
                let _type = Prop::And(fst_prop.boxed(), snd_prop.boxed());

                Ok((
                    _type.clone().into(),
                    ProofTree {
                        premisses: vec![fst_proof_tree, snd_proof_tree],
                        rule: ProofTreeRule::AndIntro,
                        conclusion: ProofTreeConclusion::PropIsTrue(_type.into()),
                    },
                ))
            }
            (_, Type::Datatype(datatype)) => Err(SynthesizeError::ExpectedProp {
                received_datatype: datatype.to_string(),
            }),
        }
    }

    fn visit_project_fst(
        &mut self,
        body: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let (body_type, body_proof_tree) = synthesize(body, self.ctx.clone())?;

        let fst = match body_type {
            Type::Prop(Prop::And(fst, _)) => fst,
            _ => {
                return Err(SynthesizeError::UnexpectedPropKind {
                    expected: vec![PropKind::And],
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
                return Err(SynthesizeError::UnexpectedPropKind {
                    expected: vec![PropKind::And],
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
        param_ident: &String,
        param_type: &Option<Type>,
        body: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let param_type = match param_type {
            Some(param_type) => param_type,
            None => return Err(SynthesizeError::NotSynthesizing(ProofTermKind::Function)),
        };

        let mut body_ctx = self.ctx.clone();
        body_ctx.insert(param_ident.clone(), param_type.clone());
        let (body_type, body_proof_tree) = synthesize(body, body_ctx.clone())?;

        match (&param_type, &body_type) {
            (Type::Datatype(ident), Type::Prop(body_type)) => {
                let _type = Prop::ForAll {
                    object_ident: param_ident.clone(),
                    object_type_ident: ident.clone(),
                    body: body_type.boxed(), // TODO have I to replace parameterized props?
                };

                Ok((
                    _type.clone().into(),
                    ProofTree {
                        premisses: vec![body_proof_tree],
                        rule: ProofTreeRule::ForAllIntro(param_ident.clone()),
                        conclusion: ProofTreeConclusion::PropIsTrue(_type),
                    },
                ))
            }
            (Type::Prop(fst), Type::Prop(snd)) => {
                let _type = Prop::Impl(fst.boxed(), snd.boxed());

                Ok((
                    _type.clone().into(),
                    ProofTree {
                        premisses: vec![body_proof_tree],
                        rule: ProofTreeRule::ImplIntro(param_ident.clone()),
                        conclusion: ProofTreeConclusion::PropIsTrue(_type),
                    },
                ))
            }
            (_, Type::Datatype(_)) => return Err(SynthesizeError::CannotReturnDatatype),
        }
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
                return Err(SynthesizeError::UnexpectedProofTermKind {
                    expected: vec![ProofTermKind::Function],
                    received: function_type,
                })
            }
        };

        let applicant_proof_tree = check_comapre_free_parameters_structurally(applicant, &requested_applicant_type, self.ctx.clone())
            .map_err(|err| Box::new(err))?;

        let conclusion = match return_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(SynthesizeError::CannotReturnDatatype),
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
        fst_ident: &String,
        snd_ident: &String,
        pair_proof_term: &ProofTerm,
        body: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
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
            let (body_type, body_proof_tree) = synthesize(body, body_ctx)?;

            // check that quantified object does not escape it's scope
            if let Type::Prop(prop) = &body_type {
                if prop.get_free_parameters().contains(&fst_ident) {
                    return Err(SynthesizeError::QuantifiedObjectEscapesScope);
                }

                Ok((
                    body_type.clone(),
                    ProofTree {
                        premisses: vec![pair_proof_term_tree, body_proof_tree],
                        rule: ProofTreeRule::ExistsElim(fst_ident.clone(), snd_ident.clone()),
                        conclusion: ProofTreeConclusion::PropIsTrue(prop.clone()),
                    },
                ))
            } else {
                Err(SynthesizeError::CannotReturnDatatype)
            }
        } else {
            Err(SynthesizeError::UnexpectedProofTermKind {
                expected: vec![ProofTermKind::ExistsPair],
                received: pair_proof_term_type,
            })
        }
    }

    fn visit_or_left(&mut self, _body: &ProofTerm) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::OrLeft))
    }

    fn visit_or_right(&mut self, _body: &ProofTerm) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::OrRight))
    }

    fn visit_case(
        &mut self,
        proof_term: &ProofTerm,
        left_ident: &String,
        left_term: &ProofTerm,
        right_ident: &String,
        right_term: &ProofTerm,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let (proof_term_type, proof_term_tree) = synthesize(proof_term, self.ctx.clone())?;

        if let Type::Prop(Prop::Or(fst, snd)) = proof_term_type {
            let mut left_ctx = self.ctx.clone();
            left_ctx.insert(left_ident.clone(), Type::Prop(*fst));
            let (fst_type, fst_proof_tree) = synthesize(left_term, left_ctx)?;

            let mut right_ctx = self.ctx.clone();
            right_ctx.insert(right_ident.clone(), Type::Prop(*snd));
            let (snd_type, snd_proof_tree) = synthesize(right_term, right_ctx)?;

            if fst_type != snd_type {
                return Err(SynthesizeError::CaseArmsDifferent { fst_type, snd_type });
            }

            if let Type::Prop(fst_type) = fst_type {
                Ok((
                    snd_type,
                    ProofTree {
                        premisses: vec![proof_term_tree, fst_proof_tree, snd_proof_tree],
                        rule: ProofTreeRule::OrElim(left_ident.clone(), right_ident.clone()),
                        conclusion: ProofTreeConclusion::PropIsTrue(fst_type),
                    },
                ))
            } else {
                Err(SynthesizeError::CannotReturnDatatype)
            }
        } else {
            Err(SynthesizeError::UnexpectedPropKind {
                expected: vec![PropKind::Or],
                received: proof_term_type,
            })
        }
    }

    fn visit_abort(&mut self, _body: &ProofTerm) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::Abort))
    }

    fn visit_unit(&mut self) -> Result<(Type, ProofTree), SynthesizeError> {
        Ok((
            Type::Prop(Prop::True),
            ProofTree {
                premisses: vec![],
                rule: ProofTreeRule::TrueIntro,
                conclusion: ProofTreeConclusion::PropIsTrue(Prop::True),
            },
        ))
    }
}
