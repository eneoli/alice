use std::fmt::{self, Debug};

use super::proof_term::Type;

#[derive(Clone, PartialEq, Eq)]
pub enum Prop {
    Any,
    Atom(String, Vec<String>),
    And(Box<Prop>, Box<Prop>),
    Or(Box<Prop>, Box<Prop>),
    Impl(Box<Prop>, Box<Prop>),

    ForAll {
        object_ident: String,
        object_type_ident: String,
        body: Box<Prop>,
    },
    Exists {
        object_ident: String,
        object_type_ident: String,
        body: Box<Prop>,
    },

    True,
    False,
}

impl Prop {
    pub fn boxed(&self) -> Box<Self> {
        Box::new(self.clone())
    }

    // This can only replace with single identifiers, but that should be ok at the current state of the type checker.
    // Substituent: The identifier that gets replaced.
    // Substitutor: The identifier that will be replaced with.
    pub fn substitue_free_parameter(&mut self, substituent: &String, substitutor: &String) {
        match self {
            Prop::True => (),
            Prop::False => (),
            Prop::Any => (),
            Prop::And(ref mut fst, ref mut snd) => {
                Prop::substitue_free_parameter(fst, substituent, substitutor);
                Prop::substitue_free_parameter(snd, substituent, substitutor);
            }
            Prop::Or(ref mut fst, ref mut snd) => {
                Prop::substitue_free_parameter(fst, substituent, substitutor);
                Prop::substitue_free_parameter(snd, substituent, substitutor);
            }
            Prop::Impl(ref mut fst, ref mut snd) => {
                Prop::substitue_free_parameter(fst, substituent, substitutor);
                Prop::substitue_free_parameter(snd, substituent, substitutor);
            }
            Prop::Atom(_, params) => params.iter_mut().for_each(|param| {
                if *param == *substituent {
                    *param = substitutor.clone()
                }
            }),
            Prop::Exists {
                object_ident,
                ref mut body,
                ..
            } => {
                if object_ident != substituent {
                    Prop::substitue_free_parameter(body, substituent, substitutor);
                }
            }
            Prop::ForAll {
                object_ident,
                ref mut body,
                ..
            } => {
                if object_ident != substituent {
                    Prop::substitue_free_parameter(body, substituent, substitutor);
                }
            }
        }
    }
}

impl Into<Type> for Prop {
    fn into(self) -> Type {
        Type::Prop(self)
    }
}

impl Debug for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Prop::Any => write!(f, "*"),
            Prop::Atom(ident, params) => {
                if params.len() > 0 {
                    write!(f, "{}({})", format!("{:?}", ident), format!("{:?}", params))
                } else {
                    write!(f, "{}", format!("{:?}", ident))
                }
            }

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
            Prop::ForAll {
                object_ident,
                object_type_ident,
                body,
            } => write!(
                f,
                "∀{}:{}. ({})",
                object_ident,
                object_type_ident,
                format!("{:?}", body)
            ),
            Prop::Exists {
                object_ident,
                object_type_ident,
                body,
            } => write!(
                f,
                "∃{}:{}. ({})",
                object_ident,
                object_type_ident,
                format!("{:?}", body)
            ),

            Prop::True => write!(f, "T"),
            Prop::False => write!(f, "⊥"),
        }
    }
}
