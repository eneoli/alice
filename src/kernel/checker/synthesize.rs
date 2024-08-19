use std::{ops::Range, vec};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tsify_next::Tsify;

use crate::kernel::{
    proof_term::{
        Abort, Application, Case, Function, Ident, LetIn, OrLeft, OrRight, Pair, ProjectFst,
        ProjectSnd, ProofTerm, ProofTermKind, ProofTermVisitor, Type, TypeAscription,
    },
    proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
    prop::{InstatiationError, Prop, PropKind, PropParameter},
};

use super::{
    check::{check_allowing_free_params, CheckError},
    identifier::IdentifierFactory,
    identifier_context::IdentifierContext,
    TypeCheckerResult,
};

#[derive(Debug, Error, PartialEq, Eq, Serialize, Deserialize, Clone, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum SynthesizeError {
    #[error("Checking type failed")]
    CheckError(Box<CheckError>),

    #[error("Unknown identifier")]
    UnknownIdentifier(String, Option<Range<usize>>),

    #[error("Type Annotations needed")]
    TypeAnnotationsNeeded(Option<Range<usize>>),

    #[error("Synthesis yielded unexpected Proposition")]
    UnexpectedPropKind {
        expected: Vec<PropKind>,
        received: Type,
        span: Option<Range<usize>>,
    },

    #[error("Expected Proposition but go a datatype")]
    ExpectedPropAsSecondPairComponent {
        received_datatype: String,
        span: Option<Range<usize>>,
    },

    #[error("Cannot return datatype")]
    CannotReturnDatatype(Option<Range<usize>>),

    #[error("The given proof term is not synthesizing")]
    NotSynthesizing(ProofTermKind, Option<Range<usize>>),

    #[error("Both case arms must return the same type")]
    CaseArmsDifferent {
        fst_type: Type,
        snd_type: Type,
        span: Option<Range<usize>>,
    },

    #[error("Quantified object would escape it's scope")]
    QuantifiedObjectEscapesScope(Option<Range<usize>>),
}

pub fn synthesize(
    proof_term: &ProofTerm,
    ctx: &IdentifierContext,
    identifier_factory: &mut IdentifierFactory,
) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
    let mut visitor = SynthesizeVisitor::new(ctx, identifier_factory);

    proof_term.visit(&mut visitor)
}

struct SynthesizeVisitor<'a> {
    ctx: &'a IdentifierContext,
    identifier_factory: &'a mut IdentifierFactory,
}

impl<'a> SynthesizeVisitor<'a> {
    pub fn new(ctx: &'a IdentifierContext, identifier_factory: &'a mut IdentifierFactory) -> Self {
        Self {
            ctx,
            identifier_factory,
        }
    }
}

impl<'a> ProofTermVisitor<Result<(Type, TypeCheckerResult), SynthesizeError>>
    for SynthesizeVisitor<'a>
{
    fn visit_ident(&mut self, ident: &Ident) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let Ident(ident, ident_span) = ident;

        // lookup identifier
        let (identifier, ident_type) = match self.ctx.get_by_name(ident) {
            Some(ident_type) => ident_type,
            None => {
                return Err(SynthesizeError::UnknownIdentifier(
                    ident.clone(),
                    ident_span.clone(),
                ))
            }
        };

        // decide whether proposition is true or type judgment
        let conclusion = match ident_type {
            Type::Prop(prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            Type::Datatype(datatype) => {
                ProofTreeConclusion::TypeJudgement(identifier.clone(), datatype.clone())
            }
        };

        Ok((
            ident_type.clone(),
            TypeCheckerResult {
                goals: vec![],
                proof_tree: ProofTree {
                    premisses: vec![],
                    rule: ProofTreeRule::Ident(identifier.clone()),
                    conclusion,
                },
            },
        ))
    }

    fn visit_pair(&mut self, pair: &Pair) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let Pair(fst, snd, span) = pair;

        let (fst_type, fst_result) = synthesize(fst, self.ctx, self.identifier_factory)?;
        let (snd_type, snd_result) = synthesize(snd, self.ctx, self.identifier_factory)?;

        match (&fst_type, &snd_type) {
            // Exists
            (Type::Datatype(_), Type::Prop(_)) => {
                Err(SynthesizeError::TypeAnnotationsNeeded(span.clone()))
            }

            // And
            (Type::Prop(fst_prop), Type::Prop(snd_prop)) => {
                let _type = Prop::And(fst_prop.boxed(), snd_prop.boxed());

                Ok((
                    _type.clone().into(),
                    TypeCheckerResult {
                        goals: [fst_result.goals, snd_result.goals].concat(),
                        proof_tree: ProofTree {
                            premisses: vec![fst_result.proof_tree, snd_result.proof_tree],
                            rule: ProofTreeRule::AndIntro,
                            conclusion: ProofTreeConclusion::PropIsTrue(_type),
                        },
                    },
                ))
            }

            // other
            (_, Type::Datatype(datatype)) => {
                Err(SynthesizeError::ExpectedPropAsSecondPairComponent {
                    received_datatype: datatype.to_string(),
                    span: span.clone(),
                })
            }
        }
    }

    fn visit_project_fst(
        &mut self,
        projection: &ProjectFst,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let ProjectFst(body, span) = projection;

        let (body_type, body_result) = synthesize(body, self.ctx, self.identifier_factory)?;

        let fst = match body_type {
            Type::Prop(Prop::And(fst, _)) => fst,
            _ => {
                return Err(SynthesizeError::UnexpectedPropKind {
                    expected: vec![PropKind::And],
                    received: body_type,
                    span: span.clone(),
                })
            }
        };

        Ok((
            Type::Prop(*fst.clone()),
            TypeCheckerResult {
                goals: body_result.goals,
                proof_tree: ProofTree {
                    premisses: vec![body_result.proof_tree],
                    rule: ProofTreeRule::AndElimFst,
                    conclusion: ProofTreeConclusion::PropIsTrue(*fst),
                },
            },
        ))
    }

    fn visit_project_snd(
        &mut self,
        projection: &ProjectSnd,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let ProjectSnd(body, span) = projection;

        let (body_type, body_result) = synthesize(body, self.ctx, self.identifier_factory)?;

        let snd = match body_type {
            Type::Prop(Prop::And(_, snd)) => snd,
            _ => {
                return Err(SynthesizeError::UnexpectedPropKind {
                    expected: vec![PropKind::And],
                    received: body_type,
                    span: span.clone(),
                })
            }
        };

        Ok((
            Type::Prop(*snd.clone()),
            TypeCheckerResult {
                goals: body_result.goals,
                proof_tree: ProofTree {
                    premisses: vec![body_result.proof_tree],
                    rule: ProofTreeRule::AndElimSnd,
                    conclusion: ProofTreeConclusion::PropIsTrue(*snd),
                },
            },
        ))
    }

    fn visit_function(
        &mut self,
        function: &Function,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let Function {
            param_ident,
            param_type,
            body,
            span,
        } = function;

        // require param annotation
        let param_type = match param_type {
            Some(param_type) => param_type,
            None => return Err(SynthesizeError::TypeAnnotationsNeeded(span.clone())),
        };

        // check that if parameter is Prop, it only has known identifiers as free occurrences
        let mut bound_param_type = param_type.clone();
        bound_param_type
            .instantiate_parameters_with_context(&self.ctx)
            .map_err(|err| match err {
                InstatiationError::UnknownIdentifier(ident) => {
                    SynthesizeError::UnknownIdentifier(ident, span.clone())
                }
            })?;

        // add param to context
        let param_identifier = self.identifier_factory.create(param_ident.clone());
        let mut body_ctx = self.ctx.clone();
        body_ctx.insert(param_identifier.clone(), bound_param_type.clone());
        let (body_type, body_result) = synthesize(body, &body_ctx, self.identifier_factory)?;

        match (&bound_param_type, &body_type) {
            // Forall
            (Type::Datatype(ident), Type::Prop(body_type)) => {
                let _type = Prop::ForAll {
                    object_ident: param_ident.clone(),
                    object_type_ident: ident.clone(),
                    body: body_type.boxed(),
                };

                Ok((
                    _type.clone().into(),
                    TypeCheckerResult {
                        goals: body_result.goals,
                        proof_tree: ProofTree {
                            premisses: vec![body_result.proof_tree],
                            rule: ProofTreeRule::ForAllIntro(param_identifier),
                            conclusion: ProofTreeConclusion::PropIsTrue(_type),
                        },
                    },
                ))
            }

            // Implication
            (Type::Prop(fst), Type::Prop(snd)) => {
                let _type = Prop::Impl(fst.boxed(), snd.boxed());

                Ok((
                    _type.clone().into(),
                    TypeCheckerResult {
                        goals: body_result.goals,
                        proof_tree: ProofTree {
                            premisses: vec![body_result.proof_tree],
                            rule: ProofTreeRule::ImplIntro(param_identifier),
                            conclusion: ProofTreeConclusion::PropIsTrue(_type),
                        },
                    },
                ))
            }

            // otherwise
            (_, Type::Datatype(_)) => {
                Err(SynthesizeError::CannotReturnDatatype(body.span().clone()))
            }
        }
    }

    fn visit_application(
        &mut self,
        application: &Application,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let Application {
            function,
            applicant,
            ..
        } = application;

        // synthesize function
        let (function_type, function_result) =
            synthesize(function, self.ctx, self.identifier_factory)?;

        let (requested_applicant_type, return_type, rule) =
            match function_type {
                // Implication
                Type::Prop(Prop::Impl(fst, snd)) => (
                    Type::Prop(*fst.clone()),
                    *snd.clone(),
                    ProofTreeRule::ImplElim,
                ),

                // Universal quantification
                Type::Prop(Prop::ForAll {
                    object_ident,
                    object_type_ident,
                    mut body,
                }) => {
                    let (param_ident, param_span) = match **applicant {
                        ProofTerm::Ident(Ident(ref ident, ref span)) => (ident, span),
                        _ => {
                            return Err(SynthesizeError::CannotReturnDatatype(
                                applicant.span().clone(),
                            ))
                        }
                    };

                    let (identifier, _) = self.ctx.get_by_name(param_ident).ok_or(
                        SynthesizeError::UnknownIdentifier(param_ident.clone(), param_span.clone()),
                    )?;
                    body.instantiate_free_parameter(&object_ident, identifier);

                    (
                        Type::Datatype(object_type_ident.clone()),
                        *body.clone(),
                        ProofTreeRule::ForAllElim,
                    )
                }

                // other
                _ => {
                    return Err(SynthesizeError::UnexpectedPropKind {
                        expected: vec![PropKind::Impl, PropKind::ForAll],
                        received: function_type,
                        span: function.span().clone(),
                    })
                }
            };

        let applicant_result = check_allowing_free_params(
            applicant,
            &requested_applicant_type,
            self.ctx,
            self.identifier_factory,
        )
        .map_err(|check_err| match check_err {
            CheckError::SynthesizeError(synth_err) => synth_err,
            _ => SynthesizeError::CheckError(Box::new(check_err)),
        })?;

        Ok((
            Type::Prop(return_type.clone()),
            TypeCheckerResult {
                goals: [function_result.goals, applicant_result.goals].concat(),
                proof_tree: ProofTree {
                    premisses: vec![function_result.proof_tree, applicant_result.proof_tree],
                    rule,
                    conclusion: ProofTreeConclusion::PropIsTrue(return_type),
                },
            },
        ))
    }

    fn visit_let_in(
        &mut self,
        let_in: &LetIn,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let LetIn {
            fst_ident,
            snd_ident,
            head,
            body,
            ..
        } = let_in;

        let (head_type, pair_result) = synthesize(head, self.ctx, self.identifier_factory)?;

        if let Type::Prop(Prop::Exists {
            object_ident,
            object_type_ident,
            body: mut exists_body,
        }) = head_type
        {
            let fst_identifier = self.identifier_factory.create(fst_ident.clone());
            let snd_identifier = self.identifier_factory.create(snd_ident.clone());

            exists_body.instantiate_free_parameter(&object_ident, &fst_identifier);

            let mut body_ctx = self.ctx.clone();
            body_ctx.insert(fst_identifier.clone(), Type::Datatype(object_type_ident));
            body_ctx.insert(snd_identifier.clone(), Type::Prop(*exists_body));
            let (body_type, body_result) = synthesize(body, &body_ctx, self.identifier_factory)?;

            if let Type::Prop(prop) = &body_type {
                // check that quantified object does not escape it's scope
                if prop
                    .get_free_parameters()
                    .contains(&PropParameter::Instantiated(fst_identifier.clone()))
                {
                    return Err(SynthesizeError::QuantifiedObjectEscapesScope(
                        body.span().clone(),
                    ));
                }

                Ok((
                    body_type.clone(),
                    TypeCheckerResult {
                        goals: [pair_result.goals, body_result.goals].concat(),
                        proof_tree: ProofTree {
                            premisses: vec![pair_result.proof_tree, body_result.proof_tree],
                            rule: ProofTreeRule::ExistsElim(fst_identifier, snd_identifier),
                            conclusion: ProofTreeConclusion::PropIsTrue(prop.clone()),
                        },
                    },
                ))
            } else {
                Err(SynthesizeError::CannotReturnDatatype(body.span().clone()))
            }
        } else {
            Err(SynthesizeError::UnexpectedPropKind {
                expected: vec![PropKind::Exists],
                received: head_type,
                span: head.span().clone(),
            })
        }
    }

    fn visit_or_left(
        &mut self,
        or_left: &OrLeft,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let OrLeft(_, span) = or_left;

        Err(SynthesizeError::NotSynthesizing(
            ProofTermKind::OrLeft,
            span.clone(),
        ))
    }

    fn visit_or_right(
        &mut self,
        or_right: &OrRight,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let OrRight(_, span) = or_right;

        Err(SynthesizeError::NotSynthesizing(
            ProofTermKind::OrRight,
            span.clone(),
        ))
    }

    fn visit_case(&mut self, case: &Case) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let Case {
            head,
            fst_ident,
            fst_term,
            snd_ident,
            snd_term,
            span,
        } = case;

        let (head_type, head_result) = synthesize(head, self.ctx, self.identifier_factory)?;

        let (fst, snd) = match head_type {
            Type::Prop(Prop::Or(fst, snd)) => (fst, snd),
            _ => {
                return Err(SynthesizeError::UnexpectedPropKind {
                    expected: vec![PropKind::Or],
                    received: head_type,
                    span: head.span().clone(),
                })
            }
        };

        // synthesize fst arm
        let fst_identifier = self.identifier_factory.create(fst_ident.clone());
        let mut fst_ctx = self.ctx.clone();
        fst_ctx.insert(fst_identifier.clone(), Type::Prop(*fst));
        let (fst_type, fst_result) = synthesize(fst_term, &fst_ctx, self.identifier_factory)?;

        // snythesize snd arm
        let snd_identifier = self.identifier_factory.create(snd_ident.clone());
        let mut snd_ctx = self.ctx.clone();
        snd_ctx.insert(snd_identifier.clone(), Type::Prop(*snd));
        let (snd_type, snd_result) = synthesize(snd_term, &snd_ctx, self.identifier_factory)?;

        // check for alpha-equivalence
        if !Type::alpha_eq(&fst_type, &snd_type) {
            return Err(SynthesizeError::CaseArmsDifferent {
                fst_type,
                snd_type,
                span: span.clone(),
            });
        }

        // check whether datatype returned
        let conclusion = match fst_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(SynthesizeError::CannotReturnDatatype(span.clone())),
        };

        Ok((
            snd_type,
            TypeCheckerResult {
                goals: [head_result.goals, fst_result.goals, snd_result.goals].concat(),
                proof_tree: ProofTree {
                    premisses: vec![
                        head_result.proof_tree,
                        fst_result.proof_tree,
                        snd_result.proof_tree,
                    ],
                    rule: ProofTreeRule::OrElim(fst_identifier, snd_identifier),
                    conclusion,
                },
            },
        ))
    }

    fn visit_abort(&mut self, abort: &Abort) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let Abort(_, span) = abort;

        Err(SynthesizeError::NotSynthesizing(
            ProofTermKind::Abort,
            span.clone(),
        ))
    }

    fn visit_unit(
        &mut self,
        _span: Option<Range<usize>>,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        Ok((
            Type::Prop(Prop::True),
            TypeCheckerResult {
                goals: vec![],
                proof_tree: ProofTree {
                    premisses: vec![],
                    rule: ProofTreeRule::TrueIntro,
                    conclusion: ProofTreeConclusion::PropIsTrue(Prop::True),
                },
            },
        ))
    }

    fn visit_type_ascription(
        &mut self,
        type_ascription: &TypeAscription,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        let TypeAscription {
            proof_term,
            ascription,
            span,
        } = type_ascription;

        // check that  if ascription is Prop, it only has known identifiers as free occurences
        let mut instantiated_ascription = ascription.clone();
        instantiated_ascription
            .instantiate_parameters_with_context(&self.ctx)
            .map_err(|err| match err {
                InstatiationError::UnknownIdentifier(ident) => {
                    SynthesizeError::UnknownIdentifier(ident, span.clone())
                }
            })?;

        check_allowing_free_params(
            proof_term,
            &instantiated_ascription,
            self.ctx,
            self.identifier_factory,
        )
        .map(|proof_tree| (instantiated_ascription.clone(), proof_tree))
        .map_err(|check_err| match check_err {
            CheckError::SynthesizeError(synth_err) => synth_err,
            _ => SynthesizeError::CheckError(Box::new(check_err)),
        })
    }

    fn visit_sorry(
        &mut self,
        span: Option<Range<usize>>,
    ) -> Result<(Type, TypeCheckerResult), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::Sorry, span))
    }
}
