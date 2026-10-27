//! Plain-terminal styling. Honours NO_COLOR and non-terminal output.

use std::io::IsTerminal;
use std::sync::OnceLock;

use crate::state::View;

fn color() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var_os("NO_COLOR").is_none() && std::io::stdout().is_terminal())
}

fn paint(code: &str, s: &str) -> String {
    if color() {
        format!("\x1b[{code}m{s}\x1b[0m")
    } else {
        s.to_string()
    }
}

pub fn bold(s: &str) -> String {
    paint("1", s)
}
pub fn dim(s: &str) -> String {
    paint("2", s)
}
pub fn green(s: &str) -> String {
    paint("32", s)
}
pub fn red(s: &str) -> String {
    paint("31", s)
}
pub fn yellow(s: &str) -> String {
    paint("33", s)
}
pub fn cyan(s: &str) -> String {
    paint("36", s)
}

pub fn view_color(v: View, s: &str) -> String {
    match v {
        View::Approved => green(s),
        View::Stale => yellow(s),
        View::InProgress | View::Review => cyan(s),
        View::Ready => bold(s),
        View::Blocked => dim(s),
    }
}
