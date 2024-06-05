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
                body: mut exists_body,
            }) = pair_proof_term_type
            {
                exists_body.substitue_free_parameter(&object_ident, &fst_ident);
                ctx.insert(fst_ident.clone(), Type::Datatype(object_type_ident));
                ctx.insert(snd_ident.clone(), Type::Prop(*exists_body));

                let body_type = typify(&body, datatypes, ctx);

                ctx.remove(&fst_ident);
                ctx.remove(&snd_ident);

                // check that quantified object does not escape it's scope
                if let Type::Prop(prop) = &body_type {
                    println!("{:#?}", object_ident);
                    if prop.get_free_parameters().contains(&fst_ident) {
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

#[cfg(test)]
mod tests {
    use std::vec;

    use chumsky::Parser;

    use crate::core::{
        parse::{
            lexer::{self, lexer},
            proof::proof_parser,
            proof_term::proof_term_parser,
        },
        process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline},
        proof_term::Type,
        prop::Prop,
    };

    use super::{identifier_context::IdentifierContext, typify};

    #[test]
    fn test_proof_implication_to_and() {
        let tokens = lexer().parse("fn u: A => fn w: B => (u, w)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Impl(
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                    Prop::And(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("B".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                )
                .boxed(),
            ))
        );
    }

    #[test]
    fn test_commutativity_of_conjunction() {
        let tokens = lexer().parse("fn u: (A && B) => (snd u, fst u)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::And(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed()
                )
                .boxed(),
                Prop::And(
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                    Prop::Atom("A".to_string(), vec![]).boxed()
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_interaction_law_of_distributivity() {
        let tokens = lexer()
            .parse("fn u: (A -> (B & C)) => (fn w: A => fst (u w), fn w: A => snd (u w))")
            .unwrap();

        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Impl(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::And(
                        Prop::Atom("B".to_string(), vec![]).boxed(),
                        Prop::Atom("C".to_string(), vec![]).boxed(),
                    )
                    .boxed(),
                )
                .boxed(),
                Prop::And(
                    Prop::Impl(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("B".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                    Prop::Impl(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("C".to_string(), vec![]).boxed()
                    )
                    .boxed()
                )
                .boxed()
            )),
        )
    }

    #[test]
    fn test_commutativity_of_disjunction() {
        let tokens = lexer()
            .parse("fn u: A || B => case u of inl a => inr<B> a, inr b => inl<A> b")
            .unwrap();

        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Or(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                )
                .boxed(),
                Prop::Or(
                    Prop::Atom("B".to_string(), vec![]).boxed(),
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_true() {
        let tokens = lexer().parse("()").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(_type, Type::Prop(Prop::True))
    }

    #[test]
    fn test_composition() {
        let tokens = lexer()
            .parse("fn u: ((A -> B) && (B -> C)) => fn w: A => (snd u) ((fst u) w)")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::And(
                    Prop::Impl(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("B".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                    Prop::Impl(
                        Prop::Atom("B".to_string(), vec![]).boxed(),
                        Prop::Atom("C".to_string(), vec![]).boxed()
                    )
                    .boxed()
                )
                .boxed(),
                Prop::Impl(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("C".to_string(), vec![]).boxed()
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_composition_of_identities() {
        let tokens = lexer().parse("(fn u: ((A -> A) && (A -> A)) => fn w: A => (snd u) ((fst u) w)) ((fn x: A => x), (fn y: A => y))").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("A".to_string(), vec![]).boxed()
            ))
        )
    }

    #[test]
    fn test_non_minimal_identity_proof() {
        let tokens = lexer().parse("fn u: A => (fn w: A => w) u").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Atom("A".to_string(), vec![]).boxed()
            ))
        )
    }

    #[test]
    fn test_projection_function() {
        let tokens = lexer().parse("fn u: A & B => fst u").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::And(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed()
                )
                .boxed(),
                Prop::Atom("A".to_string(), vec![]).boxed(),
            ))
        )
    }

    #[test]
    fn test_implication_chain() {
        let tokens = lexer()
            .parse("fn u: (A -> A) -> B => u (fn u: A => u)")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Impl(
                    Prop::Impl(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("A".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                    Prop::Atom("B".to_string(), vec![]).boxed()
                )
                .boxed(),
                Prop::Atom("B".to_string(), vec![]).boxed()
            ))
        )
    }

    #[test]
    fn test_piano_number_2() {
        let tokens = lexer().parse("fn z: A => fn s: A -> A => s(s(z))").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Atom("A".to_string(), vec![]).boxed(),
                Prop::Impl(
                    Prop::Impl(
                        Prop::Atom("A".to_string(), vec![]).boxed(),
                        Prop::Atom("A".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                    Prop::Atom("A".to_string(), vec![]).boxed()
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_tripple_neagation_elimination() {
        // ~~~A = ((A => False) => False) => False
        let tokens = lexer()
            .parse("fn u: (~~~A) => fn v: A => u (fn w: A -> \\bot => w v)")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast, &vec![], &mut IdentifierContext::new());

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::Impl(
                    Prop::Impl(
                        Prop::Impl(
                            Prop::Atom("A".to_string(), vec![]).boxed(),
                            Prop::False.boxed()
                        )
                        .boxed(),
                        Prop::False.boxed()
                    )
                    .boxed(),
                    Prop::False.boxed()
                )
                .boxed(),
                Prop::Impl(
                    Prop::Atom("A".to_string(), vec![]).boxed(),
                    Prop::False.boxed()
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_allquant_distribution() {
        let tokens = lexer()
            .parse(
                "
                datatype t;
                fn u: (\\forall x:t. A(x) & B(x)) => (
                fn x: t => fst (u x),
                fn x: t => snd (u x)
            )",
            )
            .unwrap();
        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let _type = typify(
            &proof.proof_term,
            &proof.datatypes,
            &mut IdentifierContext::new(),
        );

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::ForAll {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::And(
                        Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
                        Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed()
                    )
                    .boxed()
                }
                .boxed(),
                Prop::And(
                    Prop::ForAll {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed()
                    }
                    .boxed(),
                    Prop::ForAll {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom("B".to_string(), vec!["x".to_string()]).boxed()
                    }
                    .boxed()
                )
                .boxed()
            ))
        )
    }

    #[test]
    fn test_reusing_allquant_ident() {
        let tokens = lexer()
            .parse("datatype t; fn u: (∀x:t. C(x, x)) => fn a:t => fn a:t => u a")
            .unwrap();

        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let _type = typify(
            &proof.proof_term,
            &proof.datatypes,
            &mut IdentifierContext::new(),
        );

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::ForAll {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Atom("C".to_string(), vec!["x".to_string(), "x".to_string()])
                        .boxed()
                }
                .boxed(),
                Prop::ForAll {
                    object_ident: "a".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::ForAll {
                        object_ident: "a".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom("C".to_string(), vec!["a".to_string(), "a".to_string()])
                            .boxed()
                    }
                    .boxed()
                }
                .boxed()
            ))
        )
    }

    #[test]
    fn test_exsists_move_unquantified() {
        let tokens = lexer().parse("datatype t; fn u: (\\forall x:t. A(x) -> C) => fn w: \\exists x:t. A(x) => let (a, proof) = w in u a proof").unwrap();

        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let _type = typify(
            &proof.proof_term,
            &proof.datatypes,
            &mut IdentifierContext::new(),
        );

        assert_eq!(
            _type,
            Type::Prop(Prop::Impl(
                Prop::ForAll {
                    object_ident: "x".to_string(),
                    object_type_ident: "t".to_string(),
                    body: Prop::Impl(
                        Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
                        Prop::Atom("C".to_string(), vec![]).boxed()
                    )
                    .boxed(),
                }
                .boxed(),
                Prop::Impl(
                    Prop::Exists {
                        object_ident: "x".to_string(),
                        object_type_ident: "t".to_string(),
                        body: Prop::Atom("A".to_string(), vec!["x".to_string()]).boxed(),
                    }
                    .boxed(),
                    Prop::Atom("C".to_string(), vec![]).boxed(),
                )
                .boxed(),
            ))
        )
    }

    #[test]
    #[should_panic]
    fn test_do_not_allow_exists_quant_escape() {
        let tokens = lexer()
            .parse("datatype t; fn u: \\exists x:t. C(x) => let (a, proof) = u in proof")
            .unwrap();

        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let _type = typify(
            &proof.proof_term,
            &proof.datatypes,
            &mut IdentifierContext::new(),
        );
    }
}
