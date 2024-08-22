use serde::{Deserialize, Serialize};
use stages::resolve_datatypes::ResolveDatatypesStageError;
use thiserror::Error;
use tsify_next::Tsify;

use super::{proof::{Proof, ProofProcessingState}, prop::Prop};

pub mod stages;

// FIXME: Stage should not decide which stage error it returns
#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum StageError {
    #[error("An error happened in the Resolve Datatypes stage")]
    ResolveDatatypesStageError(ResolveDatatypesStageError),
}

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize, Tsify)]
#[tsify(into_wasm_abi, from_wasm_abi)]
#[serde(tag = "kind", content = "value")]
pub enum ProofPipelineError {
    #[error("Proof in unexpected processing state")]
    UnexpectedProcessingState {
        expected: Vec<ProofProcessingState>,
        actual: ProofProcessingState,
    },

    #[error("There was an error in one of the stages")]
    StageError(#[from] StageError),
}

pub trait ProofPipelineStage {
    fn expected_processing_states(&self) -> Vec<ProofProcessingState>;
    fn process(&self, proof: Proof, prop: &Prop) -> Result<Proof, StageError>;
}

pub struct ProofPipeline {
    stages: Vec<Box<dyn ProofPipelineStage>>,
}

impl Default for ProofPipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl ProofPipeline {
    pub fn new() -> Self {
        Self { stages: vec![] }
    }

    pub fn pipe(&mut self, stage: Box<dyn ProofPipelineStage>) -> &mut Self {
        self.stages.push(stage);

        self
    }

    pub fn apply(&self, proof: Proof, prop: &Prop) -> Result<Proof, ProofPipelineError> {
        let mut p = proof;

        for stage in self.stages.iter() {
            if !stage
                .expected_processing_states()
                .contains(&p.processing_state)
            {
                return Err(ProofPipelineError::UnexpectedProcessingState {
                    expected: stage.expected_processing_states(),
                    actual: p.processing_state,
                });
            }

            p = stage.process(p, prop)?;
        }

        Ok(p)
    }
}
