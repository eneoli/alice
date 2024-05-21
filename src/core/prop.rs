use std::fmt::{self, Debug};

#[derive(Clone, PartialEq, Eq)]
pub enum Prop {
    Atom(String, Option<String>),
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
            Prop::Atom(ident, param) => match param {
                Some(param_ident) => write!(
                    f,
                    "{}({})",
                    format!("{:?}", ident),
                    format!("{:?}", param_ident)
                ),
                None => write!(f, "{}", format!("{:?}", ident)),
            },
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
