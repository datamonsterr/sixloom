fn main() {
    let api = jot_tui::client::from_env();
    let notes = api.list_notes().unwrap_or_default();
    let mut terminal = ratatui::init();
    let _ = terminal.draw(|f| jot_tui::render(f, &notes));
    let _ = ratatui::crossterm::event::read();
    ratatui::restore();
}
