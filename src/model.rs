use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};
use tui_widget_list::ListState;

// full model data
#[derive(Debug)]
pub struct Model {
    pub current_mode: CurrentMode,   // mode the user is in
    pub current_json: OrderedValue,  // the full json content
    pub current_field: OrderedValue, // field focused by user
    pub current_path: Vec<String>,   // full key and index path to object/array/field
    pub value_input: String,         // value of field being edited
    pub list_state: ListState,       // state of list widget
    pub running_state: RunningState, // whether application is running
}

impl Default for Model {
    fn default() -> Self {
        Self {
            current_mode: CurrentMode::Browse,
            current_json: OrderedValue::Null,
            current_field: OrderedValue::Null,
            current_path: vec![],
            value_input: String::new(),
            list_state: ListState::default(),
            running_state: RunningState::Running,
        }
    }
}

pub static LIST_STATE: LazyLock<Mutex<ListState>> =
    LazyLock::new(|| Mutex::new(Model::default().list_state));

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
impl OrderedValue {
    // parse from json string
    pub fn from_str(s: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(s)
    }

    // pretty string of value
    pub fn to_string_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    // first key of object
    pub fn first_key(&self) -> Option<&String> {
        match self {
            OrderedValue::Object(map) => map.keys().next(),
            _ => None,
        }
    }

    // nested path traversal
    pub fn get(&self, path: &[String]) -> Option<&Self> {
        let mut current = self;
        // dig into structure via each path step
        for segment in path {
            current = match current {
                OrderedValue::Array(arr) => {
                    let index: usize = segment.parse().ok()?;
                    arr.get(index)?
                }
                OrderedValue::Object(map) => map.get(segment)?,
                _ => return None,
            };
        }
        Some(current)
    }

    // get pretty string of json
    pub fn get_pretty(&self, path: &[String]) -> String {
        self.get(path)
            .expect("path not found")
            .to_string_pretty()
            .expect("failed to serialise")
    }
}

// mode the user is in
#[derive(Debug, Default, PartialEq, Eq)]
pub enum CurrentMode {
    #[default]
    Browse, // browsing json structure
    Preview, // preview in prettified json
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
