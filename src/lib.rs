use core::{
    check::typify,
    parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
    process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline}, proof_term::Type,
};

use chumsky::Parser;
use wasm_bindgen::prelude::*;

pub mod core;
pub mod util;

#[wasm_bindgen]
pub fn infer_type(proofTerm: &str) -> String {
    let src = proofTerm;

    // Step 1: Parse tokens
    let tokens = core::parse::lexer::lexer().parse(src.clone());
    println!("{:#?}", tokens);

    // Step 2: Parse Proof
    let proof = proof_parser().parse(tokens.unwrap());

    if let Err(err) = proof {
        return format!("{:#?}", err);
    }

    // Step 3: Preprocess ProofTerm

    let processed_proof = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof.unwrap());

    println!("{:#?}", processed_proof);

    let _type = typify(&processed_proof.proof_term);

    return format!("{:#?}", _type);
}

#[wasm_bindgen]
pub fn verify(prop: &str, proof_term: &str) -> bool {
    let src = proof_term;

    // Step 1: Parse tokens
    let tokens = core::parse::lexer::lexer().parse(src.clone());
    println!("{:#?}", tokens);

    // Step 2: Parse Proof
    let proof = proof_parser().parse(tokens.unwrap());

    if let Err(err) = proof {
        return false;
    }

    // Step 3: Preprocess ProofTerm

    let processed_proof = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof.unwrap());

    println!("{:#?}", processed_proof);

    let _type = typify(&processed_proof.proof_term);

    // Parse prop
    let prop_tokens = lexer().parse(prop).unwrap();
    let parsed_prop = fol_parser().parse(prop_tokens);

    return Type::Prop(parsed_prop.unwrap()) == _type.unwrap();
}
