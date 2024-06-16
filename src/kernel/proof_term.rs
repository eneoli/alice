use super::prop::Prop;
use serde::{Serialize, Deserialize};
use tsify_next::Tsify;

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum Type {
    Prop(Prop),
    Datatype(String),
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
    Unit,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
#[serde(tag = "kind", content = "value")]
pub enum ProofTerm {
    Ident(String),
    Pair(Box<ProofTerm>, Box<ProofTerm>),
    ProjectFst(Box<ProofTerm>),
    ProjectSnd(Box<ProofTerm>),
    Function {
        param_ident: String,
        param_type: Type,
        body: Box<ProofTerm>,
    },
    Application {
        function: Box<ProofTerm>,
        applicant: Box<ProofTerm>,
    },
    LetIn {
        fst_ident: String,
        snd_ident: String,
        pair_proof_term: Box<ProofTerm>,
        body: Box<ProofTerm>,
    },
    OrLeft {
        body: Box<ProofTerm>,
        other: Prop,
    },
    OrRight {
        body: Box<ProofTerm>,
        other: Prop,
    },
    Case {
        proof_term: Box<ProofTerm>,

        left_ident: String,
        left_term: Box<ProofTerm>,

        right_ident: String,
        right_term: Box<ProofTerm>,
    },
    Abort(Box<ProofTerm>),
    Unit,
}

impl ProofTerm {
    pub fn boxed(&self) -> Box<Self> {
        Box::new(self.clone())
    }

    pub fn visit<R>(&self, visitor: &mut impl ProofTermVisitor<R>) -> R {
        match self {
            ProofTerm::Ident(ident) => visitor.visit_ident(ident),
            ProofTerm::Pair(fst, snd) => visitor.visit_pair(fst, snd),
            ProofTerm::ProjectFst(body) => visitor.visit_project_fst(body),
            ProofTerm::ProjectSnd(body) => visitor.visit_project_snd(body),
            ProofTerm::Function {
                param_ident,
                param_type,
                body,
            } => visitor.visit_function(param_ident, param_type, body),
            ProofTerm::Application {
                function,
                applicant,
            } => visitor.visit_application(function, applicant),
            ProofTerm::LetIn {
                fst_ident,
                snd_ident,
                pair_proof_term,
                body,
            } => visitor.visit_let_in(fst_ident, snd_ident, pair_proof_term, body),
            ProofTerm::OrLeft { body, other } => visitor.visit_or_left(body, other),
            ProofTerm::OrRight { body, other } => visitor.visit_or_right(body, other),
            ProofTerm::Case {
                proof_term,
                left_ident,
                left_term,
                right_ident,
                right_term,
            } => visitor.visit_case(proof_term, left_ident, left_term, right_ident, right_term),
            ProofTerm::Abort(body) => visitor.visit_abort(body),
            ProofTerm::Unit => visitor.visit_unit(),
        }
    }
}

pub trait ProofTermVisitor<R> {
    fn visit_ident(&mut self, ident: &String) -> R;
    fn visit_pair(&mut self, fst: &ProofTerm, snd: &ProofTerm) -> R;
    fn visit_project_fst(&mut self, body: &ProofTerm) -> R;
    fn visit_project_snd(&mut self, body: &ProofTerm) -> R;
    fn visit_function(&mut self, param_ident: &String, param_type: &Type, body: &ProofTerm) -> R;
    fn visit_application(&mut self, function: &ProofTerm, applicant: &ProofTerm) -> R;
    fn visit_let_in(
        &mut self,
        fst_ident: &String,
        snd_ident: &String,
        pair_proof_term: &ProofTerm,
        body: &ProofTerm,
    ) -> R;
    fn visit_or_left(&mut self, body: &ProofTerm, other: &Prop) -> R;
    fn visit_or_right(&mut self, body: &ProofTerm, other: &Prop) -> R;
    fn visit_case(
        &mut self,
        proof_term: &ProofTerm,
        left_ident: &String,
        left_term: &ProofTerm,
        right_ident: &String,
        right_term: &ProofTerm,
    ) -> R;
    fn visit_abort(&mut self, body: &ProofTerm) -> R;
    fn visit_unit(&mut self) -> R;
}
