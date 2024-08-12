use std::fmt::Debug;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{error::Simple, Parser, Stream};
use itertools::Itertools;
use kernel::{
    checker::{
        check::{check, CheckError},
        identifier::Identifier,
        identifier_context::IdentifierContext,
    },
    export::{ocaml_exporter::OcamlExporter, ProofExporter},
    parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
    process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline, ProofPipelineError},
    proof::Proof,
    proof_tree::{ProofTree, ProofTreeConclusion},
    prop::{Prop, PropParameter, QuantifierKind},
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
pub fn print_prop(prop: &Prop) -> String {
    format!("{}", prop)
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
pub fn get_free_parameters(prop: &Prop) -> Vec<PropParameter> {
    prop.get_free_parameters()
}

#[wasm_bindgen]
pub fn instantiate_free_parameter(
    mut prop: Prop,
    substituent: String,
    substitutor: &Identifier,
) -> Prop {
    prop.instantiate_free_parameter(&substituent, substitutor);
    prop
}

#[wasm_bindgen]
pub fn instantiate_free_parameter_by_index(
    mut prop: Prop,
    index: usize,
    substitutor: &Identifier,
) -> Prop {
    prop.instantiate_free_parameter_by_index(index, substitutor);
    prop
}

#[wasm_bindgen]
pub fn bind_identifier(
    prop: &Prop,
    quantifier_kind: QuantifierKind,
    identifier: Identifier,
    mut identifier_indices: Vec<usize>,
    bind_name: &str,
    type_name: &str,
) -> Prop {
    prop.bind_identifier(
        quantifier_kind,
        identifier,
        &mut identifier_indices,
        bind_name,
        type_name,
    )
}

#[wasm_bindgen]
pub fn generate_proof_term_from_proof_tree(proof_tree: &ProofTree) -> String {
    let Proof {
        atoms,
        datatypes,
        proof_term,
        ..
    } = proof_tree.as_proof();

    let atom_decls = atoms
        .iter()
        .unique() // print Atom with multiple arities, let type checker throw error
        .map(|(atom_name, arity)| {
            if *arity == 0 {
                format!("atom {};", atom_name)
            } else {
                format!("atom {}({});", atom_name, arity)
            }
        })
        .join("\n");

    let datatype_decls = datatypes
        .iter()
        .unique()
        .map(|datatype| format!("datatype {};", datatype))
        .join("\n");

    format!("{}\n{}\n\n{}", atom_decls, datatype_decls, proof_term)
}

#[wasm_bindgen]
pub fn proof_tree_conclusion_alpha_eq(fst: ProofTreeConclusion, snd: ProofTreeConclusion) -> bool {
    match (fst, snd) {
        (
            ProofTreeConclusion::PropIsTrue(ref fst_prop),
            ProofTreeConclusion::PropIsTrue(ref snd_prop),
        ) => Prop::alpha_eq(fst_prop, snd_prop),
        (
            ProofTreeConclusion::TypeJudgement(fst_ident, fst_datatype),
            ProofTreeConclusion::TypeJudgement(snd_ident, snd_datatype),
        ) => (fst_ident == snd_ident) && (fst_datatype == snd_datatype),
        _ => false,
    }
}

#[wasm_bindgen]
pub fn export_as_ocaml(prop: &Prop, proof_term: &str) -> String {
    if prop.has_quantifiers() {
        return "OCaml does not support dependent types.".to_string();
    }

    let ocaml_exporter = OcamlExporter::new();

    if let Ok(proof) = parse_proof_term(proof_term) {
        ocaml_exporter.export(&proof.proof_term)
    } else {
        "Invalid proof term".to_string()
    }
}
