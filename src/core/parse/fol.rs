use chumsky::prelude::*;

use crate::core::prop::Prop;

use super::Token;

/*
    == FOL Parser ==
    ----------------

    Prop = Implication ;

    Implication = { Or, "->" }, (Or | Quantor) ; // This prevents LL(1) but isn't that dramatic.

    Or          = And, { "||", (And | Quantor) } ;

    And         = Not, { "&&", (Not | Quantor) } ;

    Not         = { "~" }, Atom ;

    Atom        = ⊤ | ⊥ | Ident, [ "(", Ident, ")" ] | "(", Prop, ")" ;

    Quantor     = Allquant | Existsquant ;

    Allquant    = "∀", Ident, ":", Ident, ".", Prop ;

    Existsquant = "∃", Ident, ":", Ident, ".", Prop ;
*/
pub fn fol_parser() -> impl Parser<Token, Prop, Error = Simple<Token>> {
    let ident = select! { Token::IDENT(ident) => ident };

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
                    .delimited_by(just(Token::LROUND), just(Token::RROUND))
                    .or_not(),
            )
            .map(|(ident, param)| Prop::Atom(ident, param))
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
    use chumsky::Parser;

    use crate::{
        core::{
            parse::{fol::fol_parser, lexer::lexer},
            prop::Prop,
        },
        s,
    };

    #[test]
    fn test_simple_prop() {
        let token = lexer().parse("A").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(ast, Prop::Atom(String::from("A"), None));
    }

    #[test]
    fn test_simple_not() {
        let token = lexer().parse("~A").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::Impl(
                Prop::Atom(String::from("A"), None).boxed(),
                Prop::False.boxed()
            )
        );
    }

    #[test]
    fn test_chained_not() {
        let token = lexer().parse("~!¬A").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::Impl(
                Prop::Impl(
                    Prop::Impl(
                        Prop::Atom(String::from("A"), None).boxed(),
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
        let token = lexer().parse("A & B").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Box::new(Prop::Atom(String::from("A"), None)),
                Box::new(Prop::Atom(String::from("B"), None)),
            )
        );
    }

    #[test]
    fn test_and_implicit_left_associative() {
        let token = lexer().parse("A & B & C").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Box::new(Prop::And(
                    Box::new(Prop::Atom(String::from("A"), None)),
                    Box::new(Prop::Atom(String::from("B"), None))
                )),
                Box::new(Prop::Atom(String::from("C"), None)),
            )
        );
    }

    #[test]
    fn test_and_explicit_left_associative() {
        let token = lexer().parse("A & (B & C)").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Box::new(Prop::Atom(String::from("A"), None)),
                Box::new(Prop::And(
                    Box::new(Prop::Atom(String::from("B"), None)),
                    Box::new(Prop::Atom(String::from("C"), None))
                )),
            )
        );
    }

    #[test]
    fn test_precedence_propositional_logic() {
        let token = lexer().parse("A || B && ~C -> D").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::Impl(
                Prop::Or(
                    Prop::Atom(s!("A"), None).boxed(),
                    Prop::And(
                        Prop::Atom(s!("B"), None).boxed(),
                        Prop::Impl(Prop::Atom(s!("C"), None).boxed(), Prop::False.boxed()).boxed()
                    )
                    .boxed()
                )
                .boxed(),
                Prop::Atom(s!("D"), None).boxed()
            )
        )
    }

    #[test]
    fn test_global_forall() {
        let token = lexer().parse("\\forall x:t. A -> B").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::ForAll {
                object_ident: String::from("x"),
                object_type_ident: String::from("t"),
                body: Prop::Impl(
                    Prop::Atom(format!("A"), None).boxed(),
                    Prop::Atom(format!("B"), None).boxed()
                )
                .boxed()
            }
        );
    }

    #[test]
    fn test_global_exists() {
        let token = lexer().parse("\\exists x:t. A -> B").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::Exists {
                object_ident: String::from("x"),
                object_type_ident: String::from("t"),
                body: Prop::Impl(
                    Prop::Atom(format!("A"), None).boxed(),
                    Prop::Atom(format!("B"), None).boxed()
                )
                .boxed()
            }
        );
    }

    #[test]
    fn test_left_forall() {
        let token = lexer().parse("A && \\forall x:t. A -> B").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Prop::Atom(format!("A"), None).boxed(),
                Prop::ForAll {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Impl(
                        Prop::Atom(s!("A"), None).boxed(),
                        Prop::Atom(s!("B"), None).boxed()
                    )
                    .boxed()
                }
                .boxed()
            )
        )
    }

    #[test]
    fn test_left_exists() {
        let token = lexer().parse("A && \\exists x:t. A -> B").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Prop::Atom(format!("A"), None).boxed(),
                Prop::Exists {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Impl(
                        Prop::Atom(s!("A"), None).boxed(),
                        Prop::Atom(s!("B"), None).boxed()
                    )
                    .boxed()
                }
                .boxed()
            )
        )
    }

    #[test]
    fn test_nested_forall() {
        let token = lexer().parse("A && (\\forall x:t. x) && C").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Prop::And(
                    Prop::Atom(s!("A"), None).boxed(),
                    Prop::ForAll {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom(s!("x"), None).boxed()
                    }
                    .boxed()
                )
                .boxed(),
                Prop::Atom(s!("C"), None).boxed()
            )
        );
    }

    #[test]
    fn test_nested_exists() {
        let token = lexer().parse("A && (\\exists x:t. x) && C").unwrap();
        let ast = fol_parser().parse(token).unwrap();

        assert_eq!(
            ast,
            Prop::And(
                Prop::And(
                    Prop::Atom(s!("A"), None).boxed(),
                    Prop::Exists {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom(s!("x"), None).boxed()
                    }
                    .boxed()
                )
                .boxed(),
                Prop::Atom(s!("C"), None).boxed()
            )
        );
    }
}
