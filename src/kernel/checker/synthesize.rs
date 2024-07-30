use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::kernel::{
    proof_term::{
        Abort, Application, Case, Function, Ident, LetIn, OrLeft, OrRight, Pair, ProjectFst,
        ProjectSnd, ProofTerm, ProofTermKind, ProofTermVisitor, Type, TypeAscription,
    },
    proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
    prop::{Prop, PropKind, PropParameter},
};

use super::{
    check::{check_allowing_free_params, CheckError},
    identifier::IdentifierFactory,
    identifier_context::IdentifierContext,
};

#[derive(Debug, Error, PartialEq, Eq, Serialize, Deserialize, Clone)]
pub enum SynthesizeError {
    #[error("Checking type failed")]
    CheckError(#[from] Box<CheckError>),

    #[error("Unknown identifier")]
    UnknownIdentifier(String),

    #[error("Cannot refer to Prop with unknown parameter")]
    UnknownPropParameter { prop: Prop, parameter_ident: String },

    #[error("Type Annotations needed")]
    TypeAnnotationsNeeded,

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
    ctx: &IdentifierContext,
    identifier_factory: &mut IdentifierFactory,
) -> Result<(Type, ProofTree), SynthesizeError> {
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

    fn bind_free_params_to_ctx(&self, _type: &mut Type) -> Result<(), SynthesizeError> {
        if let Type::Prop(ref mut prop) = _type {
            let free_params = prop.get_free_parameters_mut();

            for free_param in free_params {
                match free_param {
                    PropParameter::Uninstantiated(name) => {
                        let (identifier, _) = self
                            .ctx
                            .get_by_name(name)
                            .ok_or(SynthesizeError::UnknownIdentifier(name.clone()))?;
                        *free_param = PropParameter::Instantiated(identifier.clone())
                    }
                    PropParameter::Instantiated(identifier) => {
                        // sanity check
                        if self.ctx.get(identifier).is_none() {
                            panic!("Instantiated parameter does not exist: {:#?}", identifier);
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl<'a> ProofTermVisitor<Result<(Type, ProofTree), SynthesizeError>> for SynthesizeVisitor<'a> {
    fn visit_ident(&mut self, ident: &Ident) -> Result<(Type, ProofTree), SynthesizeError> {
        let Ident(ident) = ident;

        // lookup identifier
        let (identifier, ident_type) = match self.ctx.get_by_name(ident) {
            Some(ident_type) => ident_type,
            None => return Err(SynthesizeError::UnknownIdentifier(ident.clone())),
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
            ProofTree {
                premisses: vec![],
                rule: ProofTreeRule::Ident(ident.clone()),
                conclusion,
            },
        ))
    }

    fn visit_pair(&mut self, pair: &Pair) -> Result<(Type, ProofTree), SynthesizeError> {
        let Pair(fst, snd) = pair;

        let (fst_type, fst_proof_tree) = synthesize(fst, self.ctx, self.identifier_factory)?;
        let (snd_type, snd_proof_tree) = synthesize(snd, self.ctx, self.identifier_factory)?;

        match (&fst_type, &snd_type) {
            // Exists
            (Type::Datatype(_), Type::Prop(_)) => {
                Err(SynthesizeError::TypeAnnotationsNeeded)
            }

            // And
            (Type::Prop(fst_prop), Type::Prop(snd_prop)) => {
                let _type = Prop::And(fst_prop.boxed(), snd_prop.boxed());

                Ok((
                    _type.clone().into(),
                    ProofTree {
                        premisses: vec![fst_proof_tree, snd_proof_tree],
                        rule: ProofTreeRule::AndIntro,
                        conclusion: ProofTreeConclusion::PropIsTrue(_type),
                    },
                ))
            }

            // other
            (_, Type::Datatype(datatype)) => Err(SynthesizeError::ExpectedProp {
                received_datatype: datatype.to_string(),
            }),
        }
    }

    fn visit_project_fst(
        &mut self,
        projection: &ProjectFst,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let ProjectFst(body) = projection;

        let (body_type, body_proof_tree) =
            synthesize(body, self.ctx, self.identifier_factory)?;

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
                conclusion: ProofTreeConclusion::PropIsTrue(*fst),
            },
        ))
    }

    fn visit_project_snd(
        &mut self,
        projection: &ProjectSnd,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let ProjectSnd(body) = projection;

        let (body_type, body_proof_tree) =
            synthesize(body, self.ctx, self.identifier_factory)?;

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
                rule: ProofTreeRule::AndElimSnd,
                conclusion: ProofTreeConclusion::PropIsTrue(*snd),
            },
        ))
    }

    fn visit_function(
        &mut self,
        function: &Function,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let Function {
            param_ident,
            param_type,
            body,
        } = function;

        // require param annotation
        let param_type = match param_type {
            Some(param_type) => param_type,
            None => return Err(SynthesizeError::TypeAnnotationsNeeded),
        };

        // check that  if parameter is Prop, it only has known identifiers as free occurences
        let mut binded_param_type = param_type.clone();
        self.bind_free_params_to_ctx(&mut binded_param_type)?;

        // add param to context
        let param_identifier = self.identifier_factory.create(param_ident.clone());
        let mut body_ctx = self.ctx.clone();
        body_ctx.insert(param_identifier, binded_param_type);
        let (body_type, body_proof_tree) =
            synthesize(body, &body_ctx, self.identifier_factory)?;

        match (&param_type, &body_type) {
            // Forall
            (Type::Datatype(ident), Type::Prop(body_type)) => {
                let _type = Prop::ForAll {
                    object_ident: param_ident.clone(),
                    object_type_ident: ident.clone(),
                    body: body_type.boxed(),
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

            // Implication
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

            // otherwise
            (_, Type::Datatype(_)) => Err(SynthesizeError::CannotReturnDatatype),
        }
    }

    fn visit_application(
        &mut self,
        application: &Application,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let Application {
            function,
            applicant,
        } = application;

        // synthesize function
        let (function_type, function_proof_tree) =
            synthesize(function, self.ctx, self.identifier_factory)?;

        let (requested_applicant_type, return_type, rule) = match function_type {
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
                let param_ident = match **applicant {
                    ProofTerm::Ident(Ident(ref ident)) => ident,
                    _ => unreachable!("Datatype functions do not exist"),
                };

                let (identifier, _) = self
                    .ctx
                    .get_by_name(param_ident)
                    .ok_or(SynthesizeError::UnknownIdentifier(param_ident.clone()))?;
                body.instantiate_free_parameter(&object_ident, identifier);

                (
                    Type::Datatype(object_type_ident.clone()),
                    *body.clone(),
                    ProofTreeRule::ForAllElim,
                )
            }

            // other
            _ => {
                return Err(SynthesizeError::UnexpectedProofTermKind {
                    expected: vec![ProofTermKind::Function],
                    received: function_type,
                })
            }
        };

        let applicant_proof_tree = check_allowing_free_params(
            applicant,
            &requested_applicant_type,
            self.ctx,
            self.identifier_factory,
        )
        .map_err(Box::new)?;

        Ok((
            Type::Prop(return_type.clone()),
            ProofTree {
                premisses: vec![function_proof_tree, applicant_proof_tree],
                rule,
                conclusion: ProofTreeConclusion::PropIsTrue(return_type),
            },
        ))
    }

    fn visit_let_in(&mut self, let_in: &LetIn) -> Result<(Type, ProofTree), SynthesizeError> {
        let LetIn {
            fst_ident,
            snd_ident,
            head,
            body,
        } = let_in;

        let (head_type, pair_proof_term_tree) =
            synthesize(head, self.ctx, self.identifier_factory)?;

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
            body_ctx.insert(snd_identifier, Type::Prop(*exists_body));
            let (body_type, body_proof_tree) =
                synthesize(body, &body_ctx, self.identifier_factory)?;

            if let Type::Prop(prop) = &body_type {
                // check that quantified object does not escape it's scope
                if prop
                    .get_free_parameters()
                    .contains(&PropParameter::Instantiated(fst_identifier))
                {
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
                received: head_type,
            })
        }
    }

    fn visit_or_left(&mut self, _body: &OrLeft) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::OrLeft))
    }

    fn visit_or_right(&mut self, _body: &OrRight) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::OrRight))
    }

    fn visit_case(&mut self, case: &Case) -> Result<(Type, ProofTree), SynthesizeError> {
        let Case {
            head,
            fst_ident,
            fst_term,
            snd_ident,
            snd_term,
        } = case;

        let (proof_term_type, proof_term_tree) =
            synthesize(head, self.ctx, self.identifier_factory)?;

        let (fst, snd) = match proof_term_type {
            Type::Prop(Prop::Or(fst, snd)) => (fst, snd),
            _ => {
                return Err(SynthesizeError::UnexpectedPropKind {
                    expected: vec![PropKind::Or],
                    received: proof_term_type,
                })
            }
        };

        // synthesize fst arm
        let fst_identifier = self.identifier_factory.create(fst_ident.clone());
        let mut fst_ctx = self.ctx.clone();
        fst_ctx.insert(fst_identifier, Type::Prop(*fst));
        let (fst_type, fst_proof_tree) =
            synthesize(fst_term, &fst_ctx, self.identifier_factory)?;

        // snythesize snd arm
        let snd_identifier = self.identifier_factory.create(snd_ident.clone());
        let mut snd_ctx = self.ctx.clone();
        snd_ctx.insert(snd_identifier, Type::Prop(*snd));
        let (snd_type, snd_proof_tree) =
            synthesize(snd_term, &snd_ctx, self.identifier_factory)?;

        // check for alpha-equivalence
        if !Type::alpha_eq(&fst_type, &snd_type) {
            return Err(SynthesizeError::CaseArmsDifferent { fst_type, snd_type });
        }

        // check whether datatype returned
        let conclusion = match fst_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(SynthesizeError::CannotReturnDatatype),
        };

        Ok((
            snd_type,
            ProofTree {
                premisses: vec![proof_term_tree, fst_proof_tree, snd_proof_tree],
                rule: ProofTreeRule::OrElim(fst_ident.clone(), snd_ident.clone()),
                conclusion,
            },
        ))
    }

    fn visit_abort(&mut self, _abort: &Abort) -> Result<(Type, ProofTree), SynthesizeError> {
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

    fn visit_type_ascription(
        &mut self,
        type_ascription: &TypeAscription,
    ) -> Result<(Type, ProofTree), SynthesizeError> {
        let TypeAscription {
            proof_term,
            ascription,
        } = type_ascription;

        // check that  if ascription is Prop, it only has known identifiers as free occurences
        let mut binded_ascription = ascription.clone();
        self.bind_free_params_to_ctx(&mut binded_ascription)?;

        check_allowing_free_params(
            proof_term,
            &binded_ascription,
            self.ctx,
            self.identifier_factory,
        )
        .map(|proof_tree| (ascription.clone(), proof_tree))
        .map_err(|check_err| SynthesizeError::CheckError(Box::new(check_err)))
    }
    
    fn visit_sorry(&mut self) -> Result<(Type, ProofTree), SynthesizeError> {
        Err(SynthesizeError::NotSynthesizing(ProofTermKind::Sorry))
    }
}
