use alice::kernel::{
    parse::{fol::fol_parser, lexer::lexer, proof::proof_parser},
    process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline}, prove::prove,
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

    let processed_proof = ProofPipeline::new()
        .pipe(ResolveDatatypes::boxed())
        .apply(proof.unwrap())
        .unwrap();

    // Step 3: Preprocess ProofTerm

    println!("{:#?}", processed_proof);
    let fol = "~~(A || ~A)";
    let fol_tokens = lexer().parse(fol).unwrap();
    let fol_len = fol.chars().count();

    let prop = fol_parser()
        .parse(Stream::from_iter(
            fol_len..fol_len + 1,
            fol_tokens.into_iter(),
        ))
        .unwrap();

    // println!("{:#?}", prop.get_free_parameters());

    // let _type = check(
    //     &processed_proof.proof_term,
    //     &prop,
    //     &IdentifierContext::new(),
    // );

    // println!("{:#?}", _type);

    println!("{}", prove(&prop).unwrap());
}
