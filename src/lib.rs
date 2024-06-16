use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{error::Simple, Parser, Stream};
use kernel::{
    check::{typify, TypeError},
    parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
    process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline},
    proof::Proof,
    proof_term::Type,
    proof_tree::ProofTree,
};

use wasm_bindgen::prelude::*;

pub mod kernel;
pub mod util;

use serde::{Deserialize, Serialize};
use thiserror::Error;
use tsify_next::Tsify;

#[derive(Clone, PartialEq, Eq, Tsify, Serialize, Deserialize, Error, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum BackendError {
    #[error("A lexer error occured.")]
    LexerError(String),

    #[error("A parser errror occured.")]
    ParserError(String),

    #[error("Failed to infer type.")]
    TypeInferenceFailed(TypeError),
}

pub fn format_errors<T: std::hash::Hash + Eq + std::fmt::Display>(
    errors: Vec<Simple<T>>,
    src: &str,
) -> String {
    let mut error_output = Vec::new();

    errors.into_iter().for_each(|e| {
        Report::build(ReportKind::Error, (), e.span().start)
            .with_message(e.to_string())
            .with_label(Label::new(e.span()).with_color(Color::Red))
            .finish()
            .write(Source::from(src), &mut error_output)
            .unwrap();
    });

    return String::from_utf8_lossy(&error_output).to_string();
}

#[wasm_bindgen]
pub fn infer_type(proof_term: &str) -> Result<Type, BackendError> {
    let len = proof_term.chars().count();

    // Step 1: Parse tokens
    let tokens = lexer().parse(proof_term);

    if let Err(err) = tokens {
        return Err(BackendError::LexerError(format_errors(err, proof_term)));
    }

    // Step 2: Parse Proof
    let proof = proof_parser().parse(Stream::from_iter(len..len + 1, tokens.unwrap().into_iter()));

    if let Err(err) = proof {
        return Err(BackendError::ParserError(format_errors(err, proof_term)));
    }

    // Step 3: Preprocess ProofTerm
    let processed_proof = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof.unwrap());

    let (_type, _) = typify(&processed_proof.proof_term).unwrap();

    return Ok(_type);
}

#[wasm_bindgen]
pub fn verify(prop: &str, proof_term: &str) -> Result<bool, BackendError> {
    let proof_term_len = proof_term.chars().count();

    // Step 1: Parse tokens
    let tokens = lexer()
        .parse(proof_term)
        .map_err(|err| BackendError::LexerError(format_errors(err, proof_term)))?;

    // Step 2: Parse Proof
    let proof = proof_parser()
        .parse(Stream::from_iter(
            proof_term_len..proof_term_len + 1,
            tokens.into_iter(),
        ))
        .map_err(|err| BackendError::ParserError(format_errors(err, proof_term)))?;

    // Step 3: Preprocess ProofTerm
    let processed_proof = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof);

    let (_type, _) =
        typify(&processed_proof.proof_term).map_err(|err| BackendError::TypeInferenceFailed(err))?;

    // Parse prop
    let prop_len = prop.chars().count();
    let prop_tokens = lexer()
        .parse(prop)
        .map_err(|err| BackendError::LexerError(format_errors(err, prop)))?;

    let parsed_prop = fol_parser()
        .parse(Stream::from_iter(
            prop_len..prop_len + 1,
            prop_tokens.into_iter(),
        ))
        .map_err(|err| BackendError::ParserError(format_errors(err, prop)))?;

    return Ok(Type::Prop(parsed_prop) == _type);
}

#[wasm_bindgen]
pub fn parse_proof_term(proof_term: &str) -> Result<Proof, BackendError> {
    let len = proof_term.chars().count();

    // Step 1: Parse tokens
    let tokens = lexer()
        .parse(proof_term)
        .map_err(|err| BackendError::LexerError(format_errors(err, proof_term)))?;

    // Step 2: Parse Proof
    let proof = proof_parser()
        .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
        .map_err(|err| BackendError::ParserError(format_errors(err, proof_term)))?;

    // Step 3: Preprocess ProofTerm
    Ok(ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof))
}

#[wasm_bindgen]
pub fn annotate_proof_term(proof_term: &str) -> Result<ProofTree, BackendError> {
    let len = proof_term.chars().count();

    let tokens = lexer()
        .parse(proof_term)
        .map_err(|err| BackendError::LexerError(format_errors(err, proof_term)))?;

    // Step 2: Parse Proof
    let proof = proof_parser()
        .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
        .map_err(|err| BackendError::ParserError(format_errors(err, proof_term)))?;

    // Step 3: Preprocess ProofTerm
    let processed_proof = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof);

    typify(&processed_proof.proof_term)
        .map(|result| result.1)
        .map_err(|err| BackendError::TypeInferenceFailed(err))
}
