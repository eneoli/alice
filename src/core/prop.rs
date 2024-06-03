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

    pub fn get_free_parameters(&self) -> Vec<String> {
        fn _get_free_parameters(prop: &Prop, binded_idents: &mut Vec<String>) -> Vec<String> {
            match prop {
                Prop::True => vec![],
                Prop::False => vec![],
                Prop::Any => vec![],
                Prop::And(fst, snd) => {
                    let mut fst_idents = fst.get_free_parameters();
                    let mut snd_idents = snd.get_free_parameters();

                    fst_idents.append(&mut snd_idents);

                    fst_idents
                }
                Prop::Or(fst, snd) => {
                    let mut fst_idents = fst.get_free_parameters();
                    let mut snd_idents = snd.get_free_parameters();

                    fst_idents.append(&mut snd_idents);

                    fst_idents
                }
                Prop::Impl(fst, snd) => {
                    let mut fst_idents = fst.get_free_parameters();
                    let mut snd_idents = snd.get_free_parameters();

                    fst_idents.append(&mut snd_idents);

                    fst_idents
                }
                Prop::Exists {
                    object_ident, body, ..
                } => {
                    binded_idents.push(object_ident.to_string());

                    _get_free_parameters(body, binded_idents)
                }
                Prop::ForAll {
                    object_ident, body, ..
                } => {
                    binded_idents.push(object_ident.to_string());

                    _get_free_parameters(body, binded_idents)
                }
                Prop::Atom(_, params) => {
                    let mut free_params = params.clone();
                    free_params.retain(|param| !binded_idents.contains(param));

                    free_params
                }
            }
        }

        let mut binded_idents = Vec::new();

        _get_free_parameters(self, &mut binded_idents)
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

// ==== TESTS ====

#[cfg(test)]
mod tests {
    use std::vec;

    use super::Prop;

    #[test]
    fn test_free_parameters_none() {
        let free_params = Prop::Atom("A".to_string(), vec![]).get_free_parameters();

        assert_eq!(free_params.len(), 0)
    }

    #[test]
    fn test_free_parameters_some() {
        let free_params = Prop::Atom(
            "A".to_string(),
            vec!["x".to_string(), "y".to_string(), "z".to_string()],
        )
        .get_free_parameters();

        assert_eq!(free_params.len(), 3);
        assert_eq!(free_params.contains(&"x".to_string()), true);
        assert_eq!(free_params.contains(&"y".to_string()), true);
        assert_eq!(free_params.contains(&"z".to_string()), true);
    }

    #[test]
    fn test_free_parameters_and() {
        let free_params = Prop::And(
            Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
            Prop::Atom("B".to_string(), vec!["y".to_string()]).boxed(),
        )
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(free_params.contains(&"x".to_string()), true);
        assert_eq!(free_params.contains(&"y".to_string()), true);
    }

    #[test]
    fn test_free_parameters_or() {
        let free_params = Prop::Or(
            Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
            Prop::Atom("B".to_string(), vec!["y".to_string()]).boxed(),
        )
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(free_params.contains(&"x".to_string()), true);
        assert_eq!(free_params.contains(&"y".to_string()), true);
    }

    #[test]
    fn test_free_parameters_impl() {
        let free_params = Prop::Impl(
            Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
            Prop::Atom("B".to_string(), vec!["y".to_string()]).boxed(),
        )
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(free_params.contains(&"x".to_string()), true);
        assert_eq!(free_params.contains(&"y".to_string()), true);
    }

    #[test]
    fn test_allquant_free_parameters_none() {
        let free_params = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
        }
        .get_free_parameters();

        assert_eq!(free_params.len(), 0);
    }

    #[test]
    fn test_allquant_free_parameters_some() {
        let free_params = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec!["x".to_string(), "y".to_string(), "z".to_string()],
            )
            .boxed(),
        }
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(free_params.contains(&"y".to_string()), true);
        assert_eq!(free_params.contains(&"z".to_string()), true);
    }

    #[test]
    fn test_existsquant_free_parameters_none() {
        let free_params = Prop::Exists {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
        }
        .get_free_parameters();

        assert_eq!(free_params.len(), 0);
    }

    #[test]
    fn test_existsquant_free_parameters_some() {
        let free_params = Prop::Exists {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec!["x".to_string(), "y".to_string(), "z".to_string()],
            )
            .boxed(),
        }
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(free_params.contains(&"y".to_string()), true);
        assert_eq!(free_params.contains(&"z".to_string()), true);
    }

    #[test]
    fn test_substitute_none() {
        let mut prop = Prop::Atom("A".to_string(), vec![]);
        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());

        assert_eq!(prop, Prop::Atom("A".to_string(), vec![]))
    }

    #[test]
    fn test_substitute_some() {
        let mut prop = Prop::Atom("A".to_string(), vec!["x".to_string()]);
        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());

        assert_eq!(prop, Prop::Atom("A".to_string(), vec!["y".to_string()]))
    }

    #[test]
    fn test_substitute_and() {
        let mut prop = Prop::And(
            Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
            Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed(),
        );

        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());

        assert_eq!(
            prop,
            Prop::And(
                Prop::Atom("A".to_string(), vec!["y".to_string()]).boxed(),
                Prop::Atom("B".to_string(), vec!["y".to_string()]).boxed(),
            )
        )
    }

    #[test]
    fn test_substitute_or() {
        let mut prop = Prop::Or(
            Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
            Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed(),
        );

        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());

        assert_eq!(
            prop,
            Prop::Or(
                Prop::Atom("A".to_string(), vec!["y".to_string()]).boxed(),
                Prop::Atom("B".to_string(), vec!["y".to_string()]).boxed(),
            )
        )
    }

    #[test]
    fn test_substitute_impl() {
        let mut prop = Prop::Impl(
            Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
            Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed(),
        );

        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());

        assert_eq!(
            prop,
            Prop::Impl(
                Prop::Atom("A".to_string(), vec!["y".to_string()]).boxed(),
                Prop::Atom("B".to_string(), vec!["y".to_string()]).boxed(),
            )
        )
    }

    #[test]
    fn test_substitute_allquant_none() {
        let mut prop = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
        };

        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());

        assert_eq!(
            prop,
            Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
            }
        )
    }

    #[test]
    fn test_substitute_allquant_some() {
        let mut prop = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom("A".to_string(), vec!["x".to_string(), "z".to_string()]).boxed(),
        };

        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());
        prop.substitue_free_parameter(&"z".to_string(), &"y".to_string());

        assert_eq!(
            prop,
            Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom("A".to_string(), vec!["x".to_string(), "y".to_string()]).boxed(),
            }
        )
    }

    #[test]
    fn test_substitute_existsquant_none() {
        let mut prop = Prop::Exists {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
        };

        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());

        assert_eq!(
            prop,
            Prop::Exists {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
            }
        )
    }

    #[test]
    fn test_substitute_existsquant_some() {
        let mut prop = Prop::Exists {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom("A".to_string(), vec!["x".to_string(), "z".to_string()]).boxed(),
        };

        prop.substitue_free_parameter(&"x".to_string(), &"y".to_string());
        prop.substitue_free_parameter(&"z".to_string(), &"y".to_string());

        assert_eq!(
            prop,
            Prop::Exists {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom("A".to_string(), vec!["x".to_string(), "y".to_string()]).boxed(),
            }
        )
    }
}
