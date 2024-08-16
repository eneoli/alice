use chumsky::prelude::*;
use itertools::{Either, Itertools};

use crate::kernel::proof::{Proof, ProofProcessingState};

use super::{proof_term::proof_term_parser, Token};

/**
 *     == Proof Parser ==
 *
 *     Proof    = {Datatype | Atom}, ProofTerm ;
 *     Datatype = "datatype", Ident, ";" ;
 *     Atom     = "atom", Ident, [ "(", Num, ")" ] , ";" ;
 */
pub fn proof_parser() -> impl Parser<Token, Proof, Error = Simple<Token>> {
    let ident = select! { Token::IDENT(ident) => ident };
    let num = select! { Token::NUM(num) => num };

    enum DeclarationType {
        Atom(String, usize),
        Datatype(String),
    }

    let atom = just(Token::ATOM)
        .ignore_then(ident)
        .then(
            just(Token::LROUND)
                .ignore_then(num)
                .then_ignore(just(Token::RROUND))
                .or_not(),
        )
        .then_ignore(just(Token::SEMICOLON))
        .map(|(atom, arity)| DeclarationType::Atom(atom, arity.unwrap_or(0)))
        .boxed();

    let datatype = just(Token::DATATYPE)
        .ignore_then(ident)
        .then_ignore(just(Token::SEMICOLON))
        .map(DeclarationType::Datatype)
        .boxed();

    choice((datatype, atom))
        .repeated()
        .then(proof_term_parser())
        .try_map(|(declarations, proof_term), span| {
            let (atoms, datatypes): (Vec<(String, usize)>, Vec<String>) =
                declarations.into_iter().partition_map(|decl| match decl {
                    DeclarationType::Atom(atom, arity) => Either::Left((atom, arity)),
                    DeclarationType::Datatype(datatype) => Either::Right(datatype),
                });

            let idents = [atoms.iter().map(|s| s.0.as_str()).collect::<Vec<&str>>(),
                datatypes.iter().map(|s| s.as_str()).collect()]
            .concat();

            // check that each identifier is unique
            let mut seen_idents: Vec<&str> = vec![];
            for ident in idents {
                if seen_idents.contains(&ident) {
                    return Err(Simple::custom(
                        span,
                        format!("Identifier \"{}\" declared multiple times", ident),
                    ));
                }

                seen_idents.push(ident);
            }

            Ok(Proof {
                processing_state: ProofProcessingState::Parsed,
                datatypes,
                atoms,
                proof_term,
            })
        })
        .boxed()
}

// ==== TESTS ====

#[cfg(test)]
mod tests {
    use chumsky::{primitive::end, Parser, Stream};

    use crate::kernel::{
        parse::lexer::lexer,
        proof::{Proof, ProofProcessingState},
        proof_term::{Function, Ident, ProofTerm},
    };

    use super::proof_parser;

    #[test]
    fn test_no_datatypes_no_atoms_function() {
        let proof_term = "fn u => u";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Proof {
                processing_state: ProofProcessingState::Parsed,
                datatypes: vec![],
                atoms: vec![],
                proof_term: ProofTerm::Function(Function {
                    param_type: None,
                    param_ident: "u".to_string(),
                    body: ProofTerm::Ident(Ident("u".to_string(), Some(8..9))).boxed()
                })
            }
        )
    }

    #[test]
    fn test_no_atoms_no_datatypes_unit() {
        let proof_term = "()";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Proof {
                processing_state: ProofProcessingState::Parsed,
                atoms: vec![],
                datatypes: vec![],
                proof_term: ProofTerm::Unit
            }
        )
    }

    #[test]
    fn test_no_atoms_one_datatype_function() {
        let proof_term = "datatype nat; fn u => u";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Proof {
                processing_state: ProofProcessingState::Parsed,
                atoms: vec![],
                datatypes: vec!["nat".to_string()],
                proof_term: ProofTerm::Function(Function {
                    param_ident: "u".to_string(),
                    param_type: None,
                    body: ProofTerm::Ident(Ident("u".to_string(), Some(22..23))).boxed()
                })
            }
        )
    }

    #[test]
    fn test_one_atom_no_datatype_function() {
        let proof_term = "atom A; fn u => u";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Proof {
                processing_state: ProofProcessingState::Parsed,
                atoms: vec![("A".to_string(), 0)],
                datatypes: vec![],
                proof_term: ProofTerm::Function(Function {
                    param_ident: "u".to_string(),
                    param_type: None,
                    body: ProofTerm::Ident(Ident("u".to_string(), Some(16..17))).boxed()
                })
            }
        )
    }

    #[test]
    fn test_one_datatype_unit() {
        let proof_term = "datatype nat; ()";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Proof {
                processing_state: ProofProcessingState::Parsed,
                atoms: vec![],
                datatypes: vec!["nat".to_string()],
                proof_term: ProofTerm::Unit
            }
        )
    }

    #[test]
    fn test_some_datatypes_and_atoms_function() {
        let proof_term =
            "datatype nat; atom A; datatype t; atom B(1); datatype list; atom C(2); fn u => u";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Proof {
                processing_state: ProofProcessingState::Parsed,
                atoms: vec![
                    ("A".to_string(), 0),
                    ("B".to_string(), 1),
                    ("C".to_string(), 2)
                ],
                datatypes: vec!["nat".to_string(), "t".to_string(), "list".to_string()],
                proof_term: ProofTerm::Function(Function {
                    param_ident: "u".to_string(),
                    param_type: None,
                    body: ProofTerm::Ident(Ident("u".to_string(), Some(79..80))).boxed()
                })
            }
        )
    }

    #[test]
    fn test_some_datatypes_unit() {
        let proof_term = "datatype nat; atom A; datatype t; atom B(42); datatype list; ()";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();
        let ast = proof_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        assert_eq!(
            ast,
            Proof {
                processing_state: ProofProcessingState::Parsed,
                atoms: vec![("A".to_string(), 0), ("B".to_string(), 42)],
                datatypes: vec!["nat".to_string(), "t".to_string(), "list".to_string()],
                proof_term: ProofTerm::Unit
            }
        )
    }

    #[test]
    fn test_datatypes_after_proof_term() {
        let proof_term = "datatype nat; (fn u => u) datatype uff;";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();

        let ast = proof_parser()
            .then_ignore(end())
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }

    #[test]
    fn test_atoms_after_proof_term() {
        let proof_term = "datatype nat; (fn u => u) atom uff;";
        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();

        let ast = proof_parser()
            .then_ignore(end())
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }

    #[test]
    fn test_enforce_unqiue_identifiers_datatypes() {
        let proof_term = "datatype t; datatype nat; datatype t; fn u => u";

        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();

        let ast = proof_parser()
            .then_ignore(end())
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }

    #[test]
    fn test_enforce_unqiue_identifiers_atoms() {
        let proof_term = "atom A; atom B; atom A(42); fn u => u";

        let len = proof_term.chars().count();

        let tokens = lexer().parse(proof_term).unwrap();

        let ast = proof_parser()
            .then_ignore(end())
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()));

        assert!(ast.is_err())
    }
}
