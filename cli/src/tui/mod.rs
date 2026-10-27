//! Terminal UI: the project dashboard and the `init` wizard.

pub mod dashboard;
pub mod wizard;

use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

use crate::state::View;

pub const ACCENT: Color = Color::Rgb(124, 92, 255);
pub const MUTED: Color = Color::Rgb(128, 128, 140);
pub const OK: Color = Color::Rgb(80, 200, 120);
pub const WARN: Color = Color::Rgb(240, 180, 60);
pub const BAD: Color = Color::Rgb(235, 90, 90);
pub const INFO: Color = Color::Rgb(90, 170, 240);

pub fn view_style(v: View) -> Style {
    match v {
        View::Approved => Style::new().fg(OK),
        View::Stale => Style::new().fg(WARN).add_modifier(Modifier::BOLD),
        View::InProgress | View::Review => Style::new().fg(INFO),
        View::Ready => Style::new().add_modifier(Modifier::BOLD),
        View::Blocked => Style::new().fg(MUTED),
    }
}

pub fn title(s: &str) -> Span<'static> {
    Span::styled(
        format!(" {s} "),
        Style::new().fg(ACCENT).add_modifier(Modifier::BOLD),
    )
}

/// Footer of `key label` pairs.
pub fn keys(pairs: &[(&str, &str)]) -> Line<'static> {
    let mut spans = vec![];
    for (k, label) in pairs {
        spans.push(Span::styled(
            format!(" {k} "),
            Style::new().fg(Color::Black).bg(MUTED),
        ));
        spans.push(Span::styled(format!(" {label}  "), Style::new().fg(MUTED)));
    }
    Line::from(spans)
}
