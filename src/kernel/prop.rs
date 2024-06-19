use std::fmt::{self, Debug};

use super::proof_term::Type;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;

#[derive(Clone, Debug, Serialize, Deserialize, Tsify, PartialEq, Eq)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum PropKind {
    Atom,
    And,
    Or,
    Impl,
    ForAll,
    Exists,
    True,
    Fakse,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
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

    pub fn has_free_parameters(&self) -> bool {
        self.get_free_parameters().len() > 0
    }

    pub fn get_free_parameters(&self) -> Vec<String> {
        fn _get_free_parameters(prop: &Prop, binded_idents: &mut Vec<String>) -> Vec<String> {
            match prop {
                Prop::True => vec![],
                Prop::False => vec![],
                Prop::Any => vec![],
                Prop::And(fst, snd) => {
                    let mut fst_idents = _get_free_parameters(fst, &mut binded_idents.clone());
                    let mut snd_idents = _get_free_parameters(snd, binded_idents);

                    fst_idents.append(&mut snd_idents);

                    fst_idents
                }
                Prop::Or(fst, snd) => {
                    let mut fst_idents = _get_free_parameters(fst, &mut binded_idents.clone());
                    let mut snd_idents = _get_free_parameters(snd, binded_idents);

                    fst_idents.append(&mut snd_idents);

                    fst_idents
                }
                Prop::Impl(fst, snd) => {
                    let mut fst_idents = _get_free_parameters(fst, &mut binded_idents.clone());
                    let mut snd_idents = _get_free_parameters(snd, binded_idents);

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

    pub fn alpha_eq(&self, other: &Prop) -> bool {
        let env = vec![];

        Self::_alpha_eq(self, other, false, env)
    }

    pub fn alpha_eq_compare_free_occurences_by_structure(&self, other: &Prop) -> bool {
        let env = vec![];

        Self::_alpha_eq(self, other, true, env)
    }

    fn _alpha_eq<'a>(
        fst: &'a Prop,
        snd: &'a Prop,
        compare_free_occurences_by_structure: bool,
        mut env: Vec<(&'a String, &'a String)>,
    ) -> bool {
        match (fst, snd) {
            (Prop::True, Prop::True) => true,
            (Prop::False, Prop::False) => true,
            (Prop::And(l1, l2), Prop::And(r1, r2)) => {
                Self::_alpha_eq(l1, r1, compare_free_occurences_by_structure, env.clone())
                    && Self::_alpha_eq(l2, r2, compare_free_occurences_by_structure, env)
            }
            (Prop::Or(l1, l2), Prop::Or(r1, r2)) => {
                Self::_alpha_eq(l1, r1, compare_free_occurences_by_structure, env.clone())
                    && Self::_alpha_eq(l2, r2, compare_free_occurences_by_structure, env)
            }
            (Prop::Impl(l1, l2), Prop::Impl(r1, r2)) => {
                Self::_alpha_eq(l1, r1, compare_free_occurences_by_structure, env.clone())
                    && Self::_alpha_eq(l2, r2, compare_free_occurences_by_structure, env)
            }
            (
                Prop::Exists {
                    object_ident: l_object_ident,
                    object_type_ident: l_object_type_ident,
                    body: l_body,
                },
                Prop::Exists {
                    object_ident: r_object_ident,
                    object_type_ident: r_object_type_ident,
                    body: r_body,
                },
            ) => {
                env.push((l_object_ident, r_object_ident));

                l_object_type_ident == r_object_type_ident
                    && Self::_alpha_eq(l_body, r_body, compare_free_occurences_by_structure, env)
            }

            (
                Prop::ForAll {
                    object_ident: l_object_ident,
                    object_type_ident: l_object_type_ident,
                    body: l_body,
                },
                Prop::ForAll {
                    object_ident: r_object_ident,
                    object_type_ident: r_object_type_ident,
                    body: r_body,
                },
            ) => {
                env.push((l_object_ident, r_object_ident));

                l_object_type_ident == r_object_type_ident
                    && Self::_alpha_eq(l_body, r_body, compare_free_occurences_by_structure, env)
            }
            (Prop::Atom(l_ident, l_params), Prop::Atom(r_ident, r_params)) => {
                if l_ident != r_ident {
                    return false;
                }

                if l_params.len() != r_params.len() {
                    return false;
                }

                let mut free_occurrences_env = Vec::new();
                for (l_param, r_param) in Iterator::zip(l_params.iter(), r_params.iter()) {
                    // search for identifiers
                    let pair = env
                        .iter()
                        .rev()
                        .find(|(x, y)| *x == l_param || *y == r_param);

                    if let Some((x, y)) = pair {
                        if *x != l_param || *y != r_param {
                            return false;
                        }
                    } else {
                        // free occurences and no structural comaprison, check for name equality
                        if !compare_free_occurences_by_structure && l_param != r_param {
                            return false;
                        }

                        // otherwise check if structural identical
                        let pair = free_occurrences_env
                            .iter()
                            .rev()
                            .find(|(x, y)| *x == l_param || *y == r_param);

                        if let Some((x, y)) = pair {
                            if *x != l_param || *y != r_param {
                                return false;
                            }
                        } else {
                            free_occurrences_env.push((l_param, r_param));
                        }
                    }
                }

                return true;
            }
            _ => false,
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

    // HELPER

    use chumsky::{Parser, Stream};

    use crate::kernel::parse::{fol::fol_parser, lexer::lexer};

    fn parse_prop(prop: &str) -> Prop {
        let len = prop.chars().count();

        let tokens = lexer().parse(prop).unwrap();
        let prop = fol_parser()
            .parse(Stream::from_iter(len..len + 1, tokens.into_iter()))
            .unwrap();

        prop
    }

    // END Helper

    #[test]
    fn test_alpha_eq_no_quantors_equivalent() {
        let fst = Prop::And(
            Prop::Atom("A".to_string(), vec![]).boxed(),
            Prop::Or(
                Prop::Atom("B".to_string(), vec![]).boxed(),
                Prop::Atom("C".to_string(), vec![]).boxed(),
            )
            .boxed(),
        );

        let snd = Prop::And(
            Prop::Atom("A".to_string(), vec![]).boxed(),
            Prop::Or(
                Prop::Atom("B".to_string(), vec![]).boxed(),
                Prop::Atom("C".to_string(), vec![]).boxed(),
            )
            .boxed(),
        );

        assert!(Prop::alpha_eq(&fst, &snd))
    }

    #[test]
    fn test_alpha_eq_no_quantors_not_equivalent() {
        let fst = Prop::And(
            Prop::Atom("A".to_string(), vec![]).boxed(),
            Prop::Or(
                Prop::Atom("C".to_string(), vec![]).boxed(),
                Prop::Atom("C".to_string(), vec![]).boxed(),
            )
            .boxed(),
        );

        let snd = Prop::And(
            Prop::Atom("A".to_string(), vec![]).boxed(),
            Prop::Or(
                Prop::Atom("B".to_string(), vec![]).boxed(),
                Prop::Atom("C".to_string(), vec![]).boxed(),
            )
            .boxed(),
        );

        assert!(!Prop::alpha_eq(&fst, &snd))
    }

    #[test]
    fn test_alpha_eq_quantors_equivalent() {
        let fst = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::ForAll {
                object_ident: "y".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::And(
                    Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
                    Prop::Atom("B".to_string(), vec!["y".to_string()]).boxed(),
                )
                .boxed(),
            }
            .boxed(),
        };

        let snd = Prop::ForAll {
            object_ident: "y".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::And(
                    Prop::Atom("A".to_string(), vec!["y".to_string()]).boxed(),
                    Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed(),
                )
                .boxed(),
            }
            .boxed(),
        };

        assert!(Prop::alpha_eq(&fst, &snd))
    }

    #[test]
    fn test_alpha_eq_quantors_not_equivalent() {
        let fst = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::ForAll {
                object_ident: "y".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::And(
                    Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
                    Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed(),
                )
                .boxed(),
            }
            .boxed(),
        };

        let snd = Prop::ForAll {
            object_ident: "y".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::And(
                    Prop::Atom("A".to_string(), vec!["y".to_string()]).boxed(),
                    Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed(),
                )
                .boxed(),
            }
            .boxed(),
        };

        assert!(!Prop::alpha_eq(&fst, &snd))
    }
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

    #[test]
    fn test_alpha_eq_atom_no_params() {
        assert!(parse_prop("A").alpha_eq(&parse_prop("A")))
    }

    #[test]
    fn test_not_alpha_eq_atom_no_params() {
        assert!(!parse_prop("A").alpha_eq(&parse_prop("B")))
    }

    #[test]
    fn test_alpha_eq_atom_params() {
        assert!(Prop::alpha_eq(
            &parse_prop("A(x, y, z)"),
            &parse_prop("A(x, y, z)")
        ))
    }

    #[test]
    fn test_not_alpha_eq_atom_params() {
        assert!(!Prop::alpha_eq(
            &parse_prop("A(x, y, z)"),
            &parse_prop("A(x, z, z)")
        ));

        assert!(!Prop::alpha_eq(
            &parse_prop("A(x, y, z)"),
            &parse_prop("A(x, y)")
        ));

        assert!(!Prop::alpha_eq(
            &parse_prop("A(x, y, z)"),
            &parse_prop("B(x, y, z)")
        ))
    }

    #[test]
    fn test_alpha_eq_atom_params_structural() {
        assert!(!Prop::alpha_eq(
            &parse_prop("A(x, y, z)"),
            &parse_prop("A(a, b, c)")
        ));
    }

    #[test]
    fn test_not_alpha_eq_atom_params_structural() {
        assert!(!Prop::alpha_eq(
            &parse_prop("A(x, y, z)"),
            &parse_prop("A(a, b, b)")
        ));

        assert!(!Prop::alpha_eq(
            &parse_prop("A(x, y, z)"),
            &parse_prop("A(a, b)")
        ));

        assert!(!Prop::alpha_eq(
            &parse_prop("A(x, y, z)"),
            &parse_prop("B(a, b, c)")
        ))
    }

    #[test]
    fn test_alpha_eq_and() {
        assert!(Prop::alpha_eq(&parse_prop("A && B"), &parse_prop("A && B")));
    }

    #[test]
    fn test_not_alpha_eq_and() {
        assert!(!Prop::alpha_eq(
            &parse_prop("A && B"),
            &parse_prop("B && B")
        ));
    }

    #[test]
    fn test_alpha_eq_or() {
        assert!(Prop::alpha_eq(&parse_prop("A || B"), &parse_prop("A || B")));
    }

    #[test]
    fn test_not_alpha_eq_or() {
        assert!(!Prop::alpha_eq(
            &parse_prop("A || B"),
            &parse_prop("A || A")
        ));
    }

    #[test]
    fn test_alpha_eq_implication() {
        assert!(Prop::alpha_eq(&parse_prop("A -> B"), &parse_prop("A -> B")));
    }

    #[test]
    fn test_not_alpha_eq_implication() {
        assert!(!Prop::alpha_eq(
            &parse_prop("A -> B"),
            &parse_prop("A -> A")
        ));
    }

    #[test]
    fn test_for_all() {
        assert!(Prop::alpha_eq(
            &parse_prop("\\forall x:t. A(x)"),
            &parse_prop("\\forall w:t. A(w)")
        ));
    }

    #[test]
    fn test_exists() {
        assert!(Prop::alpha_eq(
            &parse_prop("\\exists x:t. A(x)"),
            &parse_prop("\\exists w:t. A(w)")
        ));
    }
}
