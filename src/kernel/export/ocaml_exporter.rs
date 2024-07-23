use crate::kernel::proof_term::{
    Abort, Application, Case, Function, Ident, OrLeft, OrRight, Pair, ProjectFst, ProjectSnd,
    ProofTerm, Type, TypeAscription,
};

use super::ProofExporter;

const PREAMBLE: &'static str = "type empty = | 
type ('a, 'b) disjunction = Inl of 'a | Inr of 'b
let fst (x, _) = x
let snd (_, y) = y
let abort m : empty -> 'a = failwith \"abort\"
let rec sorry () = sorry ()
";

pub struct OcamlExporter {}

impl OcamlExporter {
    pub fn new() -> Self {
        Self {}
    }

    fn should_wrap_unary(parent_precedence: usize, child_precedence: usize) -> bool {
        parent_precedence > child_precedence
    }

    fn generate_ocaml_term(proof_term: &ProofTerm) -> String {
        match proof_term {
            ProofTerm::Unit => "()".to_string(),
            ProofTerm::Sorry => "sorry ()".to_string(),
            ProofTerm::Ident(Ident(ident)) => ident.clone(),
            ProofTerm::Abort(Abort(body)) => {
                if Self::should_wrap_unary(proof_term.precedence(), body.precedence()) {
                    format!("abort ({})", Self::generate_ocaml_term(body.as_ref()))
                } else {
                    format!("abort {}", Self::generate_ocaml_term(body.as_ref()))
                }
            }
            ProofTerm::Pair(Pair(fst, snd)) => format!(
                "({}, {})",
                Self::generate_ocaml_term(fst),
                Self::generate_ocaml_term(snd)
            ),
            ProofTerm::ProjectFst(ProjectFst(body)) => {
                if Self::should_wrap_unary(proof_term.precedence(), body.precedence()) {
                    format!("fst ({})", Self::generate_ocaml_term(body))
                } else {
                    format!("fst {}", Self::generate_ocaml_term(body))
                }
            }
            ProofTerm::ProjectSnd(ProjectSnd(body)) => {
                if Self::should_wrap_unary(proof_term.precedence(), body.precedence()) {
                    format!("snd ({})", Self::generate_ocaml_term(body))
                } else {
                    format!("snd {}", Self::generate_ocaml_term(body))
                }
            }
            ProofTerm::OrLeft(OrLeft(body)) => {
                if Self::should_wrap_unary(proof_term.precedence(), body.precedence()) {
                    format!("Inl ({})", body)
                } else {
                    format!("Inl {}", body)
                }
            }
            ProofTerm::OrRight(OrRight(body)) => {
                if Self::should_wrap_unary(proof_term.precedence(), body.precedence()) {
                    format!("Inr ({})", body)
                } else {
                    format!("Inr {}", body)
                }
            }
            ProofTerm::Case(Case {
                head,
                fst_ident,
                fst_term,
                snd_ident,
                snd_term,
            }) => {
                format!(
                    "match {} with | Inl {} -> {} | Inr {} -> {}",
                    Self::generate_ocaml_term(head),
                    fst_ident,
                    Self::generate_ocaml_term(fst_term),
                    snd_ident,
                    Self::generate_ocaml_term(snd_term),
                )
            }
            ProofTerm::Function(Function {
                param_ident, body, ..
            }) => {
                format!("fun {} -> {}", param_ident, Self::generate_ocaml_term(body))
            }
            ProofTerm::Application(Application {
                function,
                applicant,
            }) => {
                let own_precedence = proof_term.precedence();
                let function_precedence = function.precedence();
                let applicant_precedence = applicant.precedence();

                let should_wrap_left = (function_precedence < own_precedence)
                    || ((function_precedence == own_precedence) && function.right_associative());
                let should_wrap_right = (applicant_precedence < own_precedence)
                    || ((applicant_precedence == own_precedence) && applicant.left_associative());

                let left_side = if should_wrap_left {
                    format!("({})", Self::generate_ocaml_term(function))
                } else {
                    format!("{}", Self::generate_ocaml_term(function))
                };

                let right_side = if should_wrap_right {
                    format!("({})", Self::generate_ocaml_term(applicant))
                } else {
                    format!("{}", Self::generate_ocaml_term(applicant))
                };

                format!("{} {}", left_side, right_side,)
            }
            ProofTerm::TypeAscription(TypeAscription { proof_term, .. }) => {
                Self::generate_ocaml_term(proof_term)
            }
            ProofTerm::LetIn(_) => panic!(""),
        }
    }
}

impl ProofExporter for OcamlExporter {
    fn export(&self, proof_term: &ProofTerm) -> String {
        let code = Self::generate_ocaml_term(proof_term);

        format!("{} \nlet proof = {}", PREAMBLE, code)
    }

    fn can_export(&self, proof_term: &ProofTerm) -> bool {
        match proof_term {
            ProofTerm::Unit => true,
            ProofTerm::Ident(_) => true,
            ProofTerm::Sorry => true,
            ProofTerm::TypeAscription(_) => true,
            ProofTerm::Abort(Abort(body)) => self.can_export(body),
            ProofTerm::OrLeft(OrLeft(body)) => self.can_export(body),
            ProofTerm::OrRight(OrRight(body)) => self.can_export(body),
            ProofTerm::Case(Case {
                head,
                fst_term,
                snd_term,
                ..
            }) => self.can_export(head) && self.can_export(&fst_term) && self.can_export(&snd_term),
            ProofTerm::Pair(Pair(fst, snd)) => self.can_export(fst) && self.can_export(snd),
            ProofTerm::ProjectFst(ProjectFst(body)) => self.can_export(body),
            ProofTerm::ProjectSnd(ProjectSnd(body)) => self.can_export(body),
            ProofTerm::Application(Application {
                function,
                applicant,
            }) => self.can_export(&function) && self.can_export(&applicant),
            ProofTerm::Function(Function {
                param_type, body, ..
            }) => {
                if let Some(param_type) = param_type {
                    if param_type.has_quantifiers() {
                        return false;
                    }
                }

                self.can_export(body)
            }
            ProofTerm::LetIn(_) => false,
        }
    }

    fn can_export_for_type(&self, _type: Type) -> bool {
        let Type::Prop(prop) = _type else {
            return false;
        };

        prop.has_quantifiers()
    }
}
