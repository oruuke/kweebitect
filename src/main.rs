use better_panic;
use clap::Parser;
use color_eyre::eyre::WrapErr;
use std::{fs, path::PathBuf};

mod model;
mod update;
mod view;
use crate::{
    model::{Model, OrderedValue, RunningState},
    update::{handle_event, update},
    view::view,
};

#[derive(Debug, Parser)]
#[command(name = "kweebitect", version, about)]
struct Args {
    #[arg(value_name = "FILE", value_hint = clap::ValueHint::FilePath)]
    input: PathBuf,
}

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    better_panic::install();

    // validate and transform json to custom serde value
    let args = Args::parse();
    let json = fs::read_to_string(&args.input)
        .wrap_err_with(|| format!("failed to read file: {}", args.input.display()))?;
    let data: OrderedValue = OrderedValue::from_str(&json)
        .wrap_err_with(|| format!("failed to parse json: {}", args.input.display()))?;

    ratatui::run(|terminal| {
        let mut model = Model::default();

        model.current_json = data;
        model.ensure_root_segment();

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
    })
}
