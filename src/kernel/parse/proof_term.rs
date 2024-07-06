use chumsky::prelude::*;

use crate::kernel::proof_term::{
    Abort, Application, Case, Function, Ident, LetIn, OrLeft, OrRight, Pair, ProjectFst,
    ProjectSnd, ProofTerm, Type, TypeAscription,
};

use super::{fol::fol_parser, Token};

/*
    == Proof Term Parser ==

    Expr            = (Function | Case | Application | LetIn), [ TypeAscription ] ;
    TypeAscription  = ":", Prop ;
    Unit            = "(", ")" ;
    Pair            = "(", Expr, ",", Expr, [ "," ], ")" ;
    Atom            = "(", Expr, ")" | Ident | Pair | Unit ;
    Function        = "fn", Ident, [ ":", Prop ], "=>", Expr ;
    CaseExpr        = Case | Application | LetIn;
    Case            = "case", CaseExpr, "of", "inl", Ident, "=>", Expr, ",", "inr", Ident, "=>", Expr, [","] ;
    Application     = Atom, { Atom } ;
    LetIn           = "let", "(", Ident, ",", Ident, ")", "=", Expr, "in", Expr ;
*/
pub fn proof_term_parser() -> impl Parser<Token, ProofTerm, Error = Simple<Token>> {
    let ident_token = select! { Token::IDENT(ident) => ident }.labelled("identifier");

    let proof_term = recursive(|proof_term| {
        let ident_term = ident_token
            .map(|ident| ProofTerm::Ident(Ident(ident)))
            .boxed();

        let unit = just(Token::LROUND)
            .then(just(Token::RROUND))
            .map(|_| ProofTerm::Unit)
            .boxed();

        let pair = just(Token::LROUND)
            .ignore_then(proof_term.clone())
            .then_ignore(just(Token::COMMA))
            .then(proof_term.clone())
            .then_ignore(just(Token::COMMA).or_not())
            .then_ignore(just(Token::RROUND))
            .map(|(fst, snd)| ProofTerm::Pair(Pair(Box::new(fst), Box::new(snd))))
            .boxed();

        let atom = choice((
            proof_term
                .clone()
                .delimited_by(just(Token::LROUND), just(Token::RROUND)),
            ident_term.clone(),
            pair.clone(),
            unit.clone(),
        ))
        .boxed();

        let function = just(Token::FN)
            .ignore_then(ident_token)
            .then(just(Token::COLON).ignore_then(fol_parser()).or_not())
            .then_ignore(just(Token::ARROW))
            .then(proof_term.clone())
            .map(|((param_ident, param_prop), body)| {
                ProofTerm::Function(Function {
                    param_ident,
                    param_type: param_prop.map(|prop| Type::Prop(prop)),
                    body: Box::new(body),
                })
            })
            .boxed();

        let let_in = just(Token::LET)
            .ignore_then(just(Token::LROUND))
            .ignore_then(ident_token.clone())
            .then_ignore(just(Token::COMMA))
            .then(ident_token.clone())
            .then_ignore(just(Token::RROUND))
            .then_ignore(just(Token::EQUAL))
            .then(proof_term.clone())
            .then_ignore(just(Token::IN))
            .then(proof_term.clone())
            .map(|(((fst_ident, snd_ident), pair_proof_term), body)| {
                ProofTerm::LetIn(LetIn {
                    fst_ident,
                    snd_ident,
                    head: Box::new(pair_proof_term),
                    body: Box::new(body),
                })
            });

        let case = |application: Recursive<'static, Token, ProofTerm, Simple<Token>>| {
            recursive(|case| {
                let case_expr = choice((case.clone(), application.clone(), let_in.clone()));

                just(Token::CASE)
                    .ignore_then(case_expr.clone())
                    .then_ignore(just(Token::OF))
                    //
                    .then_ignore(just(Token::IDENT("inl".to_string())))
                    .then(ident_token.clone())
                    .then_ignore(just(Token::ARROW))
                    .then(proof_term.clone())
                    .then_ignore(just(Token::COMMA))
                    //
                    .then_ignore(just(Token::IDENT("inr".to_string())))
                    .then(ident_token.clone())
                    .then_ignore(just(Token::ARROW))
                    .then(proof_term.clone())
                    .then_ignore(just(Token::COMMA).or_not())
                    .map(
                        |((((proof_term, left_ident), left_term), right_ident), right_term)| {
                            ProofTerm::Case(Case {
                                head: Box::new(proof_term),
                                fst_ident: left_ident,
                                fst_term: Box::new(left_term),
                                snd_ident: right_ident,
                                snd_term: Box::new(right_term),
                            })
                        },
                    )
                    .boxed()
            })
        };

        let application = recursive(|application| {
            atom.clone()
                .then(
                    choice((
                        atom.clone(),
                        function.clone(),
                        case(application).clone(),
                        let_in.clone(),
                    ))
                    .repeated(),
                )
                .try_map(|(lhs, rhs), span| {
                    //  check that if lhs is constructor/destructor, we got a rhs
                    let identifiers = ["inl", "inr", "abort", "fst", "snd"];
                    if let ProofTerm::Ident(Ident(ref ident)) = lhs {
                        if identifiers.contains(&ident.as_str()) && rhs.len() == 0 {
                            return Err(Simple::custom(span, "Missing applicant"));
                        }
                    }

                    // check rhs does not include constructor/destructor
                    for element in rhs.iter() {
                        if let ProofTerm::Ident(ident) = element {
                            if identifiers.contains(&ident.as_str()) {
                                return Err(Simple::custom(
                                    span.clone(),
                                    format!(
                                        "Right-hand side of an applicant cannot contain {}",
                                        ident.as_str()
                                    ),
                                ));
                            }
                        }
                    }

                    Ok((lhs, rhs))
                })
                .foldl(|lhs, rhs| {
                    if let ProofTerm::Ident(ident) = lhs.clone() {
                        match ident.as_str() {
                            "inl" => ProofTerm::OrLeft(OrLeft(Box::new(rhs))),
                            "inr" => ProofTerm::OrRight(OrRight(Box::new(rhs))),
                            "abort" => ProofTerm::Abort(Abort(Box::new(rhs))),
                            "fst" => ProofTerm::ProjectFst(ProjectFst(Box::new(rhs))),
                            "snd" => ProofTerm::ProjectSnd(ProjectSnd(Box::new(rhs))),
                            _ => ProofTerm::Application(Application {
                                function: Box::new(lhs),
                                applicant: Box::new(rhs),
                            }),
                        }
                    } else {
                        ProofTerm::Application(Application {
                            function: Box::new(lhs),
                            applicant: Box::new(rhs),
                        })
                    }
                })
                .boxed()
        });

        let type_ascription = just(Token::COLON).ignore_then(fol_parser());

        choice((function, case(application.clone()), application, let_in))
            .then(type_ascription.or_not())
            .map(|(proof_term, ascription)| {
                if let Some(ascription) = ascription {
                    ProofTerm::TypeAscription(TypeAscription {
                        proof_term: proof_term.boxed(),
                        ascription: Type::Prop(ascription),
                    })
                } else {
                    proof_term
                }
            })
            .boxed()
    });

    proof_term.then_ignore(end())
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use chumsky::{Parser, Stream};

    use crate::kernel::{
        parse::lexer::lexer,
        proof_term::{
            Abort, Application, Case, Function, Ident, LetIn, OrLeft, OrRight, Pair, ProjectFst,
            ProjectSnd, ProofTerm, Type,
        },
        prop::Prop,
    };

    use super::proof_term_parser;

    #[test]
    pub fn test_id_function() {
        let proof_term = "fn x => x";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "x".to_string(),
                param_type: None,
                body: ProofTerm::Ident(Ident("x".to_string())).boxed()
            })
        )
    }

    #[test]
    pub fn test_id_function_annotated() {
        let proof_term = "fn x: A => x";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "x".to_string(),
                param_type: Some(Type::Prop(Prop::Atom("A".to_string(), vec![]))),
                body: ProofTerm::Ident(Ident("x".to_string())).boxed()
            })
        )
    }

    #[test]
    pub fn test_swap_function() {
        let proof_term = "fn x => (snd x, fst x)";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "x".to_string(),
                param_type: None,
                body: ProofTerm::Pair(Pair(
                    ProofTerm::ProjectSnd(ProjectSnd(
                        ProofTerm::Ident(Ident("x".to_string())).boxed()
                    ))
                    .boxed(),
                    ProofTerm::ProjectFst(ProjectFst(
                        ProofTerm::Ident(Ident("x".to_string())).boxed()
                    ))
                    .boxed(),
                ))
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_swap_function_annotated() {
        let proof_term = "fn x: A & B => (snd x, fst x)";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "x".to_string(),
                param_type: Some(Type::Prop(Prop::And(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                ))),
                body: ProofTerm::Pair(Pair(
                    ProofTerm::ProjectSnd(ProjectSnd(
                        ProofTerm::Ident(Ident("x".to_string())).boxed()
                    ))
                    .boxed(),
                    ProofTerm::ProjectFst(ProjectFst(
                        ProofTerm::Ident(Ident("x".to_string())).boxed()
                    ))
                    .boxed(),
                ))
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_y_combinator() {
        let proof_term = "fn f => (fn x => f (x x)) (fn x => f (x x))";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "f".to_string(),
                param_type: None,
                body: ProofTerm::Application(Application {
                    function: ProofTerm::Function(Function {
                        param_ident: "x".to_string(),
                        param_type: None,
                        body: ProofTerm::Application(Application {
                            function: ProofTerm::Ident(Ident("f".to_string())).boxed(),
                            applicant: ProofTerm::Application(Application {
                                function: ProofTerm::Ident(Ident("x".to_string())).boxed(),
                                applicant: ProofTerm::Ident(Ident("x".to_string())).boxed()
                            })
                            .boxed()
                        })
                        .boxed()
                    })
                    .boxed(),
                    applicant: ProofTerm::Function(Function {
                        param_ident: "x".to_string(),
                        param_type: None,
                        body: ProofTerm::Application(Application {
                            function: ProofTerm::Ident(Ident("f".to_string())).boxed(),
                            applicant: ProofTerm::Application(Application {
                                function: ProofTerm::Ident(Ident("x".to_string())).boxed(),
                                applicant: ProofTerm::Ident(Ident("x".to_string())).boxed()
                            })
                            .boxed()
                        })
                        .boxed()
                    })
                    .boxed()
                })
                .boxed(),
            })
        )
    }

    #[test]
    pub fn test_y_combinator_annotated() {
        let proof_term = "fn f: (A) => (fn x: (B) => f (x x)) (fn x: (B) => f (x x))";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "f".to_string(),
                param_type: Some(Type::Prop(Prop::Atom("A".to_string(), vec![]))),
                body: ProofTerm::Application(Application {
                    function: ProofTerm::Function(Function {
                        param_ident: "x".to_string(),
                        param_type: Some(Type::Prop(Prop::Atom("B".to_string(), vec![]))),
                        body: ProofTerm::Application(Application {
                            function: ProofTerm::Ident(Ident("f".to_string())).boxed(),
                            applicant: ProofTerm::Application(Application {
                                function: ProofTerm::Ident(Ident("x".to_string())).boxed(),
                                applicant: ProofTerm::Ident(Ident("x".to_string())).boxed()
                            })
                            .boxed()
                        })
                        .boxed()
                    })
                    .boxed(),
                    applicant: ProofTerm::Function(Function {
                        param_ident: "x".to_string(),
                        param_type: Some(Type::Prop(Prop::Atom("B".to_string(), vec![]))),
                        body: ProofTerm::Application(Application {
                            function: ProofTerm::Ident(Ident("f".to_string())).boxed(),
                            applicant: ProofTerm::Application(Application {
                                function: ProofTerm::Ident(Ident("x".to_string())).boxed(),
                                applicant: ProofTerm::Ident(Ident("x".to_string())).boxed()
                            })
                            .boxed()
                        })
                        .boxed()
                    })
                    .boxed()
                })
                .boxed(),
            })
        )
    }

    #[test]
    pub fn test_root_pair() {
        let proof_term = "(a, b)";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Pair(Pair(
                ProofTerm::Ident(Ident("a".to_string())).boxed(),
                ProofTerm::Ident(Ident("b".to_string())).boxed(),
            ))
        )
    }

    #[test]
    pub fn test_root_ident() {
        let proof_term = "hiThere";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(ast, ProofTerm::Ident(Ident("hiThere".to_string())))
    }

    #[test]
    pub fn test_root_unit() {
        let proof_term = "()";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(ast, ProofTerm::Unit)
    }

    #[test]
    pub fn test_simple_application() {
        let proof_term = "f a";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Application(Application {
                function: ProofTerm::Ident(Ident("f".to_string())).boxed(),
                applicant: ProofTerm::Ident(Ident("a".to_string())).boxed(),
            })
        )
    }

    #[test]
    pub fn test_higher_order_function_application() {
        let proof_term = "(fn u => u) fn x => x";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Application(Application {
                function: ProofTerm::Function(Function {
                    param_ident: "u".to_string(),
                    param_type: None,
                    body: ProofTerm::Ident(Ident("u".to_string())).boxed(),
                })
                .boxed(),
                applicant: ProofTerm::Function(Function {
                    param_ident: "x".to_string(),
                    param_type: None,
                    body: ProofTerm::Ident(Ident("x".to_string())).boxed(),
                })
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_higher_order_function_application_annotated() {
        let proof_term = "(fn u: \\top => u) fn x: \\bot => x";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Application(Application {
                function: ProofTerm::Function(Function {
                    param_ident: "u".to_string(),
                    param_type: Some(Type::Prop(Prop::True)),
                    body: ProofTerm::Ident(Ident("u".to_string())).boxed(),
                })
                .boxed(),
                applicant: ProofTerm::Function(Function {
                    param_ident: "x".to_string(),
                    param_type: Some(Type::Prop(Prop::False)),
                    body: ProofTerm::Ident(Ident("x".to_string())).boxed(),
                })
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_higher_order_function_return() {
        let proof_term = "fn u => fn x => x";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "u".to_string(),
                param_type: None,
                body: ProofTerm::Function(Function {
                    param_ident: "x".to_string(),
                    param_type: None,
                    body: ProofTerm::Ident(Ident("x".to_string())).boxed(),
                })
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_higher_order_function_return_annotated() {
        let proof_term = "fn u: \\top => fn x: \\bot => x";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "u".to_string(),
                param_type: Some(Type::Prop(Prop::True)),
                body: ProofTerm::Function(Function {
                    param_ident: "x".to_string(),
                    param_type: Some(Type::Prop(Prop::False)),
                    body: ProofTerm::Ident(Ident("x".to_string())).boxed(),
                })
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_fst_projection() {
        let proof_term = "fst (a, b)";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::ProjectFst(ProjectFst(
                ProofTerm::Pair(Pair(
                    ProofTerm::Ident(Ident("a".to_string())).boxed(),
                    ProofTerm::Ident(Ident("b".to_string())).boxed(),
                ))
                .boxed()
            ))
        )
    }

    #[test]
    pub fn test_inl() {
        let proof_term = "inl a";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::OrLeft(OrLeft(ProofTerm::Ident(Ident("a".to_string())).boxed()))
        )
    }

    #[test]
    pub fn test_inr() {
        let proof_term = "inr a";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::OrRight(OrRight(ProofTerm::Ident(Ident("a".to_string())).boxed()))
        )
    }

    #[test]
    pub fn test_inl_inr_case() {
        let proof_term = "fn u => case u of inl a => inr a, inr b => inl b";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function(Function {
                param_ident: "u".to_string(),
                param_type: None,
                body: ProofTerm::Case(Case {
                    head: ProofTerm::Ident(Ident("u".to_string())).boxed(),
                    fst_ident: "a".to_string(),
                    fst_term: ProofTerm::OrRight(OrRight(
                        ProofTerm::Ident(Ident("a".to_string())).boxed()
                    ))
                    .boxed(),
                    snd_ident: "b".to_string(),
                    snd_term: ProofTerm::OrLeft(OrLeft(
                        ProofTerm::Ident(Ident("b".to_string())).boxed()
                    ))
                    .boxed()
                })
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_inr_no_applicant() {
        let proof_term = "inr";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }

    #[test]
    pub fn test_inl_no_applicant() {
        let proof_term = "inl";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }

    #[test]
    pub fn test_no_nested_inl() {
        let proof_term = "fst inl u";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }

    #[test]
    pub fn test_no_nested_inr() {
        let proof_term = "fst inr u";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }

    #[test]
    pub fn test_snd_projection() {
        let proof_term = "snd (a, b)";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::ProjectSnd(ProjectSnd(
                ProofTerm::Pair(Pair(
                    ProofTerm::Ident(Ident("a".to_string())).boxed(),
                    ProofTerm::Ident(Ident("b".to_string())).boxed(),
                ))
                .boxed()
            ))
        )
    }

    #[test]
    pub fn test_abort() {
        let proof_term = "abort a";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Abort(Abort(ProofTerm::Ident(Ident("a".to_string())).boxed()))
        )
    }

    #[test]
    pub fn test_simple_case() {
        let proof_term = "case (a,b) of inl u => u, inr u => u,";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::Case(Case {
                head: ProofTerm::Pair(Pair(
                    ProofTerm::Ident(Ident("a".to_string())).boxed(),
                    ProofTerm::Ident(Ident("b".to_string())).boxed(),
                ))
                .boxed(),
                fst_ident: "u".to_string(),
                fst_term: ProofTerm::Ident(Ident("u".to_string())).boxed(),
                snd_ident: "u".to_string(),
                snd_term: ProofTerm::Ident(Ident("u".to_string())).boxed(),
            })
        )
    }

    #[test]
    pub fn test_root_let_in() {
        let proof_term = "let (a, b) = M in (b, a)";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::LetIn(LetIn {
                fst_ident: "a".to_string(),
                snd_ident: "b".to_string(),
                head: ProofTerm::Ident(Ident("M".to_string())).boxed(),
                body: ProofTerm::Pair(Pair(
                    ProofTerm::Ident(Ident("b".to_string())).boxed(),
                    ProofTerm::Ident(Ident("a".to_string())).boxed()
                ))
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_root_let_in_with_funtion() {
        let proof_term = "let (a, b) = M in fn x => a";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::LetIn(LetIn {
                fst_ident: "a".to_string(),
                snd_ident: "b".to_string(),
                head: ProofTerm::Ident(Ident("M".to_string())).boxed(),
                body: ProofTerm::Function(Function {
                    param_type: None,
                    param_ident: "x".to_string(),
                    body: ProofTerm::Ident(Ident("a".to_string())).boxed(),
                })
                .boxed()
            })
        )
    }

    #[test]
    pub fn test_root_let_in_with_funtion_annotated() {
        let proof_term = "let (a, b) = M in fn x: A => a";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            ProofTerm::LetIn(LetIn {
                fst_ident: "a".to_string(),
                snd_ident: "b".to_string(),
                head: ProofTerm::Ident(Ident("M".to_string())).boxed(),
                body: ProofTerm::Function(Function {
                    param_type: Some(Type::Prop(Prop::Atom("A".to_string(), vec![]))),
                    param_ident: "x".to_string(),
                    body: ProofTerm::Ident(Ident("a".to_string())).boxed(),
                })
                .boxed()
            })
        )
    }
}
