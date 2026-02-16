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
            model.current_path.push(String::from("0"));
        }
        Message::Up => {}
        Message::Down => {}
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
