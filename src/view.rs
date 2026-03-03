use crate::model::{
    CurrentMode, Model, OrderedValue, container_entries_for, format_container_entry_lines,
    value_for_entry,
};
//use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind};
use itertools::Itertools;
use ratatui::{
    Frame,
    layout::{Constraint, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};
use tui_widget_list::{ListBuilder, ListView, ScrollAxis};

// return width of string
fn display_width(s: &str) -> usize {
    s.chars().count()
}

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

    // top current path strip
    frame.render_widget(path, top);
    // main view, rendering either preview or browser
    match model.current_mode {
        CurrentMode::Preview => {
            // render preview in middle area
            frame.render_widget(json, middle);
        }
        _ => {
            // render browser in middle area
            HorizontalBlocks::new(model).render(frame, middle);
        }
    }
    // bottom current mode strip
    frame.render_widget(mode, bottom);
}

// thin wrapper for mutable state access
pub struct HorizontalBlocks<'a> {
    model: &'a mut Model,
}

// horizontal list for representing depth
impl<'a> HorizontalBlocks<'a> {
    pub fn new(model: &'a mut Model) -> Self {
        Self { model }
    }

    // render proxy for passing state
    pub fn render(self, frame: &mut Frame, area: Rect) {
        // setup root depth state
        self.model.ensure_root_segment();
        let column_count = self.model.current_path.len();
        struct ColumnData {
            title: String,
            container: OrderedValue,
            entries: Vec<String>,
            desired_width: u16,
        }

        // calculate ideal widths based on keys and values
        let mut columns: Vec<ColumnData> = Vec::with_capacity(column_count.max(1));
        for depth in 0..column_count {
            let title = if depth == 0 {
                "root".to_string()
            } else {
                self.model.current_path[depth - 1].key.clone()
            };

            let (container, entries) = {
                // get data of depth for reference
                let segment = &mut self.model.current_path[depth];
                let container = segment.value.clone();
                let entries = container_entries_for(&container);

                // ensure valid selection for rendering column
                if !entries.is_empty() {
                    // defaul to first entry when no selection
                    if segment.list_state.selected.is_none() {
                        segment.list_state.select(Some(0));
                    }

                    if let Some(selected_index) = segment.list_state.selected {
                        // clamp selection if entries changed sincelast render
                        let selected_index = selected_index.min(entries.len().saturating_sub(1));
                        segment.list_state.select(Some(selected_index));
                        // sync model's key wit selected row
                        segment.key = entries[selected_index].clone();
                    }
                }

                (container, entries)
            };

            // get widest rendered line to determine suitable width
            let mut max_line_width = display_width(&title);
            for entry_label in &entries {
                // simulate key/value string for maintaining width
                let row_value = value_for_entry(&container, entry_label).unwrap_or(&container);
                for line in format_container_entry_lines(entry_label, row_value) {
                    max_line_width = max_line_width.max(display_width(&line));
                }
            }

            // add column wit new width and padding on top
            let desired_width =
                (max_line_width.saturating_add(4)).clamp(16, u16::MAX as usize) as u16;
            columns.push(ColumnData {
                title,
                container,
                entries,
                desired_width,
            });
        }

        // preview column for containers
        if let Some(active_segment) = self.model.current_path.last() {
            if let Some(selected_value) =
                value_for_entry(&active_segment.value, &active_segment.key)
            {
                // only preview containers
                if matches!(
                    selected_value,
                    OrderedValue::Array(_) | OrderedValue::Object(_)
                ) {
                    let title = format!("{}", active_segment.key);
                    let container = selected_value.clone();
                    let entries = container_entries_for(&container);

                    // calculate column width
                    let mut max_line_width = display_width(&title);
                    for entry_label in &entries {
                        let row_value =
                            value_for_entry(&container, entry_label).unwrap_or(&container);
                        for line in format_container_entry_lines(entry_label, row_value) {
                            max_line_width = max_line_width.max(display_width(&line));
                        }
                    }

                    let desired_width =
                        (max_line_width.saturating_add(4)).clamp(16, u16::MAX as usize) as u16;
                    columns.push(ColumnData {
                        title,
                        container,
                        entries,
                        desired_width,
                    });
                }
            }
        }

        // setup scrolling viewport
        let spacing: u16 = 1;
        let viewport_width = area.width;
        let end = columns.len();
        let mut start = end;
        let mut used_width: u16 = 0;
        // maintin scrolling
        while start > 0 {
            let w = columns[start - 1].desired_width;
            let add = if used_width == 0 {
                w
            } else {
                w.saturating_add(spacing)
            };
            if used_width != 0 && used_width.saturating_add(add) > viewport_width {
                break;
            }
            used_width = used_width.saturating_add(add);
            start -= 1;
        }

        // setup layout for each level of depth
        let constraints: Vec<_> = columns[start..end]
            .iter()
            .map(|c| Constraint::Length(c.desired_width))
            .collect();

        // split available area into column rects
        let layout = Layout::horizontal(constraints)
            .flex(ratatui::layout::Flex::Start)
            .spacing(spacing)
            .split(area);

        // initialise preview
        let mut preview_state = tui_widget_list::ListState::default();
        preview_state.select(None);

        // render each column
        for (visible_index, column_index) in (start..end).enumerate() {
            let column = &columns[column_index];
            // clone into row builder closure
            let container_value_owned = column.container.clone();
            let entries_cloned = column.entries.clone();
            let builder = ListBuilder::new(move |context| {
                // shared base styling
                let mut style = Style::default();
                if context.is_selected {
                    style = style.add_modifier(Modifier::REVERSED);
                }

                // get entry label for dis row wit stable fallback
                let entry_label = entries_cloned
                    .get(context.index)
                    .cloned()
                    .unwrap_or_else(|| "?".to_string());

                // get the row value for the dis entry
                let row_value = value_for_entry(&container_value_owned, &entry_label)
                    .unwrap_or(&container_value_owned);

                // render the row wit potential for container
                let lines = format_container_entry_lines(&entry_label, row_value)
                    .into_iter()
                    .map(Line::from)
                    .collect::<Vec<_>>();

                // return height along wit widget to handle multi-line
                let height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
                let widget = Paragraph::new(Text::from(lines)).style(style);
                (widget, height)
            });

            // wrap list in a titled block for current column
            let list = ListView::new(builder, column.entries.len())
                .scroll_axis(ScrollAxis::Vertical)
                .infinite_scrolling(true)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(column.title.clone()),
                );

            // render active and preview columns
            if column_index < column_count {
                let state = &mut self.model.current_path[column_index].list_state;
                frame.render_stateful_widget(list, layout[visible_index], state);
            } else {
                frame.render_stateful_widget(list, layout[visible_index], &mut preview_state);
            }
        }
    }
}
