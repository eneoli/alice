use std::vec;

use chumsky::prelude::*;

use crate::kernel::prop::Prop;

use super::Token;

/*
    == FOL Parser ==
    ----------------

    Prop = Implication ;

    Implication = { Or, "->" }, (Or | Quantor) ; // This prevents LL(1) but isn't that dramatic.

    Or          = And, { "||", (And | Quantor) } ;

    And         = Not, { "&&", (Not | Quantor) } ;

    Not         = { "~" }, Atom ;

    Atom        = ⊤ | ⊥ | Ident, [ "(", Ident, { ",", Ident }, [","],  ")" ] | "(", Prop, ")" ;

    Quantor     = Allquant | Existsquant ;

    Allquant    = "∀", Ident, ":", Ident, ".", Prop ;

    Existsquant = "∃", Ident, ":", Ident, ".", Prop ;
*/
pub fn fol_parser() -> impl Parser<Token, Prop, Error = Simple<Token>> {
    let ident = select! { Token::IDENT(ident) => ident }.labelled("identifier");

    let prop = recursive(|prop: Recursive<Token, Prop, Simple<Token>>| {
        let allquant = just(Token::FORALL)
            .ignore_then(ident)
            .then_ignore(just(Token::COLON))
            .then(ident)
            .then_ignore(just(Token::DOT))
            .then(prop.clone())
            .map(|((object_ident, object_type_ident), body)| Prop::ForAll {
                object_ident,
                object_type_ident,
                body: Box::new(body.clone()),
            })
            .boxed();

        let existsquant = just(Token::EXISTS)
            .ignore_then(ident)
            .then_ignore(just(Token::COLON))
            .then(ident)
            .then_ignore(just(Token::DOT))
            .then(prop.clone())
            .map(|((object_ident, object_type_ident), body)| Prop::Exists {
                object_ident,
                object_type_ident,
                body: Box::new(body.clone()),
            })
            .boxed();

        let quantor = choice((existsquant, allquant)).boxed();

        let atom = ident
            .then(
                ident
                    .then(
                        just(Token::COMMA)
                            .ignore_then(ident)
                            .repeated()
                            .then_ignore(just(Token::COMMA).or_not()),
                    )
                    .delimited_by(just(Token::LROUND), just(Token::RROUND))
                    .or_not(),
            )
            .map(|(ident, params)| {
                if let Some((head, mut tail)) = params {
                    tail.insert(0, head);
                    Prop::Atom(ident, tail)
                } else {
                    Prop::Atom(ident, vec![])
                }
            })
            .or(prop
                .clone()
                .delimited_by(just(Token::LROUND), just(Token::RROUND)))
            .or(just(Token::TRUE).map(|_| Prop::True))
            .or(just(Token::FALSE).map(|_| Prop::False))
            .boxed();

        let not = just(Token::NOT)
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Prop::Impl(Box::new(rhs), Box::new(Prop::False)))
            .boxed();

        let and = not
            .clone()
            .then(
                just(Token::AND)
                    .to(Prop::And)
                    .then(choice((not, quantor.clone())))
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)))
            .boxed();

        let or = and
            .clone()
            .then(
                just(Token::OR)
                    .to(Prop::Or)
                    .then(choice((and, quantor.clone())))
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)))
            .boxed();

        let implication = or
            .clone()
            .then(just(Token::IMPLICATION).to(Prop::Impl))
            .repeated()
            .then(choice((or, quantor.clone())))
            .foldr(|(lhs, op), rhs| op(Box::new(lhs), Box::new(rhs)))
            .boxed();

        implication
    });

    prop
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use chumsky::{Parser, Stream};

    use crate::{
        kernel::{
            parse::{fol::fol_parser, lexer::lexer},
            prop::Prop,
        },
        s,
    };

    #[test]
    fn test_simple_prop() {
        let fol = "A";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(ast, Prop::Atom(String::from("A"), vec![]));
    }

    #[test]
    fn test_parameterized_prop_one() {
        let fol = "A(x)";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(ast, Prop::Atom("A".to_string(), vec!["x".to_string()]));
    }

    #[test]
    fn test_parameterized_prop_one_trailling_comma() {
        let fol = "A(x,)";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(ast, Prop::Atom("A".to_string(), vec!["x".to_string()]));
    }

    #[test]
    fn test_parameterized_prop_two() {
        let fol = "A(x, y)";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Atom("A".to_string(), vec!["x".to_string(), "y".to_string()])
        );
    }

    #[test]
    fn test_parameterized_prop_two_trailling_comma() {
        let fol = "A(x, y, )";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Atom("A".to_string(), vec!["x".to_string(), "y".to_string()])
        );
    }

    #[test]
    fn test_parameterized_prop_three() {
        let fol = "A(x, y, z)";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Atom(
                "A".to_string(),
                vec!["x".to_string(), "y".to_string(), "z".to_string()]
            )
        );
    }

    #[test]
    fn test_parameterized_prop_three_trailling_comma() {
        let fol = "A(x, y, z,)";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Atom(
                "A".to_string(),
                vec!["x".to_string(), "y".to_string(), "z".to_string()]
            )
        );
    }

    #[test]
    pub fn test_parameterized_prop_nested() {
        let fol = "A & B(x, y) || C(x) -> \\forall z:t. Z(z, x)";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Impl(
                Prop::Or(
                    Prop::And(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("B".to_string(), vec!["x".to_string(), "y".to_string()]).boxed()
                    )
                    .boxed(),
                    Prop::Atom("C".to_string(), vec!["x".to_string()]).boxed()
                )
                .boxed(),
                Prop::ForAll {
                    object_ident: "z".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Atom("Z".to_string(), vec!["z".to_string(), "x".to_string()])
                        .boxed(),
                }
                .boxed()
            ),
        );
    }

    #[test]
    fn test_simple_not() {
        let fol = "~A";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Impl(
                Prop::Atom(String::from("A"), vec![]).boxed(),
                Prop::False.boxed()
            )
        );
    }

    #[test]
    fn test_chained_not() {
        let fol = "~!¬A";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Impl(
                Prop::Impl(
                    Prop::Impl(
                        Prop::Atom(String::from("A"), vec![]).boxed(),
                        Prop::False.boxed()
                    )
                    .boxed(),
                    Prop::False.boxed()
                )
                .boxed(),
                Prop::False.boxed()
            )
        );
    }

    #[test]
    fn test_simple_and() {
        let fol = "A & B";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Box::new(Prop::Atom(String::from("A"), vec![])),
                Box::new(Prop::Atom(String::from("B"), vec![])),
            )
        );
    }

    #[test]
    fn test_and_implicit_left_associative() {
        let fol = "A & B & C";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Box::new(Prop::And(
                    Box::new(Prop::Atom(String::from("A"), vec![])),
                    Box::new(Prop::Atom(String::from("B"), vec![]))
                )),
                Box::new(Prop::Atom(String::from("C"), vec![])),
            )
        );
    }

    #[test]
    fn test_and_explicit_left_associative() {
        let fol = "A & (B & C)";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Box::new(Prop::Atom(String::from("A"), vec![])),
                Box::new(Prop::And(
                    Box::new(Prop::Atom(String::from("B"), vec![])),
                    Box::new(Prop::Atom(String::from("C"), vec![]))
                )),
            )
        );
    }

    #[test]
    fn test_precedence_propositional_logic() {
        let fol = "A || B && ~C -> D";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Impl(
                Prop::Or(
                    Prop::Atom(s!("A"), vec![]).boxed(),
                    Prop::And(
                        Prop::Atom(s!("B"), vec![]).boxed(),
                        Prop::Impl(Prop::Atom(s!("C"), vec![]).boxed(), Prop::False.boxed())
                            .boxed()
                    )
                    .boxed()
                )
                .boxed(),
                Prop::Atom(s!("D"), vec![]).boxed()
            )
        );
    }

    #[test]
    fn test_global_forall() {
        let fol = "\\forall x:t. A -> B";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::ForAll {
                object_ident: String::from("x"),
                object_type_ident: String::from("t"),
                body: Prop::Impl(
                    Prop::Atom(format!("A"), vec![]).boxed(),
                    Prop::Atom(format!("B"), vec![]).boxed()
                )
                .boxed()
            }
        );
    }

    #[test]
    fn test_global_exists() {
        let fol = "\\exists x:t. A -> B";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::Exists {
                object_ident: String::from("x"),
                object_type_ident: String::from("t"),
                body: Prop::Impl(
                    Prop::Atom(format!("A"), vec![]).boxed(),
                    Prop::Atom(format!("B"), vec![]).boxed()
                )
                .boxed()
            }
        );
    }

    #[test]
    fn test_left_forall() {
        let fol = "A && \\forall x:t. A -> B";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Prop::Atom(format!("A"), vec![]).boxed(),
                Prop::ForAll {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Impl(
                        Prop::Atom(s!("A"), vec![]).boxed(),
                        Prop::Atom(s!("B"), vec![]).boxed()
                    )
                    .boxed()
                }
                .boxed()
            )
        );
    }

    #[test]
    fn test_left_exists() {
        let fol = "A && \\exists x:t. A -> B";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Prop::Atom(format!("A"), vec![]).boxed(),
                Prop::Exists {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Impl(
                        Prop::Atom(s!("A"), vec![]).boxed(),
                        Prop::Atom(s!("B"), vec![]).boxed()
                    )
                    .boxed()
                }
                .boxed()
            )
        );
    }

    #[test]
    fn test_nested_forall() {
        let fol = "A && (\\forall x:t. x) && C";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Prop::And(
                    Prop::Atom(s!("A"), vec![]).boxed(),
                    Prop::ForAll {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom(s!("x"), vec![]).boxed()
                    }
                    .boxed()
                )
                .boxed(),
                Prop::Atom(s!("C"), vec![]).boxed()
            )
        );
    }

    #[test]
    fn test_nested_exists() {
        let fol = "A && (\\exists x:t. x) && C";
        let len = fol.chars().count();

        let tokens = lexer().parse(fol).unwrap();
        let ast = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Prop::And(
                    Prop::Atom(s!("A"), vec![]).boxed(),
                    Prop::Exists {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom(s!("x"), vec![]).boxed()
                    }
                    .boxed()
                )
                .boxed(),
                Prop::Atom(s!("C"), vec![]).boxed()
            )
        );
    }
}
