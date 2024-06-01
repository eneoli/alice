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
