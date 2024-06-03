use identifier_context::IdentifierContext;

use super::{
    proof_term::{ProofTerm, Type},
    prop::Prop,
};

pub mod identifier_context;

// TODO: Schon erledigt? checken das der Typ einer quantifizierten Variable auch ein Datentyp und kein Prop ist.
// TODO: Schon erledigt? checken das paramitriserte Atomns A(...) nur identifier haben die auch eingeführt wurden. (besonders (aber nicht nur) bei Exists)

pub fn typify(
    proof_term: &ProofTerm,
    datatypes: &Vec<String>,
    ctx: &mut IdentifierContext,
) -> Type {
    match proof_term {
        ProofTerm::Unit => Prop::True.into(),
        ProofTerm::Abort(body) => {
            let body_type = typify(&body, datatypes, ctx);

            if let Type::Prop(Prop::False) = body_type {
                return Prop::Any.into();
            }

            panic!("Failed to type check: abort Statement needs body of type falsum.");
        }
        ProofTerm::Pair(fst, snd) => {
            let fst_type = typify(&fst, datatypes, ctx);
            let snd_type = typify(&snd, datatypes, ctx);

            match (fst_type, snd_type) {
                (Type::Datatype(type_ident), Type::Prop(snd_prop)) => {
                    if let ProofTerm::Ident(ident) = &**fst {
                        Prop::Exists {
                            object_ident: ident.to_string(),
                            object_type_ident: type_ident.to_string(),
                            body: snd_prop.boxed(),
                        }
                    } else {
                        panic!("Architecture error: Expected identifier. Are you implementing datatytpe functions?");
                    }
                }
                (Type::Prop(fst_prop), Type::Prop(snd_prop)) => {
                    Prop::And(fst_prop.boxed(), snd_prop.boxed()).into()
                }
                (Type::Datatype(_), Type::Datatype(_)) => panic!("Snd has to be a Prop."),
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
            let body_type = typify(&body, datatypes, ctx);
            ctx.remove(&param_ident);

            match (param_type, body_type) {
                (Type::Datatype(ident), Type::Prop(body_type)) => {
                    Prop::ForAll {
                        object_ident: param_ident.clone(),
                        object_type_ident: ident.clone(),
                        body: body_type.boxed(), // TODO have I to replace parameterized props?
                    }
                }
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
            let function_type = typify(&function, datatypes, ctx);
            let applicant_type = typify(&applicant, datatypes, ctx);

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
                if applicant_type == Type::Datatype(object_type_ident) {
                    if let ProofTerm::Ident(ident) = &**applicant {
                        let mut substitued_body = *body.clone();
                        substitued_body.substitue_free_parameter(&object_ident, &ident);
                        return Type::Prop(substitued_body);
                    } else {
                        panic!("Architecture error: Expected identifier. Are you implementing datatytpe functions?");
                    }
                }

                panic!("Failed to type check: Applicant does not match.");
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
            let proof_term_type = typify(&proof_term, datatypes, ctx);

            if let Type::Prop(Prop::Or(fst, snd)) = proof_term_type {
                // fst
                ctx.insert(left_ident.clone(), Type::Prop(*fst));
                let fst_type = typify(&left_term, datatypes, ctx);
                ctx.remove(&left_ident);

                // snd
                ctx.insert(right_ident.clone(), Type::Prop(*snd));
                let snd_type = typify(&right_term, datatypes, ctx);
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
            let body_type = typify(&body, datatypes, ctx);

            if let Type::Prop(Prop::And(fst, _)) = body_type {
                return Type::Prop(*fst);
            }

            panic!("Failed to type check: Body is not a Pair.");
        }

        ProofTerm::ProjectSnd(body) => {
            let body_type = typify(&body, datatypes, ctx);

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
            let pair_proof_term_type = typify(&pair_proof_term, datatypes, ctx);

            if let Type::Prop(Prop::Exists {
                object_ident,
                object_type_ident,
                body: exists_body,
            }) = pair_proof_term_type
            {
                ctx.insert(fst_ident.clone(), Type::Datatype(object_type_ident));
                ctx.insert(snd_ident.clone(), Type::Prop(*exists_body));

                let body_type = typify(&body, datatypes, ctx);

                ctx.remove(&fst_ident);
                ctx.remove(&snd_ident);

                // check that quantified object does not escape it's scope
                if let Type::Prop(prop) = &body_type {
                    if prop.get_free_parameters().contains(&object_ident) {
                        panic!("Failed to type check: Quantified object cannot escape it's scope.");
                    }
                }

                body_type
            } else {
                panic!("Failed to type check: Proof Term is not an Exists pair.")
            }
        }

        ProofTerm::OrLeft { body, other } => {
            let body_type = typify(&body, datatypes, ctx);

            if let Type::Prop(body_prop) = body_type {
                Type::Prop(Prop::Or(body_prop.boxed(), other.boxed()))
            } else {
                panic!("Failed to type check: Expected Prop")
            }
        }

        ProofTerm::OrRight { body, other } => {
            let body_type = typify(&body, datatypes, ctx);

            if let Type::Prop(body_prop) = body_type {
                Type::Prop(Prop::Or(other.boxed(), body_prop.boxed()))
            } else {
                panic!("Failed to type check: Expected Prop")
            }
        }
    }
}
