use crate::model::Model;
use ratatui::{
    Frame,
    symbols::border,
    widgets::{Block, Paragraph},
};

// rendering view to always produce same ui representation for given model
pub fn view(model: &mut Model, frame: &mut Frame) {
    let block = Block::bordered().border_set(border::THICK);
    let p = Paragraph::new(format!("counter: {}", model.counter))
        .centered()
        .block(block);
    frame.render_widget(p, frame.area());
}
