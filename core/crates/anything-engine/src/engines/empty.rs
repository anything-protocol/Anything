use crate::{context::ExecutionContext, error::EngineResult, types::Process};

use super::Engine;
use anything_graph::flow::action::EmptyAction;
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct EmptyEngine {
    pub config: EmptyAction,
    pub process: Option<Process>,
}

impl EmptyEngine {
    pub fn new(config: EmptyAction) -> Self {
        Self {
            config,
            process: Some(Process::default()),
        }
    }
}

impl Engine for EmptyEngine {
    fn run(&mut self, _context: &ExecutionContext) -> EngineResult<()> {
        self.process = Some(Process::default());
        Ok(())
    }
    fn config(&self) -> &dyn std::any::Any {
        &self.config
    }
    fn process(&self) -> Option<Process> {
        self.process.clone()
    }
}
