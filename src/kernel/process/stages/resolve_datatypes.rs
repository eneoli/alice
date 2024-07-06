use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::kernel::{
    process::{ProofPipelineStage, StageError},
    proof::{Proof, ProofProcessingState},
    proof_term::{
        Abort, Application, Case, Function, LetIn, OrLeft, OrRight, Pair, ProjectFst, ProjectSnd,
        ProofTerm, Type, TypeAscription,
    },
    prop::Prop,
};

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ResolveDatatypesStageError {
    #[error("Proposition contains a datatype identifier")]
    PropContainsDatatypeIdentifier,

    #[error("Identifier \"{0}\" is unknown")]
    AtomUnknown(String),

    #[error("Arity of {ident} wrong: expected {expected}, actual {actual}")]
    ArityWrong {
        ident: String,
        expected: usize,
        actual: usize,
    },
}

pub struct ResolveDatatypes {}

impl ResolveDatatypes {
    pub fn new() -> Self {
        Self {}
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self {})
    }

    fn has_datatype_identifier(&self, prop: &Prop, datatypes: &Vec<String>) -> bool {
        match prop {
            Prop::Atom(ident, _) => datatypes.contains(&ident),
            Prop::And(fst, snd) => {
                self.has_datatype_identifier(fst, datatypes)
                    || self.has_datatype_identifier(snd, datatypes)
            }
            Prop::Or(fst, snd) => {
                self.has_datatype_identifier(fst, datatypes)
                    || self.has_datatype_identifier(snd, datatypes)
            }
            Prop::False => false,
            Prop::True => false,
            Prop::Impl(fst, snd) => {
                self.has_datatype_identifier(fst, datatypes)
                    || self.has_datatype_identifier(snd, datatypes)
            }
            Prop::Exists { body, .. } => self.has_datatype_identifier(body, datatypes),
            Prop::ForAll { body, .. } => self.has_datatype_identifier(body, datatypes),
        }
    }

    fn resolve_datatypes(
        &self,
        proof_term: ProofTerm,
        atoms: &HashMap<String, usize>,
        datatypes: &Vec<String>,
    ) -> Result<ProofTerm, ResolveDatatypesStageError> {
        let get_real_type = |_type: Type| match _type {
            // Atom that is in fact a datatype
            Type::Prop(Prop::Atom(ident, params))
                if params.is_empty() && datatypes.contains(&ident) =>
            {
                Ok(Type::Datatype(ident.clone()))
            }

            // Prop that includes datatype
            Type::Prop(prop) if self.has_datatype_identifier(&prop, &datatypes) => {
                return Err(ResolveDatatypesStageError::PropContainsDatatypeIdentifier);
            }

            // Actual Atom
            Type::Prop(Prop::Atom(ref ident, ref params)) => {
                if let Some(expected_arity) = atoms.get(ident) {
                    if *expected_arity != params.len() {
                        return Err(ResolveDatatypesStageError::ArityWrong {
                            ident: ident.clone(),
                            expected: *expected_arity,
                            actual: params.len(),
                        });
                    }
                } else {
                    return Err(ResolveDatatypesStageError::AtomUnknown(ident.clone()));
                }

                Ok(_type)
            }
            _ => Ok(_type),
        };

        let result = match proof_term {
            ProofTerm::Unit => ProofTerm::Unit,
            ProofTerm::Ident(ident) => ProofTerm::Ident(ident),
            ProofTerm::Pair(Pair(fst, snd)) => ProofTerm::Pair(Pair(
                self.resolve_datatypes(*fst, atoms, datatypes)?.boxed(),
                self.resolve_datatypes(*snd, atoms, datatypes)?.boxed(),
            )),
            ProofTerm::Abort(Abort(body)) => ProofTerm::Abort(Abort(
                self.resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            )),
            ProofTerm::Application(Application {
                function,
                applicant,
            }) => ProofTerm::Application(Application {
                function: self.resolve_datatypes(*function, atoms, datatypes)?.boxed(),
                applicant: self
                    .resolve_datatypes(*applicant, atoms, datatypes)?
                    .boxed(),
            }),
            ProofTerm::Case(Case {
                head,
                fst_ident: left_ident,
                fst_term: left_term,
                snd_ident: right_ident,
                snd_term: right_term,
            }) => ProofTerm::Case(Case {
                head: self.resolve_datatypes(*head, atoms, datatypes)?.boxed(),
                fst_ident: left_ident,
                fst_term: self
                    .resolve_datatypes(*left_term, atoms, datatypes)?
                    .boxed(),
                snd_ident: right_ident,
                snd_term: self
                    .resolve_datatypes(*right_term, atoms, datatypes)?
                    .boxed(),
            }),
            ProofTerm::Function(Function {
                param_ident,
                param_type: None,
                body,
            }) => ProofTerm::Function(Function {
                param_ident,
                param_type: None,
                body: self.resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            }),
            ProofTerm::Function(Function {
                param_ident,
                param_type: Some(param_type),
                body,
            }) => ProofTerm::Function(Function {
                param_ident,
                param_type: Some(get_real_type(param_type)?),
                body: self.resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            }),

            ProofTerm::LetIn(LetIn {
                fst_ident,
                snd_ident,
                head,
                body,
            }) => ProofTerm::LetIn(LetIn {
                fst_ident,
                snd_ident,
                head: self.resolve_datatypes(*head, atoms, datatypes)?.boxed(),
                body: self.resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            }),

            ProofTerm::OrLeft(OrLeft(body)) => ProofTerm::OrLeft(OrLeft(
                self.resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            )),
            ProofTerm::OrRight(OrRight(body)) => ProofTerm::OrRight(OrRight(
                self.resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            )),
            ProofTerm::ProjectFst(ProjectFst(body)) => ProofTerm::ProjectFst(ProjectFst(
                self.resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            )),
            ProofTerm::ProjectSnd(ProjectSnd(body)) => ProofTerm::ProjectSnd(ProjectSnd(
                self.resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            )),
            ProofTerm::TypeAscription(TypeAscription {
                ascription,
                proof_term,
            }) => ProofTerm::TypeAscription(TypeAscription {
                ascription: get_real_type(ascription)?,
                proof_term: self
                    .resolve_datatypes(*proof_term, atoms, datatypes)?
                    .boxed(),
            }),
        };

        Ok(result)
    }
}

impl ProofPipelineStage for ResolveDatatypes {
    fn expected_processing_states(&self) -> Vec<ProofProcessingState> {
        vec![ProofProcessingState::Parsed]
    }

    fn process(&self, proof: Proof) -> Result<Proof, StageError> {
        let Proof {
            proof_term,
            atoms,
            datatypes,
            ..
        } = proof;

        let atom_map = HashMap::from_iter(atoms.clone().into_iter());

        let new_proof_term = self
            .resolve_datatypes(proof_term, &atom_map, &datatypes)
            .map_err(StageError::ResolveDatatypesStageError)?;

        Ok(Proof {
            processing_state: ProofProcessingState::TypesResolved,
            proof_term: new_proof_term,
            atoms,
            datatypes,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::kernel::{
        proof_term::{Function, Pair, ProofTerm, Type},
        prop::{Prop, PropParameter},
    };

    use super::ResolveDatatypes;

    #[test]
    fn test_simple_resolve() {
        let resolver = ResolveDatatypes::new();

        let mut proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::Atom("nat".to_string(), vec![]))),
            body: ProofTerm::Unit.boxed(),
        });

        proof_term = resolver
            .resolve_datatypes(proof_term, &HashMap::new(), &vec!["nat".to_string()])
            .unwrap();

        assert_eq!(
            proof_term,
            ProofTerm::Function(Function {
                param_ident: "u".to_string(),
                param_type: Some(Type::Datatype("nat".to_string())),
                body: ProofTerm::Unit.boxed(),
            })
        )
    }

    #[test]
    fn test_resolve_nested() {
        let resolver = ResolveDatatypes::new();

        let mut proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::Atom("nat".to_string(), vec![]))),
            body: ProofTerm::Function(Function {
                param_ident: "v".to_string(),
                param_type: Some(Type::Prop(Prop::Atom("list".to_string(), vec![]))),
                body: ProofTerm::Pair(Pair(
                    ProofTerm::Function(Function {
                        param_ident: "w".to_string(),
                        param_type: Some(Type::Prop(Prop::Atom("A".to_string(), vec![]))),
                        body: ProofTerm::Unit.boxed(),
                    })
                    .boxed(),
                    ProofTerm::Function(Function {
                        param_ident: "x".to_string(),
                        param_type: Some(Type::Prop(Prop::Atom("t".to_string(), vec![]))),
                        body: ProofTerm::Unit.boxed(),
                    })
                    .boxed(),
                ))
                .boxed(),
            })
            .boxed(),
        });

        proof_term = resolver
            .resolve_datatypes(
                proof_term,
                &HashMap::from([("A".to_string(), 0)]),
                &vec!["nat".to_string(), "list".to_string(), "t".to_string()],
            )
            .unwrap();

        assert_eq!(
            proof_term,
            ProofTerm::Function(Function {
                param_ident: "u".to_string(),
                param_type: Some(Type::Datatype("nat".to_string())),
                body: ProofTerm::Function(Function {
                    param_ident: "v".to_string(),
                    param_type: Some(Type::Datatype("list".to_string())),
                    body: ProofTerm::Pair(Pair(
                        ProofTerm::Function(Function {
                            param_ident: "w".to_string(),
                            param_type: Some(Type::Prop(Prop::Atom("A".to_string(), vec![]))),
                            body: ProofTerm::Unit.boxed(),
                        })
                        .boxed(),
                        ProofTerm::Function(Function {
                            param_ident: "x".to_string(),
                            param_type: Some(Type::Datatype("t".to_string())),
                            body: ProofTerm::Unit.boxed(),
                        })
                        .boxed(),
                    ))
                    .boxed(),
                })
                .boxed(),
            })
        )
    }

    #[test]
    #[should_panic]
    fn test_no_nested_datatypes() {
        let resolver = ResolveDatatypes::new();

        let proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::And(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("nat".to_string(), vec![]).boxed(),
            ))),
            body: ProofTerm::Unit.boxed(),
        });

        resolver
            .resolve_datatypes(
                proof_term,
                &HashMap::from([("A".to_string(), 0)]),
                &vec!["nat".to_string()],
            )
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_no_datatypes_with_params() {
        let resolver = ResolveDatatypes::new();

        let proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::Atom(
                "nat".to_string(),
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Uninstantiated("y".to_string()),
                ],
            ))),
            body: ProofTerm::Unit.boxed(),
        });

        resolver
            .resolve_datatypes(proof_term, &HashMap::new(), &vec!["nat".to_string()])
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_arity() {
        let resolver = ResolveDatatypes::new();

        let proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::Atom(
                "A".to_string(),
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Uninstantiated("y".to_string()),
                ],
            ))),
            body: ProofTerm::Unit.boxed(),
        });

        resolver
            .resolve_datatypes(proof_term, &HashMap::from([("A".to_string(), 3)]), &vec![])
            .unwrap();
    }
}
