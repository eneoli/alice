use core::fmt;

pub mod fol;
pub mod lexer;
pub mod proof;
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
    LANGLE,
    RANGLE,
    EXISTS,
    FORALL,
    DOT,
    COMMA,
    COLON,
    SEMICOLON,
    TRUE,
    FALSE,

    FN,
    CASE,
    OF,
    LET,
    IN,
    EQUAL,
    DATATYPE,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::IDENT(s) => write!(f, "{}", s),
            Token::AND => write!(f, "∧"),
            Token::OR => write!(f, "∨"),
            Token::ARROW => write!(f, "=>"),
            Token::IMPLICATION => write!(f, "→"),
            Token::NOT => write!(f, "¬"),
            Token::LROUND => write!(f, "("),
            Token::RROUND => write!(f, ")"),
            Token::LANGLE => write!(f, "<"),
            Token::RANGLE => write!(f, ">"),
            Token::EXISTS => write!(f, "∃"),
            Token::FORALL => write!(f, "∀"),
            Token::DOT => write!(f, "."),
            Token::COMMA => write!(f, ","),
            Token::COLON => write!(f, ":"),
            Token::SEMICOLON => write!(f, ";"),
            Token::TRUE => write!(f, "⊤"),
            Token::FALSE => write!(f, "⊥"),

            Token::FN => write!(f, "fn"),
            Token::CASE => write!(f, "case"),
            Token::OF => write!(f, "of"),
            Token::LET => write!(f, "let"),
            Token::IN => write!(f, "in"),
            Token::EQUAL => write!(f, "="),
            Token::DATATYPE => write!(f, "datatype"),
        }
    }
}
