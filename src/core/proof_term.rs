use super::prop::Prop;

#[derive(Debug, Clone)]
pub enum ProofTerm {

    Ident(String),
    Pair(Box<ProofTerm>, Box<ProofTerm>),
    Function {
        param_ident: String,
        param_prop: Prop,
        body: Box<ProofTerm>,
    },
    Application {
        function: Box<ProofTerm>,
        applicant: Box<ProofTerm>,
    },
    LetIn,
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
