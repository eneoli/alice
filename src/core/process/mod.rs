use super::proof::Proof;

pub mod stages;

pub trait ProofPipelineStage {
    fn process(&self, proof: Proof) -> Proof;
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

    pub fn apply(&self, proof: Proof) -> Proof {
        let mut p = proof;

        for stage in self.stages.iter() {
            p = stage.process(p);
        }

        p
    }
}
