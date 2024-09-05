use identifier_generator::IdentifierGenerator;
use itertools::Itertools;

use crate::kernel::proof_term::{ProjectFst, ProjectSnd};

use super::{
    checker::{check::check, identifier_context::IdentifierContext},
    proof_term::{
        Abort, Application, Case, Function, Ident, OrLeft, OrRight, Pair, ProofTerm, Type,
        TypeAscription,
    },
    prop::Prop,
};

mod identifier_generator;
mod tests;

#[derive(Debug, Clone)]
pub struct TypeJudgment {
    pub prop: Prop,
    pub proof_term: ProofTerm,
}

impl TypeJudgment {
    pub fn new(prop: Prop, proof_term: ProofTerm) -> Self {
        TypeJudgment { prop, proof_term }
    }
}

#[derive(Debug, Clone)]
struct Sequent<'a> {
    unordered_ctx: Vec<TypeJudgment>,
    ordered_ctx: Vec<TypeJudgment>,
    goal: &'a Prop,
}

impl<'a> Sequent<'a> {
    pub fn new(goal: &'a Prop) -> Self {
        Self {
            unordered_ctx: vec![],
            ordered_ctx: vec![],
            goal,
        }
    }

    pub fn with_new_goal(&self, goal: &'a Prop) -> Self {
        Self {
            unordered_ctx: self.unordered_ctx.clone(),
            ordered_ctx: self.ordered_ctx.clone(),
            goal,
        }
    }

    pub fn append_ordered(&mut self, type_judgment: TypeJudgment) {
        self.ordered_ctx.push(type_judgment);
    }

    pub fn append_unordered(&mut self, type_judgment: TypeJudgment) {
        self.unordered_ctx.push(type_judgment);
    }

    pub fn find_in_unordered_context_by_prop(&self, prop: &Prop) -> Option<&TypeJudgment> {
        self.unordered_ctx.iter().find(|elem| elem.prop == *prop)
    }
}

pub fn prove(prop: &Prop) -> Option<ProofTerm> {
    prove_with_ctx(prop, &IdentifierContext::new())
}

pub fn prove_with_ctx(prop: &Prop, ctx: &IdentifierContext) -> Option<ProofTerm> {
    let assumptions = ctx
        .get_all_visible()
        .iter()
        .filter_map(|(ident, _type)| {
            let Type::Prop(prop) = _type else {
                return None;
            };

            let name = ident.name();

            Some(TypeJudgment {
                prop: prop.clone(),
                proof_term: Ident::create(name.clone()),
            })
        })
        .collect_vec();

    let proof_term = Prover::prove_with_assumptions(prop, assumptions)?;

    // sanity check
    let check_result = check(&proof_term, prop, ctx);
    if check_result.is_err() {
        panic!(
            "Prover returned wrong proof. Prop: {:#?}, proof term: {}, context: {:#?}, error: {:#?}",
            prop, proof_term, ctx, check_result,
        );
    }

    Some(proof_term)
}

struct Prover {
    forbidden_idents: Vec<String>,
    identifier_generator: IdentifierGenerator,
}

impl Prover {
    pub fn prove_with_assumptions(
        prop: &Prop,
        assumptions: Vec<TypeJudgment>,
    ) -> Option<ProofTerm> {
        if prop.has_free_parameters() {
            panic!("Cannot prove propositions with quantifiers.");
        }

        let mut sequent = Sequent::new(prop);

        let mut forbidden_idents = vec![];
        for assumption in assumptions {
            if let ProofTerm::Ident(Ident(ref name, _)) = assumption.proof_term {
                forbidden_idents.push(name.clone());
            }

            sequent.append_ordered(assumption);
        }

        let mut prover = Prover::new();
        prover.forbidden_idents = forbidden_idents;

        prover.prove_right(sequent)
    }

    fn new() -> Self {
        Self {
            identifier_generator: IdentifierGenerator::new(),
            forbidden_idents: vec![],
        }
    }

    fn prove_right(&mut self, sequent: Sequent) -> Option<ProofTerm> {
        match sequent.goal {
            Prop::True => Some(ProofTerm::Unit(None)),
            Prop::False => self.prove_left(sequent),
            Prop::Atom(_, _) => self.prove_left(sequent),
            Prop::Or(_, _) => self.prove_left(sequent),
            Prop::And(_, _) => self.handle_and_right(sequent),
            Prop::Impl(_, _) => self.handle_impl_right(sequent),
            Prop::ForAll { .. } => panic!("Cannot prove quantified propositions."),
            Prop::Exists { .. } => panic!("Cannot prove quantified propositions."),
        }
    }

    fn handle_and_right(&mut self, sequent: Sequent) -> Option<ProofTerm> {
        let Prop::And(fst, snd) = sequent.goal else {
            panic!("Expected conjunction.");
        };

        let fst_sequent = sequent.with_new_goal(fst);
        let fst_proof_term = self.prove_right(fst_sequent)?;

        let snd_sequent = sequent.with_new_goal(snd);
        let snd_proof_term = self.prove_right(snd_sequent)?;

        Some(Pair::create(
            fst_proof_term.boxed(),
            snd_proof_term.boxed(),
            None,
        ))
    }

    fn handle_impl_right(&mut self, mut sequent: Sequent) -> Option<ProofTerm> {
        let Prop::Impl(fst, snd) = sequent.goal else {
            panic!("Expected implication.");
        };

        let param_ident = self.generate_identifier();
        let param_judgment = TypeJudgment::new(*fst.clone(), Ident::create(param_ident.clone()));

        sequent.append_ordered(param_judgment);
        sequent.goal = snd;

        let body_proof_term = self.prove_right(sequent)?;

        Some(Function::create(
            param_ident,
            None,
            body_proof_term.boxed(),
            None,
        ))
    }

    fn prove_left(&mut self, mut sequent: Sequent) -> Option<ProofTerm> {
        if sequent.ordered_ctx.is_empty() {
            return self.search(sequent);
        }

        let type_judgment = sequent.ordered_ctx.pop().unwrap();

        match &type_judgment.prop {
            Prop::True => self.prove_left(sequent),
            Prop::False => Some(Abort::create(type_judgment.proof_term.boxed(), None)),
            Prop::Atom(_, _) => self.handle_atom_left(type_judgment, sequent),
            Prop::And(_, _) => self.handle_and_left(type_judgment, sequent),
            Prop::Or(_, _) => self.handle_or_left(type_judgment, sequent),
            Prop::Impl(_, _) => self.handle_impl_left(type_judgment, sequent),
            Prop::ForAll { .. } => panic!("Cannot prove quantified propositions."),
            Prop::Exists { .. } => panic!("Cannot prove quantified propositions."),
        }
    }

    fn handle_and_left(
        &mut self,
        type_judgment: TypeJudgment,
        mut sequent: Sequent,
    ) -> Option<ProofTerm> {
        let TypeJudgment { prop, proof_term } = type_judgment;
        let Prop::And(fst, snd) = prop else {
            panic!("Exptected conjunction");
        };

        let fst_judgment = TypeJudgment::new(*fst, ProjectFst::create(proof_term.boxed(), None));
        sequent.append_ordered(fst_judgment);

        let snd_judgment = TypeJudgment::new(*snd, ProjectSnd::create(proof_term.boxed(), None));
        sequent.append_ordered(snd_judgment);

        self.prove_left(sequent)
    }

    fn handle_or_left(
        &mut self,
        type_judgment: TypeJudgment,
        sequent: Sequent,
    ) -> Option<ProofTerm> {
        let TypeJudgment { prop, proof_term } = type_judgment;
        let Prop::Or(fst, snd) = prop else {
            panic!("Expected disjunction");
        };

        let mut fst_sequent = sequent.clone();
        let fst_ident = self.generate_identifier();
        let fst_judgment = TypeJudgment::new(*fst, Ident::create(fst_ident.clone()));
        fst_sequent.append_ordered(fst_judgment);
        let fst_term = self.prove_left(fst_sequent)?;

        let mut snd_sequent = sequent;
        let snd_ident = self.generate_identifier();
        let snd_judgment = TypeJudgment::new(*snd, Ident::create(snd_ident.clone()));
        snd_sequent.append_ordered(snd_judgment);
        let snd_term = self.prove_left(snd_sequent)?;

        Some(Case::create(
            proof_term.boxed(),
            fst_ident,
            fst_term.boxed(),
            snd_ident,
            snd_term.boxed(),
            None,
        ))
    }

    fn handle_atom_left(
        &mut self,
        type_judgment: TypeJudgment,
        mut sequent: Sequent,
    ) -> Option<ProofTerm> {
        let TypeJudgment { ref prop, .. } = type_judgment;
        let Prop::Atom(_, _) = prop else {
            panic!("Expected atom");
        };

        sequent.append_unordered(type_judgment);
        self.prove_left(sequent)
    }

    fn handle_impl_left(
        &mut self,
        type_judgment: TypeJudgment,
        mut sequent: Sequent,
    ) -> Option<ProofTerm> {
        let TypeJudgment { prop, proof_term } = type_judgment;
        let Prop::Impl(fst, snd) = prop else {
            panic!("Expected implication");
        };

        match *fst {
            Prop::True => {
                let application_proof_term =
                    Application::create(proof_term.boxed(), ProofTerm::Unit(None).boxed(), None);
                let application_judgment = TypeJudgment::new(*snd, application_proof_term);
                sequent.append_ordered(application_judgment);
                self.prove_left(sequent)
            }

            Prop::And(and_fst, and_snd) => {
                let new_prop = Prop::Impl(and_fst, Prop::Impl(and_snd, snd).boxed());

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
                            body: Application::create(
                                proof_term.boxed(),
                                Pair::create(
                                    Ident::create(fst_ident).boxed(),
                                    Ident::create(snd_ident).boxed(),
                                    None,
                                )
                                .boxed(),
                                None,
                            )
                            .boxed(),
                            span: None,
                        })
                        .boxed(),
                        span: None,
                    })
                    .boxed(),
                    span: None,
                });
                let new_judgment = TypeJudgment::new(new_prop, new_proof_term);
                sequent.append_ordered(new_judgment);

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
                                ProofTerm::Ident(Ident(or_fst_ident, None)).boxed(),
                                None,
                            ))
                            .boxed(),
                            span: None,
                        })
                        .boxed(),
                        span: None,
                    })
                    .boxed(),
                    span: None,
                });
                let or_fst_judgment = TypeJudgment::new(or_fst_prop, or_fst_proof_term);
                sequent.append_ordered(or_fst_judgment);

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
                                ProofTerm::Ident(Ident(or_snd_ident, None)).boxed(),
                                None,
                            ))
                            .boxed(),
                            span: None,
                        })
                        .boxed(),
                        span: None,
                    })
                    .boxed(),
                    span: None,
                });
                let or_snd_judgment = TypeJudgment::new(or_snd_prop, or_snd_proof_term);
                sequent.append_ordered(or_snd_judgment);

                self.prove_left(sequent)
            }

            Prop::False => self.prove_left(sequent),
            Prop::Atom(_, _) => {
                sequent.append_unordered(TypeJudgment::new(Prop::Impl(fst, snd), proof_term));
                self.prove_left(sequent)
            }
            Prop::Impl(_, _) => {
                sequent.append_unordered(TypeJudgment::new(Prop::Impl(fst, snd), proof_term));
                self.prove_left(sequent)
            }
            Prop::ForAll { .. } => panic!("Cannot prove quantified propositions."),
            Prop::Exists { .. } => panic!("Cannot prove quantified propositions."),
        }
    }

    fn search(&mut self, sequent: Sequent) -> Option<ProofTerm> {
        assert!(
            sequent.ordered_ctx.is_empty(),
            "Do not search when ordered context is not empty."
        );

        let goal_in_unordered_context = sequent.find_in_unordered_context_by_prop(sequent.goal);

        // id rule
        if let (Prop::Atom(_, _), Some(elem)) = (sequent.goal, goal_in_unordered_context) {
            return Some(elem.proof_term.clone());
        }

        // falsum rule
        if let (Prop::False, Some(elem)) = (sequent.goal, goal_in_unordered_context) {
            return Some(elem.proof_term.clone());
        }

        // or left rule
        if let Prop::Or(fst, _) = sequent.goal {
            if let Some(proof_term) = self.prove_right(sequent.with_new_goal(fst)) {
                return Some(OrLeft::create(proof_term.boxed(), None));
            }
        }

        // or right rule
        if let Prop::Or(_, snd) = sequent.goal {
            if let Some(proof_term) = self.prove_right(sequent.with_new_goal(snd)) {
                return Some(OrRight::create(proof_term.boxed(), None));
            }
        }

        // try Impl Rules for every element in unordered ctx.
        for i in 0..sequent.unordered_ctx.len() {
            let mut searching_sequent = sequent.clone();
            let TypeJudgment { prop, proof_term } = searching_sequent.unordered_ctx.remove(i);
            let Prop::Impl(impl_fst, impl_snd) = prop else {
                continue;
            };

            // impl atom rule
            if let Prop::Atom(_, _) = *impl_fst {
                if let Some(elem) = sequent.find_in_unordered_context_by_prop(&impl_fst) {
                    let mut new_sequent = searching_sequent.clone();
                    new_sequent.append_ordered(TypeJudgment::new(
                        *impl_snd.clone(),
                        Application::create(proof_term.boxed(), elem.proof_term.boxed(), None),
                    ));

                    if let Some(result_proof_term) = self.prove_left(new_sequent) {
                        return Some(result_proof_term);
                    }
                }
            }

            // Impl Impl left rule
            if let Prop::Impl(impl_impl_fst, impl_impl_snd) = *impl_fst {
                let fst_goal = Prop::Impl(impl_impl_fst, impl_impl_snd.clone());
                let mut fst_sequent = searching_sequent.with_new_goal(&fst_goal);
                let first_param_ident = self.generate_identifier();

                let new_prop = Prop::Impl(impl_impl_snd.boxed(), impl_snd.boxed());
                fst_sequent.append_unordered(TypeJudgment::new(
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
                                    body: Ident::create(first_param_ident).boxed(),
                                    span: None,
                                })
                                .boxed(),
                                span: None,
                            })
                            .boxed(),
                            span: None,
                        })
                        .boxed(),
                        span: None,
                    }),
                ));

                if let Some(fst_proof_term) = self.prove_right(fst_sequent) {
                    let mut snd_sequent = searching_sequent.clone();
                    snd_sequent.append_unordered(TypeJudgment::new(
                        *impl_snd,
                        Application::create(proof_term.boxed(), fst_proof_term.boxed(), None),
                    ));

                    if let Some(final_proof_term) = self.prove_left(snd_sequent) {
                        return Some(final_proof_term);
                    }
                }
            }
        }

        None
    }

    fn generate_identifier(&mut self) -> String {
        let mut ident = self.identifier_generator.generate();

        while self.forbidden_idents.contains(&ident) {
            ident = self.identifier_generator.generate();
        }

        ident
    }
}
