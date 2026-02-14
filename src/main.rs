use better_panic;
use color_eyre;

mod model;
mod update;
mod view;
use crate::{
    model::{Model, RunningState},
    update::{handle_event, update},
    view::view,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    better_panic::install();

    let mut terminal = ratatui::init();
    let mut model = Model::default();

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
