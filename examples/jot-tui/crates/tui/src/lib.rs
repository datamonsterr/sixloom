pub mod client;
pub mod mocks;

use ratatui::widgets::{Block, BorderType, List, Paragraph};
use ratatui::Frame;

/// Widget names tests use as handles.
pub const NOTES: &str = "notes";

/// S-01 Notes list.
// sxl: F-001.AC1, F-001.AC2
pub fn render(f: &mut Frame, notes: &[client::Note]) {
    let block = Block::bordered().border_type(BorderType::Rounded).title(" Jot ");
    if notes.is_empty() {
        f.render_widget(Paragraph::new("No notes yet").block(block), f.area());
        return;
    }
    let items: Vec<String> = notes.iter().map(|n| n.text.clone()).collect();
    f.render_widget(List::new(items).block(block), f.area());
}
