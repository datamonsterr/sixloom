use jot_tui::mocks::{MockClient, Scenario};
use jot_tui::client::Api;
use ratatui::backend::TestBackend;
use ratatui::Terminal;

fn screen(scenario: Scenario) -> Vec<String> {
    let notes = MockClient::new(scenario).list_notes().unwrap();
    let mut t = Terminal::new(TestBackend::new(30, 6)).unwrap();
    t.draw(|f| jot_tui::render(f, &notes)).unwrap();
    let buf = t.backend().buffer().clone();
    (0..buf.area.height)
        .map(|y| (0..buf.area.width).map(|x| buf[(x, y)].symbol().to_string()).collect::<String>().trim_end().to_string())
        .collect()
}

#[test]
fn f001_ac1_newest_first() {
    let lines = screen(Scenario::Ready);
    assert_eq!(lines[1], "│Call landlord               │");
    assert_eq!(lines[2], "│Buy coffee                  │");
}

#[test]
fn f001_ac2_no_notes_yet() {
    let lines = screen(Scenario::Empty);
    assert_eq!(lines[1], "│No notes yet                │");
}
