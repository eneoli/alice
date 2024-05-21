use std::fmt;
use std::fmt::Debug;

use chumsky::prelude::*;

use crate::s;

/*
    Grammar:

    <Program> ::= <Pair>       |
                  "fst" <Pair> |
                  "snd" <Pair> |
                  "Function" <Program> |
                  "Function" |
                  "OrLeft" <Program> |
                  "OrRight" <Program> |
                  <Case> |
                  <Abort> |
                  "true" |
                  "false"

    <Pair> ::= "("<Program>, <Program>")"

    <Function> ::= "assuming <Ident> "as" <Prop> "show" <Prop> "{" <Program> "}"

    <Ident> ::= "a" | "b" | "c" | "d" | "e" | "f"

    <Prop> ::=  "A" | "B" | "C" | "D" | "E" | "F"
                "("<Prop>")"      |
                <Prop> "v" <Prop> |
                <Prop> "&" <Prop> |
                <Prop> "=>" <Prop>|
             "\forall" <Ident>":" <Prop>"." <Prop>|
             "\exists" <Ident>":" <Prop>"." <Prop>

    <Case> ::= "case" <Program> "of" "inL" <Program> "=>" <Program>";" "inR" <Program> "=>" <Program>";"
    <Abort> ::= "abort" "false"
*/

/*

   <Program> ::= <Pair>       |
                 "fst" <Pair> |
                 "snd" <Pair> |
                 "Function" <Program> |
                 "Function" |
                 "OrLeft" <Program> |
                 "OrRight" <Program> |
                 <Case> |
                 <Abort> |
                 "true" |
                 "false"

   <Pair> ::= "(" <Program> "," <Program> ")"

   <Function> ::= "assuming" <Ident> "as" <Prop> "show" <Prop> "{" <Program> "}"

   <Ident> ::= "a" | "b" | "c" | "d" | "e" | "f"

   <Prop> ::=  "A" | "B" | "C" | "D" | "E" | "F"
               "(" <Prop> ")"      |
               <Prop> "v" <Prop> |
               <Prop> "&" <Prop> |
               <Prop> "=>" <Prop> |
            "\forall" <Ident> ":" <Prop> "." <Prop> |
            "\exists" <Ident> ":" <Prop> "." <Prop>

   <Case> ::= "case" <Program> "of" "inL" <Program> "=>" <Program> ";" "inR" <Program> "=>" <Program> ";"
   <Abort> ::= "abort" "false"


*/

/*

<Program> ::= <Pair> |
              "fst" <Pair> | "snd" <Pair> |
              <Function> <Program> |
              <Function> |
              "OrLeft" <Program> |
              "OrRight" <Program> |
              <Case> |
              <Abort> |
              "true" |
              "false"

<Pair> ::= "(" <Program> "," <Program> ")"
<Function> ::= "assuming" <Ident> "as" <Prop> "show" <Prop> "{" <Program> "}"
<Ident> ::= "a" | "b" | "c" | "d" | "e" | "f"
<Prop> ::=  "A" | "B" | "C" | "D" | "E" | "F" | "(" <Prop> ")" | <Prop> "v" <Prop> | <Prop> "&" <Prop> | <Prop> "=>" <Prop> | "\\forall" <Ident> ":" <Prop> "." <Prop> | "\\exists" <Ident> ":" <Prop> "." <Prop>
<Case> ::= "case" <Program> "of" "inL" <Program> "=>" <Program> ";" "inR" <Program> "=>" <Program> ";"
<Abort> ::= "abort" "false"

 */

/* Tokens */
/*

<ident> ::= a | .. | z | .. // C-Style
*/

#[derive(Clone, PartialEq, Eq)]
pub enum Prop {
    Atom(String),
    And(Box<Prop>, Box<Prop>),
    Or(Box<Prop>, Box<Prop>),
    Impl(Box<Prop>, Box<Prop>),

    ForAll { ident: String, body: Box<Prop> },

    Exists { ident: String, body: Box<Prop> },

    True,
    False,
}

impl Prop {
    pub fn boxed(&self) -> Box<Self> {
        Box::new(self.clone())
    }
}

impl Debug for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Prop::Atom(ident) => ident.fmt(f),
            Prop::And(left, right) => write!(
                f,
                "({}) ∧ ({})",
                format!("{:?}", left),
                format!("{:?}", right)
            ),
            Prop::Or(left, right) => write!(
                f,
                "({}) ∨ ({})",
                format!("{:?}", left),
                format!("{:?}", right)
            ),
            Prop::Impl(left, right) => write!(
                f,
                "({}) => ({})",
                format!("{:?}", left),
                format!("{:?}", right)
            ),
            Prop::ForAll { ident, body } => write!(f, "∀{}. ({})", ident, format!("{:?}", body)),
            Prop::Exists { ident, body } => write!(f, "∃{}. ({})", ident, format!("{:?}", body)),

            Prop::True => write!(f, "T"),
            Prop::False => write!(f, "⊥"),
        }
    }
}

enum Expr {
    // (<first>, <right>)
    Pair(Box<Expr>, Box<Expr>),

    // assuming <ident> as <in_prop> show <show_prop> { <body> }
    Function {
        ident: String,
        in_prop: Prop,
        show_prop: Prop,
        body: Box<Expr>,
    },

    // <function> <applicant>
    Application {
        function: Box<Expr>,
        applicant: Box<Expr>,
    },

    // inL <expr>
    OrLeft(Box<Expr>),

    // inR <expr>
    OrRight(Box<Expr>),

    // case <expr> of inL <left_ident> => { <left_expr> } inR <right_ident> => { <right_expr> }
    Case {
        expr: Box<Expr>,

        left_ident: String,
        left_expr: Box<Expr>,

        right_ident: String,
        right_expr: Box<Expr>,
    },

    // abort <expr>
    Abort(Box<Expr>),

    // true
    True,

    // false
    False,
}

/*
    == Lexer ==
    -----------
*/

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Token {
    IDENT(String),
    AND,
    OR,
    IMPLICATION,
    NOT,
    LROUND,
    RROUND,
    EXISTS,
    FORALL,
    DOT,
    TRUE,
    FALSE,
}

pub fn lexer() -> impl Parser<char, Vec<Token>, Error = Simple<char>> {
    let truth = choice((
        just("true"),
        just("True"),
        just("⊤"),
        just("T"),
        just("\\top"),
    ))
    .map(|_| Token::TRUE);

    let falsum = choice((
        just("false"),
        just("False"),
        just("⊥"),
        just("\\bot"),
        just("\\bottom"),
    ))
    .map(|_| Token::FALSE);

    let ident = text::ident().map(|s| Token::IDENT(s));

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

    let comment = choice((comment_single_line, comment_multi_line));

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

/*
    == FOL Parser ==
    ----------------

    Prop = Implication ;

    Implication = { Or, "=>" }, (Or | Quantor) ;

    Or          = And, { "||", (And | Quantor) } ;

    And         = Not, { "&&", (Not | Quantor) } ;

    Not         = { "~" }, Atom ;

    Atom        = ⊤ | ⊥ | Ident | "(", Prop, ")" ;

    Quantor     = Allquant | Existsquant ;

    Allquant    = "∀", Ident, ".", Prop ;

    Existsquant = "∃", Ident, ".", Prop ;
*/

pub fn fol_parser() -> impl Parser<Token, Prop, Error = Simple<Token>> {
    let ident = select! { Token::IDENT(ident) => ident };

    let prop = recursive(|prop: Recursive<Token, Prop, Simple<Token>>| {
        let allquant = just(Token::FORALL)
            .ignore_then(ident)
            .then_ignore(just(Token::DOT))
            .then(prop.clone())
            .map(|(ident, body)| Prop::ForAll {
                ident,
                body: Box::new(body.clone()),
            })
            .boxed();

        let existsquant = just(Token::EXISTS)
            .ignore_then(ident)
            .then_ignore(just(Token::DOT))
            .then(prop.clone())
            .map(|(ident, body)| Prop::Exists {
                ident,
                body: Box::new(body.clone()),
            })
            .boxed();

        let quantor = choice((existsquant, allquant)).boxed();

        let atom = ident
            .map(Prop::Atom)
            .or(prop
                .clone()
                .delimited_by(just(Token::LROUND), just(Token::RROUND)))
            .or(just(Token::TRUE).map(|_| Prop::True))
            .or(just(Token::FALSE).map(|_| Prop::False))
            .boxed();

        let not = just(Token::NOT)
            .repeated()
            .then(atom)
            .foldr(|_op, rhs| Prop::Impl(Box::new(rhs), Box::new(Prop::False)))
            .boxed();

        let and = not
            .clone()
            .then(
                just(Token::AND)
                    .to(Prop::And)
                    .then(choice((not, quantor.clone())))
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)))
            .boxed();

        let or = and
            .clone()
            .then(
                just(Token::OR)
                    .to(Prop::Or)
                    .then(choice((and, quantor.clone())))
                    .repeated(),
            )
            .foldl(|lhs, (op, rhs)| op(Box::new(lhs), Box::new(rhs)))
            .boxed();

        let implication = or
            .clone()
            .then(just(Token::IMPLICATION).to(Prop::Impl))
            .repeated()
            .then(choice((or, quantor.clone())))
            .foldr(|(lhs, op), rhs| op(Box::new(lhs), Box::new(rhs)))
            .boxed();

        implication
    });

    prop.then_ignore(end())
}

// TESTS
// =====

#[test]
fn test_simple_prop() {
    let token = lexer().parse("A").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(ast, Prop::Atom(String::from("A")));
}

#[test]
fn test_simple_not() {
    let token = lexer().parse("~A").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::Impl(Prop::Atom(String::from("A")).boxed(), Prop::False.boxed())
    );
}

#[test]
fn test_chained_not() {
    let token = lexer().parse("~!¬A").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::Impl(
            Prop::Impl(
                Prop::Impl(Prop::Atom(String::from("A")).boxed(), Prop::False.boxed()).boxed(),
                Prop::False.boxed()
            )
            .boxed(),
            Prop::False.boxed()
        )
    );
}

#[test]
fn test_simple_and() {
    let token = lexer().parse("A & B").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::And(
            Box::new(Prop::Atom(String::from("A"))),
            Box::new(Prop::Atom(String::from("B"))),
        )
    );
}

#[test]
fn test_and_implicit_left_associative() {
    let token = lexer().parse("A & B & C").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::And(
            Box::new(Prop::And(
                Box::new(Prop::Atom(String::from("A"))),
                Box::new(Prop::Atom(String::from("B")))
            )),
            Box::new(Prop::Atom(String::from("C"))),
        )
    );
}

#[test]
fn test_and_explicit_left_associative() {
    let token = lexer().parse("A & (B & C)").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::And(
            Box::new(Prop::Atom(String::from("A"))),
            Box::new(Prop::And(
                Box::new(Prop::Atom(String::from("B"))),
                Box::new(Prop::Atom(String::from("C")))
            )),
        )
    );
}

#[test]
fn test_precedence_propositional_logic() {
    let token = lexer().parse("A || B && ~C => D").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::Impl(
            Prop::Or(
                Prop::Atom(s!("A")).boxed(),
                Prop::And(
                    Prop::Atom(s!("B")).boxed(),
                    Prop::Impl(Prop::Atom(s!("C")).boxed(), Prop::False.boxed()).boxed()
                )
                .boxed()
            )
            .boxed(),
            Prop::Atom(s!("D")).boxed()
        )
    )
}

#[test]
fn test_global_forall() {
    let token = lexer().parse("\\forall x. A => B").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::ForAll {
            ident: String::from("x"),
            body: Prop::Impl(
                Prop::Atom(format!("A")).boxed(),
                Prop::Atom(format!("B")).boxed()
            )
            .boxed()
        }
    );
}

#[test]
fn test_global_exists() {
    let token = lexer().parse("\\exists x. A => B").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::Exists {
            ident: String::from("x"),
            body: Prop::Impl(
                Prop::Atom(format!("A")).boxed(),
                Prop::Atom(format!("B")).boxed()
            )
            .boxed()
        }
    );
}

#[test]
fn test_left_forall() {
    let token = lexer().parse("A && \\forall x. A => B").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::And(
            Prop::Atom(format!("A")).boxed(),
            Prop::ForAll {
                ident: format!("x"),
                body: Prop::Impl(Prop::Atom(s!("A")).boxed(), Prop::Atom(s!("B")).boxed()).boxed()
            }
            .boxed()
        )
    )
}

#[test]
fn test_left_exists() {
    let token = lexer().parse("A && \\exists x. A => B").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::And(
            Prop::Atom(format!("A")).boxed(),
            Prop::Exists {
                ident: format!("x"),
                body: Prop::Impl(Prop::Atom(s!("A")).boxed(), Prop::Atom(s!("B")).boxed()).boxed()
            }
            .boxed()
        )
    )
}

#[test]
fn test_nested_forall() {
    let token = lexer().parse("A && (\\forall x. x) && C").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::And(
            Prop::And(
                Prop::Atom(s!("A")).boxed(),
                Prop::ForAll {
                    ident: s!("x"),
                    body: Prop::Atom(s!("x")).boxed()
                }
                .boxed()
            )
            .boxed(),
            Prop::Atom(s!("C")).boxed()
        )
    );
}

#[test]
fn test_nested_exists() {
    let token = lexer().parse("A && (\\exists x. x) && C").unwrap();
    let ast = fol_parser().parse(token).unwrap();

    assert_eq!(
        ast,
        Prop::And(
            Prop::And(
                Prop::Atom(s!("A")).boxed(),
                Prop::Exists {
                    ident: s!("x"),
                    body: Prop::Atom(s!("x")).boxed()
                }
                .boxed()
            )
            .boxed(),
            Prop::Atom(s!("C")).boxed()
        )
    );
}
