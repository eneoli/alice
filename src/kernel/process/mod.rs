use serde::{Deserialize, Serialize};
use stages::resolve_datatypes::ResolveDatatypesStageError;
use thiserror::Error;

use super::proof::{Proof, ProofProcessingState};

pub mod stages;

// FIXME: Stage should not decide which stage error it returns
#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum StageError {
    #[error("An error happened in the Resolve Datatypes stage")]
    ResolveDatatypesStageError(ResolveDatatypesStageError),
}

#[derive(Debug, Error, PartialEq, Eq, Clone, Serialize, Deserialize)]
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
    fn process(&self, proof: Proof) -> Result<Proof, StageError>;
}

pub struct ProofPipeline {
    stages: Vec<Box<dyn ProofPipelineStage>>,
}

impl ProofPipeline {
    pub fn new() -> Self {
        Self { stages: vec![] }
    }

    pub fn pipe(&mut self, stage: Box<dyn ProofPipelineStage>) -> &mut Self {
        self.stages.push(stage);

        self
    }

    pub fn apply(&self, proof: Proof) -> Result<Proof, ProofPipelineError> {
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

            p = stage.process(p)?;
        }

        Ok(p)
    }
}
