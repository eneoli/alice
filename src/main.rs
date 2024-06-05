use chumsky::Parser;
use server::core::{
    check::{identifier_context::IdentifierContext, typify},
    parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
    process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline},
    proof::Proof,
    proof_term::Type,
};

fn main() {
    let src = std::fs::read_to_string("test.proof").unwrap();

    // Step 1: Parse tokens
    let tokens = lexer().parse(src.clone());
    println!("{:#?}", tokens);

    // Step 2: Parse Proof
    let proof: Proof = proof_parser().parse(tokens.unwrap()).unwrap();

    // Step 3: Preprocess ProofTerm

    let processed_proof = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof);

    println!("{:#?}", processed_proof);

    let _type = typify(&processed_proof.proof_term);

    println!("{:#?}", _type);
}
