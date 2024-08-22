use alice::kernel::{
    checker::{check::check, identifier_context::IdentifierContext},
    export::{ocaml_exporter::OcamlExporter, ProofExporter},
    parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
    process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline},
    prove::prove,
};
use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::{Parser, Stream};

fn main() {
    let src = std::fs::read_to_string("test.proof").unwrap();

    // Step 1: Parse tokens
    let tokens = lexer().parse(src.clone());

    println!("{:#?}", tokens);

    if let Err(err) = tokens.clone() {
        err.into_iter().for_each(|e| {
            Report::build(ReportKind::Error, (), e.span().start)
                .with_message(e.to_string())
                .with_label(Label::new(e.span()).with_color(Color::Red))
                .finish()
                .print(Source::from(src.clone()))
                .unwrap();
        });

        return;
    }

    // Step 2: Parse Proof
    let len = src.chars().count();
    let proof = proof_parser().parse(Stream::from_iter(len..len + 1, tokens.unwrap().into_iter()));

    if let Err(err) = proof.clone() {
        println!("{:#?}", err);

        err.into_iter().for_each(|e| {
            Report::build(ReportKind::Error, (), e.span().start)
                .with_message(e.to_string())
                .with_label(Label::new(e.span()).with_color(Color::Red))
                .finish()
                .print(Source::from(src.clone()))
                .unwrap();
        });

        return;
    }

    let fol = "A -> A";
    let fol_tokens = lexer().parse(fol).unwrap();
    let fol_len = fol.chars().count();

    let prop = fol_parser()
        .parse(Stream::from_iter(
            fol_len..fol_len + 1,
            fol_tokens.into_iter(),
        ))
        .unwrap();

    let processed_proof = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof.unwrap(), &prop)
        .unwrap();

    // Step 3: Preprocess ProofTerm

    println!("{:#?}", processed_proof);

    // println!("{:#?}", prop.get_free_parameters());

    let _type = check(
        &processed_proof.proof_term,
        &prop,
        &IdentifierContext::new(),
    );

    println!("{:#?}", _type);

    println!("{}", prove(&prop).unwrap());

    let ocaml_exporter = OcamlExporter::new();

    println!("{}", ocaml_exporter.export(&processed_proof.proof_term));

    // let proof_tree = ProofTree {
    //     premisses: vec![
    //         ProofTree {
    //             premisses: vec![ProofTree {
    //                 premisses: vec![],
    //                 rule: ProofTreeRule::Ident("w".to_string()),
    //                 conclusion: ProofTreeConclusion::PropIsTrue(parse_prop("A").unwrap()),
    //             }],
    //             rule: ProofTreeRule::ImplIntro("w".to_string()),
    //             conclusion: ProofTreeConclusion::PropIsTrue(
    //                 parse_prop("(A -> A) -> A -> A").unwrap(),
    //             ),
    //         },
    //         ProofTree {
    //             premisses: vec![ProofTree {
    //                 premisses: vec![],
    //                 rule: ProofTreeRule::Ident("v".to_string()),
    //                 conclusion: ProofTreeConclusion::PropIsTrue(parse_prop("A").unwrap()),
    //             }],
    //             rule: ProofTreeRule::ImplIntro("v".to_string()),
    //             conclusion: ProofTreeConclusion::PropIsTrue(parse_prop("A -> A").unwrap()),
    //         },
    //     ],
    //     rule: ProofTreeRule::ImplElim,
    //     conclusion: ProofTreeConclusion::PropIsTrue(parse_prop("A -> A").unwrap()),
    // };

    // println!("{}", proof_tree.as_proof_term());
    // println!(
    //     "{:#?}",
    //     check(
    //         &proof_tree.as_proof_term(),
    //         &parse_prop("A -> A").unwrap(),
    //         &IdentifierContext::new()
    //     )
    // );
}
