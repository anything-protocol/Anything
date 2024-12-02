use super::react_flow_types::{HandleProps, NodePresentation};
use crate::types::general::Variable;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Action {
    pub anything_action_version: String,
    pub r#type: ActionType,
    pub plugin_id: String,
    pub action_id: String,
    pub plugin_version: String,
    pub label: String,
    pub description: Option<String>,
    pub icon: String,
    pub variables: Variable,
    pub variables_locked: Option<bool>,
    pub variables_schema: Variable,
    pub variables_schema_locked: Option<bool>,
    pub input: Variable,
    pub input_locked: Option<bool>,
    pub input_schema: Variable,
    pub input_schema_locked: Option<bool>,
    pub presentation: Option<NodePresentation>,
    pub handles: Option<Vec<HandleProps>>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ActionType {
    Trigger,  // Trigger action
    Action,   // General action
    Loop,     // Loop action
    Decision, // Decision action
    Filter,   // Filter action
    Response, // Response action for making api endpoints
    Input,    // Input action for subflows
    Output,   // Output action for subflows
}

impl ActionType {
    pub fn as_str(&self) -> &str {
        match self {
            ActionType::Input => "input",
            ActionType::Trigger => "trigger",
            ActionType::Response => "response",
            ActionType::Action => "action",
            ActionType::Loop => "loop",
            ActionType::Decision => "decision",
            ActionType::Filter => "filter",
            ActionType::Output => "output",
        }
    }
}
