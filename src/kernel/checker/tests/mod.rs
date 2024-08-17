#[cfg(test)]
mod tests {
    use std::vec;

    use chumsky::{primitive::end, Parser, Stream};

    use crate::{
        kernel::{
            checker::{
                check::check,
                identifier::IdentifierFactory,
                identifier_context::IdentifierContext,
                synthesize::{synthesize, SynthesizeError},
            },
            parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
            process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline},
            proof_term::ProofTerm,
            proof_tree::{ProofTree, ProofTreeConclusion, ProofTreeRule},
            prop::Prop,
        },
        util::counter::Counter,
    };

    // HELPER

    fn parse_proof(proof: &str) -> ProofTerm {
        let len = proof.chars().count();
        let tokens = lexer().parse(proof).unwrap();

        let proof_ast = proof_parser()
            .then_ignore(end())
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        let processed_proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof_ast)
            .unwrap();

        processed_proof.proof_term
    }

    fn parse_prop(prop: &str) -> Prop {
        let len = prop.chars().count();

        let tokens = lexer().parse(prop).unwrap();
        let ast = fol_parser()
            .then_ignore(end())
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        ast
    }

    fn check_proof_term(proof: &str, prop: &str) -> (Prop, ProofTree) {
        let proof_term_ast = parse_proof(proof);
        let prop_ast = parse_prop(prop);

        (
            prop_ast.clone(),
            check(&proof_term_ast, &prop_ast, &IdentifierContext::new()).unwrap(),
        )
    }

    // END HELPER

    #[test]
    fn test_proof_implication_to_and() {
        let (expected_type, proof_tree) =
            check_proof_term("fn u => fn w => (u, w)", "A -> B -> A & B");

        // check proof tree
        assert_eq!(
            proof_tree,
            ProofTree {
                premisses: vec![ProofTree {
                    premisses: vec![ProofTree {
                        premisses: vec![
                            ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident("u".to_string()),
                                conclusion: ProofTreeConclusion::PropIsTrue(Prop::Atom(
                                    "A".to_string(),
                                    vec![]
                                )),
                            },
                            ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident("w".to_string()),
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
                conclusion: ProofTreeConclusion::PropIsTrue(expected_type),
            }
        )
    }

    #[test]
    fn test_commutativity_of_conjunction() {
        let (expected_type, proof_tree) =
            check_proof_term("fn u => (snd u, fst u)", "A & B -> B & A");

        // test proof tree
        assert_eq!(
            proof_tree,
            ProofTree {
                premisses: vec![ProofTree {
                    premisses: vec![
                        ProofTree {
                            premisses: vec![ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident("u".to_string()),
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
                                rule: ProofTreeRule::Ident("u".to_string()),
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
        let (expected_type, proof_tree) = check_proof_term(
            "fn u => (fn w => fst (u w), fn w => snd (u w))",
            "(A -> B & C) -> (A -> B) && (A -> C)",
        );

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
                                            rule: ProofTreeRule::Ident("u".to_string()),
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
                                            rule: ProofTreeRule::Ident("w".to_string()),
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
                                            rule: ProofTreeRule::Ident("u".to_string()),
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
                                            rule: ProofTreeRule::Ident("w".to_string()),
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
    fn test_truth_expansion() {
        check_proof_term("fn u => ()", "\\top -> \\top");
    }

    #[test]
    fn falsum_expansion() {
        check_proof_term("fn u => abort u", "\\bot -> \\bot");
    }

    #[test]
    fn test_and_expansion() {
        check_proof_term("fn u => (fst u, snd u)", "A & B -> A & B");
    }

    #[test]
    fn test_implication_expansion() {
        check_proof_term("fn u => fn w => u w", "(A -> A) -> A -> A");
    }

    #[test]
    fn or_expansion() {
        check_proof_term(
            "fn u  => case u of inl a => inl a, inr b => inr b",
            "A || B -> A || B",
        );
    }

    #[test]
    fn test_forall_expansion() {
        check_proof_term(
            "datatype t; fn u => fn w => u w",
            "(\\forall x:t. A(x)) -> (\\forall x:t. A(x))",
        );
    }

    #[test]
    fn test_exsists_expansion() {
        check_proof_term(
            "fn u => let (w, p) = u in (w, p)",
            "(\\exists x:t. A(x)) -> (\\exists x:t. A(x))",
        );
    }

    #[test]
    #[should_panic]
    fn test_wrong_identity() {
        check_proof_term("fn u => u", "A & B -> A");
    }

    #[test]
    #[should_panic]
    fn test_no_free_params() {
        check_proof_term("fn u => u", "A(x) -> A(x)");
    }

    #[test]
    fn test_commutativity_of_disjunction() {
        let (expected_type, proof_tree) = check_proof_term(
            "fn u => case u of inl a => inr a, inr b => inl b",
            "A || B -> B || A",
        );

        // check proof tree
        assert_eq!(
            proof_tree,
            ProofTree {
                premisses: vec![ProofTree {
                    premisses: vec![
                        ProofTree {
                            premisses: vec![],
                            rule: ProofTreeRule::Ident("u".to_string()),
                            conclusion: ProofTreeConclusion::PropIsTrue(Prop::Or(
                                Prop::Atom("A".to_string(), vec![]).boxed(),
                                Prop::Atom("B".to_string(), vec![]).boxed(),
                            )),
                        },
                        ProofTree {
                            premisses: vec![ProofTree {
                                premisses: vec![],
                                rule: ProofTreeRule::Ident("a".to_string()),
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
                                rule: ProofTreeRule::Ident("b".to_string()),
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
        let (_, proof_tree) = check_proof_term("()", "\\top");

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
        check_proof_term(
            "fn u => fn w => (snd u) ((fst u) w)",
            "(A -> B) && (B -> C) -> A -> C",
        );
    }

    #[test]
    fn test_composition_of_identities_annotated_left() {
        check_proof_term(
            "
                atom A;
                (fn u: (A -> A) && (A -> A) => fn w: A => (snd u) ((fst u) w)) ((fn x => x), (fn y => y))
            ",
            "A -> A",
        );
    }

    #[test]
    fn test_composition_of_identities_annotated_right() {
        check_proof_term(
            "
                atom A;
                (fn u => fn w: A => (snd u) ((fst u) w)) ((fn x: A => x), (fn y: A => y))
            ",
            "A -> A",
        );
    }

    #[test]
    fn test_non_minimal_identity_proof() {
        check_proof_term("fn u => (fn w => w) u", "A -> A");
    }

    #[test]
    fn test_projection_function() {
        check_proof_term("fn u => fst u", "A & B -> A");
    }

    #[test]
    fn test_implication_chain() {
        check_proof_term("fn u => u (fn u => u)", "((A -> A) -> B) -> B");
    }

    #[test]
    fn test_piano_number_2() {
        check_proof_term("fn z => fn s => s(s(z))", "A -> ((A -> A) -> A)");
    }

    #[test]
    fn test_tripple_neagation_elimination() {
        // ~~~A = ((A => False) => False) => False
        check_proof_term("fn u => fn v => u (fn w => w v)", "~~~A -> ~A");
    }

    #[test]
    fn test_allquant_distribution() {
        check_proof_term(
            "
        datatype t;
        atom A(1);
        atom B(1);

        fn u: (\\forall x:t. A(x) & B(x)) => (
            fn x: t => fst (u x),
            fn x: t => snd (u x)
        )
        ",
            "(\\forall x:t. A(x) & B(x)) -> (\\forall x:t. A(x)) && (\\forall x:t. B(x))",
        );
    }

    #[test]
    fn test_reusing_allquant_ident() {
        check_proof_term(
            "
                atom C(2);
                datatype t;
                
                fn u: (∀x:t. C(x, x)) => fn a:t => fn a:t => u a
            ",
            "(∀x:t. C(x, x)) -> \\forall a:t. \\forall a:t. C(a, a)",
        );
    }

    #[test]
    fn test_exsists_move_unquantified() {
        check_proof_term(
            "
                datatype t;
                atom A(1);
                atom C;
                
                fn u: (\\forall x:t. A(x) -> C) => fn w: \\exists x:t. A(x) => let (a, proof) = w in u a proof
            ",
            "(\\forall x:t. A(x) -> C) -> (\\exists x:t. A(x)) -> C"
        );
    }

    #[test]
    fn test_do_not_allow_exists_quant_escape() {
        let proof_term = parse_proof(
            "
                datatype t;
                atom C(1);

                fn u: \\exists x:t. C(x) => let (a, proof) = u in proof
            ",
        );

        let _type = synthesize(
            &proof_term,
            &IdentifierContext::new(),
            &mut IdentifierFactory::new(Counter::new()),
        );

        assert_eq!(
            _type,
            Err(SynthesizeError::QuantifiedObjectEscapesScope(Some(122..127)))
        )
    }

    #[test]
    #[should_panic]
    fn test_do_not_allow_free_params_function_annotations() {
        check_proof_term(
            "
            atom A(1);

            fn u => snd (fn u: A(a) => (), ())
        ",
            "B -> \\top",
        );
    }

    #[test]
    #[should_panic]
    fn test_do_not_allow_free_params_type_ascription() {
        check_proof_term(
            "
            atom A(1);

            fn u => snd (abort u : A(a), ())
        ",
            "\\bot -> \\top",
        );
    }

    #[test]
    fn test_allow_bound_params_function_annotations() {
        check_proof_term(
            "
                atom A(1);

                fn u => fn a => snd (fn x: A(a) => (), ())
            ",
            "(\\forall x:t. A(x)) -> \\forall x:t. \\top",
        );
    }

    #[test]
    fn test_do_allow_bound_params_type_ascription() {
        check_proof_term(
            "
                atom A(1);
                fn u => fn a => snd (abort u : A(a), ())
            ",
            "\\bot -> \\forall x:t. \\top",
        );
    }

    #[test]
    #[should_panic]
    fn test_exists_does_not_imply_forall() {
        check_proof_term(
            "
                atom A(1);
                datatype t;

                fn u: (\\exists x:t. A(x)) => let (a, proof) = u in fn a => proof
            ",
            "(\\forall x:t. A(x)) -> (\\exists x:t. A(x))",
        );
    }

    #[test]
    fn test_root_sorry() {
        check_proof_term("sorry", "A");
    }

    #[test]
    fn test_sorry_in_function_body() {
        check_proof_term("fn u => sorry", "A -> B");
    }

    #[test]
    #[should_panic]
    fn test_sorry_in_function_body_invalid() {
        check_proof_term("fn u => sorry", "\\top");
    }

    #[test]
    fn test_sorry_in_pair() {
        check_proof_term("(sorry, sorry)", "A & B");
    }

    #[test]
    fn test_sorry_in_application_as_applicant() {
        check_proof_term(
            "
            atom A;
            (fn u: A => u) sorry
        ",
            "A",
        );
    }

    #[test]
    fn test_sorry_in_application_as_function() {
        check_proof_term(
            "
            atom A;
            sorry ()
        ",
            "A",
        );
    }
}
