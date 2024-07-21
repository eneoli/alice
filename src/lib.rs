use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{error::Simple, Parser, Stream};
use kernel::{
    checker::{
        check::{check, CheckError}, identifier::Identifier, identifier_context::IdentifierContext
    },
    parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
    process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline, ProofPipelineError},
    proof::Proof,
    proof_tree::ProofTree, prop::Prop,
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

    #[error("Failed to process proof term")]
    ProofTermProcessingError(#[from] ProofPipelineError),

    #[error("Failed to type check")]
    CheckError(#[from] CheckError),
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
pub fn verify(prop: &str, proof_term: &str) -> Result<ProofTree, BackendError> {
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
        .apply(proof)?;

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

    let proof_tree = check(
        &processed_proof.proof_term,
        &parsed_prop,
        &IdentifierContext::new(),
    )?;

    Ok(proof_tree)
}

#[wasm_bindgen]
pub fn parse_prop(prop: &str) -> Result<Prop, BackendError> {
    let len = prop.chars().count();

    // Step 1: Parse tokens
    let tokens = lexer()
        .parse(prop)
        .map_err(|err| BackendError::LexerError(format_errors(err, prop)))?;

    // Step 2: Parse Proof
    let prop_ast = fol_parser()
        .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
        .map_err(|err| BackendError::ParserError(format_errors(err, prop)))?;

    Ok(prop_ast)
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
        .apply(proof)?)
}

#[wasm_bindgen]
pub fn instantiate_free_parameter(mut prop: Prop, substituent: String, substitutor: &Identifier) -> Prop {
    prop.instantiate_free_parameter(&substituent, substitutor);
    prop
}