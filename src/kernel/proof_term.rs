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
    pub fn as_str(&self) -> &str {
        &self.0.as_str()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Pair(pub Box<ProofTerm>, pub Box<ProofTerm>);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct ProjectFst(pub Box<ProofTerm>);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct ProjectSnd(pub Box<ProofTerm>);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Function {
    pub param_ident: String,
    pub param_type: Option<Type>,
    pub body: Box<ProofTerm>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Application {
    pub function: Box<ProofTerm>,
    pub applicant: Box<ProofTerm>,
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

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct OrRight(pub Box<ProofTerm>);

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Case {
    pub head: Box<ProofTerm>,

    pub fst_ident: String,
    pub fst_term: Box<ProofTerm>,

    pub snd_ident: String,
    pub snd_term: Box<ProofTerm>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Abort(pub Box<ProofTerm>);

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
            ProofTerm::TypeAscription(type_ascription) => visitor.visit_type_ascription(type_ascription),
            ProofTerm::Unit => visitor.visit_unit(),
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
