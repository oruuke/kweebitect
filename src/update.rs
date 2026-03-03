use std::time::Duration;

use ratatui::crossterm::event::{self, Event, KeyCode};
use tui_widget_list::ListState;

use crate::model::{
    CurrentMode, Model, OrderedValue, PathSegment, RunningState, container_entries_for,
    value_for_entry,
};

// update handling with a message for each action/event (logic)
#[derive(PartialEq)]
pub enum Message {
    Out,
    In,
    Up,
    Down,
    ViewPreview,
    EditValue,
    DeleteField,
    DeleteObject,
    CreateBelow,
    CreateAbove,
    ConfirmObject,
    ConfirmValue,
    ConfirmCommand,
    Quit,
}

// core event listener
pub fn handle_event(model: &Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key, model));
            }
        }
    }
    Ok(None)
}

// event distribution to message for action
fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => Some(Message::Out),
        KeyCode::Char('l') | KeyCode::Right => Some(Message::In),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::Up),
        KeyCode::Char('p') => Some(Message::ViewPreview),
        KeyCode::Enter => match model.current_mode {
            CurrentMode::Browse => Some(Message::EditValue),
            CurrentMode::Create => Some(Message::ConfirmValue),
            CurrentMode::Select => Some(Message::ConfirmValue),
            CurrentMode::Edit => Some(Message::ConfirmValue),
            CurrentMode::Command => Some(Message::ConfirmCommand),
            CurrentMode::Preview => None,
        },
        KeyCode::Char('o') => Some(Message::CreateBelow),
        KeyCode::Char('O') => Some(Message::CreateAbove),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

// message hander to call actions
pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    // match all possible messages and return new model reflecting changes
    match msg {
        Message::Out => {
            // remove one level of path depth
            model.current_path.pop();
        }
        Message::In => {
            enter_container(model);
        }
        Message::Up => {
            // move selection wit negative delta
            navigate_active_container(model, -1);
        }
        Message::Down => {
            // move selection wit positive delta
            navigate_active_container(model, 1);
        }
        Message::ViewPreview => match model.current_mode {
            // toggle between preview and browse
            CurrentMode::Preview => {
                model.current_mode = CurrentMode::Browse;
            }
            _ => {
                model.current_mode = CurrentMode::Preview;
            }
        },
        Message::EditValue => {
            model.current_mode = CurrentMode::Edit;
        }
        Message::DeleteField => {}
        Message::DeleteObject => {}
        Message::CreateBelow => {
            model.current_mode = CurrentMode::Create;
        }
        Message::CreateAbove => {
            model.current_mode = CurrentMode::Create;
        }
        Message::ConfirmObject => {
            model.current_mode = CurrentMode::Browse;
        }
        Message::ConfirmValue => {
            model.current_mode = CurrentMode::Browse;
        }
        Message::ConfirmCommand => {}
        Message::Quit => {
            model.running_state = RunningState::Done;
        }
    };
    None
}

// vertical navigation within container
fn navigate_active_container(model: &mut Model, delta: isize) {
    let Some(segment) = model.current_path.last_mut() else {
        return;
    };

    // get entries for this container
    let entries = container_entries_for(&segment.value);
    if entries.is_empty() {
        return;
    }

    // ensure selection exists before moving
    if segment.list_state.selected.is_none() {
        segment.list_state.select(Some(0));
    }

    // move selection
    if delta < 0 {
        segment.list_state.previous();
    } else {
        segment.list_state.next();
    }

    // map selection back into a key/index string
    let Some(selected_index) = segment.list_state.selected else {
        return;
    };
    let Some(selected_key) = entries.get(selected_index).cloned() else {
        return;
    };

    // update selection key at this depth
    segment.key = selected_key;
}

// horizontal navigation between containers
fn enter_container(model: &mut Model) {
    // get active depth segment
    let Some(parent) = model.current_path.last() else {
        return;
    };

    // get selected value witin the active container
    let Some(selected_value) = value_for_entry(&parent.value, &parent.key) else {
        return;
    };

    // only enter if the selected value is a container wit values
    let is_container = matches!(
        selected_value,
        OrderedValue::Array(_) | OrderedValue::Object(_)
    );
    if !is_container {
        return;
    };
    let selected_container = selected_value.clone();
    let entries = container_entries_for(&selected_container);
    if entries.is_empty() {
        return;
    }

    // start from first entry
    let mut child_state = ListState::default();
    child_state.select(Some(0));
    let child_key = entries[0].clone();

    // push new segment for child container depth
    model
        .current_path
        .push(PathSegment::new(child_key, selected_container, child_state));
}
