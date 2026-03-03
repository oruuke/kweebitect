use crate::model::{
    CurrentMode, Model, OrderedValue, container_entries_for, format_container_entry_lines,
    value_for_entry,
};
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

// calculate column width from title and rendered row lines
fn get_column_width(title: &str, container: &OrderedValue, entries: &[String]) -> u16 {
    // include title width
    let mut max_line_width = display_width(title);
    for entry_label in entries {
        // ensure all entry widths are accounted for
        let row_value = value_for_entry(container, entry_label).unwrap_or(container);
        for line in format_container_entry_lines(entry_label, row_value) {
            max_line_width = max_line_width.max(display_width(&line));
        }
    }

    // return final width, including padding
    (max_line_width.saturating_add(4)).clamp(16, u16::MAX as usize) as u16
}

// build a rendered row for a container entry wit selection styling
fn build_entry_row(
    container: &OrderedValue,
    entries: &[String],
    index: usize,
    is_selected: bool,
) -> (Paragraph<'static>, u16) {
    // row highlighting
    let mut style = Style::default();
    if is_selected {
        style = style.add_modifier(Modifier::REVERSED);
    }

    // safely get label and value
    let entry_label = entries
        .get(index)
        .cloned()
        .unwrap_or_else(|| "?".to_string());
    let row_value = value_for_entry(container, &entry_label).unwrap_or(container);

    // render one or many lines depending on value type
    let lines = format_container_entry_lines(&entry_label, row_value)
        .into_iter()
        .map(Line::from)
        .collect::<Vec<_>>();
    let height = u16::try_from(lines.len()).unwrap_or(u16::MAX);
    let widget = Paragraph::new(Text::from(lines)).style(style);
    (widget, height)
}

// apply shared list config for rendered columns
fn with_standard_list_options<B>(list: ListView<B>, title: String) -> ListView<B> {
    list.scroll_axis(ScrollAxis::Vertical)
        .infinite_scrolling(true)
        .block(Block::default().borders(Borders::ALL).title(title))
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
    // width at which only two path columns are visible
    const TWO_COL_WIDTH: u16 = 150;

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
        let mut path_columns: Vec<ColumnData> = Vec::with_capacity(column_count.max(1));
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

            // add column wit new width and padding on top
            let desired_width = get_column_width(&title, &container, &entries);
            path_columns.push(ColumnData {
                title,
                container,
                entries,
                desired_width,
            });
        }

        // preview column for containers
        let mut preview_column: Option<ColumnData> = None;
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
                    let desired_width = get_column_width(&title, &container, &entries);
                    preview_column = Some(ColumnData {
                        title,
                        container,
                        entries,
                        desired_width,
                    });
                }
            }
        }

        // setup responsive viewport wit either 4 or 2 max visible columns
        let spacing: u16 = 1;
        let max_visible_path_columns = if area.width <= Self::TWO_COL_WIDTH {
            2
        } else {
            4
        };
        let path_end = path_columns.len();
        let path_start = path_end.saturating_sub(max_visible_path_columns);
        let visible_path_columns = &path_columns[path_start..path_end];

        // setup layout for each level of depth
        let mut constraints: Vec<_> = visible_path_columns
            .iter()
            .map(|c| Constraint::Length(c.desired_width))
            .collect();
        if preview_column.is_some() {
            constraints.push(Constraint::Fill(1));
        }

        // split available area into path column rects and optional trailing preview
        let layout = Layout::horizontal(constraints)
            .flex(ratatui::layout::Flex::Start)
            .spacing(spacing)
            .split(area);

        // initialise preview
        let mut preview_state = tui_widget_list::ListState::default();
        preview_state.select(None);

        // render each visible column of the path
        for (visible_index, column) in visible_path_columns.iter().enumerate() {
            // clone into row builder closure
            let container_value_owned = column.container.clone();
            let entries_cloned = column.entries.clone();
            let builder = ListBuilder::new(move |context| {
                build_entry_row(
                    &container_value_owned,
                    &entries_cloned,
                    context.index,
                    context.is_selected,
                )
            });

            // wrap list in a titled block for current column
            let list = with_standard_list_options(
                ListView::new(builder, column.entries.len()),
                column.title.clone(),
            );

            // render active columns mapped back to path depth
            let path_index = path_start + visible_index;
            let state = &mut self.model.current_path[path_index].list_state;
            frame.render_stateful_widget(list, layout[visible_index], state);
        }

        // render preview in remaining space
        if let Some(column) = preview_column {
            let preview_rect_index = visible_path_columns.len();
            if let Some(preview_rect) = layout.get(preview_rect_index).copied() {
                if preview_rect.width > 0 && preview_rect.height > 0 {
                    // build stateful list widget
                    let container_value_owned = column.container.clone();
                    let entries_cloned = column.entries.clone();
                    let builder = ListBuilder::new(move |context| {
                        build_entry_row(
                            &container_value_owned,
                            &entries_cloned,
                            context.index,
                            context.is_selected,
                        )
                    });

                    // wrap list in a titled block for current column
                    let list = with_standard_list_options(
                        ListView::new(builder, column.entries.len()),
                        column.title.clone(),
                    );
                    frame.render_stateful_widget(list, preview_rect, &mut preview_state);
                }
            }
        }
    }
}
