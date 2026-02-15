use indexmap::IndexMap;
use serde::{Deserialize, Serialize};

// full model data
#[derive(Debug, Default)]
pub struct Model {
    pub current_mode: CurrentMode,   // mode the user is in
    pub current_json: OrderedValue,  // the full json content
    pub current_path: Vec<String>,   // full key and index path to object/array/field
    pub value_input: String,         // value of field being edited
    pub running_state: RunningState, // whether application is running
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum OrderedValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<OrderedValue>),
    Object(IndexMap<String, OrderedValue>),
}
impl Default for OrderedValue {
    fn default() -> Self {
        OrderedValue::Object(IndexMap::new())
    }
}

// mode the user is in
#[derive(Debug, Default, PartialEq, Eq)]
pub enum CurrentMode {
    #[default]
    Browse, // browsing json structure
    Create,  // creating new objects
    Select,  // selecting preset field values
    Edit,    // typing custom field value
    Command, // typing new command
}

// whether application is running
#[derive(Debug, Default, PartialEq, Eq)]
pub enum RunningState {
    #[default]
    Running,
    Done,
}
