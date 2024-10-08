use core::fmt;

pub mod fol;
pub mod lexer;
pub mod proof;
pub mod proof_term;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    IDENT(String),
    NUM(usize),
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
    SEMICOLON,
    TRUE,
    FALSE,

    FN,
    CASE,
    OF,
    LET,
    IN,
    EQUAL,
    ATOM,
    DATATYPE,
    SORRY,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::IDENT(s) => write!(f, "{}", s),
            Token::NUM(n) => write!(f, "NUM({})", n),
            Token::AND => write!(f, "∧"),
            Token::OR => write!(f, "∨"),
            Token::ARROW => write!(f, "=>"),
            Token::IMPLICATION => write!(f, "→"),
            Token::NOT => write!(f, "¬"),
            Token::LROUND => write!(f, "("),
            Token::RROUND => write!(f, ")"),
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
            Token::ATOM => write!(f, "atom"),
            Token::DATATYPE => write!(f, "datatype"),

            Token::SORRY => write!(f, "sorry"),
        }
    }
}
