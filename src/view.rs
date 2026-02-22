use crate::model::{CurrentMode, LIST_STATE, Model};

#[path = "common/lib.rs"]
mod common;

use common::{Colors, item_container::ListItemContainer};
//use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    prelude::*,
    style::Stylize,
    widgets::{Block, Borders, Padding, Paragraph},
};
use tui_widget_list::{ListBuilder, ListState, ListView, ScrollAxis};

// rendering view to always produce same ui representation for given model
pub fn view(model: &mut Model, frame: &mut Frame) {
    // layout setup
    let area = frame.area();
    let [top, middle, bottom] = area.layout(&Layout::vertical([
        Constraint::Length(3),
        Constraint::Fill(1),
        Constraint::Length(3),
    ]));

    // path strip
    let pretty_path = model.current_path.join("/");
    let block_path = Block::bordered();
    let path = Paragraph::new(format!("{}", pretty_path)).block(block_path);

    // main browser
    let pretty_json = model.current_json.get_pretty(&model.current_path);
    let block_json = Block::bordered();
    let json = Paragraph::new(format!("{}", pretty_json)).block(block_json);

    // mode strip
    let pretty_mode = match model.current_mode {
        CurrentMode::Browse => "browse",
        CurrentMode::Preview => "preview",
        CurrentMode::Create => "create",
        CurrentMode::Select => "select",
        CurrentMode::Edit => "edit",
        CurrentMode::Command => "command",
    };
    let block_mode = Block::bordered();
    let mode = Paragraph::new(format!("{}", pretty_mode)).block(block_mode);

    // rendering layouts
    frame.render_widget(path, top);
    match model.current_mode {
        CurrentMode::Preview => {
            frame.render_widget(json, middle);
        }
        _ => {
            frame.render_widget(HorizontalList, middle);
        }
    }
    frame.render_widget(mode, bottom);
}

pub struct HorizontalList;
impl Widget for HorizontalList {
    fn render(self, area: Rect, buf: &mut Buffer) {
        const ITEMS: [&str; 4] = ["blahaj", "skirts", "thigh highs", "converse"];
        let builder = ListBuilder::new(move |context| {
            let line = Line::from(ITEMS[context.index]).alignment(Alignment::Center);
            let item = ListItemContainer::new(line, Padding::vertical(1));

            let item = match context.is_selected {
                true => item.bg(Colors::ORANGE).fg(Colors::CHARCOAL),
                false if context.index % 2 == 0 => item.bg(Colors::CHARCOAL),
                false => item.bg(Colors::BLACK),
            };

            (item, 20)
        });

        let list = ListView::new(builder, ITEMS.len())
            .scroll_axis(ScrollAxis::Horizontal)
            .infinite_scrolling(false)
            .block(Block::default().borders(Borders::ALL));

        if let Ok(mut list_state) = LIST_STATE.lock() {
            StatefulWidget::render(list, area, buf, &mut list_state);
        }
    }
}
