use identifier_context::IdentifierContext;

use super::{
    proof_term::{ProofTerm, ProofTermKind, ProofTermVisitor, Type},
    proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
    prop::Prop,
};

pub mod identifier_context;

// TODO: Schon erledigt? checken das der Typ einer quantifizierten Variable auch ein Datentyp und kein Prop ist.
// TODO: Schon erledigt? checken das paramitriserte Atomns A(...) nur identifier haben die auch eingeführt wurden. (besonders (aber nicht nur) bei Exists)

#[derive(Debug, PartialEq, Eq)]
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

struct TypifyVisitor {
    ctx: IdentifierContext,
}

impl TypifyVisitor {
    pub fn new() -> Self {
        Self {
            ctx: IdentifierContext::new(),
        }
    }
}

impl ProofTermVisitor<Result<(Type, ProofTree), TypeError>> for TypifyVisitor {
    fn visit_ident(&mut self, ident: &String) -> Result<(Type, ProofTree), TypeError> {
        let ident_type = self.ctx.get(&ident);

        if let Some(Type::Prop(_type)) = ident_type {
            Ok((
                Type::Prop(_type.clone()),
                ProofTree {
                    premisses: vec![],
                    rule: ProofTreeRule::Ident(Some(ident.clone())),
                    conclusion: ProofTreeConclusion::PropIsTrue(_type.clone()),
                },
            ))
        } else if let Some(Type::Datatype(_type)) = ident_type {
            Ok((
                Type::Datatype(_type.clone()),
                ProofTree {
                    premisses: vec![],
                    rule: ProofTreeRule::Ident(Some(ident.clone())),
                    conclusion: ProofTreeConclusion::TypeJudgement(ident.clone(), _type.clone()),
                },
            ))
        } else {
            Err(TypeError::UnknownIdent {
                ident: ident.to_string(),
            })
        }
    }

    fn visit_pair(
        &mut self,
        fst: &ProofTerm,
        snd: &ProofTerm,
    ) -> Result<(Type, ProofTree), TypeError> {
        let (fst_type, fst_proof_tree) = fst.visit(self)?;
        let (snd_type, snd_proof_tree) = snd.visit(self)?;

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
            (_, Type::Datatype(_)) => Err(TypeError::ExpectedProp {
                received: snd_type.clone(),
            }),
        }
    }

    fn visit_project_fst(&mut self, body: &ProofTerm) -> Result<(Type, ProofTree), TypeError> {
        let (body_type, body_proof_tree) = body.visit(self)?;

        if let Type::Prop(Prop::And(fst, _)) = body_type {
            return Ok((
                Type::Prop(*fst.clone()),
                ProofTree {
                    premisses: vec![body_proof_tree],
                    rule: ProofTreeRule::AndElimFst,
                    conclusion: ProofTreeConclusion::PropIsTrue(*fst),
                },
            ));
        }

        Err(TypeError::UnexpectedKind {
            expected: ProofTermKind::Pair,
            received: body_type,
        })
    }

    fn visit_project_snd(&mut self, body: &ProofTerm) -> Result<(Type, ProofTree), TypeError> {
        let (body_type, body_proof_tree) = body.visit(self)?;

        if let Type::Prop(Prop::And(_, snd)) = body_type {
            return Ok((
                Type::Prop(*snd.clone()),
                ProofTree {
                    premisses: vec![body_proof_tree],
                    rule: ProofTreeRule::AndElimSnd,
                    conclusion: ProofTreeConclusion::PropIsTrue(*snd),
                },
            ));
        }

        Err(TypeError::UnexpectedKind {
            expected: ProofTermKind::Pair,
            received: body_type,
        })
    }

    fn visit_function(
        &mut self,
        param_ident: &String,
        param_type: &Type,
        body: &ProofTerm,
    ) -> Result<(Type, ProofTree), TypeError> {
        self.ctx.insert(param_ident.clone(), param_type.clone());
        let (body_type, body_proof_tree) = body.visit(self)?;
        self.ctx.remove(&param_ident);

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
            (_, Type::Datatype(_)) => Err(TypeError::ExpectedProp {
                received: body_type,
            }),
        }
        .into()
    }

    fn visit_application(
        &mut self,
        function: &ProofTerm,
        applicant: &ProofTerm,
    ) -> Result<(Type, ProofTree), TypeError> {
        let (function_type, function_proof_tree) = function.visit(self)?;
        let (applicant_type, applicant_proof_tree) = applicant.visit(self)?;

        // either implication or allquant

        if let Type::Prop(Prop::Impl(param_prop, body_type)) = function_type {
            let param_type = Type::Prop(*param_prop);
            if param_type != applicant_type {
                return Err(TypeError::UnexpectedType {
                    expected: param_type,
                    received: applicant_type,
                });
            }

            return Ok((
                Type::Prop(*body_type.clone()),
                ProofTree {
                    premisses: vec![function_proof_tree, applicant_proof_tree],
                    rule: ProofTreeRule::ImplElim,
                    conclusion: ProofTreeConclusion::PropIsTrue(*body_type),
                },
            ));
        }

        if let Type::Prop(Prop::ForAll {
            object_ident,
            object_type_ident,
            body,
        }) = function_type
        {
            // check if applicant is datatype
            let object_type = Type::Datatype(object_type_ident);
            if applicant_type == object_type {
                if let ProofTerm::Ident(ident) = applicant {
                    let mut substitued_body = *body.clone();
                    substitued_body.substitue_free_parameter(&object_ident, &ident);

                    return Ok((
                        Type::Prop(substitued_body.clone()),
                        ProofTree {
                            premisses: vec![function_proof_tree, applicant_proof_tree],
                            rule: ProofTreeRule::ForAllElim,
                            conclusion: ProofTreeConclusion::PropIsTrue(substitued_body),
                        },
                    ));
                } else {
                    panic!("Architecture error: Expected identifier. Are you implementing datatytpe functions?")
                }
            }

            return Err(TypeError::UnexpectedType {
                expected: object_type,
                received: applicant_type,
            });
        }

        Err(TypeError::UnexpectedKind {
            expected: ProofTermKind::Function,
            received: function_type,
        })
    }

    fn visit_let_in(
        &mut self,
        fst_ident: &String,
        snd_ident: &String,
        pair_proof_term: &ProofTerm,
        body: &ProofTerm,
    ) -> Result<(Type, ProofTree), TypeError> {
        let (pair_proof_term_type, pair_proof_term_tree) = pair_proof_term.visit(self)?;

        if let Type::Prop(Prop::Exists {
            object_ident,
            object_type_ident,
            body: mut exists_body,
        }) = pair_proof_term_type
        {
            exists_body.substitue_free_parameter(&object_ident, &fst_ident);
            self.ctx
                .insert(fst_ident.clone(), Type::Datatype(object_type_ident));
            self.ctx.insert(snd_ident.clone(), Type::Prop(*exists_body));

            let (body_type, body_proof_tree) = body.visit(self)?;

            self.ctx.remove(&fst_ident);
            self.ctx.remove(&snd_ident);

            // check that quantified object does not escape it's scope
            if let Type::Prop(prop) = &body_type {
                if prop.get_free_parameters().contains(&fst_ident) {
                    return Err(TypeError::QuantifiedObjectEscapesScope);
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
                Err(TypeError::ExpectedProp {
                    received: body_type,
                })
            }
        } else {
            Err(TypeError::UnexpectedKind {
                expected: ProofTermKind::ExistsPair,
                received: pair_proof_term_type,
            })
        }
    }

    fn visit_or_left(
        &mut self,
        body: &ProofTerm,
        other: &Prop,
    ) -> Result<(Type, ProofTree), TypeError> {
        let (body_type, body_proof_tree) = body.visit(self)?;

        if let Type::Prop(body_prop) = body_type {
            let _type = Prop::Or(body_prop.boxed(), other.boxed());

            Ok((
                _type.clone().into(),
                ProofTree {
                    premisses: vec![body_proof_tree],
                    rule: ProofTreeRule::OrIntroFst,
                    conclusion: ProofTreeConclusion::PropIsTrue(_type),
                },
            ))
        } else {
            Err(TypeError::ExpectedProp {
                received: body_type,
            })
        }
    }

    fn visit_or_right(
        &mut self,
        body: &ProofTerm,
        other: &Prop,
    ) -> Result<(Type, ProofTree), TypeError> {
        let (body_type, body_proof_tree) = body.visit(self)?;

        if let Type::Prop(body_prop) = body_type {
            let _type = Prop::Or(other.boxed(), body_prop.boxed());

            Ok((
                Type::Prop(_type.clone()),
                ProofTree {
                    premisses: vec![body_proof_tree],
                    rule: ProofTreeRule::OrIntroSnd,
                    conclusion: ProofTreeConclusion::PropIsTrue(_type),
                },
            ))
        } else {
            Err(TypeError::ExpectedProp {
                received: body_type,
            })
        }
    }

    fn visit_case(
        &mut self,
        proof_term: &ProofTerm,
        left_ident: &String,
        left_term: &ProofTerm,
        right_ident: &String,
        right_term: &ProofTerm,
    ) -> Result<(Type, ProofTree), TypeError> {
        let (proof_term_type, proof_term_tree) = proof_term.visit(self)?;

        if let Type::Prop(Prop::Or(fst, snd)) = proof_term_type {
            // fst
            self.ctx.insert(left_ident.clone(), Type::Prop(*fst));
            let (fst_type, fst_proof_tree) = left_term.visit(self)?;
            self.ctx.remove(&left_ident);

            // snd
            self.ctx.insert(right_ident.clone(), Type::Prop(*snd));
            let (snd_type, snd_proof_tree) = right_term.visit(self)?;
            self.ctx.remove(&right_ident);

            if fst_type != snd_type {
                return Err(TypeError::CaseArmsDifferent { fst_type, snd_type });
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
                Err(TypeError::ExpectedProp { received: fst_type })
            }
        } else {
            Err(TypeError::UnexpectedKind {
                expected: ProofTermKind::Pair,
                received: proof_term_type,
            })
        }
    }

    fn visit_abort(&mut self, body: &ProofTerm) -> Result<(Type, ProofTree), TypeError> {
        let (body_type, body_proof_tree) = body.visit(self)?;

        if let Type::Prop(Prop::False) = body_type {
            Ok((
                Prop::Any.into(),
                ProofTree {
                    premisses: vec![body_proof_tree],
                    rule: ProofTreeRule::FalsumElim,
                    conclusion: ProofTreeConclusion::PropIsTrue(Prop::Any),
                },
            ))
        } else {
            Err(TypeError::UnexpectedType {
                expected: Prop::False.into(),
                received: body_type,
            })
        }
    }

    fn visit_unit(&mut self) -> Result<(Type, ProofTree), TypeError> {
        Ok((
            Prop::True.into(),
            ProofTree {
                premisses: vec![],
                rule: ProofTreeRule::TrueIntro,
                conclusion: ProofTreeConclusion::PropIsTrue(Prop::True),
            },
        ))
    }
}

pub fn typify(proof_term: &ProofTerm) -> Result<(Type, ProofTree), TypeError> {
    let mut visitor = TypifyVisitor::new();
    proof_term.visit(&mut visitor)
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use std::vec;

    use chumsky::Parser;

    use crate::core::{
        check::{typify, TypeError},
        parse::{lexer::lexer, proof::proof_parser, proof_term::proof_term_parser},
        process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline},
        proof_term::Type,
        proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
        prop::Prop,
    };

    #[test]
    fn test_proof_implication_to_and() {
        let tokens = lexer().parse("fn u: A => fn w: B => (u, w)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, proof_tree) = typify(&ast).unwrap();

        let expected_type = Type::Prop(Prop::Impl(
            Prop::Atom("A".to_string(), vec![]).boxed(),
            Prop::Impl(
                Prop::Atom("B".to_string(), vec![]).boxed(),
                Prop::And(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                )
                .boxed(),
            )
            .boxed(),
        ));

        // check type
        assert_eq!(_type, expected_type);

        // check proof tree
        assert_eq!(
            proof_tree,
            ProofTree {
                premisses: vec![ProofTree {
                    premisses: vec![ProofTree {
                        premisses: vec![
                            ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident(Some("u".to_string())),
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                    "A".to_string(),
                                    vec![]
                                )),
                            },
                            ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident(Some("w".to_string())),
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                    "B".to_string(),
                                    vec![]
                                )),
                            }
                        ],
                        rule: ProofTreeRule::AndIntro,
                        conclusion: ProofTreeConclusion::PropIsTrue(Prop::And(
                            Prop::Atom("A".to_string(), vec![]).boxed(),
                            Prop::Atom("B".to_string(), vec![]).boxed(),
                        ))
                    }],
                    rule: ProofTreeRule::ImplIntro("w".to_string()),
                    conclusion: ProofTreeConclusion::PropIsTrue(Prop::Impl(
                        Prop::Atom("B".to_string(), vec![]).boxed(),
                        Prop::And(
                            Prop::Atom("A".to_string(), vec![]).boxed(),
                            Prop::Atom("B".to_string(), vec![]).boxed(),
                        )
                        .boxed(),
                    )),
                }],
                rule: ProofTreeRule::ImplIntro("u".to_string()),
                conclusion: ProofTreeConclusion::PropIsTrue(expected_type.into()),
            }
        )
    }

    #[test]
    fn test_commutativity_of_conjunction() {
        let tokens = lexer().parse("fn u: (A && B) => (snd u, fst u)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, proof_tree) = typify(&ast).unwrap();

        let expected_type = Prop::Impl(
            Prop::And(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("B".to_string(), vec![]).boxed(),
            )
            .boxed(),
            Prop::And(
                Prop::Atom("B".to_string(), vec![]).boxed(),
                Prop::Atom("A".to_string(), vec![]).boxed(),
            )
            .boxed(),
        );

        // test type
        assert_eq!(_type, Type::Prop(expected_type.clone()),);

        // test proof tree
        assert_eq!(
            proof_tree,
            ProofTree {
                premisses: vec![ProofTree {
                    premisses: vec![
                        ProofTree {
                            premisses: vec![ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident(Some("u".to_string())),
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::And(
                                    Prop::Atom("A".to_string(), vec![]).boxed(),
                                    Prop::Atom("B".to_string(), vec![]).boxed(),
                                ))
                            }],
                            rule: ProofTreeRule::AndElimSnd,
                            conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                "B".to_string(),
                                vec![]
                            )),
                        },
                        ProofTree {
                            premisses: vec![ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident(Some("u".to_string())),
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::And(
                                    Prop::Atom("A".to_string(), vec![]).boxed(),
                                    Prop::Atom("B".to_string(), vec![]).boxed(),
                                ))
                            }],
                            rule: ProofTreeRule::AndElimFst,
                            conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                "A".to_string(),
                                vec![]
                            )),
                        }
                    ],
                    rule: ProofTreeRule::AndIntro,
                    conclusion: ProofTreeConclusion::PropIsTrue(Prop::And(
                        Prop::Atom("B".to_string(), vec![]).boxed(),
                        Prop::Atom("A".to_string(), vec![]).boxed()
                    )),
                }],
                rule: ProofTreeRule::ImplIntro("u".to_string()),
                conclusion: ProofTreeConclusion::PropIsTrue(expected_type),
            }
        )
    }

    #[test]
    fn test_interaction_law_of_distributivity() {
        let tokens = lexer()
            .parse("fn u: (A -> (B & C)) => (fn w: A => fst (u w), fn w: A => snd (u w))")
            .unwrap();

        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, proof_tree) = typify(&ast).unwrap();

        let expected_type = Prop::Impl(
            Prop::Impl(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::And(
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                    Prop::Atom("C".to_string(), vec![]).boxed(),
                )
                .boxed(),
            )
            .boxed(),
            Prop::And(
                Prop::Impl(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                )
                .boxed(),
                Prop::Impl(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("C".to_string(), vec![]).boxed(),
                )
                .boxed(),
            )
            .boxed(),
        );

        // test type
        assert_eq!(_type, Type::Prop(expected_type.clone()));

        // test proof tree
        assert_eq!(
            proof_tree,
            ProofTree {
                premisses: vec![ProofTree {
                    premisses: vec![
                        ProofTree {
                            premisses: vec![ProofTree {
                                premisses: vec![ProofTree {
                                    premisses: vec![
                                        ProofTree {
                                            premisses: vec![],
                                            rule: ProofTreeRule::Ident(Some("u".to_string())),
                                            conclusion: ProofTreeConclusion::PropIsTrue(
                                                Prop::Impl(
                                                    Prop::Atom("A".to_string(), vec![]).boxed(),
                                                    Prop::And(
                                                        Prop::Atom("B".to_string(), vec![]).boxed(),
                                                        Prop::Atom("C".to_string(), vec![]).boxed(),
                                                    )
                                                    .boxed(),
                                                )
                                            )
                                        },
                                        ProofTree {
                                            premisses: vec![],
                                            rule: ProofTreeRule::Ident(Some("w".to_string())),
                                            conclusion: ProofTreeConclusion::PropIsTrue(
                                                Prop::Atom("A".to_string(), vec![])
                                            ),
                                        }
                                    ],
                                    rule: ProofTreeRule::ImplElim,
                                    conclusion: ProofTreeConclusion::PropIsTrue(Prop::And(
                                        Prop::Atom("B".to_string(), vec![]).boxed(),
                                        Prop::Atom("C".to_string(), vec![]).boxed(),
                                    )),
                                }],
                                rule: ProofTreeRule::AndElimFst,
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                    "B".to_string(),
                                    vec![]
                                )),
                            }],
                            rule: ProofTreeRule::ImplIntro("w".to_string()),
                            conclusion: ProofTreeConclusion::PropIsTrue(Prop::Impl(
                                Prop::Atom("A".to_string(), vec![]).boxed(),
                                Prop::Atom("B".to_string(), vec![]).boxed(),
                            ))
                        },
                        ProofTree {
                            premisses: vec![ProofTree {
                                premisses: vec![ProofTree {
                                    premisses: vec![
                                        ProofTree {
                                            premisses: vec![],
                                            rule: ProofTreeRule::Ident(Some("u".to_string())),
                                            conclusion: ProofTreeConclusion::PropIsTrue(
                                                Prop::Impl(
                                                    Prop::Atom("A".to_string(), vec![]).boxed(),
                                                    Prop::And(
                                                        Prop::Atom("B".to_string(), vec![]).boxed(),
                                                        Prop::Atom("C".to_string(), vec![]).boxed(),
                                                    )
                                                    .boxed(),
                                                )
                                            )
                                        },
                                        ProofTree {
                                            premisses: vec![],
                                            rule: ProofTreeRule::Ident(Some("w".to_string())),
                                            conclusion: ProofTreeConclusion::PropIsTrue(
                                                Prop::Atom("A".to_string(), vec![])
                                            ),
                                        }
                                    ],
                                    rule: ProofTreeRule::ImplElim,
                                    conclusion: ProofTreeConclusion::PropIsTrue(Prop::And(
                                        Prop::Atom("B".to_string(), vec![]).boxed(),
                                        Prop::Atom("C".to_string(), vec![]).boxed(),
                                    )),
                                }],
                                rule: ProofTreeRule::AndElimSnd,
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                    "C".to_string(),
                                    vec![]
                                )),
                            }],
                            rule: ProofTreeRule::ImplIntro("w".to_string()),
                            conclusion: ProofTreeConclusion::PropIsTrue(Prop::Impl(
                                Prop::Atom("A".to_string(), vec![]).boxed(),
                                Prop::Atom("C".to_string(), vec![]).boxed(),
                            ))
                        }
                    ],
                    rule: ProofTreeRule::AndIntro,
                    conclusion: ProofTreeConclusion::PropIsTrue(Prop::And(
                        Prop::Impl(
                            Prop::Atom("A".to_string(), vec![]).boxed(),
                            Prop::Atom("B".to_string(), vec![]).boxed(),
                        )
                        .boxed(),
                        Prop::Impl(
                            Prop::Atom("A".to_string(), vec![]).boxed(),
                            Prop::Atom("C".to_string(), vec![]).boxed(),
                        )
                        .boxed(),
                    ))
                }],
                rule: ProofTreeRule::ImplIntro("u".to_string()),
                conclusion: ProofTreeConclusion::PropIsTrue(expected_type),
            }
        )
    }

    #[test]
    fn test_commutativity_of_disjunction() {
        let tokens = lexer()
            .parse("fn u: A || B => case u of inl a => inr<B> a, inr b => inl<A> b")
            .unwrap();

        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, proof_tree) = typify(&ast).unwrap();

        let expected_type = Prop::Impl(
            Prop::Or(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("B".to_string(), vec![]).boxed(),
            )
            .boxed(),
            Prop::Or(
                Prop::Atom("B".to_string(), vec![]).boxed(),
                Prop::Atom("A".to_string(), vec![]).boxed(),
            )
            .boxed(),
        );

        // check type
        assert_eq!(_type, Type::Prop(expected_type.clone()),);

        // check proof tree
        assert_eq!(
            proof_tree,
            ProofTree {
                premisses: vec![ProofTree {
                    premisses: vec![
                        ProofTree {
                            premisses: vec![],
                            rule: ProofTreeRule::Ident(Some("u".to_string())),
                            conclusion: ProofTreeConclusion::PropIsTrue(Prop::Or(
                                Prop::Atom("A".to_string(), vec![]).boxed(),
                                Prop::Atom("B".to_string(), vec![]).boxed(),
                            )),
                        },
                        ProofTree {
                            premisses: vec![ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident(Some("a".to_string())),
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                    "A".to_string(),
                                    vec![]
                                )),
                            }],
                            rule: ProofTreeRule::OrIntroSnd,
                            conclusion: ProofTreeConclusion::PropIsTrue(Prop::Or(
                                Prop::Atom("B".to_string(), vec![]).boxed(),
                                Prop::Atom("A".to_string(), vec![]).boxed(),
                            ))
                        },
                        ProofTree {
                            premisses: vec![ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident(Some("b".to_string())),
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                    "B".to_string(),
                                    vec![]
                                )),
                            }],
                            rule: ProofTreeRule::OrIntroFst,
                            conclusion: ProofTreeConclusion::PropIsTrue(Prop::Or(
                                Prop::Atom("B".to_string(), vec![]).boxed(),
                                Prop::Atom("A".to_string(), vec![]).boxed(),
                            ))
                        }
                    ],
                    rule: ProofTreeRule::OrElim("a".to_string(), "b".to_string()),
                    conclusion: ProofTreeConclusion::PropIsTrue(Prop::Or(
                        Prop::Atom("B".to_string(), vec![]).boxed(),
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                    )),
                }],
                rule: ProofTreeRule::ImplIntro("u".to_string()),
                conclusion: ProofTreeConclusion::PropIsTrue(expected_type),
            }
        )
    }

    #[test]
    fn test_true() {
        let tokens = lexer().parse("()").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, proof_tree) = typify(&ast).unwrap();

        assert_eq!(_type, Type::Prop(Prop::True));

        assert_eq!(
            proof_tree,
            ProofTree {
                premisses: vec![],
                rule: ProofTreeRule::TrueIntro,
                conclusion: ProofTreeConclusion::PropIsTrue(Prop::True),
            }
        );
    }

    #[test]
    fn test_composition() {
        let tokens = lexer()
            .parse("fn u: ((A -> B) && (B -> C)) => fn w: A => (snd u) ((fst u) w)")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, _) = typify(&ast).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::And(
                    Prop::Impl(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("B".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                    Prop::Impl(
                        Prop::Atom("B".to_string(), vec![]).boxed(),
                        Prop::Atom("C".to_string(), vec![]).boxed()
                    )
                    .boxed()
                )
                .boxed(),
                Prop::Impl(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("C".to_string(), vec![]).boxed()
                )
                .boxed()
            ))
        );
    }

    #[test]
    fn test_composition_of_identities() {
        let tokens = lexer().parse("(fn u: ((A -> A) && (A -> A)) => fn w: A => (snd u) ((fst u) w)) ((fn x: A => x), (fn y: A => y))").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, _) = typify(&ast).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("A".to_string(), vec![]).boxed()
            ))
        )
    }

    #[test]
    fn test_non_minimal_identity_proof() {
        let tokens = lexer().parse("fn u: A => (fn w: A => w) u").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, _) = typify(&ast).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("A".to_string(), vec![]).boxed()
            ))
        )
    }

    #[test]
    fn test_projection_function() {
        let tokens = lexer().parse("fn u: A & B => fst u").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, _) = typify(&ast).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::And(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed()
                )
                .boxed(),
                Prop::Atom("A".to_string(), vec![]).boxed(),
            ))
        )
    }

    #[test]
    fn test_implication_chain() {
        let tokens = lexer()
            .parse("fn u: (A -> A) -> B => u (fn u: A => u)")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, _) = typify(&ast).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Impl(
                    Prop::Impl(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("A".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed()
                )
                .boxed(),
                Prop::Atom("B".to_string(), vec![]).boxed()
            ))
        )
    }

    #[test]
    fn test_piano_number_2() {
        let tokens = lexer().parse("fn z: A => fn s: A -> A => s(s(z))").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, _) = typify(&ast).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Impl(
                    Prop::Impl(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("A".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                    Prop::Atom("A".to_string(), vec![]).boxed()
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_tripple_neagation_elimination() {
        // ~~~A = ((A => False) => False) => False
        let tokens = lexer()
            .parse("fn u: (~~~A) => fn v: A => u (fn w: A -> \\bot => w v)")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let (_type, _) = typify(&ast).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Impl(
                    Prop::Impl(
                        Prop::Impl(
                            Prop::Atom("A".to_string(), vec![]).boxed(),
                            Prop::False.boxed()
                        )
                        .boxed(),
                        Prop::False.boxed()
                    )
                    .boxed(),
                    Prop::False.boxed()
                )
                .boxed(),
                Prop::Impl(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::False.boxed()
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_allquant_distribution() {
        let tokens = lexer()
            .parse(
                "
                    datatype t;
                    fn u: (\\forall x:t. A(x) & B(x)) => (
                    fn x: t => fst (u x),
                    fn x: t => snd (u x)
                )",
            )
            .unwrap();
        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let (_type, _) = typify(&proof.proof_term).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::ForAll {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::And(
                        Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
                        Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed()
                    )
                    .boxed()
                }
                .boxed(),
                Prop::And(
                    Prop::ForAll {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed()
                    }
                    .boxed(),
                    Prop::ForAll {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed()
                    }
                    .boxed()
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_reusing_allquant_ident() {
        let tokens = lexer()
            .parse("datatype t; fn u: (∀x:t. C(x, x)) => fn a:t => fn a:t => u a")
            .unwrap();

        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let (_type, _) = typify(&proof.proof_term).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::ForAll {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Atom("C".to_string(), vec!["x".to_string(), "x".to_string()])
                        .boxed()
                }
                .boxed(),
                Prop::ForAll {
                    object_ident: "a".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::ForAll {
                        object_ident: "a".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom("C".to_string(), vec!["a".to_string(), "a".to_string()])
                            .boxed()
                    }
                    .boxed()
                }
                .boxed()
            ))
        )
    }

    #[test]
    fn test_exsists_move_unquantified() {
        let tokens = lexer().parse("datatype t; fn u: (\\forall x:t. A(x) -> C) => fn w: \\exists x:t. A(x) => let (a, proof) = w in u a proof").unwrap();

        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let (_type, _) = typify(&proof.proof_term).unwrap();

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::ForAll {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Impl(
                        Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
                        Prop::Atom("C".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                }
                .boxed(),
                Prop::Impl(
                    Prop::Exists {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
                    }
                    .boxed(),
                    Prop::Atom("C".to_string(), vec![]).boxed(),
                )
                .boxed(),
            ))
        )
    }

    #[test]
    fn test_do_not_allow_exists_quant_escape() {
        let tokens = lexer()
            .parse("datatype t; fn u: \\exists x:t. C(x) => let (a, proof) = u in proof")
            .unwrap();

        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let _type = typify(&proof.proof_term);

        assert_eq!(_type, Err(TypeError::QuantifiedObjectEscapesScope))
    }
}
