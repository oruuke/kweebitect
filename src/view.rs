use crate::model::{CurrentMode, Model, OrderedValue, PathSegment};

#[path = "common/lib.rs"]
mod common;

use common::{Colors, item_container::ListItemContainer};
//use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use itertools::Itertools;
use ratatui::{
    Frame,
    layout::{Constraint, Constraint::Length, Layout, Rect},
    style::Stylize,
    widgets::{Block, Borders, List as RatatuiList, ListItem, Padding, Paragraph},
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
    let pretty_path = model.current_path.iter().map(|p| &p.key).join("/");
    let block_path = Block::bordered();
    let path = Paragraph::new(format!("{}", pretty_path)).block(block_path);

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

    // top current path strip
    frame.render_widget(path, top);
    // main view, rendering either preview or browser
    match model.current_mode {
        CurrentMode::Preview => {
            // build preview paragraph
            let pretty_json = model.current_json.get_pretty(&model.current_path);
            let block_json = Block::bordered();
            let json = Paragraph::new(format!("{}", pretty_json)).block(block_json);
            // render preview in middle area
            frame.render_widget(json, middle);
        }
        _ => {
            // render browser in middle area
            HorizontalList::new(model).render(frame, middle);
        }
    }
    // bottom current mode strip
    frame.render_widget(mode, bottom);
}

// thin wrapper for mutable state access
pub struct HorizontalList<'a> {
    model: &'a mut Model,
}

// horizontal list for representing depth
impl<'a> HorizontalList<'a> {
    pub fn new(model: &'a mut Model) -> Self {
        Self { model }
    }

    // render proxy for passing state
    pub fn render(self, frame: &mut Frame, area: Rect) {
        // setup layout for each level of depth
        let constraints: Vec<_> = std::iter::once(Length(40))
            .chain(self.model.current_path.iter().map(|_| Length(40)))
            .collect();
        let layout = Layout::horizontal(constraints)
            .flex(ratatui::layout::Flex::Start)
            .spacing(1)
            .split(area);
        // render root to give user somewhere to start
        let pretty_json = self.model.current_json.to_string_pretty().expect("failed");
        let root = Paragraph::new(format!("{}", pretty_json))
            .block(Block::default().borders(Borders::ALL).title("root"));
        frame.render_widget(root, layout[0]);

        // iterate rendering
        for (i, segment) in self.model.current_path.iter().enumerate() {
            // build key path up to scoped segment
            let keys: Vec<&str> = self.model.current_path[..=i]
                .iter()
                .map(|p| p.key.as_str())
                .collect();
            // get printable type of scoped segment
            let segment_type = self
                .model
                .current_json
                .get(&keys)
                .map(|v| v.to_string())
                .unwrap_or_else(|| "unknown".to_string());
            // render block for segment
            let pretty_json = self.model.current_json.get_pretty(&keys);
            let json = Paragraph::new(format!("{}", pretty_json))
                .block(Block::default().borders(Borders::ALL).title(segment_type));

            frame.render_widget(json, layout[i + 1]);
        }
    }
}
