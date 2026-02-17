use better_panic;
use clap::Parser;
use color_eyre;
use std::fs;
use std::io::Write;
use std::path::Path;

mod model;
mod update;
mod view;
use crate::{
    model::{Model, OrderedValue, RunningState},
    update::{handle_event, update},
    view::view,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    better_panic::install();

    let mut terminal = ratatui::init();
    let mut model = Model::default();

    let json = r#"{
        "type": "Abstract",
        "Debug": "DisplayState",
        "Instructions": [
            {
                "Sensor": {
                    "Type": "State",
                    "State": "Idle"
                },
                "Instructions": []
            },
            {
                "Sensor": {
                    "Type": "State",
                    "State": "Sleep"
                },
                "Instructions": []
            }
        ]
    }"#;
    let data: OrderedValue = OrderedValue::from_str(json)?;
    model.current_json = data;

    while model.running_state != RunningState::Done {
        // render current  view
        terminal.draw(|f| view(&mut model, f))?;

        // handle events and map to a message
        let mut current_msg = handle_event(&model)?;

        // process updates until none message
        while current_msg.is_some() {
            current_msg = update(&mut model, current_msg.unwrap());
        }
    }

    ratatui::restore();
    Ok(())
}
