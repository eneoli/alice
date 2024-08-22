use std::fmt::Debug;

use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{error::Simple, prelude::end, Parser, Stream};
use itertools::Itertools;
use kernel::{
    checker::{
        check::{check, CheckError},
        identifier::Identifier,
        identifier_context::IdentifierContext,
        TypeCheckerResult,
    },
    export::{ocaml_exporter::OcamlExporter, ProofExporter},
    parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
    process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline, ProofPipelineError},
    proof::Proof,
    proof_tree::{ProofTree, ProofTreeConclusion},
    prop::{Prop, PropParameter, QuantifierKind},
    prove::prove,
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

#[derive(Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
pub enum VerificationResultSolvableStatus {
    Solvable,
    Unsolvable,
    Unknown,
}

#[derive(Clone, PartialEq, Eq, Tsify, Serialize, Deserialize)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum VerificationResult {
    LexerError {
        error_message: String,
        solvable: VerificationResultSolvableStatus,
    },

    ParserError {
        error_message: String,
        solvable: VerificationResultSolvableStatus,
    },

    ProofPipelineError {
        error: ProofPipelineError,
        solvable: VerificationResultSolvableStatus,
    },

    TypeCheckerError {
        error: CheckError,
        solvable: VerificationResultSolvableStatus,
    },

    TypeCheckSucceeded {
        result: TypeCheckerResult,
        solvable: VerificationResultSolvableStatus,
    },
}

#[wasm_bindgen]
pub fn verify(prop: &Prop, proof_term: &str) -> VerificationResult {
    let get_prop_solvable_status = |prop: &Prop| {
        let mut status = VerificationResultSolvableStatus::Unknown;

        if !prop.has_quantifiers() && !prop.has_free_parameters() {
            let solvable = prove(&prop).is_some();

            if solvable {
                status = VerificationResultSolvableStatus::Solvable;
            } else {
                let negative_solvable =
                    prove(&Prop::Impl(prop.boxed(), Prop::False.boxed())).is_some();

                if negative_solvable {
                    status = VerificationResultSolvableStatus::Unsolvable;
                }
            }
        }

        status
    };

    let proof_term_len = proof_term.chars().count();

    // Step 1: Parse ProofTerm tokens
    let token_result = lexer().then_ignore(end()).parse(proof_term);

    if let Err(err) = token_result {
        return VerificationResult::LexerError {
            error_message: format_errors(err, proof_term),
            solvable: get_prop_solvable_status(prop),
        };
    }

    let tokens = token_result.unwrap();

    // Step 2: Parse ProofTerm
    let proof_result = proof_parser().then_ignore(end()).parse(Stream::from_iter(
        proof_term_len..proof_term_len + 1,
        tokens.into_iter(),
    ));

    if let Err(err) = proof_result {
        return VerificationResult::ParserError {
            error_message: format_errors(err, proof_term),
            solvable: get_prop_solvable_status(prop),
        };
    }

    let proof = proof_result.unwrap();

    // Step 3: Preprocess ProofTerm
    let processed_proof_result = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof, prop);

    if let Err(err) = processed_proof_result {
        return VerificationResult::ProofPipelineError {
            error: err,
            solvable: get_prop_solvable_status(prop),
        };
    }

    let processed_proof = processed_proof_result.unwrap();

    // Step 4: Type Checking
    let type_checking_result = check(
        &processed_proof.proof_term,
        &prop,
        &IdentifierContext::new(),
    );

    // Step 5: Prepare response

    if type_checking_result.is_err() {
        return VerificationResult::TypeCheckerError {
            error: type_checking_result.unwrap_err(),
            solvable: get_prop_solvable_status(&prop),
        };
    }

    let checker_result = type_checking_result.unwrap();

    // Check first if every goal has a solution.
    // If not, use Prover to determine provability.

    let mut status = VerificationResultSolvableStatus::Unknown;
    let all_goals_have_solution = checker_result
        .goals
        .iter()
        .all(|goal| goal.solution.is_some());

    if all_goals_have_solution {
        status = VerificationResultSolvableStatus::Solvable;
    } else {
        status = get_prop_solvable_status(prop);
    }

    VerificationResult::TypeCheckSucceeded {
        result: checker_result,
        solvable: status,
    }
}

#[wasm_bindgen]
pub fn parse_prop(prop: &str) -> Result<Prop, BackendError> {
    let len = prop.chars().count();

    // Step 1: Parse tokens
    let tokens = lexer()
        .then_ignore(end())
        .parse(prop)
        .map_err(|err| BackendError::LexerError(format_errors(err, prop)))?;

    // Step 2: Parse Proof
    let prop_ast = fol_parser()
        .then_ignore(end())
        .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
        .map_err(|err| BackendError::ParserError(format_errors(err, prop)))?;

    Ok(prop_ast)
}

#[wasm_bindgen]
pub fn parse_proof_term(proof_term: &str, prop: &Prop) -> Result<Proof, BackendError> {
    let len = proof_term.chars().count();

    // Step 1: Parse tokens
    let tokens = lexer()
        .then_ignore(end())
        .parse(proof_term)
        .map_err(|err| BackendError::LexerError(format_errors(err, proof_term)))?;

    // Step 2: Parse Proof
    let proof = proof_parser()
        .then_ignore(end())
        .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
        .map_err(|err| BackendError::ParserError(format_errors(err, proof_term)))?;

    // Step 3: Preprocess ProofTerm
    Ok(ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof, prop)?)
}

#[wasm_bindgen]
pub fn get_free_parameters(prop: &Prop) -> Vec<PropParameter> {
    prop.get_free_parameters()
}

#[wasm_bindgen]
pub fn has_quantifiers(prop: &Prop) -> bool {
    prop.has_quantifiers()
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
pub fn generate_proof_term_from_proof_tree(proof_tree: &ProofTree, prop: &Prop) -> String {
    let Proof {
        atoms,
        datatypes,
        proof_term,
        ..
    } = proof_tree.as_proof();

    let atom_decls = [atoms, prop.get_atoms()]
        .concat()
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

    let datatype_decls = [datatypes, prop.get_datatypes()]
        .concat()
        .iter()
        .unique()
        .map(|datatype| format!("datatype {};", datatype))
        .join("\n");

    format!("{}\n{}\n\n{}", atom_decls, datatype_decls, proof_term)
}

#[wasm_bindgen]
pub fn print_prop_decls(prop: &Prop) -> String {
    let atom_decls = print_atom_decls(prop.get_atoms());
    let datatype_decls = print_datatype_decls(prop.get_datatypes());

    format!("{}\n{}", atom_decls, datatype_decls)
}

pub fn print_atom_decls(atoms: Vec<(String, usize)>) -> String {
    atoms
        .iter()
        .unique() // print Atom with multiple arities, let type checker throw error
        .map(|(atom_name, arity)| {
            if *arity == 0 {
                format!("atom {};", atom_name)
            } else {
                format!("atom {}({});", atom_name, arity)
            }
        })
        .join("\n")
}

pub fn print_datatype_decls(datatypes: Vec<String>) -> String {
    datatypes
        .iter()
        .unique()
        .map(|datatype| format!("datatype {};", datatype))
        .join("\n")
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

    if let Ok(proof) = parse_proof_term(proof_term, prop) {
        ocaml_exporter.export(&proof.proof_term)
    } else {
        "Invalid proof term".to_string()
    }
}
