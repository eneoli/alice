use chumsky::prelude::*;

use super::Token;

/*
    == Lexer ==
    -----------
*/
pub fn lexer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let truth = choice((
        just("true"),
        just("True"),
        just("⊤"),
        just("T"),
        just("\\top"),
    ))
    .map(|_| Token::TRUE)
    .boxed();

    let falsum = choice((
        just("false"),
        just("False"),
        just("⊥"),
        just("\\bot"),
        just("\\bottom"),
    ))
    .map(|_| Token::FALSE)
    .boxed();

    let ident = text::ident().map(|s: String| match s.to_lowercase().as_str() {
        "fn" => Token::FN,
        _ => Token::IDENT(s),
    });

    let and = choice((just("&&"), just("&"), just("^"), just("∧")))
        .map(|_| Token::AND)
        .boxed();

    let or = choice((just("||"), just("|"), just("∨")))
        .map(|_| Token::OR)
        .boxed();

    let implication = choice((
        just("=>"),
        just("->"),
        just("⇒"),
        just("⟹"),
        just("→"),
        just("⊃"),
    ))
    .map(|_| Token::IMPLICATION)
    .boxed();

    let not = choice((just("~"), just("!"), just("¬")))
        .map(|_| Token::NOT)
        .boxed();

    let lround = just("(").map(|_| Token::LROUND).boxed();

    let rround = just(")").map(|_| Token::RROUND).boxed();

    let dot = just(".").map(|_| Token::DOT).boxed();

    let comma = just(",").map(|_| Token::COMMA).boxed();

    let colon = just(":").map(|_| Token::COLON).boxed();

    let exists = choice((just("∃"), just("\\exists")))
        .map(|_| Token::EXISTS)
        .boxed();

    let forall = choice((just("∀"), just("\\forall")))
        .map(|_| Token::FORALL)
        .boxed();

    let comment_single_line = just("//")
        .then(text::newline().not().repeated().then(text::newline()))
        .padded()
        .map(|_| ())
        .boxed();

    let comment_multi_line = just("/*")
        .then(just("*/").not().repeated().then(just("*/")))
        .padded()
        .map(|_| ())
        .boxed();

    let comment = choice((comment_single_line, comment_multi_line)).boxed();

    choice((
        truth,
        falsum,
        ident,
        and,
        or,
        implication,
        not,
        lround,
        rround,
        dot,
        comma,
        colon,
        forall,
        exists,
    ))
    .padded_by(comment.repeated())
    .padded()
    .repeated()
    .then_ignore(end())
    .boxed()
    .collect()
}
