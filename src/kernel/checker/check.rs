use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    kernel::{
        proof_term::{
            Abort, Application, Case, Function, Ident, LetIn, OrLeft, OrRight, Pair, ProjectFst,
            ProjectSnd, ProofTerm, ProofTermKind, ProofTermVisitor, Type, TypeAscription,
        },
        proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
        prop::{Prop, PropKind, PropParameter},
    },
    util::counter::Counter,
};

use super::{
    identifier::{Identifier, IdentifierFactory},
    identifier_context::IdentifierContext,
    synthesize::{synthesize, SynthesizeError},
};

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum CheckError {
    #[error("An error happened while synthesizing")]
    SynthesizeError(#[from] SynthesizeError),

    #[error("Checkable type cannot include parameters")]
    PropHasFreeParameters(Prop),

    #[error("Identifier {0} unknown")]
    UnknownIdentifier(String),

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

    #[error("Ascription does not match expected type")]
    UnexpectedTypeAscription { expected: Type, ascription: Type },

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

    let mut identifier_factory = IdentifierFactory::new(Counter::new());

    check_allowing_free_params(
        proof_term,
        &Type::Prop(expected_prop.clone()),
        ctx,
        &mut identifier_factory,
    )
}

pub(super) fn check_allowing_free_params(
    proof_term: &ProofTerm,
    expected_type: &Type,
    ctx: &IdentifierContext,
    identifier_factory: &mut IdentifierFactory,
) -> Result<ProofTree, CheckError> {
    let mut visitor = CheckVisitor::new(expected_type.clone(), ctx, identifier_factory);
    proof_term.visit(&mut visitor)
}

struct CheckVisitor<'a> {
    expected_type: Type,
    ctx: &'a IdentifierContext,
    identifier_factory: &'a mut IdentifierFactory,
}

impl<'a> CheckVisitor<'a> {
    pub fn new(
        expected_type: Type,
        ctx: &'a IdentifierContext,
        identifier_factory: &'a mut IdentifierFactory,
    ) -> Self {
        Self {
            expected_type,
            ctx,
            identifier_factory,
        }
    }
}

impl<'a> ProofTermVisitor<Result<ProofTree, CheckError>> for CheckVisitor<'a> {
    fn visit_ident(&mut self, ident: &Ident) -> Result<ProofTree, CheckError> {
        // use =>
        //     <= rule

        let (_type, proof_tree) = synthesize(
            &ProofTerm::Ident(ident.clone()),
            self.ctx,
            self.identifier_factory,
        )?;

        if Type::alpha_eq(&_type, &self.expected_type) {
            Ok(proof_tree)
        } else {
            Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: _type,
            })
        }
    }

    fn visit_pair(&mut self, pair: &Pair) -> Result<ProofTree, CheckError> {
        let Pair(fst_term, snd_term) = pair;

        let (expected_fst_type, expected_snd_type, rule, conclusion) = match self.expected_type {
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
                let substitution = match **fst_term {
                    ProofTerm::Ident(Ident(ref ident)) => ident.clone(),
                    _ => unreachable!(),
                };

                // instantiate body
                let (identifier, _) = self
                    .ctx
                    .get_by_name(&substitution)
                    .ok_or(CheckError::UnknownIdentifier(substitution.clone()))?;
                let mut substitued_body = *body.clone();
                substitued_body.instantiate_free_parameter(object_ident, identifier);

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
                    proof_term: ProofTerm::Pair(pair.clone()),
                })
            }
        };

        // check pair components
        let fst_proof_tree = check_allowing_free_params(
            fst_term,
            &expected_fst_type,
            self.ctx,
            self.identifier_factory,
        )?;
        let snd_proof_tree = check_allowing_free_params(
            snd_term,
            &expected_snd_type,
            self.ctx,
            self.identifier_factory,
        )?;

        Ok(ProofTree {
            premisses: vec![fst_proof_tree, snd_proof_tree],
            rule,
            conclusion,
        })
    }

    fn visit_project_fst(&mut self, projection: &ProjectFst) -> Result<ProofTree, CheckError> {
        // use =>
        //     <= rule

        let (projection_type, projection_proof_tree) = synthesize(
            &ProofTerm::ProjectFst(projection.clone()),
            self.ctx,
            self.identifier_factory,
        )?;

        if Type::alpha_eq(&self.expected_type, &projection_type) {
            Ok(projection_proof_tree)
        } else {
            Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: projection_type,
            })
        }
    }

    fn visit_project_snd(&mut self, projection: &ProjectSnd) -> Result<ProofTree, CheckError> {
        // use =>
        //     <= rule

        let (projection_type, projection_proof_tree) = synthesize(
            &ProofTerm::ProjectSnd(projection.clone()),
            self.ctx,
            self.identifier_factory,
        )?;

        if Type::alpha_eq(&self.expected_type, &projection_type) {
            Ok(projection_proof_tree)
        } else {
            Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: projection_type,
            })
        }
    }

    fn visit_function(&mut self, function: &Function) -> Result<ProofTree, CheckError> {
        let Function {
            param_ident,
            param_type,
            body,
        } = function;

        let param_identifier = self.identifier_factory.create(param_ident.clone());

        let (expected_param_type, expected_body_prop, rule, conclusion) = match self.expected_type {
            // Implication
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
                // instantiate body with parameter name of function to account for alpha-Equivalence
                let mut expected_body_prop = *body.clone();
                expected_body_prop.instantiate_free_parameter(object_ident, &param_identifier);

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
                    proof_term: ProofTerm::Function(function.clone()),
                })
            }
        };

        // fail if type annotation is not expected type
        if let Some(unboxed_param_type) = param_type {
            if !Type::alpha_eq(unboxed_param_type, &expected_param_type) {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::Function(function.clone()),
                });
            }
        }

        // check body of function
        let mut body_ctx = self.ctx.clone();
        body_ctx.insert(param_identifier, expected_param_type.clone());
        let body_proof_tree = check_allowing_free_params(
            body,
            &Type::Prop(expected_body_prop),
            &body_ctx,
            self.identifier_factory,
        )?;

        Ok(ProofTree {
            premisses: vec![body_proof_tree],
            rule,
            conclusion,
        })
    }

    fn visit_application(&mut self, application: &Application) -> Result<ProofTree, CheckError> {
        let Application {
            function,
            applicant,
        } = application;

        let (expected_return_prop, conclusion) = match self.expected_type {
            Type::Prop(ref prop) => (prop, ProofTreeConclusion::PropIsTrue(prop.clone())),
            Type::Datatype(_) => return Err(CheckError::CannotReturnDatatype),
        };

        // synthesize applicant
        let applicant_synthesize_result = synthesize(applicant, self.ctx, self.identifier_factory);

        // use  ⊃ E<= rule
        if let Ok((Type::Prop(applicant_prop), applicant_proof_tree)) = applicant_synthesize_result
        {
            // check function
            let expected_function_type = Type::Prop(Prop::Impl(
                applicant_prop.boxed(),
                expected_return_prop.boxed(),
            ));

            let function_proof_tree = check_allowing_free_params(
                function,
                &expected_function_type,
                self.ctx,
                self.identifier_factory,
            )?;

            return Ok(ProofTree {
                premisses: vec![function_proof_tree, applicant_proof_tree],
                rule: ProofTreeRule::ImplElim,
                conclusion,
            });
        }

        // use =>
        //     <= rule

        // synthesize application
        let (application_type, application_proof_tree) = synthesize(
            &ProofTerm::Application(application.clone()),
            self.ctx,
            self.identifier_factory,
        )?;

        // test for alpha-equivalence
        if !Type::alpha_eq(&application_type, &self.expected_type) {
            return Err(CheckError::UnexpectedType {
                expected: self.expected_type.clone(),
                received: application_type,
            });
        }

        Ok(application_proof_tree)
    }

    fn visit_let_in(&mut self, let_in: &LetIn) -> Result<ProofTree, CheckError> {
        let LetIn {
            fst_ident,
            snd_ident,
            head,
            body,
        } = let_in;

        let (head_type, head_proof_tree) = synthesize(head, self.ctx, self.identifier_factory)?;

        if let Type::Prop(Prop::Exists {
            object_ident,
            object_type_ident,
            body: mut exists_body,
        }) = head_type
        {
            let fst_identifier = self.identifier_factory.create(fst_ident.clone());
            let snd_identifier = self.identifier_factory.create(snd_ident.clone());

            // instantiate proof with given name for witness
            exists_body.instantiate_free_parameter(&object_ident, &fst_identifier);

            // check body
            let mut body_ctx = self.ctx.clone();
            body_ctx.insert(fst_identifier.clone(), Type::Datatype(object_type_ident));
            body_ctx.insert(snd_identifier, Type::Prop(*exists_body));
            let body_proof_tree = check_allowing_free_params(
                body,
                &self.expected_type,
                &body_ctx,
                self.identifier_factory,
            )?;

            if let Type::Prop(prop) = &self.expected_type {
                // check that quantified object does not escape it's scope
                if prop
                    .get_free_parameters()
                    .contains(&PropParameter::Instantiated(fst_identifier))
                {
                    return Err(CheckError::QuantifiedObjectEscapesScope);
                }

                Ok(ProofTree {
                    premisses: vec![head_proof_tree, body_proof_tree],
                    rule: ProofTreeRule::ExistsElim(fst_ident.clone(), snd_ident.clone()),
                    conclusion: ProofTreeConclusion::PropIsTrue(prop.clone()),
                })
            } else {
                Err(CheckError::CannotReturnDatatype)
            }
        } else {
            Err(CheckError::UnexpectedProofTermKind {
                expected: vec![ProofTermKind::ExistsPair],
                received: head_type,
            })
        }
    }

    fn visit_or_left(&mut self, or_left: &OrLeft) -> Result<ProofTree, CheckError> {
        let OrLeft(body) = or_left;

        let (expected_body_type, conclusion) = match self.expected_type {
            Type::Prop(ref prop @ Prop::Or(ref fst, _)) => (
                Type::Prop(*fst.clone()),
                ProofTreeConclusion::PropIsTrue(prop.clone()),
            ),
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::OrLeft(or_left.clone()),
                })
            }
        };

        let body_proof_tree = check_allowing_free_params(
            body,
            &expected_body_type,
            self.ctx,
            self.identifier_factory,
        )?;

        Ok(ProofTree {
            premisses: vec![body_proof_tree],
            rule: ProofTreeRule::OrIntroFst,
            conclusion,
        })
    }

    fn visit_or_right(&mut self, or_right: &OrRight) -> Result<ProofTree, CheckError> {
        let OrRight(body) = or_right;

        let (expected_body_type, conclusion) = match self.expected_type {
            Type::Prop(ref prop @ Prop::Or(_, ref snd)) => (
                Type::Prop(*snd.clone()),
                ProofTreeConclusion::PropIsTrue(prop.clone()),
            ),
            _ => {
                return Err(CheckError::IncompatibleProofTerm {
                    expected_type: self.expected_type.clone(),
                    proof_term: ProofTerm::OrRight(or_right.clone()),
                })
            }
        };

        let body_proof_tree = check_allowing_free_params(
            body,
            &expected_body_type,
            self.ctx,
            self.identifier_factory,
        )?;

        Ok(ProofTree {
            premisses: vec![body_proof_tree],
            rule: ProofTreeRule::OrIntroSnd,
            conclusion,
        })
    }

    fn visit_case(&mut self, case: &Case) -> Result<ProofTree, CheckError> {
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
                return Err(CheckError::UnexpectedPropKind {
                    expected: vec![PropKind::Or],
                    received: proof_term_type,
                })
            }
        };

        // check fst case arm
        let fst_identifier = self.identifier_factory.create(fst_ident.clone());
        let mut fst_ctx = self.ctx.clone();
        fst_ctx.insert(fst_identifier, Type::Prop(*fst));
        let fst_proof_tree = check_allowing_free_params(
            fst_term,
            &self.expected_type,
            &fst_ctx,
            self.identifier_factory,
        )?;

        // check snd case arm
        let snd_identifier = self.identifier_factory.create(snd_ident.clone());
        let mut snd_ctx = self.ctx.clone();
        snd_ctx.insert(snd_identifier, Type::Prop(*snd));
        let snd_proof_tree = check_allowing_free_params(
            snd_term,
            &self.expected_type,
            &snd_ctx,
            self.identifier_factory,
        )?;

        // check whether datatype returned
        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            _ => return Err(CheckError::CannotReturnDatatype),
        };

        Ok(ProofTree {
            premisses: vec![proof_term_tree, fst_proof_tree, snd_proof_tree],
            rule: ProofTreeRule::OrElim(fst_ident.clone(), snd_ident.clone()),
            conclusion,
        })
    }

    fn visit_abort(&mut self, abort: &Abort) -> Result<ProofTree, CheckError> {
        let Abort(body) = abort;

        let body_proof_tree = check_allowing_free_params(
            body,
            &Type::Prop(Prop::False),
            self.ctx,
            self.identifier_factory,
        )?;

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

    fn visit_type_ascription(
        &mut self,
        type_ascription: &TypeAscription,
    ) -> Result<ProofTree, CheckError> {
        let TypeAscription {
            ascription,
            proof_term,
        } = type_ascription;

        if !Type::alpha_eq(&self.expected_type, ascription) {
            return Err(CheckError::UnexpectedTypeAscription {
                expected: self.expected_type.clone(),
                ascription: ascription.clone(),
            });
        }

        check_allowing_free_params(
            proof_term,
            &self.expected_type,
            self.ctx,
            self.identifier_factory,
        )
    }

    fn visit_sorry(&mut self) -> Result<ProofTree, CheckError> {
        let conclusion = match self.expected_type {
            Type::Prop(ref prop) => ProofTreeConclusion::PropIsTrue(prop.clone()),
            Type::Datatype(ref datatype) => {
                ProofTreeConclusion::TypeJudgement(Identifier::sorry(), datatype.clone())
            }
        };

        Ok(ProofTree {
            premisses: vec![],
            rule: ProofTreeRule::Sorry,
            conclusion,
        })
    }
}
