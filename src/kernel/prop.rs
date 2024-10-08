use std::fmt::{self, Debug, Display};
use std::vec;

use super::checker::identifier_context::IdentifierContext;
use super::{checker::identifier::Identifier, proof_term::Type};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use tsify_next::Tsify;
use wasm_bindgen::prelude::*;

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
    False,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum QuantifierKind {
    ForAll,
    Exists,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum PropParameter {
    Uninstantiated(String),
    Instantiated(Identifier),
}

impl PropParameter {
    pub fn name(&self) -> &String {
        match self {
            Self::Uninstantiated(ident) => ident,
            Self::Instantiated(ident) => ident.name(),
        }
    }

    pub fn unique_id(&self) -> Option<usize> {
        match self {
            Self::Uninstantiated(_) => None,
            Self::Instantiated(ident) => Some(ident.unique_id()),
        }
    }

    pub fn is_instantiated(&self) -> bool {
        match self {
            Self::Uninstantiated(_) => false,
            Self::Instantiated(_) => true,
        }
    }

    pub fn is_uninstantiated(&self) -> bool {
        match self {
            Self::Uninstantiated(_) => true,
            Self::Instantiated(_) => false,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InstatiationError {
    UnknownIdentifier(String),
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum Prop {
    Atom(String, Vec<PropParameter>),
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

    pub fn precedence(&self) -> usize {
        match self {
            Prop::Atom(_, _) => 999,
            Prop::True => 999,
            Prop::False => 999,
            Prop::And(_, _) => 4,
            Prop::Or(_, _) => 3,
            Prop::Impl(_, snd) => {
                if **snd == Prop::False {
                    999
                } else {
                    2
                }
            }
            Prop::ForAll { .. } => 1,
            Prop::Exists { .. } => 1,
        }
    }

    pub fn left_associative(&self) -> bool {
        match self {
            Prop::Atom(_, _) => false,
            Prop::True => false,
            Prop::False => false,
            Prop::And(_, _) => true,
            Prop::Or(_, _) => true,
            Prop::Impl(_, _) => false,
            Prop::ForAll { .. } => false,
            Prop::Exists { .. } => false,
        }
    }

    pub fn right_associative(&self) -> bool {
        match self {
            Prop::Atom(_, _) => false,
            Prop::True => false,
            Prop::False => false,
            Prop::And(_, _) => false,
            Prop::Or(_, _) => false,
            Prop::Impl(_, _) => true,
            Prop::ForAll { .. } => false,
            Prop::Exists { .. } => false,
        }
    }

    pub fn get_atoms(&self) -> Vec<(String, usize)> {
        match self {
            Prop::True => vec![],
            Prop::False => vec![],
            Prop::Atom(atom, params) => vec![(atom.clone(), params.len())],
            Prop::ForAll { body, .. } | Prop::Exists { body, .. } => body.get_atoms(),
            Prop::And(fst, snd) | Prop::Or(fst, snd) | Prop::Impl(fst, snd) => {
                [fst.get_atoms(), snd.get_atoms()].concat()
            }
        }
    }

    pub fn get_datatypes(&self) -> Vec<String> {
        match self {
            Prop::True | Prop::False | Prop::Atom(_, _) => vec![],
            Prop::ForAll {
                object_type_ident,
                body,
                 ..
            }
            | Prop::Exists {
                object_type_ident,
                body,
                ..
            } => [vec![object_type_ident.clone()], body.get_datatypes()].concat(),
            Prop::And(fst, snd) | Prop::Or(fst, snd) | Prop::Impl(fst, snd) => {
                [fst.get_datatypes(), snd.get_datatypes()].concat()
            }
        }
    }

    pub fn has_quantifiers(&self) -> bool {
        match self {
            Prop::Atom(_, _) => false,
            Prop::True => false,
            Prop::False => false,
            Prop::And(fst, snd) => fst.has_quantifiers() || snd.has_quantifiers(),
            Prop::Or(fst, snd) => fst.has_quantifiers() || snd.has_quantifiers(),
            Prop::Impl(fst, snd) => fst.has_quantifiers() || snd.has_quantifiers(),
            Prop::ForAll { .. } => true,
            Prop::Exists { .. } => true,
        }
    }

    pub fn has_free_parameters(&self) -> bool {
        !self.get_free_parameters().is_empty()
    }

    pub fn get_free_parameters(&self) -> Vec<PropParameter> {
        fn _get_free_parameters(prop: &Prop, bound_idents: &mut Vec<String>) -> Vec<PropParameter> {
            match prop {
                Prop::True => vec![],
                Prop::False => vec![],
                Prop::And(fst, snd) => {
                    let fst_idents = _get_free_parameters(fst, &mut bound_idents.clone());
                    let snd_idents = _get_free_parameters(snd, bound_idents);

                    [fst_idents, snd_idents].concat()
                }
                Prop::Or(fst, snd) => {
                    let fst_idents = _get_free_parameters(fst, &mut bound_idents.clone());
                    let snd_idents = _get_free_parameters(snd, bound_idents);

                    [fst_idents, snd_idents].concat()
                }
                Prop::Impl(fst, snd) => {
                    let fst_idents = _get_free_parameters(fst, &mut bound_idents.clone());
                    let snd_idents = _get_free_parameters(snd, bound_idents);

                    [fst_idents, snd_idents].concat()
                }
                Prop::Exists {
                    object_ident, body, ..
                } => {
                    bound_idents.push(object_ident.to_string());

                    _get_free_parameters(body, bound_idents)
                }
                Prop::ForAll {
                    object_ident, body, ..
                } => {
                    bound_idents.push(object_ident.to_string());

                    _get_free_parameters(body, bound_idents)
                }
                Prop::Atom(_, params) => {
                    let mut free_params = params.clone();
                    free_params.retain(|param| !bound_idents.contains(param.name()));

                    free_params
                }
            }
        }

        let mut bound_idents = Vec::new();

        _get_free_parameters(self, &mut bound_idents)
    }

    pub fn get_free_parameters_mut(&mut self) -> Vec<&mut PropParameter> {
        fn _get_free_parameters<'a>(
            prop: &'a mut Prop,
            bound_idents: &mut Vec<String>,
        ) -> Vec<&'a mut PropParameter> {
            match prop {
                Prop::True => vec![],
                Prop::False => vec![],
                Prop::And(fst, snd) => {
                    let mut fst_idents = _get_free_parameters(fst, &mut bound_idents.clone());
                    let mut snd_idents = _get_free_parameters(snd, bound_idents);

                    fst_idents.append(&mut snd_idents);
                    fst_idents
                }
                Prop::Or(fst, snd) => {
                    let mut fst_idents = _get_free_parameters(fst, &mut bound_idents.clone());
                    let mut snd_idents = _get_free_parameters(snd, bound_idents);

                    fst_idents.append(&mut snd_idents);
                    fst_idents
                }
                Prop::Impl(fst, snd) => {
                    let mut fst_idents = _get_free_parameters(fst, &mut bound_idents.clone());
                    let mut snd_idents = _get_free_parameters(snd, bound_idents);

                    fst_idents.append(&mut snd_idents);
                    fst_idents
                }
                Prop::Exists {
                    object_ident, body, ..
                } => {
                    bound_idents.push(object_ident.to_string());

                    _get_free_parameters(body, bound_idents)
                }
                Prop::ForAll {
                    object_ident, body, ..
                } => {
                    bound_idents.push(object_ident.to_string());

                    _get_free_parameters(body, bound_idents)
                }
                Prop::Atom(_, ref mut params) => {
                    let mut free_params = params.iter_mut().collect::<Vec<&'a mut PropParameter>>();
                    free_params.retain(|param| !bound_idents.contains(param.name()));

                    free_params
                }
            }
        }

        let mut bound_idents = Vec::new();

        _get_free_parameters(self, &mut bound_idents)
    }

    // This can only replace with single identifiers, but that should be ok at the current state of the type checker.
    // Substituent: The identifier that gets replaced. (Uninstantiated)
    // Substitutor: The identifier that will be replaced with.
    pub fn instantiate_free_parameter(&mut self, substituent: &String, substitutor: &Identifier) {
        match self {
            Prop::True => (),
            Prop::False => (),
            Prop::And(ref mut fst, ref mut snd) => {
                Prop::instantiate_free_parameter(fst, substituent, substitutor);
                Prop::instantiate_free_parameter(snd, substituent, substitutor);
            }
            Prop::Or(ref mut fst, ref mut snd) => {
                Prop::instantiate_free_parameter(fst, substituent, substitutor);
                Prop::instantiate_free_parameter(snd, substituent, substitutor);
            }
            Prop::Impl(ref mut fst, ref mut snd) => {
                Prop::instantiate_free_parameter(fst, substituent, substitutor);
                Prop::instantiate_free_parameter(snd, substituent, substitutor);
            }
            Prop::Exists {
                object_ident,
                ref mut body,
                ..
            } => {
                if object_ident != substituent {
                    Prop::instantiate_free_parameter(body, substituent, substitutor);
                }
            }
            Prop::ForAll {
                object_ident,
                ref mut body,
                ..
            } => {
                if object_ident != substituent {
                    Prop::instantiate_free_parameter(body, substituent, substitutor);
                }
            }
            Prop::Atom(_, params) => params.iter_mut().for_each(|param| {
                if param.is_uninstantiated() && *param.name() == *substituent {
                    *param = PropParameter::Instantiated(substitutor.clone())
                }
            }),
        }
    }

    pub fn instantiate_free_parameter_by_index(&mut self, index: usize, substitutor: &Identifier) {
        fn _instantiate<'a>(
            prop: &'a mut Prop,
            index: usize,
            substitutor: &Identifier,
            current_index: &mut usize,
            bound_identifiers: &mut Vec<&'a str>,
        ) {
            match prop {
                Prop::True => {}
                Prop::False => {}
                Prop::And(fst, snd) | Prop::Or(fst, snd) | Prop::Impl(fst, snd) => {
                    _instantiate(
                        fst,
                        index,
                        substitutor,
                        current_index,
                        &mut bound_identifiers.clone(),
                    );

                    if *current_index > index {
                        return;
                    }

                    _instantiate(snd, index, substitutor, current_index, bound_identifiers);
                }
                Prop::ForAll {
                    object_ident,
                    ref mut body,
                    ..
                }
                | Prop::Exists {
                    object_ident,
                    ref mut body,
                    ..
                } => {
                    bound_identifiers.push(object_ident.as_str());

                    _instantiate(body, index, substitutor, current_index, bound_identifiers);
                }
                Prop::Atom(_, params) => {
                    let param_limit = index - *current_index + 1;
                    if params.len() < param_limit {
                        *current_index += params.len();
                        return;
                    }

                    let Some(param) = params.get_mut(index - *current_index) else {
                        panic!("Invalid index.");
                    };

                    *param = PropParameter::Instantiated(substitutor.clone());
                    *current_index = index;
                }
            }
        }

        let mut current_index = 0;
        _instantiate(self, index, substitutor, &mut current_index, &mut vec![]);

        // sanity check
        if current_index < index {
            panic!("Proposition has not enough parameters.");
        }
    }

    // Warning: This might bind parameters to an inner quantifier
    // Choose bind name with care.
    pub fn bind_identifier(
        &self,
        quantifier_kind: QuantifierKind,
        identifier: Identifier,
        identifier_indices: Option<&Vec<usize>>,
        bind_name: &str,
        type_name: &str,
    ) -> Prop {
        fn _bind_identifier<'a>(
            prop: &'a Prop,
            identifier: &Identifier,
            identifier_indices: Option<&Vec<usize>>,
            bind_name: &str,
            bound_identifiers: &mut Vec<&'a str>,
            current_index: &mut usize,
        ) -> Prop {
            match prop {
                Prop::True => Prop::True,
                Prop::False => Prop::False,
                Prop::And(fst, snd) => Prop::And(
                    _bind_identifier(
                        fst,
                        identifier,
                        identifier_indices,
                        bind_name,
                        &mut bound_identifiers.clone(),
                        current_index,
                    )
                    .boxed(),
                    _bind_identifier(
                        snd,
                        identifier,
                        identifier_indices,
                        bind_name,
                        bound_identifiers,
                        current_index,
                    )
                    .boxed(),
                ),
                Prop::Or(fst, snd) => Prop::Or(
                    _bind_identifier(
                        fst,
                        identifier,
                        identifier_indices,
                        bind_name,
                        &mut bound_identifiers.clone(),
                        current_index,
                    )
                    .boxed(),
                    _bind_identifier(
                        snd,
                        identifier,
                        identifier_indices,
                        bind_name,
                        bound_identifiers,
                        current_index,
                    )
                    .boxed(),
                ),
                Prop::Impl(fst, snd) => Prop::Impl(
                    _bind_identifier(
                        fst,
                        identifier,
                        identifier_indices,
                        bind_name,
                        &mut bound_identifiers.clone(),
                        current_index,
                    )
                    .boxed(),
                    _bind_identifier(
                        snd,
                        identifier,
                        identifier_indices,
                        bind_name,
                        bound_identifiers,
                        current_index,
                    )
                    .boxed(),
                ),
                Prop::Exists {
                    object_ident,
                    object_type_ident,
                    body,
                } => {
                    bound_identifiers.push(object_ident.as_str());
                    Prop::Exists {
                        object_ident: object_ident.clone(),
                        object_type_ident: object_type_ident.clone(),
                        body: _bind_identifier(
                            body,
                            identifier,
                            identifier_indices,
                            bind_name,
                            bound_identifiers,
                            current_index,
                        )
                        .boxed(),
                    }
                }
                Prop::ForAll {
                    object_ident,
                    object_type_ident,
                    body,
                } => {
                    bound_identifiers.push(object_ident.as_str());
                    Prop::ForAll {
                        object_ident: object_ident.clone(),
                        object_type_ident: object_type_ident.clone(),
                        body: _bind_identifier(
                            body,
                            identifier,
                            identifier_indices,
                            bind_name,
                            bound_identifiers,
                            current_index,
                        )
                        .boxed(),
                    }
                }
                Prop::Atom(atom_ident, params) => {
                    let new_params = params
                        .iter()
                        .map(|param| {
                            if param.is_instantiated()
                                && param.name() == identifier.name()
                                && param.unique_id().unwrap() == identifier.unique_id()
                            {
                                if identifier_indices.is_none() || identifier_indices.unwrap().contains(&current_index) {
                                    *current_index += 1;
                                    return PropParameter::Uninstantiated(bind_name.to_string());
                                }

                                *current_index += 1;
                            }

                            param.clone()
                        })
                        .collect();

                    Prop::Atom(atom_ident.clone(), new_params)
                }
            }
        }

        let bound_body = _bind_identifier(
            self,
            &identifier,
            identifier_indices,
            &bind_name,
            &mut vec![],
            &mut 0,
        );

        match quantifier_kind {
            QuantifierKind::Exists => Prop::Exists {
                object_ident: bind_name.to_string(),
                object_type_ident: type_name.to_string(),
                body: bound_body.boxed(),
            },
            QuantifierKind::ForAll => Prop::ForAll {
                object_ident: bind_name.to_string(),
                object_type_ident: type_name.to_string(),
                body: bound_body.boxed(),
            },
        }
    }

    pub fn instantiate_parameters_with_context(
        &mut self,
        ctx: &IdentifierContext,
    ) -> Result<(), InstatiationError> {
        fn _instantiate_with_ctx<'a>(
            prop: &'a mut Prop,
            ctx: &IdentifierContext,
            mut bound_idents: Vec<&'a str>,
        ) -> Result<(), InstatiationError> {
            match prop {
                Prop::True => {}
                Prop::False => {}
                Prop::And(ref mut fst, ref mut snd)
                | Prop::Or(ref mut fst, ref mut snd)
                | Prop::Impl(ref mut fst, ref mut snd) => {
                    _instantiate_with_ctx(fst, ctx, bound_idents.clone())?;
                    _instantiate_with_ctx(snd, ctx, bound_idents)?;
                }
                Prop::ForAll {
                    ref object_ident,
                    body,
                    ..
                }
                | Prop::Exists {
                    ref object_ident,
                    body,
                    ..
                } => {
                    bound_idents.push(object_ident);

                    _instantiate_with_ctx(body, ctx, bound_idents)?;
                }
                Prop::Atom(_, params) => {
                    for param in params.iter_mut() {
                        // sanity check
                        if let PropParameter::Instantiated(identifier) = param {
                            if ctx.get(identifier).is_none() {
                                panic!("Instantiated parameter does not exist: {:#?}", identifier);
                            }

                            if let Some(Type::Prop(prop)) = ctx.get(&identifier) {
                                panic!(
                                    "Parameter is a proposition: {:#?}, {:#?}",
                                    identifier, prop
                                );
                            }

                            continue;
                        }

                        let PropParameter::Uninstantiated(name) = param else {
                            continue;
                        };

                        if bound_idents.contains(&name.as_str()) {
                            continue;
                        }

                        let Some((identifier, _)) = ctx.get_by_name(&name) else {
                            return Err(InstatiationError::UnknownIdentifier(name.clone()));
                        };

                        *param = PropParameter::Instantiated(identifier.clone());
                    }
                }
            };

            Ok(())
        }

        _instantiate_with_ctx(self, ctx, vec![])
    }

    pub fn alpha_eq(&self, other: &Prop) -> bool {
        let env = vec![];

        Self::_alpha_eq(self, other, env)
    }

    fn _alpha_eq<'a>(fst: &'a Prop, snd: &'a Prop, mut env: Vec<(&'a String, &'a String)>) -> bool {
        match (fst, snd) {
            (Prop::True, Prop::True) => true,
            (Prop::False, Prop::False) => true,
            (Prop::And(l1, l2), Prop::And(r1, r2)) => {
                Self::_alpha_eq(l1, r1, env.clone()) && Self::_alpha_eq(l2, r2, env)
            }
            (Prop::Or(l1, l2), Prop::Or(r1, r2)) => {
                Self::_alpha_eq(l1, r1, env.clone()) && Self::_alpha_eq(l2, r2, env)
            }
            (Prop::Impl(l1, l2), Prop::Impl(r1, r2)) => {
                Self::_alpha_eq(l1, r1, env.clone()) && Self::_alpha_eq(l2, r2, env)
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

                l_object_type_ident == r_object_type_ident && Self::_alpha_eq(l_body, r_body, env)
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

                l_object_type_ident == r_object_type_ident && Self::_alpha_eq(l_body, r_body, env)
            }
            (left @ Prop::Atom(l_ident, l_params), right @ Prop::Atom(r_ident, r_params)) => {
                if l_ident != r_ident {
                    return false;
                }

                // sanity check
                if l_params.len() != r_params.len() {
                    panic!(
                        "Error: Cannot have the same Atom with different aity: {:#?}, {:#?}",
                        left, right
                    );
                }

                for (l_param, r_param) in Iterator::zip(l_params.iter(), r_params.iter()) {
                    if let (
                        PropParameter::Uninstantiated(l_param_name),
                        PropParameter::Uninstantiated(r_param_name),
                    ) = (l_param, r_param)
                    {
                        // search for uninstantiated identifiers
                        let pair = env
                            .iter()
                            .rev()
                            .find(|(x, y)| *x == l_param_name || *y == r_param_name);

                        if let Some((x, y)) = pair {
                            if *x != l_param_name || *y != r_param_name {
                                return false;
                            }
                        } else {
                            panic!("Found uninstantiated parameter that is not bound by a quantor. left: {:#?}, right: {:#?}", left, right);
                        }
                    } else if l_param != r_param {
                        return false;
                    }
                }

                true
            }
            _ => false,
        }
    }
}

impl From<Prop> for Type {
    fn from(val: Prop) -> Self {
        Type::Prop(val)
    }
}

impl Display for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Prop::Atom(name, params) = self {
            if params.len() == 0 {
                return write!(f, "{}", name);
            }

            let param_list = params.iter().map(PropParameter::name).join(", ");

            return write!(f, "{}({})", name, param_list);
        }

        if let Prop::True = self {
            return write!(f, "⊤");
        }

        if let Prop::False = self {
            return write!(f, "⊥");
        }

        if let Prop::ForAll {
            object_ident,
            object_type_ident,
            body,
        } = self
        {
            return write!(f, "∀{}:{}. {}", object_ident, object_type_ident, body);
        }

        if let Prop::Exists {
            object_ident,
            object_type_ident,
            body,
        } = self
        {
            return write!(f, "∃{}:{}. {}", object_ident, object_type_ident, body);
        }

        // Binary connective

        let wrap = |prop: &Prop, should_wrap: bool| {
            if should_wrap {
                return format!("({})", prop);
            }

            format!("{}", prop)
        };

        let (connective_symbol, fst, snd) = match self {
            Prop::And(ref fst, ref snd) => ("∧", fst, snd),
            Prop::Or(ref fst, ref snd) => ("∨", fst, snd),
            Prop::Impl(ref fst, ref snd) => ("⊃", fst, snd),
            _ => unreachable!(),
        };

        let prop_precedence = self.precedence();
        let fst_precedence = fst.precedence();
        let snd_precedence = snd.precedence();

        // Special display for A -> False
        if let (Prop::Impl(_, _), Prop::False) = (self, &(**snd)) {
            let should_wrap = prop_precedence > fst_precedence;
            return write!(f, "¬{}", wrap(fst, should_wrap));
        }

        let should_wrap_fst = (prop_precedence > fst_precedence)
            || (prop_precedence == fst_precedence && self.right_associative());
        let should_wrap_snd = (prop_precedence > snd_precedence)
            || (prop_precedence == snd_precedence && self.left_associative());

        write!(
            f,
            "{} {} {}",
            wrap(fst, should_wrap_fst),
            connective_symbol,
            wrap(snd, should_wrap_snd)
        )
    }
}

impl Debug for Prop {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Prop::Atom(ident, params) => {
                if !params.is_empty() {
                    write!(f, "{:?}({:?})", ident, params)
                } else {
                    write!(f, "{:?}", ident)
                }
            }

            Prop::And(left, right) => write!(f, "({:?}) ∧ ({:?})", left, right),
            Prop::Or(left, right) => write!(f, "({:?}) ∨ ({:?})", left, right),
            Prop::Impl(left, right) => write!(f, "({:?}) => ({:?})", left, right),
            Prop::ForAll {
                object_ident,
                object_type_ident,
                body,
            } => write!(f, "∀{}:{}. ({:?})", object_ident, object_type_ident, body),
            Prop::Exists {
                object_ident,
                object_type_ident,
                body,
            } => write!(f, "∃{}:{}. ({:?})", object_ident, object_type_ident, body),

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

    use crate::kernel::{
        checker::{identifier::Identifier, identifier_context::IdentifierContext},
        parse::{fol::fol_parser, lexer::lexer},
        proof_term::Type,
        prop::{InstatiationError, PropParameter, QuantifierKind},
    };

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
                    Prop::Atom(
                        "A".to_string(),
                        vec![PropParameter::Uninstantiated("x".to_string())],
                    )
                    .boxed(),
                    Prop::Atom(
                        "B".to_string(),
                        vec![PropParameter::Uninstantiated("y".to_string())],
                    )
                    .boxed(),
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
                    Prop::Atom(
                        "A".to_string(),
                        vec![PropParameter::Uninstantiated("y".to_string())],
                    )
                    .boxed(),
                    Prop::Atom(
                        "B".to_string(),
                        vec![PropParameter::Uninstantiated("x".to_string())],
                    )
                    .boxed(),
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
                    Prop::Atom(
                        "A".to_string(),
                        vec![PropParameter::Uninstantiated("x".to_string())],
                    )
                    .boxed(),
                    Prop::Atom(
                        "B".to_string(),
                        vec![PropParameter::Uninstantiated("x".to_string())],
                    )
                    .boxed(),
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
                    Prop::Atom(
                        "A".to_string(),
                        vec![PropParameter::Uninstantiated("y".to_string())],
                    )
                    .boxed(),
                    Prop::Atom(
                        "B".to_string(),
                        vec![PropParameter::Uninstantiated("x".to_string())],
                    )
                    .boxed(),
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
            vec![
                PropParameter::Uninstantiated("x".to_string()),
                PropParameter::Uninstantiated("y".to_string()),
                PropParameter::Uninstantiated("z".to_string()),
            ],
        )
        .get_free_parameters();

        assert_eq!(free_params.len(), 3);
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("x".to_string())),
            true
        );
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("y".to_string())),
            true
        );
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("z".to_string())),
            true
        );
    }

    #[test]
    fn test_free_parameters_and() {
        let free_params = Prop::And(
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
            Prop::Atom(
                "B".to_string(),
                vec![PropParameter::Uninstantiated("y".to_string())],
            )
            .boxed(),
        )
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("x".to_string())),
            true
        );
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("y".to_string())),
            true
        );
    }

    #[test]
    fn test_free_parameters_or() {
        let free_params = Prop::Or(
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
            Prop::Atom(
                "B".to_string(),
                vec![PropParameter::Uninstantiated("y".to_string())],
            )
            .boxed(),
        )
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("x".to_string())),
            true
        );
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("y".to_string())),
            true
        );
    }

    #[test]
    fn test_free_parameters_impl() {
        let free_params = Prop::Impl(
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
            Prop::Atom(
                "B".to_string(),
                vec![PropParameter::Uninstantiated("y".to_string())],
            )
            .boxed(),
        )
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("x".to_string())),
            true
        );
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("y".to_string())),
            true
        );
    }

    #[test]
    fn test_allquant_free_parameters_none() {
        let free_params = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
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
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Uninstantiated("y".to_string()),
                    PropParameter::Uninstantiated("z".to_string()),
                ],
            )
            .boxed(),
        }
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("y".to_string())),
            true
        );
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("z".to_string())),
            true
        );
    }

    #[test]
    fn test_existsquant_free_parameters_none() {
        let free_params = Prop::Exists {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
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
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Uninstantiated("y".to_string()),
                    PropParameter::Uninstantiated("z".to_string()),
                ],
            )
            .boxed(),
        }
        .get_free_parameters();

        assert_eq!(free_params.len(), 2);
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("y".to_string())),
            true
        );
        assert_eq!(
            free_params.contains(&PropParameter::Uninstantiated("z".to_string())),
            true
        );
    }

    #[test]
    fn test_substitute_none() {
        let mut prop = Prop::Atom("A".to_string(), vec![]);
        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(prop, Prop::Atom("A".to_string(), vec![]))
    }

    #[test]
    fn test_substitute_some() {
        let mut prop = Prop::Atom(
            "A".to_string(),
            vec![PropParameter::Uninstantiated("x".to_string())],
        );
        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(
            prop,
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Instantiated(Identifier::new(
                    "y".to_string(),
                    42
                ))]
            )
        )
    }

    #[test]
    fn test_do_not_substitute_instantiated_param() {
        let mut prop = Prop::Atom(
            "A".to_string(),
            vec![PropParameter::Instantiated(Identifier::new(
                "x".to_string(),
                1,
            ))],
        );
        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 2));

        assert_eq!(
            prop,
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Instantiated(Identifier::new(
                    "x".to_string(),
                    1
                ))]
            )
        )
    }

    #[test]
    fn test_substitute_and() {
        let mut prop = Prop::And(
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
            Prop::Atom(
                "B".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
        );

        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(
            prop,
            Prop::And(
                Prop::Atom(
                    "A".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "y".to_string(),
                        42
                    ))]
                )
                .boxed(),
                Prop::Atom(
                    "B".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "y".to_string(),
                        42
                    ))]
                )
                .boxed(),
            )
        )
    }

    #[test]
    fn test_substitute_or() {
        let mut prop = Prop::Or(
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
            Prop::Atom(
                "B".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
        );

        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(
            prop,
            Prop::Or(
                Prop::Atom(
                    "A".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "y".to_string(),
                        42
                    ))]
                )
                .boxed(),
                Prop::Atom(
                    "B".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "y".to_string(),
                        42
                    ))]
                )
                .boxed(),
            )
        )
    }

    #[test]
    fn test_substitute_impl() {
        let mut prop = Prop::Impl(
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
            Prop::Atom(
                "B".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
        );

        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(
            prop,
            Prop::Impl(
                Prop::Atom(
                    "A".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "y".to_string(),
                        42
                    ))]
                )
                .boxed(),
                Prop::Atom(
                    "B".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "y".to_string(),
                        42
                    ))]
                )
                .boxed(),
            )
        )
    }

    #[test]
    fn test_substitute_allquant_none() {
        let mut prop = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
        };

        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(
            prop,
            Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom(
                    "A".to_string(),
                    vec![PropParameter::Uninstantiated("x".to_string())]
                )
                .boxed(),
            }
        )
    }

    #[test]
    fn test_substitute_allquant_some() {
        let mut prop = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Uninstantiated("z".to_string()),
                ],
            )
            .boxed(),
        };

        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));
        prop.instantiate_free_parameter(&"z".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(
            prop,
            Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom(
                    "A".to_string(),
                    vec![
                        PropParameter::Uninstantiated("x".to_string()),
                        PropParameter::Instantiated(Identifier::new("y".to_string(), 42))
                    ]
                )
                .boxed(),
            }
        )
    }

    #[test]
    fn test_substitute_existsquant_none() {
        let mut prop = Prop::Exists {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )
            .boxed(),
        };

        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(
            prop,
            Prop::Exists {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom(
                    "A".to_string(),
                    vec![PropParameter::Uninstantiated("x".to_string())]
                )
                .boxed(),
            }
        )
    }

    #[test]
    fn test_substitute_existsquant_some() {
        let mut prop = Prop::Exists {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Uninstantiated("z".to_string()),
                ],
            )
            .boxed(),
        };

        prop.instantiate_free_parameter(&"x".to_string(), &Identifier::new("y".to_string(), 42));
        prop.instantiate_free_parameter(&"z".to_string(), &Identifier::new("y".to_string(), 42));

        assert_eq!(
            prop,
            Prop::Exists {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom(
                    "A".to_string(),
                    vec![
                        PropParameter::Uninstantiated("x".to_string()),
                        PropParameter::Instantiated(Identifier::new("y".to_string(), 42))
                    ]
                )
                .boxed(),
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
    #[should_panic]
    fn test_no_free_uninstantiated_params() {
        Prop::alpha_eq(&parse_prop("A(a)"), &parse_prop("A(a)"));
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

    #[test]
    fn test_bind_atom_without_parameters() {
        assert_eq!(
            &parse_prop("A").bind_identifier(
                QuantifierKind::ForAll,
                Identifier::new("a".to_string(), 42),
                Some(&mut vec![]),
                "x",
                "t"
            ),
            &parse_prop("\\forall x:t. A"),
        )
    }

    #[test]
    fn test_bind_atom_with_one_parameter_replacement() {
        let prop = Prop::Atom(
            "A".to_string(),
            vec![PropParameter::Instantiated(Identifier::new(
                "a".to_string(),
                42,
            ))],
        );

        assert_eq!(
            &prop.bind_identifier(
                QuantifierKind::ForAll,
                Identifier::new("a".to_string(), 42),
                Some(&mut vec![0]),
                "x",
                "t"
            ),
            &parse_prop("\\forall x:t. A(x)"),
        )
    }

    #[test]
    fn test_bind_atom_with_two_parameter_replacement() {
        let prop = Prop::Atom(
            "A".to_string(),
            vec![
                PropParameter::Instantiated(Identifier::new("a".to_string(), 42)),
                PropParameter::Instantiated(Identifier::new("a".to_string(), 42)),
            ],
        );

        assert_eq!(
            &prop.bind_identifier(
                QuantifierKind::ForAll,
                Identifier::new("a".to_string(), 42),
                Some(&mut vec![0, 1]),
                "x",
                "t"
            ),
            &parse_prop("\\forall x:t. A(x, x)"),
        )
    }

    #[test]
    fn test_bind_atom_with_two_parameter_one_replacement() {
        let prop = Prop::Atom(
            "A".to_string(),
            vec![
                PropParameter::Instantiated(Identifier::new("a".to_string(), 0)),
                PropParameter::Instantiated(Identifier::new("a".to_string(), 42)),
            ],
        );

        assert_eq!(
            prop.bind_identifier(
                QuantifierKind::ForAll,
                Identifier::new("a".to_string(), 42),
                Some(&mut vec![0]),
                "x",
                "t",
            ),
            Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom(
                    "A".to_string(),
                    vec![
                        PropParameter::Instantiated(Identifier::new("a".to_string(), 0)),
                        PropParameter::Uninstantiated("x".to_string()),
                    ]
                )
                .boxed(),
            }
        )
    }

    #[test]
    fn test_bind_instantiated_parameter_that_is_under_quantifier() {
        // The Test tests for the right behaviour.
        // An instantiated parameter was instantiated through a quantor elimintation (ForAll) and
        // introdutciont (Exists) respectively.

        let prop = Prop::Exists {
            object_ident: "a".to_string(),
            object_type_ident: "t".to_string(),
            body: Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Instantiated(Identifier::new(
                    "a".to_string(),
                    42,
                ))],
            )
            .boxed(),
        };

        assert_eq!(
            prop.bind_identifier(
                QuantifierKind::Exists,
                Identifier::new("a".to_string(), 42),
                Some(&mut vec![0]),
                "x",
                "t",
            ),
            parse_prop("\\exists x:t. \\exists a:t. A(x)"),
        )
    }

    #[test]
    fn test_do_not_bind_already_bound_identifier() {
        assert_eq!(
            parse_prop("\\forall a:t. A(a)").bind_identifier(
                QuantifierKind::ForAll,
                Identifier::new("a".to_string(), 42),
                Some(&mut vec![0]),
                "x",
                "t",
            ),
            parse_prop("\\forall x:t. \\forall a:t. A(a)"),
        );
    }

    #[test]
    fn test_identifier_indices_atom() {
        assert_eq!(
            Prop::Atom(
                "A".to_string(),
                vec![
                    PropParameter::Instantiated(Identifier::new("a".to_string(), 42)),
                    PropParameter::Instantiated(Identifier::new("a".to_string(), 42)),
                    PropParameter::Instantiated(Identifier::new("a".to_string(), 42)),
                ],
            )
            .bind_identifier(
                QuantifierKind::ForAll,
                Identifier::new("a".to_string(), 42),
                Some(&mut vec![0, 2]),
                "x",
                "t",
            ),
            Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Prop::Atom(
                    "A".to_string(),
                    vec![
                        PropParameter::Uninstantiated("x".to_string()),
                        PropParameter::Instantiated(Identifier::new("a".to_string(), 42)),
                        PropParameter::Uninstantiated("x".to_string()),
                    ],
                )
                .boxed()
            },
        )
    }

    #[test]
    fn test_identifier_indices_binary_connective() {
        assert_eq!(
            Prop::And(
                Prop::Atom(
                    "A".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "a".to_string(),
                        42
                    ))]
                )
                .boxed(),
                Prop::Atom(
                    "B".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "a".to_string(),
                        42
                    ))]
                )
                .boxed()
            )
            .bind_identifier(
                QuantifierKind::ForAll,
                Identifier::new("a".to_string(), 42),
                Some(&mut vec![0, 1]),
                "x",
                "t",
            ),
            parse_prop("\\forall x:t. A(x) & B (x)")
        )
    }

    #[test]
    fn test_instantiate_with_context_simple() {
        let mut prop = Prop::True;
        let ctx = IdentifierContext::new();

        assert_eq!(prop.instantiate_parameters_with_context(&ctx), Ok(()));
    }

    #[test]
    fn test_instantiate_with_context_atom() {
        let mut prop = Prop::Atom(
            "A".to_string(),
            vec![PropParameter::Uninstantiated("x".to_string())],
        );

        let identifier = Identifier::new("x".to_string(), 42);
        let mut ctx = IdentifierContext::new();
        ctx.insert(identifier.clone(), Type::Datatype("t".to_string()));

        assert_eq!(prop.instantiate_parameters_with_context(&ctx), Ok(()));
        if let Prop::Atom(_, params) = prop {
            assert_eq!(params[0], PropParameter::Instantiated(identifier));
        } else {
            panic!("Expected Prop::Atom");
        }
    }

    #[test]
    fn test_instantiate_with_context_nested() {
        let mut prop = Prop::And(
            Box::new(Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )),
            Box::new(Prop::Atom(
                "B".to_string(),
                vec![PropParameter::Uninstantiated("y".to_string())],
            )),
        );
        let mut ctx = IdentifierContext::new();
        let fst_ident = Identifier::new("x".to_string(), 1);
        let snd_ident = Identifier::new("y".to_string(), 2);

        ctx.insert(fst_ident.clone(), Type::Datatype("t".to_string()));
        ctx.insert(snd_ident.clone(), Type::Datatype("list".to_string()));

        assert_eq!(prop.instantiate_parameters_with_context(&ctx), Ok(()));
        if let Prop::And(prop1, prop2) = prop {
            if let Prop::Atom(_, params1) = *prop1 {
                assert_eq!(params1[0], PropParameter::Instantiated(fst_ident));
            } else {
                panic!("Expected Prop::Atom");
            }
            if let Prop::Atom(_, params2) = *prop2 {
                assert_eq!(params2[0], PropParameter::Instantiated(snd_ident));
            } else {
                panic!("Expected Prop::Atom");
            }
        } else {
            panic!("Expected Prop::And with Prop::Atom");
        }
    }

    #[test]
    fn test_instantiate_with_context_forall() {
        let mut prop = Prop::ForAll {
            object_ident: "a".to_string(),
            object_type_ident: "t".to_string(),
            body: Box::new(Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("a".to_string())],
            )),
        };
        let ctx = IdentifierContext::new();

        assert_eq!(prop.instantiate_parameters_with_context(&ctx), Ok(()));
        if let Prop::ForAll { body, .. } = prop {
            if let Prop::Atom(_, params) = *body {
                assert_eq!(params[0], PropParameter::Uninstantiated("a".to_string()));
            } else {
                panic!("Expected Prop::Atom");
            }
        } else {
            panic!("Expected Prop::ForAll");
        }
    }

    #[test]
    fn test_instantiate_with_context_unknown_identifier() {
        let mut prop = Prop::Atom(
            "A".to_string(),
            vec![PropParameter::Uninstantiated("x".to_string())],
        );
        let ctx = IdentifierContext::new();

        assert_eq!(
            prop.instantiate_parameters_with_context(&ctx),
            Err(InstatiationError::UnknownIdentifier("x".to_string()))
        );
    }

    #[test]
    fn test_instantiate_free_parameter_by_index_noop_on_true() {
        let mut prop = Prop::True;
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(0, &subst);

        assert_eq!(prop, Prop::True);
    }

    #[test]
    fn test_instantiate_free_parameter_by_index_noop_on_false() {
        let mut prop = Prop::False;
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(0, &subst);

        assert_eq!(prop, Prop::False);
    }

    #[test]
    fn test_instantiate_free_parameter_by_index_substitute_in_atom_single_param() {
        let mut prop = Prop::Atom(
            "A".to_string(),
            vec![PropParameter::Uninstantiated("x".to_string())],
        );
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(0, &subst);

        assert_eq!(
            prop,
            Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Instantiated(Identifier::new(
                    "sub".to_string(),
                    42
                ))]
            )
        );
    }

    #[test]
    fn test_instantiate_free_parameter_by_index_substitute_in_atom_multi_params() {
        let mut prop = Prop::Atom(
            "A".to_string(),
            vec![
                PropParameter::Uninstantiated("x".to_string()),
                PropParameter::Uninstantiated("y".to_string()),
            ],
        );
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(1, &subst);

        assert_eq!(
            prop,
            Prop::Atom(
                "A".to_string(),
                vec![
                    PropParameter::Uninstantiated("x".to_string()),
                    PropParameter::Instantiated(Identifier::new("sub".to_string(), 42))
                ]
            )
        );
    }

    #[test]
    #[should_panic(expected = "Proposition has not enough parameters.")]
    fn test_instantiate_free_parameter_by_index_panic_on_invalid_index() {
        let mut prop = Prop::Atom(
            "A".to_string(),
            vec![PropParameter::Uninstantiated("x".to_string())],
        );
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(10, &subst);
    }

    #[test]
    fn test_instantiate_free_parameter_by_index_substitute_in_nested_and() {
        let mut prop = Prop::And(
            Box::new(Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )),
            Box::new(Prop::Atom(
                "B".to_string(),
                vec![PropParameter::Uninstantiated("y".to_string())],
            )),
        );
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(1, &subst);

        assert_eq!(
            prop,
            Prop::And(
                Box::new(Prop::Atom(
                    "A".to_string(),
                    vec![PropParameter::Uninstantiated("x".to_string())],
                )),
                Box::new(Prop::Atom(
                    "B".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "sub".to_string(),
                        42
                    ))],
                )),
            )
        );
    }

    #[test]
    fn test_instantiate_free_parameter_by_index_substitute_in_forall_body() {
        let mut prop = Prop::ForAll {
            object_ident: "x".to_string(),
            object_type_ident: "t".to_string(),
            body: Box::new(Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )),
        };
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(0, &subst);

        assert_eq!(
            prop,
            Prop::ForAll {
                object_ident: "x".to_string(),
                object_type_ident: "t".to_string(),
                body: Box::new(Prop::Atom(
                    "A".to_string(),
                    vec![PropParameter::Instantiated(Identifier::new(
                        "sub".to_string(),
                        42
                    ))],
                )),
            }
        );
    }

    #[test]
    #[should_panic(expected = "Proposition has not enough parameters.")]
    fn test_instantiate_free_parameter_by_index_panic_when_index_too_large_for_atom() {
        let mut prop = Prop::Atom(
            "P".to_string(),
            vec![PropParameter::Uninstantiated("x".to_string())],
        );
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(5, &subst);
    }

    #[test]
    #[should_panic(expected = "Proposition has not enough parameters.")]
    fn test_instantiate_free_parameter_by_index_panic_when_index_too_large_in_complex_prop() {
        let mut prop = Prop::And(
            Box::new(Prop::Atom(
                "A".to_string(),
                vec![PropParameter::Uninstantiated("x".to_string())],
            )),
            Box::new(Prop::ForAll {
                object_ident: "y".to_string(),
                object_type_ident: "t".to_string(),
                body: Box::new(Prop::Atom(
                    "B".to_string(),
                    vec![PropParameter::Uninstantiated("z".to_string())],
                )),
            }),
        );
        let subst = Identifier::new("sub".to_string(), 42);

        prop.instantiate_free_parameter_by_index(10, &subst);
    }
}
