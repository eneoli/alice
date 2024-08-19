use core::fmt;
use std::fmt::Display;
use std::ops::Range;

use super::{
    checker::identifier_context::IdentifierContext,
    prop::{InstatiationError, Prop},
};
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

    pub fn has_quantifiers(&self) -> bool {
        match self {
            Type::Prop(prop) => prop.has_quantifiers(),
            Type::Datatype(_) => false,
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

    pub fn instantiate_parameters_with_context(
        &mut self,
        ctx: &IdentifierContext,
    ) -> Result<(), InstatiationError> {
        let Type::Prop(prop) = self else {
            return Ok(());
        };

        prop.instantiate_parameters_with_context(ctx)
    }
}

impl From<Type> for Prop {
    fn from(val: Type) -> Self {
        if let Type::Prop(_type) = val {
            return _type;
        }

        panic!("Type is not a Prop.");
    }
}

impl Display for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Type::Prop(prop) => write!(f, "{}", prop),
            Type::Datatype(datatype) => write!(f, "{}", datatype),
        }
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
    Sorry,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Ident(pub String, pub Option<Range<usize>>);

impl Ident {
    pub fn create(ident: String) -> ProofTerm {
        ProofTerm::Ident(Self(ident, None))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Pair(
    pub Box<ProofTerm>,
    pub Box<ProofTerm>,
    pub Option<Range<usize>>,
);

impl Pair {
    pub fn create(
        fst: Box<ProofTerm>,
        snd: Box<ProofTerm>,
        span: Option<Range<usize>>,
    ) -> ProofTerm {
        ProofTerm::Pair(Self(fst, snd, span))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct ProjectFst(pub Box<ProofTerm>, pub Option<Range<usize>>);

impl ProjectFst {
    pub fn create(body: Box<ProofTerm>, span: Option<Range<usize>>) -> ProofTerm {
        ProofTerm::ProjectFst(ProjectFst(body, span))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct ProjectSnd(pub Box<ProofTerm>, pub Option<Range<usize>>);

impl ProjectSnd {
    pub fn create(body: Box<ProofTerm>, span: Option<Range<usize>>) -> ProofTerm {
        ProofTerm::ProjectSnd(ProjectSnd(body, span))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Function {
    pub param_ident: String,
    pub param_type: Option<Type>,
    pub body: Box<ProofTerm>,
    pub span: Option<Range<usize>>,
}

impl Function {
    pub fn create(
        param_ident: String,
        param_type: Option<Type>,
        body: Box<ProofTerm>,
        span: Option<Range<usize>>,
    ) -> ProofTerm {
        ProofTerm::Function(Function {
            param_ident,
            param_type,
            body,
            span,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Application {
    pub function: Box<ProofTerm>,
    pub applicant: Box<ProofTerm>,
    pub span: Option<Range<usize>>,
}

impl Application {
    pub fn create(
        function: Box<ProofTerm>,
        applicant: Box<ProofTerm>,
        span: Option<Range<usize>>,
    ) -> ProofTerm {
        ProofTerm::Application(Application {
            function,
            applicant,
            span,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct LetIn {
    pub fst_ident: String,
    pub snd_ident: String,
    pub head: Box<ProofTerm>,
    pub body: Box<ProofTerm>,
    pub span: Option<Range<usize>>,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct OrLeft(pub Box<ProofTerm>, pub Option<Range<usize>>);

impl OrLeft {
    pub fn create(body: Box<ProofTerm>, span: Option<Range<usize>>) -> ProofTerm {
        ProofTerm::OrLeft(OrLeft(body, span))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct OrRight(pub Box<ProofTerm>, pub Option<Range<usize>>);

impl OrRight {
    pub fn create(body: Box<ProofTerm>, span: Option<Range<usize>>) -> ProofTerm {
        ProofTerm::OrRight(OrRight(body, span))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Case {
    pub head: Box<ProofTerm>,

    pub fst_ident: String,
    pub fst_term: Box<ProofTerm>,

    pub snd_ident: String,
    pub snd_term: Box<ProofTerm>,

    pub span: Option<Range<usize>>,
}

impl Case {
    pub fn create(
        head: Box<ProofTerm>,
        fst_ident: String,
        fst_term: Box<ProofTerm>,
        snd_ident: String,
        snd_term: Box<ProofTerm>,
        span: Option<Range<usize>>,
    ) -> ProofTerm {
        ProofTerm::Case(Case {
            head,
            fst_ident,
            fst_term,
            snd_ident,
            snd_term,
            span,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct Abort(pub Box<ProofTerm>, pub Option<Range<usize>>);

impl Abort {
    pub fn create(body: Box<ProofTerm>, span: Option<Range<usize>>) -> ProofTerm {
        ProofTerm::Abort(Abort(body, span))
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
pub struct TypeAscription {
    pub proof_term: Box<ProofTerm>,
    pub ascription: Type,
    pub span: Option<Range<usize>>,
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
    Unit(Option<Range<usize>>),
    Sorry(Option<Range<usize>>),
}

impl ProofTerm {
    pub fn boxed(&self) -> Box<Self> {
        Box::new(self.clone())
    }

    pub fn span(&self) -> &Option<Range<usize>> {
        match self {
            ProofTerm::Ident(Ident(_, span)) => span,
            ProofTerm::Pair(Pair(_, _, span)) => span,
            ProofTerm::ProjectFst(ProjectFst(_, span)) => span,
            ProofTerm::ProjectSnd(ProjectSnd(_, span)) => span,
            ProofTerm::Function(Function { span, .. }) => span,
            ProofTerm::Application(Application { span, .. }) => span,
            ProofTerm::LetIn(LetIn { span, .. }) => span,
            ProofTerm::OrLeft(OrLeft(_, span)) => span,
            ProofTerm::OrRight(OrRight(_, span)) => span,
            ProofTerm::Case(Case { span, .. }) => span,
            ProofTerm::Abort(Abort(_, span)) => span,
            ProofTerm::TypeAscription(TypeAscription { span, .. }) => span,
            ProofTerm::Unit(span) => span,
            ProofTerm::Sorry(span) => span,
        }
    }

    pub fn precedence(&self) -> usize {
        match self {
            ProofTerm::Unit(_) => 999,
            ProofTerm::Ident(_) => 999,
            ProofTerm::Sorry(_) => 999,
            ProofTerm::Abort(_) => 3,
            ProofTerm::Pair(_) => 999,
            ProofTerm::ProjectFst(_) => 3,
            ProofTerm::ProjectSnd(_) => 3,
            ProofTerm::OrLeft(_) => 3,
            ProofTerm::OrRight(_) => 3,
            ProofTerm::Case(_) => 999,
            ProofTerm::Function(_) => 1,
            ProofTerm::Application(_) => 3,
            ProofTerm::LetIn(_) => 999,
            ProofTerm::TypeAscription(_) => 2,
        }
    }

    pub fn right_associative(&self) -> bool {
        match self {
            ProofTerm::Unit(_) => false,
            ProofTerm::Ident(_) => false,
            ProofTerm::Sorry(_) => false,
            ProofTerm::Abort(_) => false,
            ProofTerm::Pair(_) => false,
            ProofTerm::ProjectFst(_) => false,
            ProofTerm::ProjectSnd(_) => false,
            ProofTerm::OrLeft(_) => false,
            ProofTerm::OrRight(_) => false,
            ProofTerm::Case(_) => false,
            ProofTerm::Function(_) => true,
            ProofTerm::Application(_) => false,
            ProofTerm::LetIn(_) => false,
            ProofTerm::TypeAscription(_) => false,
        }
    }

    pub fn left_associative(&self) -> bool {
        match self {
            ProofTerm::Unit(_) => false,
            ProofTerm::Ident(_) => false,
            ProofTerm::Sorry(_) => false,
            ProofTerm::Abort(_) => true,
            ProofTerm::Pair(_) => false,
            ProofTerm::ProjectFst(_) => true,
            ProofTerm::ProjectSnd(_) => true,
            ProofTerm::OrLeft(_) => true,
            ProofTerm::OrRight(_) => true,
            ProofTerm::Case(_) => false,
            ProofTerm::Function(_) => false,
            ProofTerm::Application(_) => true,
            ProofTerm::LetIn(_) => false,
            ProofTerm::TypeAscription(_) => true,
        }
    }

    pub fn annotation_count(&self) -> usize {
        match self {
            ProofTerm::Ident(_) => 0,
            ProofTerm::Pair(Pair(fst, snd, _)) => fst.annotation_count() + snd.annotation_count(),
            ProofTerm::ProjectFst(ProjectFst(body, _)) => body.annotation_count(),
            ProofTerm::ProjectSnd(ProjectSnd(body, _)) => body.annotation_count(),
            ProofTerm::Function(Function {
                param_type, body, ..
            }) => {
                let param_additon = if param_type.is_some() { 1 } else { 0 };

                param_additon + body.annotation_count()
            }
            ProofTerm::Application(Application {
                function,
                applicant,
                ..
            }) => function.annotation_count() + applicant.annotation_count(),
            ProofTerm::LetIn(LetIn { head, body, .. }) => {
                head.annotation_count() + body.annotation_count()
            }
            ProofTerm::OrLeft(OrLeft(body, _)) => body.annotation_count(),
            ProofTerm::OrRight(OrRight(body, _)) => body.annotation_count(),
            ProofTerm::Case(Case {
                head,
                fst_term,
                snd_term,
                ..
            }) => {
                head.annotation_count() + fst_term.annotation_count() + snd_term.annotation_count()
            }
            ProofTerm::Abort(Abort(body, _)) => body.annotation_count(),
            ProofTerm::TypeAscription(TypeAscription { proof_term, .. }) => {
                1 + proof_term.annotation_count()
            }
            ProofTerm::Unit(_) => 0,
            ProofTerm::Sorry(_) => 0,
        }
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
            ProofTerm::Unit(span) => visitor.visit_unit(span.clone()),
            ProofTerm::Sorry(span) => visitor.visit_sorry(span.clone()),
        }
    }
}

impl Display for ProofTerm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let ProofTerm::Sorry(_) = self {
            return write!(f, "sorry");
        }

        if let ProofTerm::Unit(_) = self {
            return write!(f, "()");
        }

        if let ProofTerm::Ident(Ident(ident, _)) = self {
            return write!(f, "{}", ident);
        }

        if let ProofTerm::Pair(Pair(fst, snd, _)) = self {
            return write!(f, "({}, {})", fst, snd);
        }

        if let ProofTerm::Function(Function {
            param_ident,
            param_type,
            body,
            ..
        }) = self
        {
            if let Some(param_type) = param_type {
                return write!(f, "fn {}: {} => {}", param_ident, param_type, body);
            }

            return write!(f, "fn {} => {}", param_ident, body);
        }

        if let ProofTerm::Case(Case {
            head,
            fst_ident,
            fst_term,
            snd_ident,
            snd_term,
            ..
        }) = self
        {
            return write!(
                f,
                "case {} of inl {} => {}, inr {} => {}",
                head, fst_ident, fst_term, snd_ident, snd_term
            );
        }

        if let ProofTerm::LetIn(LetIn {
            fst_ident,
            snd_ident,
            head,
            body,
            ..
        }) = self
        {
            return write!(
                f,
                "let ({}, {}) = {} in {}",
                fst_ident, snd_ident, head, body
            );
        }

        if let ProofTerm::Application(Application {
            function,
            applicant,
            ..
        }) = self
        {
            let own_precedence = self.precedence();
            let function_precedence = function.precedence();
            let applicant_precedence = applicant.precedence();

            let should_wrap_left = (function_precedence < own_precedence)
                || ((function_precedence == own_precedence) && function.right_associative());

            let should_wrap_right = (applicant_precedence < own_precedence)
                || ((applicant_precedence == own_precedence) && applicant.left_associative());

            let left_side = if should_wrap_left {
                format!("({})", function)
            } else {
                format!("{}", function)
            };

            let right_side = if should_wrap_right {
                format!("({})", applicant)
            } else {
                format!("{}", applicant)
            };

            return write!(f, "{} {}", left_side, right_side);
        }

        if let ProofTerm::TypeAscription(TypeAscription {
            proof_term,
            ascription,
            ..
        }) = self
        {
            let should_wrap = match **proof_term {
                ProofTerm::Function(_) => true,
                ProofTerm::Case(_) => true,
                ProofTerm::LetIn(_) => true,
                ProofTerm::TypeAscription(_) => true,
                _ => false,
            };

            if should_wrap {
                return write!(f, "({}): {}", proof_term, ascription);
            }

            return write!(f, "{}: {}", proof_term, ascription);
        }

        // named function call

        let (function_name, body) = match self {
            ProofTerm::ProjectFst(ProjectFst(body, _)) => ("fst", body),
            ProofTerm::ProjectSnd(ProjectSnd(body, _)) => ("snd", body),
            ProofTerm::Abort(Abort(body, _)) => ("abort", body),
            ProofTerm::OrLeft(OrLeft(body, _)) => ("inl", body),
            ProofTerm::OrRight(OrRight(body, _)) => ("inr", body),
            _ => unreachable!(),
        };

        let parent_precedence = self.precedence();
        let child_precedence = body.precedence();

        let should_wrap = parent_precedence > child_precedence;

        if should_wrap {
            return write!(f, "{} ({})", function_name, body);
        }

        write!(f, "{} {}", function_name, body)
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
    fn visit_unit(&mut self, span: Option<Range<usize>>) -> R;
    fn visit_sorry(&mut self, span: Option<Range<usize>>) -> R;
}
