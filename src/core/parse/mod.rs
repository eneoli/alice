pub mod lexer;
pub mod fol;
pub mod proof_term;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    IDENT(String),
    AND,
    OR,
    ARROW,
    IMPLICATION,
    NOT,
    LROUND,
    RROUND,
    EXISTS,
    FORALL,
    DOT,
    COMMA,
    COLON,
    TRUE,
    FALSE,

    FN,
    CASE,
    OF,
    LET,
    IN,
    EQUAL,
}
