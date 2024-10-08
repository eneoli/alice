use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tsify_next::Tsify;

use crate::kernel::{
    process::{ProofPipelineStage, StageError},
    proof::{Proof, ProofProcessingState},
    proof_term::{
        Abort, Application, Case, Function, LetIn, OrLeft, OrRight, Pair, ProjectFst, ProjectSnd,
        ProofTerm, Type, TypeAscription,
    },
    prop::Prop,
};

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ResolveDatatypesStageError {
    #[error("Proposition contains a datatype identifier")]
    PropContainsDatatypeIdentifier { prop: Prop, datatype: String },

    #[error("Atom \"{0}\" is unknown")]
    AtomUnknown(String),

    #[error("Datatype \"{0}\" is unknown")]
    DatatypeUnknown(String),

    #[error("Identifier \"{0}\" is defined multiple times")]
    DuplicateIdentifier(String),

    #[error("Arity of {ident} wrong: expected {expected}, actual {actual}")]
    ArityWrong {
        ident: String,
        expected: usize,
        actual: usize,
    },
}

pub struct ResolveDatatypes {}

impl Default for ResolveDatatypes {
    fn default() -> Self {
        Self::new()
    }
}

impl ResolveDatatypes {
    pub fn new() -> Self {
        Self {}
    }

    pub fn boxed() -> Box<Self> {
        Box::new(Self {})
    }
}

impl ProofPipelineStage for ResolveDatatypes {
    fn expected_processing_states(&self) -> Vec<ProofProcessingState> {
        vec![ProofProcessingState::Parsed]
    }

    fn process(&self, proof: Proof, prop: &Prop) -> Result<Proof, StageError> {
        let Proof {
            proof_term,
            atoms,
            datatypes,
            ..
        } = proof;

        // check for duplicates
        let atom_names: Vec<&String> = atoms.iter().map(|(name, _)| name).collect(); // collect as ref
        let datatype_names: Vec<&String> = datatypes.iter().map(|datatype| datatype).collect(); // collect as ref

        let mut seen_names = vec![];
        for name in [&atom_names[..], &datatype_names[..]].concat() {
            if seen_names.contains(&name) {
                return Err(StageError::ResolveDatatypesStageError(
                    ResolveDatatypesStageError::DuplicateIdentifier(name.clone()),
                ));
            }

            seen_names.push(name);
        }

        // check if Atom/Datatype decl from Prop is missing
        let prop_atoms = prop.get_atoms();
        let prop_datatypes = prop.get_datatypes();

        for (prop_atom_name, prop_atom_arity) in prop_atoms {
            if !atom_names.contains(&&prop_atom_name) {
                return Err(StageError::ResolveDatatypesStageError(
                    ResolveDatatypesStageError::AtomUnknown(prop_atom_name),
                ));
            }

            let mismatch_atom = atoms
                .iter()
                .filter(|(atom_name, _)| *atom_name == prop_atom_name)
                .find(|(_, arity)| *arity != prop_atom_arity);

            if let Some((_, actual_arity)) = mismatch_atom {
                return Err(StageError::ResolveDatatypesStageError(
                    ResolveDatatypesStageError::ArityWrong {
                        ident: prop_atom_name,
                        expected: prop_atom_arity,
                        actual: *actual_arity,
                    },
                ));
            }
        }

        for prop_datatype in prop_datatypes {
            if !datatypes.contains(&prop_datatype) {
                return Err(StageError::ResolveDatatypesStageError(
                    ResolveDatatypesStageError::DatatypeUnknown(prop_datatype),
                ));
            }
        }

        let atom_map = HashMap::from_iter(atoms.clone());

        let new_proof_term = resolve_datatypes(proof_term, &atom_map, &datatypes)
            .map_err(StageError::ResolveDatatypesStageError)?;

        Ok(Proof {
            processing_state: ProofProcessingState::TypesResolved,
            proof_term: new_proof_term,
            atoms,
            datatypes,
        })
    }
}

fn get_datatype_identifier(prop: &Prop, datatypes: &Vec<String>) -> Option<String> {
    match prop {
        Prop::Atom(ident, _) => {
            if datatypes.contains(ident) {
                Some(ident.clone())
            } else {
                None
            }
        }
        Prop::And(fst, snd) | Prop::Or(fst, snd) | Prop::Impl(fst, snd) => {
            if let Some(identifier) = get_datatype_identifier(fst, datatypes) {
                return Some(identifier);
            }

            get_datatype_identifier(snd, datatypes)
        }
        Prop::Exists { body, .. } | Prop::ForAll { body, .. } => {
            get_datatype_identifier(body, datatypes)
        }
        Prop::False => None,
        Prop::True => None,
    }
}

fn resolve_datatypes(
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
        Type::Prop(prop) if get_datatype_identifier(&prop, datatypes).is_some() => {
            Err(ResolveDatatypesStageError::PropContainsDatatypeIdentifier {
                prop: prop.clone(),
                datatype: get_datatype_identifier(&prop, datatypes).unwrap(),
            })
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
        ProofTerm::Unit(span) => ProofTerm::Unit(span),
        ProofTerm::Ident(ident) => ProofTerm::Ident(ident),
        ProofTerm::Pair(Pair(fst, snd, span)) => ProofTerm::Pair(Pair(
            resolve_datatypes(*fst, atoms, datatypes)?.boxed(),
            resolve_datatypes(*snd, atoms, datatypes)?.boxed(),
            span,
        )),
        ProofTerm::Abort(Abort(body, span)) => ProofTerm::Abort(Abort(
            resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            span,
        )),
        ProofTerm::Application(Application {
            function,
            applicant,
            span,
        }) => ProofTerm::Application(Application {
            function: resolve_datatypes(*function, atoms, datatypes)?.boxed(),
            applicant: resolve_datatypes(*applicant, atoms, datatypes)?.boxed(),
            span: span.clone(),
        }),
        ProofTerm::Case(Case {
            head,
            fst_ident: left_ident,
            fst_term: left_term,
            snd_ident: right_ident,
            snd_term: right_term,
            span,
        }) => ProofTerm::Case(Case {
            head: resolve_datatypes(*head, atoms, datatypes)?.boxed(),
            fst_ident: left_ident,
            fst_term: resolve_datatypes(*left_term, atoms, datatypes)?.boxed(),
            snd_ident: right_ident,
            snd_term: resolve_datatypes(*right_term, atoms, datatypes)?.boxed(),
            span,
        }),
        ProofTerm::Function(Function {
            param_ident,
            param_type: None,
            body,
            span,
        }) => ProofTerm::Function(Function {
            param_ident,
            param_type: None,
            body: resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            span,
        }),
        ProofTerm::Function(Function {
            param_ident,
            param_type: Some(param_type),
            body,
            span,
        }) => ProofTerm::Function(Function {
            param_ident,
            param_type: Some(get_real_type(param_type)?),
            body: resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            span,
        }),

        ProofTerm::LetIn(LetIn {
            fst_ident,
            snd_ident,
            head,
            body,
            span,
        }) => ProofTerm::LetIn(LetIn {
            fst_ident,
            snd_ident,
            head: resolve_datatypes(*head, atoms, datatypes)?.boxed(),
            body: resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            span,
        }),

        ProofTerm::OrLeft(OrLeft(body, span)) => ProofTerm::OrLeft(OrLeft(
            resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            span,
        )),
        ProofTerm::OrRight(OrRight(body, span)) => ProofTerm::OrRight(OrRight(
            resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            span,
        )),
        ProofTerm::ProjectFst(ProjectFst(body, span)) => ProofTerm::ProjectFst(ProjectFst(
            resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            span,
        )),
        ProofTerm::ProjectSnd(ProjectSnd(body, span)) => ProofTerm::ProjectSnd(ProjectSnd(
            resolve_datatypes(*body, atoms, datatypes)?.boxed(),
            span,
        )),
        ProofTerm::TypeAscription(TypeAscription {
            ascription,
            proof_term,
            span,
        }) => ProofTerm::TypeAscription(TypeAscription {
            ascription: get_real_type(ascription)?,
            proof_term: resolve_datatypes(*proof_term, atoms, datatypes)?.boxed(),
            span,
        }),
        ProofTerm::Sorry(span) => ProofTerm::Sorry(span),
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::kernel::{
        process::stages::resolve_datatypes::resolve_datatypes,
        proof_term::{Function, Pair, ProofTerm, Type},
        prop::{Prop, PropParameter},
    };

    #[test]
    fn test_simple_resolve() {
        let mut proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::Atom("nat".to_string(), vec![]))),
            body: ProofTerm::Unit(None).boxed(),
            span: None,
        });

        proof_term =
            resolve_datatypes(proof_term, &HashMap::new(), &vec!["nat".to_string()]).unwrap();

        assert_eq!(
            proof_term,
            ProofTerm::Function(Function {
                param_ident: "u".to_string(),
                param_type: Some(Type::Datatype("nat".to_string())),
                body: ProofTerm::Unit(None).boxed(),
                span: None,
            })
        )
    }

    #[test]
    fn test_resolve_nested() {
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
                        body: ProofTerm::Unit(None).boxed(),
                        span: None,
                    })
                    .boxed(),
                    ProofTerm::Function(Function {
                        param_ident: "x".to_string(),
                        param_type: Some(Type::Prop(Prop::Atom("t".to_string(), vec![]))),
                        body: ProofTerm::Unit(None).boxed(),
                        span: None,
                    })
                    .boxed(),
                    None,
                ))
                .boxed(),
                span: None,
            })
            .boxed(),
            span: None,
        });

        proof_term = resolve_datatypes(
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
                            body: ProofTerm::Unit(None).boxed(),
                            span: None,
                        })
                        .boxed(),
                        ProofTerm::Function(Function {
                            param_ident: "x".to_string(),
                            param_type: Some(Type::Datatype("t".to_string())),
                            body: ProofTerm::Unit(None).boxed(),
                            span: None,
                        })
                        .boxed(),
                        None,
                    ))
                    .boxed(),
                    span: None,
                })
                .boxed(),
                span: None,
            })
        )
    }

    #[test]
    #[should_panic]
    fn test_no_nested_datatypes() {
        let proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::And(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("nat".to_string(), vec![]).boxed(),
            ))),
            body: ProofTerm::Unit(None).boxed(),
            span: None,
        });

        resolve_datatypes(
            proof_term,
            &HashMap::from([("A".to_string(), 0)]),
            &vec!["nat".to_string()],
        )
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn test_no_datatypes_with_params() {
        let proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::Atom(
                "nat".to_string(),
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Uninstantiated("y".to_string()),
                ],
            ))),
            body: ProofTerm::Unit(None).boxed(),
            span: None,
        });

        resolve_datatypes(proof_term, &HashMap::new(), &vec!["nat".to_string()]).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_arity() {
        let proof_term = ProofTerm::Function(Function {
            param_ident: "u".to_string(),
            param_type: Some(Type::Prop(Prop::Atom(
                "A".to_string(),
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Uninstantiated("y".to_string()),
                ],
            ))),
            body: ProofTerm::Unit(None).boxed(),
            span: None,
        });

        resolve_datatypes(proof_term, &HashMap::from([("A".to_string(), 3)]), &vec![]).unwrap();
    }
}
