use chumsky::prelude::*;

use crate::kernel::proof::Proof;

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
    use chumsky::{primitive::end, Parser, Stream};

    use crate::kernel::{
        parse::{lexer::lexer, proof},
        proof::Proof,
        proof_term::{ProofTerm, Type},
        prop::Prop,
    };

    use super::proof_parser;

    #[test]
    fn test_no_datatypes_function() {

        let proof_term = "fn u: A => u";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter())).unwrap();

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
        let proof_term = "()";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter())).unwrap();

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
        let proof_term = "datatype nat; fn u: A => u";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter())).unwrap();

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
        let proof_term = "datatype nat; ()";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter())).unwrap();

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
        let proof_term = "datatype nat; datatype t; datatype list; fn u: A => u";
        let len = proof_term.chars().count();

        let tokens = lexer()
            .parse(proof_term)
            .unwrap();
        let ast = proof_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter())).unwrap();

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
        let proof_term = "datatype nat; datatype t; datatype list; ()";
        let len = proof_term.chars().count();

        let tokens = lexer()
            .parse(proof_term)
            .unwrap();
        let ast = proof_parser().parse(Stream::from_iter(len..len + 1, tokens.into_iter())).unwrap();

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
        let proof_term = "datatype nat; (fn u: A => u) datatype uff;";
        let len = proof_term.chars().count();

        let tokens = lexer()
            .parse(proof_term)
            .unwrap();

        let ast = proof_parser().then_ignore(end()).parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }
}
