use chumsky::prelude::*;

use crate::core::proof::Proof;

use super::{proof_term::proof_term_parser, Token};

/**
 *     == Proof Parser ==
 *
 *     Proof    = {Datatype}, ProofTerm ;
 *     Datatype = "datatype", Ident, ";" ;
 */
pub fn proof_parser() -> impl Parser<Token, Proof, Error = Simple<Token>> {
    let ident = select! { Token::IDENT(ident) => ident };

    let datatype = just(Token::DATATYPE)
        .ignore_then(ident)
        .then_ignore(just(Token::SEMICOLON));

    datatype
        .repeated()
        .then(proof_term_parser())
        .map(|(datatypes, proof_term)| Proof {
            datatypes,
            proof_term,
        })
        .boxed()
}

// ==== TESTS ====

#[cfg(test)]
mod tests {
    use chumsky::{primitive::end, Parser};

    use crate::core::{
        parse::lexer::lexer,
        proof::Proof,
        proof_term::{ProofTerm, Type},
        prop::Prop,
    };

    use super::proof_parser;

    #[test]
    fn test_no_datatypes_function() {
        let tokens = lexer().parse("fn u: A => u").unwrap();
        let ast = proof_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            Proof {
                datatypes: vec![],
                proof_term: ProofTerm::Function {
                    param_ident: "u".to_string(),
                    param_type: Type::Prop(Prop::Atom("A".to_string(), vec![])),
                    body: ProofTerm::Ident("u".to_string()).boxed()
                }
            }
        )
    }

    #[test]
    fn test_no_datatypes_unit() {
        let tokens = lexer().parse("()").unwrap();
        let ast = proof_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            Proof {
                datatypes: vec![],
                proof_term: ProofTerm::Unit
            }
        )
    }

    #[test]
    fn test_one_datatype_function() {
        let tokens = lexer().parse("datatype nat; fn u: A => u").unwrap();
        let ast = proof_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            Proof {
                datatypes: vec!["nat".to_string()],
                proof_term: ProofTerm::Function {
                    param_ident: "u".to_string(),
                    param_type: Type::Prop(Prop::Atom("A".to_string(), vec![])),
                    body: ProofTerm::Ident("u".to_string()).boxed()
                }
            }
        )
    }

    #[test]
    fn test_one_datatype_unit() {
        let tokens = lexer().parse("datatype nat; ()").unwrap();
        let ast = proof_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            Proof {
                datatypes: vec!["nat".to_string()],
                proof_term: ProofTerm::Unit
            }
        )
    }

    #[test]
    fn test_some_datatypes_function() {
        let tokens = lexer()
            .parse("datatype nat; datatype t; datatype list; fn u: A => u")
            .unwrap();
        let ast = proof_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            Proof {
                datatypes: vec!["nat".to_string(), "t".to_string(), "list".to_string()],
                proof_term: ProofTerm::Function {
                    param_ident: "u".to_string(),
                    param_type: Type::Prop(Prop::Atom("A".to_string(), vec![])),
                    body: ProofTerm::Ident("u".to_string()).boxed()
                }
            }
        )
    }

    #[test]
    fn test_some_datatypes_unit() {
        let tokens = lexer()
            .parse("datatype nat; datatype t; datatype list; ()")
            .unwrap();
        let ast = proof_parser().parse(tokens).unwrap();

        assert_eq!(
            ast,
            Proof {
                datatypes: vec!["nat".to_string(), "t".to_string(), "list".to_string()],
                proof_term: ProofTerm::Unit
            }
        )
    }

    #[test]
    fn test_datatypes_after_proof_term() {
        let tokens = lexer()
            .parse("datatype nat; (fn u: A => u) datatype uff;")
            .unwrap();

        let ast = proof_parser().then_ignore(end()).parse(tokens);

        assert!(ast.is_err())
    }
}
