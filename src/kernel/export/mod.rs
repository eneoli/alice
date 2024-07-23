use super::proof_term::{ProofTerm, Type};

pub mod ocaml_exporter;

pub trait ProofExporter {
    fn can_export(&self, proof_term: &ProofTerm) -> bool;
    fn can_export_for_type(&self, _type: Type) -> bool;
    fn export(&self, proof_term: &ProofTerm) -> String;
}