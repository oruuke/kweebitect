use std::time::Duration;

use color_eyre;
use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::model::{CurrentMode, Model, OrderedValue, RunningState};

// update handling with a message for each action/event (logic)
#[derive(PartialEq)]
pub enum Message {
    Out,
    In,
    Up,
    Down,
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

fn handle_key(key: event::KeyEvent, model: &Model) -> Option<Message> {
    match key.code {
        KeyCode::Char('h') | KeyCode::Left => Some(Message::Out),
        KeyCode::Char('l') | KeyCode::Right => Some(Message::In),
        KeyCode::Char('j') | KeyCode::Down => Some(Message::Down),
        KeyCode::Char('k') | KeyCode::Up => Some(Message::Up),
        KeyCode::Enter => match model.current_mode {
            CurrentMode::Browse => Some(Message::EditValue),
            CurrentMode::Create => Some(Message::ConfirmValue),
            CurrentMode::Select => Some(Message::ConfirmValue),
            CurrentMode::Edit => Some(Message::ConfirmValue),
            CurrentMode::Command => Some(Message::ConfirmCommand),
        },
        KeyCode::Char('o') => Some(Message::CreateBelow),
        KeyCode::Char('O') => Some(Message::CreateAbove),
        KeyCode::Char('q') => Some(Message::Quit),
        _ => None,
    }
}

pub fn update(model: &mut Model, msg: Message) -> Option<Message> {
    // match all possible messages and return new model reflecting changes
    match msg {
        Message::Out => {
            model.current_path.pop();
        }
        Message::In => {
            // get potential json at current path
            let attempted_json = model.current_json.get(&model.current_path);

            // go to first item in array, or first field in object
            if let Some(attempt) = attempted_json {
                match attempt {
                    OrderedValue::Array(arr) if !arr.is_empty() => {
                        // add first index
                        model.current_path.push(String::from("0"));
                    }
                    OrderedValue::Object(map) => {
                        if let Some((first_key, _)) = map.iter().next() {
                            // add found key
                            model.current_path.push(first_key.clone());
                        }
                    }
                    _ => {}
                }
            }
        }
        Message::Up => {
            // split current segment off from path for mutation to decrement vertically
            if let Some((current_key, parent_path)) = model.current_path.split_last_mut() {
                if let Some(next) =
                    navigate_vertically(&model.current_json, parent_path, current_key, -1)
                {
                    // replace current segment
                    *current_key = next;
                }
            }
        }
        Message::Down => {
            // split current segment off from path for mutation to increment vertically
            if let Some((current_key, parent_path)) = model.current_path.split_last_mut() {
                if let Some(next) =
                    navigate_vertically(&model.current_json, parent_path, current_key, 1)
                {
                    // replace current segment
                    *current_key = next;
                }
            }
        }
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

fn navigate_vertically(
    root: &OrderedValue,
    parent_path: &[String],
    current_segment: &str,
    delta: isize,
) -> Option<String> {
    // resolve parent container in case of early exit
    let parent = root.get(parent_path)?;

    // determine if array or object
    match parent {
        OrderedValue::Array(arr) => {
            // parse current path segment as array index
            let index = current_segment.parse::<usize>().ok()?;

            // find adjacent index
            let next = index as isize + delta;
            if next < 0 || next >= arr.len() as isize {
                return None;
            }

            Some((next as usize).to_string())
        }
        OrderedValue::Object(map) => {
            // get order number of current key
            let pos = map.get_index_of(current_segment)?;

            // find adjacent index
            let next = pos as isize + delta;
            if next < 0 || next >= map.len() as isize {
                return None;
            }

            // return key at position
            let (key, _) = map.get_index(next as usize)?;
            Some(key.clone())
        }
        _ => None,
    }
}
