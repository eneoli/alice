use crate::core::{
    process::ProofPipelineStage,
    proof::Proof,
    proof_term::{ProofTerm, Type},
    prop::Prop,
};

pub struct ResolveDatatypes {}

impl ResolveDatatypes {
    pub fn boxed() -> Box<Self> {
        Box::new(Self {})
    }

    fn has_datatype_identifier(&self, prop: &Prop, datatypes: &Vec<String>) -> bool {
        match prop {
            Prop::Atom(ident, _) => datatypes.contains(&ident),
            Prop::And(fst, snd) => {
                self.has_datatype_identifier(fst, datatypes)
                    || self.has_datatype_identifier(snd, datatypes)
            }
            Prop::Or(fst, snd) => {
                self.has_datatype_identifier(fst, datatypes)
                    || self.has_datatype_identifier(snd, datatypes)
            }
            Prop::Any => false,
            Prop::False => false,
            Prop::True => false,
            Prop::Impl(fst, snd) => {
                self.has_datatype_identifier(fst, datatypes)
                    || self.has_datatype_identifier(snd, datatypes)
            }
            Prop::Exists { body, .. } => self.has_datatype_identifier(body, datatypes),
            Prop::ForAll { body, .. } => self.has_datatype_identifier(body, datatypes),
        }
    }

    fn resolve_datatypes(&self, proof_term: ProofTerm, datatypes: &Vec<String>) -> ProofTerm {
        match proof_term {
            ProofTerm::Unit => ProofTerm::Unit,
            ProofTerm::Ident(ident) => ProofTerm::Ident(ident),
            ProofTerm::Pair(fst, snd) => ProofTerm::Pair(
                self.resolve_datatypes(*fst, datatypes).boxed(),
                self.resolve_datatypes(*snd, datatypes).boxed(),
            ),
            ProofTerm::Abort(body) => {
                ProofTerm::Abort(self.resolve_datatypes(*body, datatypes).boxed())
            }
            ProofTerm::Application {
                function,
                applicant,
            } => ProofTerm::Application {
                function: self.resolve_datatypes(*function, datatypes).boxed(),
                applicant: self.resolve_datatypes(*applicant, datatypes).boxed(),
            },
            ProofTerm::Case {
                proof_term,
                left_ident,
                left_term,
                right_ident,
                right_term,
            } => ProofTerm::Case {
                proof_term: self.resolve_datatypes(*proof_term, datatypes).boxed(),
                left_ident,
                left_term: self.resolve_datatypes(*left_term, datatypes).boxed(),
                right_ident,
                right_term: self.resolve_datatypes(*right_term, datatypes).boxed(),
            },
            ProofTerm::Function {
                param_ident,
                param_type,
                body,
            } => {
                let real_param_type = match param_type {
                    Type::Prop(Prop::Atom(ident, params))
                        if params.is_empty() && datatypes.contains(&ident) =>
                    {
                        Type::Datatype(ident.clone())
                    }
                    Type::Prop(prop) if self.has_datatype_identifier(&prop, &datatypes) => {
                        panic!("Props are not allowed to contain datatype identifers")
                    }
                    _ => param_type,
                };

                ProofTerm::Function {
                    param_ident,
                    param_type: real_param_type,
                    body: self.resolve_datatypes(*body, datatypes).boxed(),
                }
            }

            ProofTerm::LetIn {
                fst_ident,
                snd_ident,
                pair_proof_term,
                body,
            } => ProofTerm::LetIn {
                fst_ident,
                snd_ident,
                pair_proof_term: self.resolve_datatypes(*pair_proof_term, datatypes).boxed(),
                body: self.resolve_datatypes(*body, datatypes).boxed(),
            },

            ProofTerm::OrLeft { body, other } => {
                ProofTerm::OrLeft { body: self.resolve_datatypes(*body, datatypes).boxed(), other }
            }
            ProofTerm::OrRight { body, other } => {
                ProofTerm::OrRight {body: self.resolve_datatypes(*body, datatypes).boxed(), other }
            }
            ProofTerm::ProjectFst(body) => {
                ProofTerm::ProjectFst(self.resolve_datatypes(*body, datatypes).boxed())
            }
            ProofTerm::ProjectSnd(body) => {
                ProofTerm::ProjectSnd(self.resolve_datatypes(*body, datatypes).boxed())
            }
        }
    }
}

impl ProofPipelineStage for ResolveDatatypes {
    fn process(&self, proof: Proof) -> Proof {
        let Proof {
            proof_term,
            datatypes,
        } = proof;

        let new_proof_term = self.resolve_datatypes(proof_term, &datatypes);

        Proof {
            proof_term: new_proof_term,
            datatypes,
        }
    }
}
