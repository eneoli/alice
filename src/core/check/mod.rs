use identifier_context::IdentifierContext;

use super::{
    proof_term::{ProofTerm, ProofTermVisitor, Type},
    prop::Prop,
};

pub mod identifier_context;

// TODO: Schon erledigt? checken das der Typ einer quantifizierten Variable auch ein Datentyp und kein Prop ist.
// TODO: Schon erledigt? checken das paramitriserte Atomns A(...) nur identifier haben die auch eingeführt wurden. (besonders (aber nicht nur) bei Exists)

#[derive(Debug, PartialEq, Eq)]
pub enum TypeError {
    UnexpectedType { expected: Type, received: Type },
    ExpectedProp { received: Type },
    UnknownIdent { ident: String },
    QuantifiedObjectEscapesScope,
}

struct TypifyVisitor {
    ctx: IdentifierContext,
}

impl TypifyVisitor {
    pub fn new() -> Self {
        Self {
            ctx: IdentifierContext::new(),
        }
    }
}

impl ProofTermVisitor<Result<Type, TypeError>> for TypifyVisitor {
    fn visit_ident(&mut self, ident: &String) -> Result<Type, TypeError> {
        let ident_type = self.ctx.get(&ident);

        if let Some(_type) = ident_type {
            Ok(_type.clone().into())
        } else {
            Err(TypeError::UnknownIdent {
                ident: ident.to_string(),
            })
        }
    }

    fn visit_pair(&mut self, fst: &ProofTerm, snd: &ProofTerm) -> Result<Type, TypeError> {
        let fst_type = fst.visit(self)?;
        let snd_type = snd.visit(self)?;

        match (&fst_type, &snd_type) {
            (Type::Datatype(type_ident), Type::Prop(snd_prop)) => {
                if let ProofTerm::Ident(ident) = fst {
                    Ok(Prop::Exists {
                        object_ident: ident.to_string(),
                        object_type_ident: type_ident.to_string(),
                        body: snd_prop.boxed(),
                    }
                    .into())
                } else {
                    panic!("Architecture error: Expected identifier. Are you implementing datatytpe functions?")
                }
            }
            (Type::Prop(fst_prop), Type::Prop(snd_prop)) => {
                Ok(Prop::And(fst_prop.boxed(), snd_prop.boxed()).into())
            }
            (_, Type::Datatype(_)) => Err(TypeError::ExpectedProp {
                received: snd_type.clone(),
            }),
        }
    }

    fn visit_project_fst(&mut self, body: &ProofTerm) -> Result<Type, TypeError> {
        let body_type = body.visit(self)?;

        if let Type::Prop(Prop::And(fst, _)) = body_type {
            return Ok(Type::Prop(*fst));
        }

        panic!("Failed to type check: Body is not a Pair.");
    }

    fn visit_project_snd(&mut self, body: &ProofTerm) -> Result<Type, TypeError> {
        let body_type = body.visit(self)?;

        if let Type::Prop(Prop::And(_, snd)) = body_type {
            return Ok(Type::Prop(*snd));
        }

        panic!("Failed to type check: Body is not a Pair.");
    }

    fn visit_function(
        &mut self,
        param_ident: &String,
        param_type: &Type,
        body: &ProofTerm,
    ) -> Result<Type, TypeError> {
        self.ctx.insert(param_ident.clone(), param_type.clone());
        let body_type = body.visit(self)?;
        self.ctx.remove(&param_ident);

        match (&param_type, &body_type) {
            (Type::Datatype(ident), Type::Prop(body_type)) => {
                Ok(Prop::ForAll {
                    object_ident: param_ident.clone(),
                    object_type_ident: ident.clone(),
                    body: body_type.boxed(), // TODO have I to replace parameterized props?
                }
                .into())
            }
            (Type::Prop(fst), Type::Prop(snd)) => Ok(Prop::Impl(fst.boxed(), snd.boxed()).into()),
            (_, Type::Datatype(_)) => Err(TypeError::ExpectedProp {
                received: body_type,
            }),
        }
        .into()
    }

    fn visit_application(
        &mut self,
        function: &ProofTerm,
        applicant: &ProofTerm,
    ) -> Result<Type, TypeError> {
        let function_type = function.visit(self)?;
        let applicant_type = applicant.visit(self)?;

        // either implication or allquant

        if let Type::Prop(Prop::Impl(param_prop, body_type)) = function_type {
            let param_type = Type::Prop(*param_prop);
            if param_type != applicant_type {
                return Err(TypeError::UnexpectedType {
                    expected: param_type,
                    received: applicant_type,
                });
            }

            return Ok(Type::Prop(*body_type));
        }

        if let Type::Prop(Prop::ForAll {
            object_ident,
            object_type_ident,
            body,
        }) = function_type
        {
            // check if applicant is datatype
            let object_type = Type::Datatype(object_type_ident);
            if applicant_type == object_type {
                if let ProofTerm::Ident(ident) = applicant {
                    let mut substitued_body = *body.clone();
                    substitued_body.substitue_free_parameter(&object_ident, &ident);
                    return Ok(Type::Prop(substitued_body));
                } else {
                    panic!("Architecture error: Expected identifier. Are you implementing datatytpe functions?")
                }
            }

            return Err(TypeError::UnexpectedType {
                expected: object_type,
                received: applicant_type,
            });
        }

        Err(TypeError::UnexpectedType {
            expected: Type::Prop(Prop::False), // TODO Function/Allquant
            received: function_type,
        })
    }

    fn visit_let_in(
        &mut self,
        fst_ident: &String,
        snd_ident: &String,
        pair_proof_term: &ProofTerm,
        body: &ProofTerm,
    ) -> Result<Type, TypeError> {
        let pair_proof_term_type = pair_proof_term.visit(self)?;

        if let Type::Prop(Prop::Exists {
            object_ident,
            object_type_ident,
            body: mut exists_body,
        }) = pair_proof_term_type
        {
            exists_body.substitue_free_parameter(&object_ident, &fst_ident);
            self.ctx
                .insert(fst_ident.clone(), Type::Datatype(object_type_ident));
            self.ctx.insert(snd_ident.clone(), Type::Prop(*exists_body));

            let body_type = body.visit(self)?;

            self.ctx.remove(&fst_ident);
            self.ctx.remove(&snd_ident);

            // check that quantified object does not escape it's scope
            if let Type::Prop(prop) = &body_type {
                if prop.get_free_parameters().contains(&fst_ident) {
                    return Err(TypeError::QuantifiedObjectEscapesScope);
                }
            }

            Ok(body_type)
        } else {
            panic!("Failed to type check: Proof Term is not an Exists pair.")
        }
    }

    fn visit_or_left(&mut self, body: &ProofTerm, other: &Prop) -> Result<Type, TypeError> {
        let body_type = body.visit(self)?;

        if let Type::Prop(body_prop) = body_type {
            Ok(Type::Prop(Prop::Or(body_prop.boxed(), other.boxed())))
        } else {
            panic!("Failed to type check: Expected Prop")
        }
    }

    fn visit_or_right(&mut self, body: &ProofTerm, other: &Prop) -> Result<Type, TypeError> {
        let body_type = body.visit(self)?;

        if let Type::Prop(body_prop) = body_type {
            Ok(Type::Prop(Prop::Or(other.boxed(), body_prop.boxed())))
        } else {
            panic!("Failed to type check: Expected Prop")
        }
    }

    fn visit_case(
        &mut self,
        proof_term: &ProofTerm,
        left_ident: &String,
        left_term: &ProofTerm,
        right_ident: &String,
        right_term: &ProofTerm,
    ) -> Result<Type, TypeError> {
        let proof_term_type = proof_term.visit(self)?;

        if let Type::Prop(Prop::Or(fst, snd)) = proof_term_type {
            // fst
            self.ctx.insert(left_ident.clone(), Type::Prop(*fst));
            let fst_type = left_term.visit(self)?;
            self.ctx.remove(&left_ident);

            // snd
            self.ctx.insert(right_ident.clone(), Type::Prop(*snd));
            let snd_type = right_term.visit(self)?;
            self.ctx.remove(&right_ident);

            println!("{:#?}", fst_type);
            println!("{:#?}", snd_type);

            if fst_type != snd_type {
                panic!("Failed to type check: Both arms of Case expr need to have same type.");
            }

            Ok(fst_type)
        } else {
            panic!("Failed to type check: Case proof term is not a pair.");
        }
    }

    fn visit_abort(&mut self, body: &ProofTerm) -> Result<Type, TypeError> {
        let body_type = body.visit(self)?;

        if let Type::Prop(Prop::False) = body_type {
            Ok(Prop::Any.into())
        } else {
            Err(TypeError::UnexpectedType {
                expected: Prop::False.into(),
                received: body_type,
            })
        }
    }

    fn visit_unit(&mut self) -> Result<Type, TypeError> {
        Ok(Prop::True.into())
    }
}

pub fn typify(proof_term: &ProofTerm) -> Result<Type, TypeError> {
    let mut visitor = TypifyVisitor::new();
    proof_term.visit(&mut visitor)
}

// === TESTS ===

#[cfg(test)]
mod tests {
    use std::vec;

    use chumsky::Parser;

    use crate::core::{
        check::{typify, TypeError},
        parse::{lexer::lexer, proof::proof_parser, proof_term::proof_term_parser},
        process::{stages::resolve_datatypes::ResolveDatatypes, ProofPipeline},
        proof_term::Type,
        prop::Prop,
    };

    #[test]
    fn test_proof_implication_to_and() {
        let tokens = lexer().parse("fn u: A => fn w: B => (u, w)").unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

        assert_eq!(_type, Type::Prop(Prop::True))
    }

    #[test]
    fn test_composition() {
        let tokens = lexer()
            .parse("fn u: ((A -> B) && (B -> C)) => fn w: A => (snd u) ((fst u) w)")
            .unwrap();
        let ast = proof_term_parser().parse(tokens).unwrap();
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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
        let _type = typify(&ast).unwrap();

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

        let _type = typify(&proof.proof_term).unwrap();

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

        let _type = typify(&proof.proof_term).unwrap();

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

        let _type = typify(&proof.proof_term).unwrap();

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
    fn test_do_not_allow_exists_quant_escape() {
        let tokens = lexer()
            .parse("datatype t; fn u: \\exists x:t. C(x) => let (a, proof) = u in proof")
            .unwrap();

        let mut proof = proof_parser().parse(tokens).unwrap();

        proof = ProofPipeline::new()
            .pipe(ResolveDatatypes::boxed())
            .apply(proof);

        let _type = typify(&proof.proof_term);

        assert_eq!(_type, Err(TypeError::QuantifiedObjectEscapesScope))
    }
}
