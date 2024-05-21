use crate::core::Prop;

enum Ast {
    Pair(Pair),
    Fst(Fst),
    Snd(Snd),
    True,
    False,
    Abs(Abs),
    App(App),
    InL(),
    InR(),
    Case(Case),
    Abort,
    Ident(Ident),
}

struct Pair {
    fst: Box<Ast>,
    snd: Box<Ast>,
}

struct Fst {
    child: Pair,
}

struct Snd {
    child: Pair,
}

struct Ident {}

struct Abs {
    variable: Ident,
    assuming_prop: Prop,
    show_prop: Prop,
    body: Box<Ast>,
}

struct App {
    function: Abs,
    parameter: Box<Ast>,
}

struct Case {
    variable: Ident,
    leftCase: Box<Ast>,
    rightCase: Box<Ast>,
}

pub enum Certificate {
    Pair(Box<Certificate>, Box<Certificate>),
    Fst(Box<Certificate>),
    Snd(Box<Certificate>),
    True,
    Abs(Box<Certificate>, Box<Certificate>),
    App(Box<Certificate>, Box<Certificate>),
    Left(Box<Certificate>),
    Right(Box<Certificate>),
}

pub fn test() {
    let proof = Abs {
        variable: panic!(),
        assuming_prop: panic!(),
        show_prop: panic!(),
        body: panic!(),
    };
}
