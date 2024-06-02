use chumsky::prelude::*;

use crate::core::proof_term::{ProofTerm, Type};

use super::{fol::fol_parser, Token};

/*
    == Proof Term Parser ==

    Expr           = Function | Case | Application | LetIn ;
    Unit           = "(", ")" ;
    Pair           = "(", Expr, ",", Expr, ")" ;
    Atom           = "(", Expr, ")" | Ident | Pair | Unit ;
    Function       = "fn", Ident, ":", Prop, "=>", Expr ;
    CaseExpr       = Case | Application | LetIn;
    Case           = "case", CaseExpr, "of", "inl", Ident, "=>", Expr, ",", "inr", Ident, "=>", Expr, [","] ;
    Application    = Atom, {Atom | Function | Case | LetIn} ;
    LetIn          = "let", "(", Ident, ",", Ident, ")", "=", Expr, "in", Expr ;
*/
pub fn proof_term_parser() -> impl Parser<Token, ProofTerm, Error = Simple<Token>> {
    let ident_token = select! { Token::IDENT(ident) => ident };

    let proof_term = recursive(|proof_term| {
        let ident_term = ident_token.map(ProofTerm::Ident).boxed();

        let unit = just(Token::LROUND)
            .then(just(Token::RROUND))
            .map(|_| ProofTerm::Unit)
            .boxed();

        let pair = just(Token::LROUND)
            .ignore_then(proof_term.clone())
            .then_ignore(just(Token::COMMA))
            .then(proof_term.clone())
            .then_ignore(just(Token::RROUND))
            .map(|(fst, snd)| ProofTerm::Pair(Box::new(fst), Box::new(snd)))
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
            .then_ignore(just(Token::COLON))
            .then(fol_parser())
            .then_ignore(just(Token::ARROW))
            .then(proof_term.clone())
            .map(|((param_ident, param_type), body)| ProofTerm::Function {
                param_ident,
                param_type: Type::Prop(param_type),
                body: Box::new(body),
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
            .map(
                |(((fst_ident, snd_ident), pair_proof_term), body)| ProofTerm::LetIn {
                    fst_ident,
                    snd_ident,
                    pair_proof_term: Box::new(pair_proof_term),
                    body: Box::new(body),
                },
            );

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
                            ProofTerm::Case {
                                proof_term: Box::new(proof_term),
                                left_ident,
                                left_term: Box::new(left_term),
                                right_ident,
                                right_term: Box::new(right_term),
                            }
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
                .foldl(|lhs, rhs| {
                    if let ProofTerm::Ident(ident) = lhs.clone() {
                        match ident.as_str() {
                            "inl" => ProofTerm::OrLeft(Box::new(rhs)),
                            "inr" => ProofTerm::OrRight(Box::new(rhs)),
                            "abort" => ProofTerm::Abort(Box::new(rhs)),
                            "fst" => ProofTerm::ProjectFst(Box::new(rhs)),
                            "snd" => ProofTerm::ProjectSnd(Box::new(rhs)),
                            _ => ProofTerm::Application {
                                function: Box::new(lhs),
                                applicant: Box::new(rhs),
                            },
                        }
                    } else {
                        ProofTerm::Application {
                            function: Box::new(lhs),
                            applicant: Box::new(rhs),
                        }
                    }
                })
                .boxed()
        });

        choice((function, case(application.clone()), application, let_in)).boxed()
    });

    proof_term.then_ignore(end())
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use std::vec;

    use chumsky::Parser;

    use crate::core::{
        parse::lexer::lexer,
        proof_term::{ProofTerm, Type},
        prop::Prop,
    };

    use super::proof_term_parser;

    #[test]
    pub fn test_id_function() {
        let tokens = lexer().parse("fn x: (A) => x").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function {
                param_ident: "x".to_string(),
                param_type: Type::Prop(Prop::Atom("A".to_string(), vec![])),
                body: ProofTerm::Ident("x".to_string()).boxed()
            }
        )
    }

    #[test]
    pub fn test_swap_function() {
        let tokens = lexer().parse("fn x: (A & B) => (snd x, fst x)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function {
                param_ident: "x".to_string(),
                param_type: Type::Prop(Prop::And(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                )),
                body: ProofTerm::Pair(
                    ProofTerm::ProjectSnd(ProofTerm::Ident("x".to_string()).boxed()).boxed(),
                    ProofTerm::ProjectFst(ProofTerm::Ident("x".to_string()).boxed()).boxed(),
                )
                .boxed()
            }
        )
    }

    #[test]
    pub fn test_y_combinator() {
        let tokens = lexer()
            .parse("fn f: (A) => (fn x: (B) => f (x x)) (fn x: (B) => f (x x))")
            .unwrap();

        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function {
                param_ident: "f".to_string(),
                param_type: Type::Prop(Prop::Atom("A".to_string(), vec![])),
                body: ProofTerm::Application {
                    function: ProofTerm::Function {
                        param_ident: "x".to_string(),
                        param_type: Type::Prop(Prop::Atom("B".to_string(), vec![])),
                        body: ProofTerm::Application {
                            function: ProofTerm::Ident("f".to_string()).boxed(),
                            applicant: ProofTerm::Application {
                                function: ProofTerm::Ident("x".to_string()).boxed(),
                                applicant: ProofTerm::Ident("x".to_string()).boxed()
                            }
                            .boxed()
                        }
                        .boxed()
                    }
                    .boxed(),
                    applicant: ProofTerm::Function {
                        param_ident: "x".to_string(),
                        param_type: Type::Prop(Prop::Atom("B".to_string(), vec![])),
                        body: ProofTerm::Application {
                            function: ProofTerm::Ident("f".to_string()).boxed(),
                            applicant: ProofTerm::Application {
                                function: ProofTerm::Ident("x".to_string()).boxed(),
                                applicant: ProofTerm::Ident("x".to_string()).boxed()
                            }
                            .boxed()
                        }
                        .boxed()
                    }
                    .boxed()
                }
                .boxed(),
            }
        )
    }

    #[test]
    pub fn test_root_pair() {
        let tokens = lexer().parse("(a, b)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Pair(
                ProofTerm::Ident("a".to_string()).boxed(),
                ProofTerm::Ident("b".to_string()).boxed(),
            )
        )
    }

    #[test]
    pub fn test_root_ident() {
        let tokens = lexer().parse("hiThere").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(ast, ProofTerm::Ident("hiThere".to_string()))
    }

    #[test]
    pub fn test_root_unit() {
        let tokens = lexer().parse("()").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(ast, ProofTerm::Unit)
    }

    #[test]
    pub fn test_simple_application() {
        let tokens = lexer().parse("f a").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Application {
                function: ProofTerm::Ident("f".to_string()).boxed(),
                applicant: ProofTerm::Ident("a".to_string()).boxed(),
            }
        )
    }

    #[test]
    pub fn test_higher_order_function_application() {
        let tokens = lexer()
            .parse("(fn u: (T) => u) fn x: (\\bot) => x")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Application {
                function: ProofTerm::Function {
                    param_ident: "u".to_string(),
                    param_type: Type::Prop(Prop::True),
                    body: ProofTerm::Ident("u".to_string()).boxed(),
                }
                .boxed(),
                applicant: ProofTerm::Function {
                    param_ident: "x".to_string(),
                    param_type: Type::Prop(Prop::False),
                    body: ProofTerm::Ident("x".to_string()).boxed(),
                }
                .boxed()
            }
        )
    }

    #[test]
    pub fn test_higher_order_function_return() {
        let tokens = lexer().parse("fn u: (T) => fn x: (\\bot) => x").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Function {
                param_ident: "u".to_string(),
                param_type: Type::Prop(Prop::True),
                body: ProofTerm::Function {
                    param_ident: "x".to_string(),
                    param_type: Type::Prop(Prop::False),
                    body: ProofTerm::Ident("x".to_string()).boxed(),
                }
                .boxed()
            }
        )
    }

    #[test]
    pub fn test_fst_projection() {
        let tokens = lexer().parse("fst (a, b)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::ProjectFst(
                ProofTerm::Pair(
                    ProofTerm::Ident("a".to_string()).boxed(),
                    ProofTerm::Ident("b".to_string()).boxed(),
                )
                .boxed()
            )
        )
    }

    #[test]
    pub fn test_inl() {
        let tokens = lexer().parse("inl a").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::OrLeft(ProofTerm::Ident("a".to_string()).boxed())
        )
    }

    #[test]
    pub fn test_inr() {
        let tokens = lexer().parse("inr a").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::OrRight(ProofTerm::Ident("a".to_string()).boxed())
        )
    }

    #[test]
    pub fn test_snd_projection() {
        let tokens = lexer().parse("snd (a, b)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::ProjectSnd(
                ProofTerm::Pair(
                    ProofTerm::Ident("a".to_string()).boxed(),
                    ProofTerm::Ident("b".to_string()).boxed(),
                )
                .boxed()
            )
        )
    }

    #[test]
    pub fn test_abort() {
        let tokens = lexer().parse("abort a").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Abort(ProofTerm::Ident("a".to_string()).boxed())
        )
    }

    #[test]
    pub fn test_simple_case() {
        let tokens = lexer()
            .parse("case (a,b) of inl u => u, inr u => u,")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::Case {
                proof_term: ProofTerm::Pair(
                    ProofTerm::Ident("a".to_string()).boxed(),
                    ProofTerm::Ident("b".to_string()).boxed(),
                )
                .boxed(),
                left_ident: "u".to_string(),
                left_term: ProofTerm::Ident("u".to_string()).boxed(),
                right_ident: "u".to_string(),
                right_term: ProofTerm::Ident("u".to_string()).boxed(),
            }
        )
    }

    #[test]
    pub fn test_root_let_in() {
        let tokens = lexer().parse("let (a, b) = M in (b, a)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::LetIn {
                fst_ident: "a".to_string(),
                snd_ident: "b".to_string(),
                pair_proof_term: ProofTerm::Ident("M".to_string()).boxed(),
                body: ProofTerm::Pair(
                    ProofTerm::Ident("b".to_string()).boxed(),
                    ProofTerm::Ident("a".to_string()).boxed()
                )
                .boxed()
            }
        )
    }

    #[test]
    pub fn tet_root_let_in_with_funtion() {
        let tokens = lexer().parse("let (a, b) = M in fn x: (A) => a").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            ProofTerm::LetIn {
                fst_ident: "a".to_string(),
                snd_ident: "b".to_string(),
                pair_proof_term: ProofTerm::Ident("M".to_string()).boxed(),
                body: ProofTerm::Function {
                    param_ident: "x".to_string(),
                    param_type: Type::Prop(Prop::Atom("A".to_string(), vec![])),
                    body: ProofTerm::Ident("a".to_string()).boxed(),
                }
                .boxed()
            }
        )
    }
}
