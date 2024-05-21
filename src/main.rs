use chumsky::Parser;
use server::core::{parse::{fol::fol_parser, lexer::lexer}, prop::Prop};

fn main() {
    let prop = Prop::Impl(
        Box::new(Prop::And(
            Box::new(Prop::And(
                Box::new(Prop::Atom(String::from("A"), None)),
                Box::new(Prop::Impl(
                    Box::new(Prop::Atom(String::from("A"), None)),
                    Box::new(Prop::Atom(String::from("B"), None)),
                )),
            )),
            Box::new(Prop::Impl(
                Box::new(Prop::Atom(String::from("B"), None)),
                Box::new(Prop::Atom(String::from("C"), None)),
            )),
        )),
        Box::new(Prop::Atom(String::from("C"), None)),
    );

    let prop2 = Prop::Impl(
        Box::new(Prop::Atom(String::from("A"), None)),
        Box::new(Prop::Atom(String::from("A"), None)),
    );

    let src = std::fs::read_to_string("test.proof").unwrap();

    let tokens = lexer().parse(src.clone());
    println!("{:#?}", tokens);

    let ast = fol_parser().parse(tokens.unwrap());

    println!("{:#?}", ast);

    // println!("{}", prove(prop));
}
