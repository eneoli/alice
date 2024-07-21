use core::fmt;
use std::fmt::Display;

use super::prop::Prop;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum Type {
    Prop(Prop),
    Datatype(String),
}

impl Type {
    pub fn is_prop(&self) -> bool {
        match self {
            Type::Prop(_) => true,
            Type::Datatype(_) => false,
        }
    }

    pub fn is_datatype(&self) -> bool {
        match self {
            Type::Prop(_) => false,
            Type::Datatype(_) => true,
        }
    }

    pub fn has_free_parameters(&self) -> bool {
        match self {
            Type::Prop(prop) => prop.has_free_parameters(),
            Type::Datatype(_) => false,
        }
    }

    pub fn alpha_eq(&self, other: &Type) -> bool {
        match (self, other) {
            (Type::Datatype(ld), Type::Datatype(rd)) => ld == rd,
            (Type::Prop(lprop), Type::Prop(rprop)) => Prop::alpha_eq(lprop, rprop),
            _ => false,
        }
    }
}

impl Into<Prop> for Type {
    fn into(self) -> Prop {
        if let Type::Prop(_type) = self {
            return _type;
        }

        panic!("Type is not a Prop.");
    }
}

#[derive(Clone, PartialEq, Eq, Tsify, Serialize, Deserialize, Debug)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTermKind {
    Ident,
    Pair,
    ExistsPair,
    ProjectFst,
    ProjectSnd,
    Function,
    Application,
    LetIn,
    OrLeft,
    OrRight,
    Case,
    Abort,
    TypeAscription,
    Unit,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Ident(pub String);

impl Ident {
    pub fn create(ident: String) -> ProofTerm {
        ProofTerm::Ident(Self(ident))
    }

    pub fn as_str(&self) -> &str {
        &self.0.as_str()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Pair(pub Box<ProofTerm>, pub Box<ProofTerm>);

impl Pair {
    pub fn create(fst: Box<ProofTerm>, snd: Box<ProofTerm>) -> ProofTerm {
        ProofTerm::Pair(Self(fst, snd))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct ProjectFst(pub Box<ProofTerm>);

impl ProjectFst {
    pub fn create(body: Box<ProofTerm>) -> ProofTerm {
        ProofTerm::ProjectFst(ProjectFst(body))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct ProjectSnd(pub Box<ProofTerm>);

impl ProjectSnd {
    pub fn create(body: Box<ProofTerm>) -> ProofTerm {
        ProofTerm::ProjectSnd(ProjectSnd(body))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Function {
    pub param_ident: String,
    pub param_type: Option<Type>,
    pub body: Box<ProofTerm>,
}

impl Function {
    pub fn create(
        param_ident: String,
        param_type: Option<Type>,
        body: Box<ProofTerm>,
    ) -> ProofTerm {
        ProofTerm::Function(Function {
            param_ident,
            param_type,
            body,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Application {
    pub function: Box<ProofTerm>,
    pub applicant: Box<ProofTerm>,
}

impl Application {
    pub fn create(function: Box<ProofTerm>, applicant: Box<ProofTerm>) -> ProofTerm {
        ProofTerm::Application(Application {
            function,
            applicant,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct LetIn {
    pub fst_ident: String,
    pub snd_ident: String,
    pub head: Box<ProofTerm>,
    pub body: Box<ProofTerm>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct OrLeft(pub Box<ProofTerm>);

impl OrLeft {
    pub fn create(body: Box<ProofTerm>) -> ProofTerm {
        ProofTerm::OrLeft(OrLeft(body))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct OrRight(pub Box<ProofTerm>);

impl OrRight {
    pub fn create(body: Box<ProofTerm>) -> ProofTerm {
        ProofTerm::OrRight(OrRight(body))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Case {
    pub head: Box<ProofTerm>,

    pub fst_ident: String,
    pub fst_term: Box<ProofTerm>,

    pub snd_ident: String,
    pub snd_term: Box<ProofTerm>,
}

impl Case {
    pub fn create(
        head: Box<ProofTerm>,
        fst_ident: String,
        fst_term: Box<ProofTerm>,
        snd_ident: String,
        snd_term: Box<ProofTerm>,
    ) -> ProofTerm {
        ProofTerm::Case(Case {
            head,
            fst_ident,
            fst_term,
            snd_ident,
            snd_term,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Abort(pub Box<ProofTerm>);

impl Abort {
    pub fn create(body: Box<ProofTerm>) -> ProofTerm {
        ProofTerm::Abort(Abort(body))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct TypeAscription {
    pub proof_term: Box<ProofTerm>,
    pub ascription: Type,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTerm {
    Ident(Ident),
    Pair(Pair),
    ProjectFst(ProjectFst),
    ProjectSnd(ProjectSnd),
    Function(Function),
    Application(Application),
    LetIn(LetIn),
    OrLeft(OrLeft),
    OrRight(OrRight),
    Case(Case),
    Abort(Abort),
    TypeAscription(TypeAscription),
    Unit,
}

impl ProofTerm {
    pub fn boxed(&self) -> Box<Self> {
        Box::new(self.clone())
    }

    pub fn visit<R>(&self, visitor: &mut impl ProofTermVisitor<R>) -> R {
        match self {
            ProofTerm::Ident(ident) => visitor.visit_ident(ident),
            ProofTerm::Pair(pair) => visitor.visit_pair(pair),
            ProofTerm::ProjectFst(project_fst) => visitor.visit_project_fst(project_fst),
            ProofTerm::ProjectSnd(project_snd) => visitor.visit_project_snd(project_snd),
            ProofTerm::Function(function) => visitor.visit_function(function),
            ProofTerm::Application(application) => visitor.visit_application(application),
            ProofTerm::LetIn(let_in) => visitor.visit_let_in(let_in),
            ProofTerm::OrLeft(or_left) => visitor.visit_or_left(or_left),
            ProofTerm::OrRight(or_right) => visitor.visit_or_right(or_right),
            ProofTerm::Case(case) => visitor.visit_case(case),
            ProofTerm::Abort(abort) => visitor.visit_abort(abort),
            ProofTerm::TypeAscription(type_ascription) => {
                visitor.visit_type_ascription(type_ascription)
            }
            ProofTerm::Unit => visitor.visit_unit(),
        }
    }
}

impl Display for ProofTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProofTerm::Unit => write!(f, "()"),
            ProofTerm::Ident(Ident(ident)) => write!(f, "{}", ident),
            ProofTerm::Pair(Pair(fst, snd)) => write!(f, "({}, {})", fst, snd),
            ProofTerm::ProjectFst(ProjectFst(body)) => write!(f, "fst {}", body),
            ProofTerm::ProjectSnd(ProjectSnd(body)) => write!(f, "snd {}", body),
            ProofTerm::Abort(Abort(body)) => write!(f, "abort {}", body),
            ProofTerm::OrLeft(OrLeft(body)) => write!(f, "inl {}", body),
            ProofTerm::OrRight(OrRight(body)) => write!(f, "inr {}", body),
            ProofTerm::Case(Case {
                head,
                fst_ident,
                fst_term,
                snd_ident,
                snd_term,
            }) => {
                write!(
                    f,
                    "case {} of inl {} => {}, inr {} => {}",
                    head, fst_ident, fst_term, snd_ident, snd_term
                )
            }
            ProofTerm::Function(Function {
                param_ident,
                param_type,
                body,
            }) => {
                if let Some(param_type) = param_type {
                    write!(f, "fn {}: {:?} => {}", param_ident, param_type, body)
                } else {
                    write!(f, "fn {} => {}", param_ident, body)
                }
            }
            ProofTerm::Application(Application {
                function,
                applicant,
            }) => write!(f, "({}) ({})", function, applicant),
            ProofTerm::LetIn(LetIn {
                fst_ident,
                snd_ident,
                head,
                body,
            }) => write!(
                f,
                "let ({}, {}) = {} in {}",
                fst_ident, snd_ident, head, body
            ),
            ProofTerm::TypeAscription(TypeAscription {
                proof_term,
                ascription,
            }) => write!(f, "{}: {:?}", proof_term, ascription),
        }
    }
}

pub trait ProofTermVisitor<R> {
    fn visit_ident(&mut self, ident: &Ident) -> R;
    fn visit_pair(&mut self, pair: &Pair) -> R;
    fn visit_project_fst(&mut self, project_fst: &ProjectFst) -> R;
    fn visit_project_snd(&mut self, project_snd: &ProjectSnd) -> R;
    fn visit_function(&mut self, function: &Function) -> R;
    fn visit_application(&mut self, application: &Application) -> R;
    fn visit_let_in(&mut self, let_in: &LetIn) -> R;
    fn visit_or_left(&mut self, or_left: &OrLeft) -> R;
    fn visit_or_right(&mut self, or_right: &OrRight) -> R;
    fn visit_case(&mut self, case: &Case) -> R;
    fn visit_abort(&mut self, abort: &Abort) -> R;
    fn visit_type_ascription(&mut self, type_ascription: &TypeAscription) -> R;
    fn visit_unit(&mut self) -> R;
}
