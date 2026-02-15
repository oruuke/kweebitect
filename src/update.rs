use std::time::Duration;

use color_eyre;
use ratatui::crossterm::event::{self, Event, KeyCode};

use crate::model::{CurrentMode, Model, RunningState};

// update handling with a message for each action/event (logic)
#[derive(PartialEq)]
pub enum Message {
    Out,
    In,
    Up,
    Down,
    EditObject,
    EditField,
    DeleteObject,
    DeleteField,
    CreateBelow,
    CreateAbove,
    ConfirmValue,
    ConfirmCommand,
    ChangeMode,
    Quit,
}

pub fn handle_event(_: &Model) -> color_eyre::Result<Option<Message>> {
    if event::poll(Duration::from_millis(250))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press {
                return Ok(handle_key(key));
            }
        }
    }
    Ok(None)
}

fn handle_key(key: event::KeyEvent) -> Option<Message> {
    match key.code {
        KeyCode::Left => Some(Message::Out),
        KeyCode::Right => Some(Message::In),
        KeyCode::Down => Some(Message::Down),
        KeyCode::Up => Some(Message::Up),
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
            model.current_path.push(String::from("0"));
        }
        Message::Up => {}
        Message::Down => {}
        Message::EditObject => {}
        Message::EditField => {}
        Message::DeleteObject => {}
        Message::DeleteField => {}
        Message::CreateBelow => {}
        Message::CreateAbove => {}
        Message::ConfirmValue => {}
        Message::ConfirmCommand => {}
        Message::ChangeMode => {}
        Message::Quit => {
            model.running_state = RunningState::Done;
        }
    };
    None
}
