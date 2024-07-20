use crate::kernel::proof_term::{ProjectFst, ProjectSnd};

use super::{
    checker::{check::check, identifier_context::IdentifierContext},
    proof_term::{
        Abort, Application, Case, Function, Ident, OrLeft, OrRight, Pair, ProofTerm, Type,
        TypeAscription,
    },
    prop::Prop,
};

#[derive(Debug, Clone)]
struct TypeJudgment {
    prop: Prop,
    proof_term: ProofTerm,
}

#[derive(Debug, Clone)]
struct Sequent<'a> {
    unordered_ctx: Vec<TypeJudgment>,
    ordered_ctx: Vec<TypeJudgment>,
    goal: &'a Prop,
}

impl<'a> Sequent<'a> {
    pub fn new(prop: &'a Prop) -> Self {
        Self {
            unordered_ctx: vec![],
            ordered_ctx: vec![],
            goal: prop,
        }
    }

    pub fn with_new_goal(&self, prop: &'a Prop) -> Self {
        Self {
            unordered_ctx: self.unordered_ctx.clone(),
            ordered_ctx: self.ordered_ctx.clone(),
            goal: prop,
        }
    }

    pub fn append_ordered(&mut self, prop: Prop, proof_term: ProofTerm) {
        self.ordered_ctx.push(TypeJudgment { prop, proof_term });
    }

    pub fn append_unordered(&mut self, prop: Prop, proof_term: ProofTerm) {
        self.unordered_ctx.push(TypeJudgment { prop, proof_term });
    }
}

pub fn prove(prop: &Prop) -> Option<ProofTerm> {
    // we only can prove propositional logic
    if prop.has_quantifiers() {
        return None;
    }

    let mut prover = Prover::new();

    let proof_term = prover.prove_right(Sequent::new(prop))?;

    // sanity check
    let check_result = check(&proof_term, prop, &IdentifierContext::new());
    if check_result.is_err() {
        panic!(
            "Prover returned wrong proof. Prop: {:#?}, proof term: {:#?}, error: {:#?}",
            prop, proof_term, check_result,
        );
    }

    Some(proof_term)
}

struct Prover {
    identifier_generator: IdentifierGenerator,
}

impl Prover {
    pub fn new() -> Self {
        Self {
            identifier_generator: IdentifierGenerator::new(),
        }
    }

    fn prove_right(&mut self, mut sequent: Sequent) -> Option<ProofTerm> {
        match sequent.goal {
            Prop::True => Some(ProofTerm::Unit),
            Prop::And(fst, snd) => {
                let fst_sequent = sequent.with_new_goal(fst);
                let fst_proof_term = self.prove_right(fst_sequent)?;

                let snd_sequent = sequent.with_new_goal(snd);
                let snd_proof_term = self.prove_right(snd_sequent)?;

                Some(ProofTerm::Pair(Pair(
                    fst_proof_term.boxed(),
                    snd_proof_term.boxed(),
                )))
            }

            Prop::Impl(fst, snd) => {
                let param_ident = self.generate_identifier();

                sequent.append_ordered(*fst.clone(), ProofTerm::Ident(Ident(param_ident.clone())));
                sequent.goal = snd;

                let body_proof_term = self.prove_right(sequent)?;

                Some(ProofTerm::Function(Function {
                    param_ident,
                    param_type: None,
                    body: body_proof_term.boxed(),
                }))
            }

            Prop::Atom(_, _) => self.prove_left(sequent),
            Prop::Or(_, _) => self.prove_left(sequent),
            Prop::False => self.prove_left(sequent),
            Prop::ForAll { .. } => panic!("Cannot prove quantified propositions."),
            Prop::Exists { .. } => panic!("Cannot prove quantified propositions."),
        }
    }

    fn prove_left(&mut self, mut sequent: Sequent) -> Option<ProofTerm> {
        if sequent.ordered_ctx.len() == 0 {
            return self.search(sequent);
        }

        let TypeJudgment { prop, proof_term } = sequent.ordered_ctx.pop().unwrap();

        match prop {
            Prop::And(fst, snd) => {
                sequent.append_ordered(*fst, ProofTerm::ProjectFst(ProjectFst(proof_term.boxed())));
                sequent.append_ordered(*snd, ProofTerm::ProjectSnd(ProjectSnd(proof_term.boxed())));

                self.prove_left(sequent)
            }

            Prop::True => self.prove_left(sequent),

            Prop::Or(fst, snd) => {
                let mut fst_sequent = sequent.clone();
                let fst_ident = self.generate_identifier();
                fst_sequent.append_ordered(*fst, ProofTerm::Ident(Ident(fst_ident.clone())));
                let fst_term = self.prove_left(fst_sequent)?;

                let mut snd_sequent = sequent;
                let snd_ident = self.generate_identifier();
                snd_sequent.append_ordered(*snd, ProofTerm::Ident(Ident(snd_ident.clone())));
                let snd_term = self.prove_left(snd_sequent)?;

                Some(ProofTerm::Case(Case {
                    head: proof_term.boxed(),
                    fst_ident,
                    fst_term: fst_term.boxed(),
                    snd_ident,
                    snd_term: snd_term.boxed(),
                }))
            }

            Prop::False => Some(ProofTerm::Abort(Abort(proof_term.boxed()))),

            Prop::Atom(_, _) => {
                sequent.append_unordered(prop, proof_term);
                self.prove_left(sequent)
            }

            Prop::Impl(ref fst, ref snd) => match *fst.clone() {
                Prop::True => {
                    sequent.append_ordered(
                        *snd.clone(),
                        ProofTerm::Application(Application {
                            function: proof_term.boxed(),
                            applicant: ProofTerm::Unit.boxed(),
                        }),
                    );
                    self.prove_left(sequent)
                }

                Prop::And(and_fst, and_snd) => {
                    let new_prop = Prop::Impl(
                        and_fst.clone(),
                        Prop::Impl(and_snd.clone(), snd.clone()).boxed(),
                    );

                    let fst_ident = self.generate_identifier();
                    let snd_ident = self.generate_identifier();
                    let new_proof_term = ProofTerm::TypeAscription(TypeAscription {
                        ascription: Type::Prop(new_prop.clone()),
                        proof_term: ProofTerm::Function(Function {
                            param_ident: fst_ident.clone(),
                            param_type: None,
                            body: ProofTerm::Function(Function {
                                param_ident: snd_ident.clone(),
                                param_type: None,
                                body: ProofTerm::Application(Application {
                                    function: proof_term.boxed(),
                                    applicant: ProofTerm::Pair(Pair(
                                        ProofTerm::Ident(Ident(fst_ident)).boxed(),
                                        ProofTerm::Ident(Ident(snd_ident)).boxed(),
                                    ))
                                    .boxed(),
                                })
                                .boxed(),
                            })
                            .boxed(),
                        })
                        .boxed(),
                    });
                    sequent.append_ordered(new_prop, new_proof_term);

                    self.prove_left(sequent)
                }

                Prop::Or(or_fst, or_snd) => {
                    let or_fst_prop = Prop::Impl(or_fst, snd.clone());
                    let or_fst_ident = self.generate_identifier();
                    let or_fst_proof_term = ProofTerm::TypeAscription(TypeAscription {
                        ascription: Type::Prop(or_fst_prop.clone()),
                        proof_term: ProofTerm::Function(Function {
                            param_ident: or_fst_ident.clone(),
                            param_type: None,
                            body: ProofTerm::Application(Application {
                                function: proof_term.boxed(),
                                applicant: ProofTerm::OrLeft(OrLeft(
                                    ProofTerm::Ident(Ident(or_fst_ident)).boxed(),
                                ))
                                .boxed(),
                            })
                            .boxed(),
                        })
                        .boxed(),
                    });
                    sequent.append_ordered(or_fst_prop, or_fst_proof_term);

                    let or_snd_prop = Prop::Impl(or_snd.clone(), (*snd).boxed());
                    let or_snd_ident = self.generate_identifier();
                    let or_snd_proof_term = ProofTerm::TypeAscription(TypeAscription {
                        ascription: Type::Prop(or_snd_prop.clone()),
                        proof_term: ProofTerm::Function(Function {
                            param_ident: or_snd_ident.clone(),
                            param_type: None,
                            body: ProofTerm::Application(Application {
                                function: proof_term.boxed(),
                                applicant: ProofTerm::OrRight(OrRight(
                                    ProofTerm::Ident(Ident(or_snd_ident)).boxed(),
                                ))
                                .boxed(),
                            })
                            .boxed(),
                        })
                        .boxed(),
                    });
                    sequent.append_ordered(or_snd_prop, or_snd_proof_term);

                    self.prove_left(sequent)
                }

                Prop::False => self.prove_left(sequent),
                Prop::Atom(_, _) => {
                    sequent.append_unordered(prop.clone(), proof_term);
                    self.prove_left(sequent)
                }
                Prop::Impl(_, _) => {
                    sequent.append_unordered(prop, proof_term);
                    self.prove_left(sequent)
                }
                Prop::ForAll { .. } => panic!("Cannot prove quantified propositions."),
                Prop::Exists { .. } => panic!("Cannot prove quantified propositions."),
            },

            Prop::ForAll { .. } => panic!("Cannot prove quantified propositions."),
            Prop::Exists { .. } => panic!("Cannot prove quantified propositions."),
        }
    }

    fn search(&mut self, mut sequent: Sequent) -> Option<ProofTerm> {
        if sequent.ordered_ctx.len() > 0 {
            panic!("Do not search when ordered context is not empty.");
        }

        // id rule
        if let Prop::Atom(_, _) = sequent.goal {
            if let Some(elem) = sequent
                .unordered_ctx
                .iter()
                .find(|elem| elem.prop == *sequent.goal)
            {
                return Some(elem.proof_term.clone());
            }
        }

        // falsum rule
        if let Prop::False = sequent.goal {
            if let Some(elem) = sequent
                .unordered_ctx
                .iter()
                .find(|elem| elem.prop == Prop::False)
            {
                return Some(elem.proof_term.clone());
            }
        }

        // or fst rule
        if let Prop::Or(fst, _) = sequent.goal {
            if let Some(proof_term) = self.prove_right(sequent.with_new_goal(fst)) {
                return Some(ProofTerm::OrLeft(OrLeft(proof_term.boxed())));
            }
        }

        // or snd rule
        if let Prop::Or(_, snd) = sequent.goal {
            if let Some(proof_term) = self.prove_right(sequent.with_new_goal(snd)) {
                return Some(ProofTerm::OrRight(OrRight(proof_term.boxed())));
            }
        }

        for (i, TypeJudgment { prop, proof_term }) in sequent.unordered_ctx.iter().enumerate() {
            let mut my_sequent = sequent.clone();
            my_sequent.unordered_ctx.remove(i);

            // Impl Rules
            // TODO check if goal is positive
            if let Prop::Impl(impl_fst, impl_snd) = prop {
                // impl atom rule
                if let Prop::Atom(_, _) = **impl_fst {
                    if let Some(elem) = my_sequent
                        .unordered_ctx
                        .iter()
                        .find(|elem| elem.prop == **impl_fst)
                    {
                        let mut new_sequent = my_sequent.clone();
                        new_sequent.append_ordered(
                            *impl_snd.clone(),
                            ProofTerm::Application(Application {
                                function: proof_term.boxed(),
                                applicant: elem.proof_term.boxed(),
                            }),
                        );

                        if let Some(result_proof_term) = self.prove_left(new_sequent) {
                            return Some(result_proof_term);
                        }
                    }
                }

                // Impl Impl left rule
                if let Prop::Impl(ref impl_impl_fst, ref impl_impl_snd) = **impl_fst {
                    let fst_goal = Prop::Impl(impl_impl_fst.clone(), impl_impl_snd.clone());
                    let mut fst_sequent = my_sequent.with_new_goal(&fst_goal);

                    let first_param_ident = self.generate_identifier();

                    let new_prop = Prop::Impl(impl_impl_snd.boxed(), impl_snd.boxed());
                    fst_sequent.append_unordered(
                        new_prop.clone(),
                        ProofTerm::TypeAscription(TypeAscription {
                            ascription: Type::Prop(new_prop),
                            proof_term: ProofTerm::Function(Function {
                                param_ident: first_param_ident.clone(),
                                param_type: None,
                                body: ProofTerm::Application(Application {
                                    function: proof_term.boxed(),
                                    applicant: ProofTerm::Function(Function {
                                        param_ident: self.generate_identifier(),
                                        param_type: None,
                                        body: ProofTerm::Ident(Ident(first_param_ident)).boxed(),
                                    })
                                    .boxed(),
                                })
                                .boxed(),
                            })
                            .boxed(),
                        }),
                    );

                    if let Some(fst_proof_term) = self.prove_right(fst_sequent) {
                        let mut snd_sequent = my_sequent.clone();
                        snd_sequent.append_unordered(
                            *impl_snd.clone(),
                            ProofTerm::Application(Application {
                                function: proof_term.boxed(),
                                applicant: fst_proof_term.boxed(),
                            }),
                        );

                        if let Some(final_proof_term) = self.prove_left(snd_sequent) {
                            return Some(final_proof_term);
                        }
                    }
                }
            }
        }

        // we failed :(
        None
    }

    fn generate_identifier(&mut self) -> String {
        self.identifier_generator.generate()
    }
}

struct IdentifierGenerator {
    idx: usize,
}

impl IdentifierGenerator {
    pub fn new() -> Self {
        Self { idx: 0 }
    }

    pub fn generate(&mut self) -> String {
        let alphabet = "abcdefghijklmnopqrstuvwxyz".chars().collect::<Vec<char>>();
        let alphabet_length = alphabet.len();
        let num_digits = f32::floor(self.idx as f32 / alphabet_length as f32) as usize + 1;

        let mut identifier = String::new();
        for i in 0..num_digits {
            identifier.push(
                alphabet[(f32::floor(
                    (self.idx as f32) / usize::pow(alphabet_length, i.try_into().unwrap()) as f32,
                ) as usize)
                    % alphabet_length],
            );
        }

        self.idx += 1;

        identifier.chars().rev().collect()
    }
}

#[cfg(test)]
mod tests {
    use chumsky::{primitive::end, Parser, Stream};

    use crate::kernel::{
        checker::{check::check, identifier_context::IdentifierContext},
        parse::{fol::fol_parser, lexer::lexer},
    };

    use super::prove;

    pub fn assert_proof(prop: &str) {
        let prop_tokens = lexer().then_ignore(end()).parse(prop).unwrap();

        let prop_len = prop.chars().count();

        let prop = fol_parser()
            .parse(Stream::from_iter(
                prop_len..prop_len + 1,
                prop_tokens.into_iter(),
            ))
            .unwrap();

        let proof_term_result = prove(&prop);

        assert!(proof_term_result.is_some());
        assert!(check(
            &proof_term_result.unwrap(),
            &prop,
            &IdentifierContext::new()
        )
        .is_ok());
    }

    pub fn assert_no_proof(prop: &str) {
        let prop_tokens = lexer().then_ignore(end()).parse(prop).unwrap();

        let prop_len = prop.chars().count();

        let prop = fol_parser()
            .parse(Stream::from_iter(
                prop_len..prop_len + 1,
                prop_tokens.into_iter(),
            ))
            .unwrap();

        let proof_term_result = prove(&prop);

        assert!(proof_term_result.is_none());
    }

    #[test]
    fn test_no_falsum_proof() {
        assert_no_proof("\\bot");
    }

    #[test]
    fn test_truth_proof() {
        assert_proof("\\top");
    }

    #[test]
    fn test_atom_identity() {
        assert_proof("A -> A");
    }

    #[test]
    fn test_impl_with_unused() {
        assert_proof("A -> (B -> A)");
    }

    #[test]
    fn test_impl_none() {
        assert_no_proof("A -> B");
    }

    #[test]
    fn test_swap() {
        assert_proof("A & B -> B & A");
    }

    #[test]
    fn test_currying() {
        assert_proof("(A & B -> C) -> (A -> B -> C)");
    }

    #[test]
    fn test_uncurrying() {
        assert_proof("(A -> B -> C) -> (A & B -> C)");
    }

    #[test]
    fn test_project_fst() {
        assert_proof("A && B -> A");
    }

    #[test]
    fn test_project_snd() {
        assert_proof("A && B -> B");
    }

    #[test]
    fn test_impl_switch_params() {
        assert_proof("(A -> B -> C) -> (B -> A -> C)");
    }

    #[test]
    fn test_impl_elim() {
        assert_proof("A && (A -> B) -> B");
    }

    #[test]
    fn test_interaction_law_of_distributivity() {
        assert_proof("(A -> (B & C)) -> ((A -> B) & (A -> C))");
    }

    #[test]
    fn test_distribution_of_or() {
        assert_proof("A -> (B || C) -> ((A -> B) || (A -> C))");
    }

    #[test]
    fn test_undo_distribution_of_or() {
        assert_proof("((A -> B) || (A -> C)) -> A -> (B || C)");
    }

    #[test]
    fn test_commutativity_of_disjunction() {
        assert_proof("A || B -> B || A");
    }

    #[test]
    fn test_ex_falso_quodlibet() {
        assert_proof("\\bot -> A");
    }

    #[test]
    fn test_composition_of_functions() {
        assert_proof("((A -> B) & (B -> C)) -> (A -> C)");
    }

    #[test]
    fn test_apply_composition_of_functions() {
        assert_proof("A && (A -> B) && (B -> C) -> C");
    }

    #[test]
    fn test_s_combinator() {
        assert_proof("(A -> B) -> (A -> (B -> C)) -> (A -> C)");
    }

    #[test]
    fn test_no_negated_lem() {
        assert_no_proof("~(A || ~A)");
    }

    #[test]
    fn test_lem_double_negated() {
        assert_proof("~~(A || ~A)");
    }

    #[test]
    fn test_tripple_negation_elimination() {
        assert_proof("~~~A -> ~A");
    }

    #[test]
    fn test_no_double_negation_elimination() {
        assert_no_proof("~~A -> A");
    }

    #[test]
    fn test_or_impl() {
        assert_proof("A || B -> B || A -> B || A -> A || B");
    }

    #[test]
    fn test_peirces_law() {
        assert_no_proof("((A -> B) -> A) -> A");
    }

    #[test]
    fn test_double_negation_peirces_law() {
        assert_proof("~~(((A -> B) -> A) -> A)");
    }

    #[test]
    fn test_long_proof() {
        assert_proof("(P -> (C ∧ K) || (D ∧ L)) -> (~K -> S) -> (D || L) -> (P -> ~S) ∧ (S -> ~P) -> (C -> ~D) ∧ (D -> ~C) -> (K -> ~L) ∧ (L -> ~K) -> ~P");
    }

    #[test]
    fn test_impl_and_or() {
        assert_proof("(A || C) && (B -> C) -> (A -> B) -> C");
    }

    #[test]
    fn test_some_tautologies() {
        assert_proof("((A || B) && (B -> C)) -> ((A -> B) -> C)");
        assert_proof("(A || B -> C) -> (A -> C) && (B -> C)");
        assert_proof("(A -> C) && (B -> C) -> (A || B -> C)");

        assert_proof("(A && (B || C)) -> (A && B) || (A && C)");
        assert_proof("(A && B) || (A && C) -> (A && (B || C))");

        assert_proof("(A || (B && C)) -> (A || B) && (A || C)");
        assert_proof("(A || B) && (A || C) -> (A || (B && C))");

        assert_proof("((A -> B) -> (A -> C) -> A -> B)");
        assert_proof("((A -> B) -> (A -> C) -> A -> C)");
        assert_proof("A -> ((A -> B) -> (A -> C) -> B)");
        assert_proof("A -> ((A -> B) -> (A -> C) -> C)");
        assert_proof("((A -> B -> C) -> A -> B -> C)");
        assert_proof("((A -> B -> C) -> B -> A -> C)");
        assert_proof("A -> B -> (A -> B -> C) -> C");
        assert_proof("B -> A -> (A -> B -> C) -> C");

        assert_proof("((A -> B) -> A -> B)");
        assert_proof("(((A -> B) -> C) -> ((A -> B) -> C))");
        assert_proof("((((A -> B) -> C) -> D) -> (((A -> B) -> C) -> D))");
        assert_proof("(((((A -> B) -> C) -> D) -> E) -> (((A -> B) -> C) -> D) -> E)");
        assert_proof("((((((A -> B) -> C) -> D) -> E) -> F) -> (((A -> B) -> C) -> D) -> E -> F)");
        assert_proof("((((((A -> B) -> C) -> D) -> E) -> F) -> (((((A -> B) -> C) -> D) -> E) -> F) || (((((A -> B) -> C) -> D) -> E) -> F))");
        assert_proof("((A -> B) -> C) -> D -> D || D");
    }

    #[test]
    fn test_demorgan() {
        assert_no_proof("~(A && B) -> ~A || ~B");
        assert_proof("~A || ~B -> ~(A && B)");

        assert_proof("~(A || B) -> ~A && ~B");
        assert_proof("~A && ~B -> ~(A || B)");

        assert_no_proof("~(A -> B) -> A && ~B");
        assert_proof("A && ~B -> ~(A -> B)");

        assert_proof("A -> ~~A");

        assert_proof("~\\top -> \\bot");
        assert_proof("\\bot -> ~\\top");
    }

    #[test]
    fn test_no_proofs() {
        assert_no_proof("(A -> B || C) -> (A -> B) || (A -> C)");
        assert_no_proof("((A -> B) -> C) -> ((A || B) && (B -> C))");
    }

    #[test]
    fn test_three_way_composition() {
        assert_proof("(A -> B) -> (B -> C) -> (C -> D) -> (A -> D)");
    }
}
