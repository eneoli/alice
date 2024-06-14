use crate::kernel::{
    process::ProofPipelineStage,
    proof::Proof,
    proof_term::{ProofTerm, Type},
    prop::Prop,
};

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
            Prop::Any => false,
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

    fn resolve_datatypes(&self, proof_term: ProofTerm, datatypes: &Vec<String>) -> ProofTerm {
        match proof_term {
            ProofTerm::Unit => ProofTerm::Unit,
            ProofTerm::Ident(ident) => ProofTerm::Ident(ident),
            ProofTerm::Pair(fst, snd) => ProofTerm::Pair(
                self.resolve_datatypes(*fst, datatypes).boxed(),
                self.resolve_datatypes(*snd, datatypes).boxed(),
            ),
            ProofTerm::Abort(body) => {
                ProofTerm::Abort(self.resolve_datatypes(*body, datatypes).boxed())
            }
            ProofTerm::Application {
                function,
                applicant,
            } => ProofTerm::Application {
                function: self.resolve_datatypes(*function, datatypes).boxed(),
                applicant: self.resolve_datatypes(*applicant, datatypes).boxed(),
            },
            ProofTerm::Case {
                proof_term,
                left_ident,
                left_term,
                right_ident,
                right_term,
            } => ProofTerm::Case {
                proof_term: self.resolve_datatypes(*proof_term, datatypes).boxed(),
                left_ident,
                left_term: self.resolve_datatypes(*left_term, datatypes).boxed(),
                right_ident,
                right_term: self.resolve_datatypes(*right_term, datatypes).boxed(),
            },
            ProofTerm::Function {
                param_ident,
                param_type,
                body,
            } => {
                let real_param_type = match param_type {
                    Type::Prop(Prop::Atom(ident, params))
                        if params.is_empty() && datatypes.contains(&ident) =>
                    {
                        Type::Datatype(ident.clone())
                    }
                    Type::Prop(prop) if self.has_datatype_identifier(&prop, &datatypes) => {
                        panic!("Props are not allowed to contain datatype identifers")
                    }
                    _ => param_type,
                };

                ProofTerm::Function {
                    param_ident,
                    param_type: real_param_type,
                    body: self.resolve_datatypes(*body, datatypes).boxed(),
                }
            }

            ProofTerm::LetIn {
                fst_ident,
                snd_ident,
                pair_proof_term,
                body,
            } => ProofTerm::LetIn {
                fst_ident,
                snd_ident,
                pair_proof_term: self.resolve_datatypes(*pair_proof_term, datatypes).boxed(),
                body: self.resolve_datatypes(*body, datatypes).boxed(),
            },

            ProofTerm::OrLeft { body, other } => ProofTerm::OrLeft {
                body: self.resolve_datatypes(*body, datatypes).boxed(),
                other,
            },
            ProofTerm::OrRight { body, other } => ProofTerm::OrRight {
                body: self.resolve_datatypes(*body, datatypes).boxed(),
                other,
            },
            ProofTerm::ProjectFst(body) => {
                ProofTerm::ProjectFst(self.resolve_datatypes(*body, datatypes).boxed())
            }
            ProofTerm::ProjectSnd(body) => {
                ProofTerm::ProjectSnd(self.resolve_datatypes(*body, datatypes).boxed())
            }
        }
    }
}

impl ProofPipelineStage for ResolveDatatypes {
    fn process(&self, proof: Proof) -> Proof {
        let Proof {
            proof_term,
            datatypes,
        } = proof;

        let new_proof_term = self.resolve_datatypes(proof_term, &datatypes);

        Proof {
            proof_term: new_proof_term,
            datatypes,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::kernel::{
        proof_term::{ProofTerm, Type},
        prop::Prop,
    };

    use super::ResolveDatatypes;

    #[test]
    fn test_simple_resolve() {
        let resolver = ResolveDatatypes::new();

        let mut proof_term = ProofTerm::Function {
            param_ident: "u".to_string(),
            param_type: Type::Prop(Prop::Atom("nat".to_string(), vec![])),
            body: ProofTerm::Unit.boxed(),
        };

        proof_term = resolver.resolve_datatypes(proof_term, &vec!["nat".to_string()]);

        assert_eq!(
            proof_term,
            ProofTerm::Function {
                param_ident: "u".to_string(),
                param_type: Type::Datatype("nat".to_string()),
                body: ProofTerm::Unit.boxed(),
            }
        )
    }

    #[test]
    fn test_simple_resolve_none() {
        let resolver = ResolveDatatypes::new();

        let mut proof_term = ProofTerm::Function {
            param_ident: "u".to_string(),
            param_type: Type::Prop(Prop::Atom("nat".to_string(), vec![])),
            body: ProofTerm::Unit.boxed(),
        };

        proof_term = resolver.resolve_datatypes(proof_term, &vec![]);

        assert_eq!(
            proof_term,
            ProofTerm::Function {
                param_ident: "u".to_string(),
                param_type: Type::Prop(Prop::Atom("nat".to_string(), vec![])),
                body: ProofTerm::Unit.boxed(),
            }
        )
    }

    #[test]
    fn test_resolve_nested() {
        let resolver = ResolveDatatypes::new();

        let mut proof_term = ProofTerm::Function {
            param_ident: "u".to_string(),
            param_type: Type::Prop(Prop::Atom("nat".to_string(), vec![])),
            body: ProofTerm::Function {
                param_ident: "v".to_string(),
                param_type: Type::Prop(Prop::Atom("list".to_string(), vec![])),
                body: ProofTerm::Pair(
                    ProofTerm::Function {
                        param_ident: "w".to_string(),
                        param_type: Type::Prop(Prop::Atom("A".to_string(), vec![])),
                        body: ProofTerm::Unit.boxed(),
                    }
                    .boxed(),
                    ProofTerm::Function {
                        param_ident: "x".to_string(),
                        param_type: Type::Prop(Prop::Atom("t".to_string(), vec![])),
                        body: ProofTerm::Unit.boxed(),
                    }
                    .boxed(),
                )
                .boxed(),
            }
            .boxed(),
        };

        proof_term = resolver.resolve_datatypes(
            proof_term,
            &vec!["nat".to_string(), "list".to_string(), "t".to_string()],
        );

        assert_eq!(
            proof_term,
            ProofTerm::Function {
                param_ident: "u".to_string(),
                param_type: Type::Datatype("nat".to_string()),
                body: ProofTerm::Function {
                    param_ident: "v".to_string(),
                    param_type: Type::Datatype("list".to_string()),
                    body: ProofTerm::Pair(
                        ProofTerm::Function {
                            param_ident: "w".to_string(),
                            param_type: Type::Prop(Prop::Atom("A".to_string(), vec![])),
                            body: ProofTerm::Unit.boxed(),
                        }
                        .boxed(),
                        ProofTerm::Function {
                            param_ident: "x".to_string(),
                            param_type: Type::Datatype("t".to_string()),
                            body: ProofTerm::Unit.boxed(),
                        }
                        .boxed(),
                    )
                    .boxed(),
                }
                .boxed(),
            }
        )
    }

    #[test]
    #[should_panic]
    fn test_no_nested_datatypes() {
        let resolver = ResolveDatatypes::new();

        let proof_term = ProofTerm::Function {
            param_ident: "u".to_string(),
            param_type: Type::Prop(Prop::And(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("nat".to_string(), vec![]).boxed(),
            )),
            body: ProofTerm::Unit.boxed(),
        };

        resolver.resolve_datatypes(proof_term, &vec!["nat".to_string()]);
    }

    #[test]
    #[should_panic]
    fn test_no_datatypes_with_params() {
        let resolver = ResolveDatatypes::new();

        let proof_term = ProofTerm::Function {
            param_ident: "u".to_string(),
            param_type: Type::Prop(Prop::Atom(
                "nat".to_string(),
                vec!["x".to_string(), "y".to_string()],
            )),
            body: ProofTerm::Unit.boxed(),
        };

        resolver.resolve_datatypes(proof_term, &vec!["nat".to_string()]);
    }
}
