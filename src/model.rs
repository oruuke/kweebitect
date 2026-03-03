use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use strum::Display;
use tui_widget_list::ListState;

// segment structure for supplying data at each level
#[derive(Debug, Clone)]
pub struct PathSegment {
    pub key: String,
    pub value: OrderedValue,
    pub list_state: ListState,
}
impl AsRef<str> for PathSegment {
    fn as_ref(&self) -> &str {
        &self.key
    }
}

impl PathSegment {
    pub fn new(key: String, value: OrderedValue, list_state: ListState) -> Self {
        Self {
            key,        // key of current json field
            value,      // value of current json field
            list_state, // selection state of list widget
        }
    }
}

// full model data
#[derive(Debug)]
pub struct Model {
    pub current_mode: CurrentMode,      // mode the user is in
    pub current_json: OrderedValue,     // the full json content
    pub current_path: Vec<PathSegment>, // full state at each level of pathway to current depth
    pub running_state: RunningState,    // whether application is running
}

impl Default for Model {
    fn default() -> Self {
        Self {
            current_mode: CurrentMode::Browse,
            current_json: OrderedValue::Null,
            current_path: vec![],
            running_state: RunningState::Running,
        }
    }
}

impl Model {
    // ensure root container segment exists in current path
    pub fn ensure_root_segment(&mut self) {
        if !self.current_path.is_empty() {
            return;
        }

        // get entry field labels
        let entries = container_entries_for(&self.current_json);
        let mut list_state = ListState::default();
        if !entries.is_empty() {
            list_state.select(Some(0));
        }

        // get first key in container
        let key = entries
            .first()
            .cloned()
            .unwrap_or_else(|| "(value)".to_string());

        // att to end of path
        self.current_path
            .push(PathSegment::new(key, self.current_json.clone(), list_state));
    }
}

// return list of field labels for a container value
pub fn container_entries_for(container: &OrderedValue) -> Vec<String> {
    match container {
        OrderedValue::Object(map) => map.keys().cloned().collect(),
        OrderedValue::Array(arr) => (0..arr.len()).map(|i| i.to_string()).collect(),
        _ => vec!["(value)".to_string()],
    }
}

// return value for an entry label inside container
pub fn value_for_entry<'a>(
    container: &'a OrderedValue,
    entry_label: &str,
) -> Option<&'a OrderedValue> {
    match container {
        OrderedValue::Object(map) => map.get(entry_label),
        OrderedValue::Array(arr) => entry_label.parse::<usize>().ok().and_then(|i| arr.get(i)),
        _ => Some(container),
    }
}

// create hooman-readable string from container value
pub fn format_container_entry_lines(entry_label: &str, value: &OrderedValue) -> Vec<String> {
    fn format_scalar_inline(value: &OrderedValue) -> String {
        // turn json values into string-equivalent representations
        match value {
            OrderedValue::Null => "null".to_string(),
            OrderedValue::Bool(b) => b.to_string(),
            OrderedValue::Number(n) => n.to_string(),
            OrderedValue::String(s) => format!("{:?}", s),
            OrderedValue::Array(_) | OrderedValue::Object(_) => {
                serde_json::to_string(value).unwrap_or_else(|_| "<invalid json>".to_string())
            }
        }
    }

    // truncate array values to item count, and objects to just their fields
    match value {
        OrderedValue::Array(arr) => vec![format!("{entry_label}: [{}]", arr.len())],
        OrderedValue::Object(map) => {
            let mut lines = Vec::with_capacity(1 + map.len());
            lines.push(format!("{entry_label}:"));
            // contained fields
            lines.extend(map.keys().map(|k| format!("  {k}")));
            lines
        }
        _ => vec![format!("{entry_label} {}", format_scalar_inline(value))],
    }
}

// custom untyped serde structure for ordered indexing
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Display)]
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
    pub fn get(&self, path: &[impl AsRef<str>]) -> Option<&Self> {
        let mut current = self;
        // dig into structure via each path step
        for segment in path {
            let key = segment.as_ref();
            current = match current {
                OrderedValue::Array(arr) => {
                    let index: usize = key.parse().ok()?;
                    arr.get(index)?
                }
                OrderedValue::Object(map) => map.get(key)?,
                _ => return None,
            };
        }
        Some(current)
    }

    // get pretty string of json
    pub fn get_pretty(&self, path: &[impl AsRef<str>]) -> String {
        // return readable fallback
        let Some(value) = self.get(path) else {
            return "<path not found>".to_string();
        };

        value
            .to_string_pretty()
            .unwrap_or_else(|_| "<failed to serialise>".to_string())
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
