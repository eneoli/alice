use chumsky::prelude::*;

use crate::kernel::proof_term::{
    Abort, Application, Case, Function, Ident, LetIn, OrLeft, OrRight, Pair, ProjectFst,
    ProjectSnd, ProofTerm, Type, TypeAscription,
};

use super::{fol::fol_parser, Token};

/*
    == Proof Term Parser ==

    Expr            = (Function | Case | Application | LetIn ), [ TypeAscription ] ;
    TypeAscription  = ":", Prop ;
    Unit            = "(", ")" ;
    Pair            = "(", Expr, ",", Expr, [ "," ], ")" ;
    Atom            = "(", Expr, ")" | Ident | Pair | Unit | Sorry ;
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
            .map_with_span(|ident, span| ProofTerm::Ident(Ident(ident, Some(span))))
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

        let sorry = just(Token::SORRY).map(|_| ProofTerm::Sorry);

        let atom = choice((
            proof_term
                .clone()
                .delimited_by(just(Token::LROUND), just(Token::RROUND)),
            ident_term,
            pair,
            unit,
            sorry,
        ))
        .boxed();

        let function = just(Token::FN)
            .ignore_then(ident_token)
            .then(just(Token::COLON).ignore_then(fol_parser()).or_not())
            .then_ignore(just(Token::ARROW))
            .then(proof_term.clone())
            .map_with_span(|((param_ident, param_prop), body), span| {
                ProofTerm::Function(Function {
                    param_ident,
                    param_type: param_prop.map(Type::Prop),
                    body: Box::new(body),
                    span: Some(span),
                })
            })
            .boxed();

        let let_in = just(Token::LET)
            .ignore_then(just(Token::LROUND))
            .ignore_then(ident_token)
            .then_ignore(just(Token::COMMA))
            .then(ident_token)
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
                    .then(ident_token)
                    .then_ignore(just(Token::ARROW))
                    .then(proof_term.clone())
                    .then_ignore(just(Token::COMMA))
                    //
                    .then_ignore(just(Token::IDENT("inr".to_string())))
                    .then(ident_token)
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
                    if let ProofTerm::Ident(Ident(ref ident, _)) = lhs {
                        if identifiers.contains(&ident.as_str()) && rhs.is_empty() {
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

    // UTILS

    fn parse(proof_term: &str) -> ProofTerm {
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_term_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        ast
    }

    // END UTILS

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
                body: ProofTerm::Ident(Ident("x".to_string(), Some(8..9))).boxed(),
                span: Some(0..9),
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
                body: ProofTerm::Ident(Ident("x".to_string(), Some(11..12))).boxed(),
                span: Some(0..12),
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
                        ProofTerm::Ident(Ident("x".to_string(), Some(13..14))).boxed()
                    ))
                    .boxed(),
                    ProofTerm::ProjectFst(ProjectFst(
                        ProofTerm::Ident(Ident("x".to_string(), Some(20..21))).boxed()
                    ))
                    .boxed(),
                ))
                .boxed(),
                span: Some(0..22),
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
                        ProofTerm::Ident(Ident("x".to_string(), Some(20..21))).boxed()
                    ))
                    .boxed(),
                    ProofTerm::ProjectFst(ProjectFst(
                        ProofTerm::Ident(Ident("x".to_string(), Some(27..28))).boxed()
                    ))
                    .boxed(),
                ))
                .boxed(),
                span: Some(0..29),
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
                            function: ProofTerm::Ident(Ident("f".to_string(), Some(17..18)))
                                .boxed(),
                            applicant: ProofTerm::Application(Application {
                                function: ProofTerm::Ident(Ident("x".to_string(), Some(20..21)))
                                    .boxed(),
                                applicant: ProofTerm::Ident(Ident("x".to_string(), Some(22..23)))
                                    .boxed()
                            })
                            .boxed()
                        })
                        .boxed(),
                        span: Some(9..24),
                    })
                    .boxed(),
                    applicant: ProofTerm::Function(Function {
                        param_ident: "x".to_string(),
                        param_type: None,
                        body: ProofTerm::Application(Application {
                            function: ProofTerm::Ident(Ident("f".to_string(), Some(35..36)))
                                .boxed(),
                            applicant: ProofTerm::Application(Application {
                                function: ProofTerm::Ident(Ident("x".to_string(), Some(38..39)))
                                    .boxed(),
                                applicant: ProofTerm::Ident(Ident("x".to_string(), Some(40..41)))
                                    .boxed()
                            })
                            .boxed()
                        })
                        .boxed(),
                        span: Some(27..42),
                    })
                    .boxed()
                })
                .boxed(),
                span: Some(0..43),
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
                            function: ProofTerm::Ident(Ident("f".to_string(), Some(27..28)))
                                .boxed(),
                            applicant: ProofTerm::Application(Application {
                                function: ProofTerm::Ident(Ident("x".to_string(), Some(30..31)))
                                    .boxed(),
                                applicant: ProofTerm::Ident(Ident("x".to_string(), Some(32..33)))
                                    .boxed()
                            })
                            .boxed()
                        })
                        .boxed(),
                        span: Some(14..34),
                    })
                    .boxed(),
                    applicant: ProofTerm::Function(Function {
                        param_ident: "x".to_string(),
                        param_type: Some(Type::Prop(Prop::Atom("B".to_string(), vec![]))),
                        body: ProofTerm::Application(Application {
                            function: ProofTerm::Ident(Ident("f".to_string(), Some(50..51)))
                                .boxed(),
                            applicant: ProofTerm::Application(Application {
                                function: ProofTerm::Ident(Ident("x".to_string(), Some(53..54)))
                                    .boxed(),
                                applicant: ProofTerm::Ident(Ident("x".to_string(), Some(55..56)))
                                    .boxed()
                            })
                            .boxed()
                        })
                        .boxed(),
                        span: Some(37..57),
                    })
                    .boxed()
                })
                .boxed(),
                span: Some(0..58),
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
                ProofTerm::Ident(Ident("a".to_string(), Some(1..2))).boxed(),
                ProofTerm::Ident(Ident("b".to_string(), Some(4..5))).boxed(),
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

        assert_eq!(
            ast,
            ProofTerm::Ident(Ident("hiThere".to_string(), Some(0..7)))
        )
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
                function: ProofTerm::Ident(Ident("f".to_string(), Some(0..1))).boxed(),
                applicant: ProofTerm::Ident(Ident("a".to_string(), Some(2..3))).boxed(),
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
                    body: ProofTerm::Ident(Ident("u".to_string(), Some(9..10))).boxed(),
                    span: Some(1..10),
                })
                .boxed(),
                applicant: ProofTerm::Function(Function {
                    param_ident: "x".to_string(),
                    param_type: None,
                    body: ProofTerm::Ident(Ident("x".to_string(), Some(20..21))).boxed(),
                    span: Some(12..21),
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
                    body: ProofTerm::Ident(Ident("u".to_string(), Some(15..16))).boxed(),
                    span: Some(1..16),
                })
                .boxed(),
                applicant: ProofTerm::Function(Function {
                    param_ident: "x".to_string(),
                    param_type: Some(Type::Prop(Prop::False)),
                    body: ProofTerm::Ident(Ident("x".to_string(), Some(32..33))).boxed(),
                    span: Some(18..33),
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
                    body: ProofTerm::Ident(Ident("x".to_string(), Some(16..17))).boxed(),
                    span: Some(8..17),
                })
                .boxed(),
                span: Some(0..17),
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
                    body: ProofTerm::Ident(Ident("x".to_string(), Some(28..29))).boxed(),
                    span: Some(14..29),
                })
                .boxed(),
                span: Some(0..29),
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
                    ProofTerm::Ident(Ident("a".to_string(), Some(5..6))).boxed(),
                    ProofTerm::Ident(Ident("b".to_string(), Some(8..9))).boxed(),
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
            ProofTerm::OrLeft(OrLeft(
                ProofTerm::Ident(Ident("a".to_string(), Some(4..5))).boxed()
            ))
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
            ProofTerm::OrRight(OrRight(
                ProofTerm::Ident(Ident("a".to_string(), Some(4..5))).boxed()
            ))
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
                    head: ProofTerm::Ident(Ident("u".to_string(), Some(13..14))).boxed(),
                    fst_ident: "a".to_string(),
                    fst_term: ProofTerm::OrRight(OrRight(
                        ProofTerm::Ident(Ident("a".to_string(), Some(31..32))).boxed()
                    ))
                    .boxed(),
                    snd_ident: "b".to_string(),
                    snd_term: ProofTerm::OrLeft(OrLeft(
                        ProofTerm::Ident(Ident("b".to_string(), Some(47..48))).boxed()
                    ))
                    .boxed()
                })
                .boxed(),
                span: Some(0..48),
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
                    ProofTerm::Ident(Ident("a".to_string(), Some(5..6))).boxed(),
                    ProofTerm::Ident(Ident("b".to_string(), Some(8..9))).boxed(),
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
            ProofTerm::Abort(Abort(
                ProofTerm::Ident(Ident("a".to_string(), Some(6..7))).boxed()
            ))
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
                    ProofTerm::Ident(Ident("a".to_string(), Some(6..7))).boxed(),
                    ProofTerm::Ident(Ident("b".to_string(), Some(8..9))).boxed(),
                ))
                .boxed(),
                fst_ident: "u".to_string(),
                fst_term: ProofTerm::Ident(Ident("u".to_string(), Some(23..24))).boxed(),
                snd_ident: "u".to_string(),
                snd_term: ProofTerm::Ident(Ident("u".to_string(), Some(35..36))).boxed(),
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
                head: ProofTerm::Ident(Ident("M".to_string(), Some(13..14))).boxed(),
                body: ProofTerm::Pair(Pair(
                    ProofTerm::Ident(Ident("b".to_string(), Some(19..20))).boxed(),
                    ProofTerm::Ident(Ident("a".to_string(), Some(22..23))).boxed()
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
                head: ProofTerm::Ident(Ident("M".to_string(), Some(13..14))).boxed(),
                body: ProofTerm::Function(Function {
                    param_type: None,
                    param_ident: "x".to_string(),
                    body: ProofTerm::Ident(Ident("a".to_string(), Some(26..27))).boxed(),
                    span: Some(18..27),
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
                head: ProofTerm::Ident(Ident("M".to_string(), Some(13..14))).boxed(),
                body: ProofTerm::Function(Function {
                    param_type: Some(Type::Prop(Prop::Atom("A".to_string(), vec![]))),
                    param_ident: "x".to_string(),
                    body: ProofTerm::Ident(Ident("a".to_string(), Some(29..30))).boxed(),
                    span: Some(18..30),
                })
                .boxed()
            })
        )
    }

    #[test]
    fn test_root_sorry() {
        let ast = parse("sorry");

        assert_eq!(ast, ProofTerm::Sorry);
    }

    #[test]
    fn test_sorry_in_function_body() {
        let ast = parse("fn u => sorry");

        assert_eq!(
            ast,
            Function::create("u".to_string(), None, ProofTerm::Sorry.boxed(), Some(0..13))
        );
    }

    #[test]
    fn test_sorry_in_pair() {
        let ast = parse("(sorry, sorry)");

        assert_eq!(
            ast,
            Pair::create(ProofTerm::Sorry.boxed(), ProofTerm::Sorry.boxed())
        );
    }

    #[test]
    fn test_sorry_as_function_in_application() {
        let ast = parse("sorry u");
        assert_eq!(
            ast,
            Application::create(
                ProofTerm::Sorry.boxed(),
                ProofTerm::Ident(Ident("u".to_string(), Some(6..7))).boxed()
            )
        );
    }

    #[test]
    fn test_sorry_as_applicant_in_application() {
        let ast = parse("u sorry");

        assert_eq!(
            ast,
            Application::create(
                ProofTerm::Ident(Ident("u".to_string(), Some(0..1))).boxed(),
                ProofTerm::Sorry.boxed()
            )
        );
    }

    #[test]
    fn test_sorry_in_let_in_head() {
        let ast = parse("let (a, b)  = sorry in u");

        assert_eq!(
            ast,
            ProofTerm::LetIn(LetIn {
                fst_ident: "a".to_string(),
                snd_ident: "b".to_string(),
                head: ProofTerm::Sorry.boxed(),
                body: ProofTerm::Ident(Ident("u".to_string(), Some(23..24))).boxed(),
            })
        )
    }

    #[test]
    fn test_sorry_in_let_in_body() {
        let ast = parse("let (a, b) = u in sorry");

        assert_eq!(
            ast,
            ProofTerm::LetIn(LetIn {
                fst_ident: "a".to_string(),
                snd_ident: "b".to_string(),
                head: ProofTerm::Ident(Ident("u".to_string(), Some(13..14))).boxed(),
                body: ProofTerm::Sorry.boxed(),
            })
        );
    }

    #[test]
    fn test_sorry_in_case_head() {
        let ast = parse("case sorry of inl a => a, inr b => b");

        assert_eq!(
            ast,
            Case::create(
                ProofTerm::Sorry.boxed(),
                "a".to_string(),
                ProofTerm::Ident(Ident("a".to_string(), Some(23..24))).boxed(),
                "b".to_string(),
                ProofTerm::Ident(Ident("b".to_string(), Some(35..36))).boxed(),
            )
        );
    }

    #[test]
    fn test_sorry_in_case_body() {
        let ast = parse("case u of inl a => sorry, inr b => sorry");

        assert_eq!(
            ast,
            Case::create(
                ProofTerm::Ident(Ident("u".to_string(), Some(5..6))).boxed(),
                "a".to_string(),
                ProofTerm::Sorry.boxed(),
                "b".to_string(),
                ProofTerm::Sorry.boxed(),
            )
        );
    }
}
