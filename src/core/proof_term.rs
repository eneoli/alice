use super::prop::Prop;

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Type {
    Prop(Prop),
    Datatype(String),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum ProofTerm {
    Ident(String),
    Pair(Box<ProofTerm>, Box<ProofTerm>),
    ProjectFst(Box<ProofTerm>),
    ProjectSnd(Box<ProofTerm>),
    Function {
        param_ident: String,
        param_type: Type,
        body: Box<ProofTerm>,
    },
    Application {
        function: Box<ProofTerm>,
        applicant: Box<ProofTerm>,
    },
    LetIn {
        fst_ident: String,
        snd_ident: String,
        pair_proof_term: Box<ProofTerm>,
        body: Box<ProofTerm>,
    },
    OrLeft(Box<ProofTerm>),
    OrRight(Box<ProofTerm>),
    Case {
        proof_term: Box<ProofTerm>,

        left_ident: String,
        left_term: Box<ProofTerm>,

        right_ident: String,
        right_term: Box<ProofTerm>,
    },
    Abort(Box<ProofTerm>),
    Unit,
}

impl ProofTerm {
    pub fn boxed(&self) -> Box<Self> {
        Box::new(self.clone())
    }
}
