use std::path::MAIN_SEPARATOR;

use chumsky::prelude::*;

use server::core::parse::{lexer, fol_parser};
use server::core::Prop;
use server::core::Prop::Implication;

fn main() {
    let prop = Implication(
        Box::new(Prop::And(
            Box::new(Prop::And(
                Box::new(Prop::Atom(String::from("A"))),
                Box::new(Implication(
                    Box::new(Prop::Atom(String::from("A"))),
                    Box::new(Prop::Atom(String::from("B"))),
                )),
            )),
            Box::new(Implication(
                Box::new(Prop::Atom(String::from("B"))),
                Box::new(Prop::Atom(String::from("C"))),
            )),
        )),
        Box::new(Prop::Atom(String::from("C"))),
    );

    let prop2 = Implication(
        Box::new(Prop::Atom(String::from("A"))),
        Box::new(Prop::Atom(String::from("A"))),
    );

    let src = std::fs::read_to_string("test.proof").unwrap();

    let tokens = lexer().parse(src.clone());
    println!("{:#?}", tokens);

    let ast = fol_parser().parse(tokens.unwrap());

    println!("{:#?}", ast);

    // println!("{}", prove(prop));
}
