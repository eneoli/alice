use check::identifier_context::IdentifierContext;
use core::panic;
use proof_term::{ProofTerm, Type};

use self::prop::Prop;

pub mod check;
pub mod parse;
pub mod process;
pub mod proof;
pub mod proof_term;
pub mod prop;
pub mod prove;

#[derive(Clone, Debug)]
pub struct Sequent {
    goal: Prop,
    inv_ctx: Vec<Prop>,
    non_inv_ctx: Vec<Prop>,
}

impl Sequent {
    pub fn new(goal: &Prop) -> Self {
        Sequent {
            goal: goal.clone(),
            inv_ctx: Vec::new(),
            non_inv_ctx: Vec::new(),
        }
    }

    pub fn change_goal(&mut self, new_goal: &Prop) -> &mut Self {
        self.goal = new_goal.clone();

        self
    }

    pub fn with_new_goal(&self, new_goal: &Prop) -> Self {
        let mut seq = self.clone();
        seq.goal = new_goal.clone();

        seq
    }

    pub fn push_inv(&mut self, prop: &Prop) -> &mut Self {
        self.inv_ctx.push(prop.clone());

        self
    }

    pub fn pop_inv(&mut self) -> Option<Prop> {
        self.inv_ctx.pop()
    }

    pub fn add_non_inv(&mut self, prop: &Prop) -> &mut Self {
        self.non_inv_ctx.push(prop.clone());

        self
    }

    pub fn ctx_contains(&self, prop: &Prop) -> bool {
        self.inv_ctx.contains(prop) || self.non_inv_ctx.contains(prop)
    }
}

// TODO: checken das der Typ einer quantifizierten Variable auch ein Datentyp und kein Prop ist.
// TODO: checken das paramitriserte Atomns A(...) nur identifier haben die auch eingeführt wurden.

pub fn typify(proof_term: ProofTerm, datatypes: &Vec<String>, ctx: &mut IdentifierContext) -> Type {
    match proof_term {
        ProofTerm::Unit => Prop::True.into(),
        ProofTerm::Abort(body) => {
            let body_type = typify(*body, datatypes, ctx);

            if let Type::Prop(Prop::False) = body_type {
                return Prop::Any.into();
            }

            panic!("Failed to type check: abort Statement needs body of type falsum.");
        }
        ProofTerm::Pair(fst, snd) => {
            let fst_type = typify(*fst, datatypes, ctx);
            let snd_type = typify(*snd, datatypes, ctx);

            match (fst_type, snd_type) {
                (Type::Datatype(ident), Type::Prop(snd_prop)) => Prop::Exists {
                    object_ident: "x".to_string(), // TODO: unique
                    object_type_ident: ident,
                    body: snd_prop.boxed(),
                },
                (Type::Prop(fst_prop), Type::Prop(snd_prop)) => {
                    Prop::And(fst_prop.boxed(), snd_prop.boxed()).into()
                }
                (Type::Datatype(ident), Type::Datatype(_)) => panic!("Snd has to be a Prop."),
                (Type::Prop(_), Type::Datatype(_)) => panic!("Snd has to be a Prop."),
            }
            .into()
        }
        ProofTerm::Ident(ident) => {
            let ident_type = ctx.get(&ident);

            if let Some(_type) = ident_type {
                _type.clone()
            } else {
                panic!("Failed to type check: Unknown symbol: {}", ident)
            }
        }
        ProofTerm::Function {
            param_ident,
            param_type,
            body,
        } => {
            ctx.insert(param_ident.clone(), param_type.clone());
            let body_type = typify(*body, datatypes, ctx);
            ctx.remove(&param_ident);

            match (param_type, body_type) {
                (Type::Datatype(ident), Type::Prop(body_type)) => Prop::ForAll {
                    object_ident: param_ident,
                    object_type_ident: ident.clone(),
                    body: body_type.boxed(),
                },
                (Type::Prop(fst), Type::Prop(snd)) => Prop::Impl(fst.boxed(), snd.boxed()),
                (Type::Datatype(_), Type::Datatype(_)) => panic!("Body has to be Prop."),
                (Type::Prop(_), Type::Datatype(_)) => panic!("Body has to be Prop."),
            }
            .into()
        }
        ProofTerm::Application {
            function,
            applicant,
        } => {
            let function_type = typify(*function, datatypes, ctx);
            let applicant_type = typify(*applicant, datatypes, ctx);

            // either implication or allquant

            if let Type::Prop(Prop::Impl(param_type, body_type)) = function_type {
                if Type::Prop(*param_type) != applicant_type {
                    panic!("Failed to type check: Applicant type does not match.");
                }

                return Type::Prop(*body_type);
            }

            if let Type::Prop(Prop::ForAll {
                object_ident,
                object_type_ident,
                body,
            }) = function_type
            {
                // check if applicant is datatype
                if let Type::Datatype(ident) = applicant_type {
                    return Type::Prop(*body);
                }

                panic!("Failed to type check: Expected data type.");
            }

            panic!("Failed to type check: Not a function.")
        }

        ProofTerm::Case {
            proof_term,
            left_ident,
            left_term,
            right_ident,
            right_term,
        } => {
            let proof_term_type = typify(*proof_term, datatypes, ctx);

            if let Type::Prop(Prop::And(fst, snd)) = proof_term_type {
                // fst
                ctx.insert(left_ident.clone(), Type::Prop(*fst));
                let fst_type = typify(*left_term, datatypes, ctx);
                ctx.remove(&left_ident);

                // snd
                ctx.insert(right_ident.clone(), Type::Prop(*snd));
                let snd_type = typify(*right_term, datatypes, ctx);
                ctx.remove(&right_ident);

                if fst_type != snd_type {
                    panic!("Failed to type check: Both arms of Case expr need to have same type.");
                }

                fst_type
            } else {
                panic!("Failed to type check: Case proof term is not a pair.");
            }
        }
        ProofTerm::ProjectFst(body) => {
            let body_type = typify(*body, datatypes, ctx);

            if let Type::Prop(Prop::And(fst, _)) = body_type {
                return Type::Prop(*fst);
            }

            panic!("Failed to type check: Body is not a Pair.");
        }

        ProofTerm::ProjectSnd(body) => {
            let body_type = typify(*body, datatypes, ctx);

            if let Type::Prop(Prop::And(_, snd)) = body_type {
                return Type::Prop(*snd);
            }

            panic!("Failed to type check: Body is not a Pair.");
        }
        ProofTerm::LetIn {
            fst_ident,
            snd_ident,
            pair_proof_term,
            body,
        } => {
            let pair_proof_term_type = typify(*pair_proof_term, datatypes, ctx);

            if let Type::Prop(Prop::Exists {
                object_ident,
                object_type_ident,
                body: exists_body,
            }) = pair_proof_term_type
            {
                ctx.insert(fst_ident.clone(), Type::Datatype(object_type_ident));
                ctx.insert(snd_ident.clone(), Type::Prop(*exists_body));

                let body_type = typify(*body, datatypes, ctx);

                ctx.remove(&fst_ident);
                ctx.remove(&snd_ident);

                body_type
            } else {
                panic!("Failed to type check: Proof Term is not an Exists pair.")
            }
        }

        ProofTerm::OrLeft(body) => {
            let body_type = typify(*body, datatypes, ctx);

            todo!()
        }

        ProofTerm::OrRight(body) => {
            let body_type = typify(*body, datatypes, ctx);

            todo!()
        }
        _ => todo!(),
    }
}
