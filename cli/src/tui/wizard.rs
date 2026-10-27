//! `sixloom init` in a terminal: a step-by-step wizard.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Paragraph, Wrap};
use ratatui::{DefaultTerminal, Frame};
use std::path::Path;

use super::{keys, title, ACCENT, MUTED, OK};
use crate::config::{self, Config};
use crate::harness;
use crate::init::Mode;

#[derive(Clone, Copy, PartialEq)]
enum Step {
    Mode,
    Name,
    Agents,
    Frontend,
    UiKit,
    Docs,
    Design,
    Bank,
    Review,
}

const STEPS: &[(Step, &str)] = &[
    (Step::Mode, "Project"),
    (Step::Name, "Name"),
    (Step::Agents, "Coding agents"),
    (Step::Frontend, "Interfaces"),
    (Step::UiKit, "UI kit"),
    (Step::Docs, "Docs engine"),
    (Step::Design, "Design tool"),
    (Step::Bank, "Skill bank"),
    (Step::Review, "Review"),
];

struct W<'a> {
    cfg: Config,
    mode: Mode,
    path: &'a Path,
    step: usize,
    cursor: usize,
    agents: Vec<bool>,
}

fn is_react(i: &config::Interface) -> bool {
    matches!(i.framework.as_str(), "react-vite" | "nextjs")
}

impl W<'_> {
    fn react_kit(&self) -> &str {
        self.cfg
            .interfaces
            .iter()
            .find(|i| is_react(i))
            .map(|i| i.ui_kit.as_str())
            .unwrap_or("none")
    }

    fn step(&self) -> Step {
        STEPS[self.step].0
    }

    fn options(&self) -> Vec<(String, String, bool)> {
        let radio = |opts: &[(&str, &str)], cur: &str| -> Vec<(String, String, bool)> {
            opts.iter()
                .map(|(id, d)| (id.to_string(), d.to_string(), *id == cur))
                .collect()
        };
        match self.step() {
            Step::Mode => vec![
                (
                    "new".into(),
                    "Create a new project in this directory".into(),
                    self.mode == Mode::New,
                ),
                (
                    "adopt".into(),
                    "Add sixloom to an existing codebase (no app code touched)".into(),
                    self.mode == Mode::Adopt,
                ),
            ],
            Step::Agents => harness::ALL
                .iter()
                .zip(&self.agents)
                .map(|(h, on)| {
                    let note = match (h.on_path(), h.verified) {
                        (true, _) => "detected".to_string(),
                        (false, false) => "paths from docs".to_string(),
                        _ => String::new(),
                    };
                    (
                        h.id.to_string(),
                        format!("{:<20} {:<16} {note}", h.name, h.skills_dir),
                        *on,
                    )
                })
                .collect(),
            Step::Frontend => config::INTERFACES
                .iter()
                .map(|(kind, fw, label)| {
                    let on = self
                        .cfg
                        .interfaces
                        .iter()
                        .any(|i| i.kind == *kind && i.framework == *fw);
                    (
                        format!("{kind}:{fw}"),
                        format!("{label}  → apps/{kind}"),
                        on,
                    )
                })
                .collect(),
            Step::UiKit => radio(config::UI_KITS, self.react_kit()),
            Step::Docs => radio(config::DOCS, &self.cfg.docs.engine),
            Step::Design => radio(config::DESIGNS, &self.cfg.design.tool),
            Step::Bank => vec![
                (
                    "no".into(),
                    "Install only the skills for my choices".into(),
                    !self.cfg.skills.bank,
                ),
                (
                    "yes".into(),
                    "Also put every other skill in an other-skill bank (.otherskill/bank)".into(),
                    self.cfg.skills.bank,
                ),
            ],
            Step::Name | Step::Review => vec![],
        }
    }

    fn choose(&mut self) {
        let opts = self.options();
        let Some((id, _, _)) = opts.get(self.cursor).cloned() else {
            return;
        };
        match self.step() {
            Step::Mode => {
                self.mode = if id == "adopt" {
                    Mode::Adopt
                } else {
                    Mode::New
                }
            }
            Step::Agents => self.agents[self.cursor] = !self.agents[self.cursor],
            Step::Frontend => {
                let (kind, fw) = id.split_once(':').unwrap();
                if let Some(pos) = self
                    .cfg
                    .interfaces
                    .iter()
                    .position(|i| i.kind == kind && i.framework == fw)
                {
                    self.cfg.interfaces.remove(pos);
                } else if let Ok(i) = config::parse_interface(&id, &self.cfg.interfaces) {
                    self.cfg.interfaces.push(i);
                }
            }
            Step::UiKit => {
                for i in self.cfg.interfaces.iter_mut().filter(|i| is_react(i)) {
                    i.ui_kit = id.clone();
                }
            }
            Step::Docs => {
                self.cfg.docs.content_dir = config::content_dir_for(&id).into();
                self.cfg.docs.engine = id;
            }
            Step::Design => self.cfg.design.tool = id,
            Step::Bank => self.cfg.skills.bank = id == "yes",
            _ => {}
        }
    }

    fn advance(&mut self, by: isize) {
        loop {
            let next = self.step as isize + by;
            if next < 0 || next >= STEPS.len() as isize {
                return;
            }
            self.step = next as usize;
            // The UI kit question only applies to React front ends.
            let skip = self.step() == Step::UiKit && !self.cfg.interfaces.iter().any(is_react);
            if !skip {
                break;
            }
        }
        self.cursor = self
            .options()
            .iter()
            .position(|o| o.2)
            .filter(|_| !matches!(self.step(), Step::Agents | Step::Frontend))
            .unwrap_or(0);
    }
}

/// Returns None when the user cancels.
pub fn run(cfg: Config, mode: Mode, path: &Path) -> Result<Option<(Config, Mode)>> {
    let agents = harness::ALL
        .iter()
        .map(|h| cfg.agents.harnesses.iter().any(|a| a == h.id) || h.on_path())
        .collect();
    let mut w = W {
        cfg,
        mode,
        path,
        step: 0,
        cursor: if mode == Mode::Adopt { 1 } else { 0 },
        agents,
    };
    let mut t = ratatui::init();
    let res = event_loop(&mut t, &mut w);
    ratatui::restore();
    Ok(match res? {
        true => {
            w.cfg.agents.harnesses = harness::ALL
                .iter()
                .zip(&w.agents)
                .filter(|(_, on)| **on)
                .map(|(h, _)| h.id.to_string())
                .collect();
            Some((w.cfg, w.mode))
        }
        false => None,
    })
}

fn event_loop(t: &mut DefaultTerminal, w: &mut W) -> Result<bool> {
    loop {
        t.draw(|f| draw(f, w))?;
        let Event::Key(k) = event::read()? else {
            continue;
        };
        if k.kind != KeyEventKind::Press {
            continue;
        }
        if k.modifiers.contains(KeyModifiers::CONTROL) && k.code == KeyCode::Char('c') {
            return Ok(false);
        }
        let n = w.options().len();
        match (w.step(), k.code) {
            (_, KeyCode::Esc) => return Ok(false),
            (Step::Name, KeyCode::Char(c)) => w.cfg.project.name.push(c),
            (Step::Name, KeyCode::Backspace) => {
                w.cfg.project.name.pop();
            }
            (Step::Review, KeyCode::Enter) => return Ok(true),
            (Step::Agents, KeyCode::Char(' ')) => w.choose(),
            (Step::Agents, KeyCode::Enter) => {
                if w.agents.iter().any(|a| *a) {
                    w.advance(1);
                }
            }
            (Step::Frontend, KeyCode::Char(' ')) => w.choose(),
            (Step::Frontend, KeyCode::Enter) => {
                if !w.cfg.interfaces.is_empty() {
                    w.advance(1);
                }
            }
            (_, KeyCode::Enter) => {
                if n > 0 {
                    w.choose();
                }
                if !w.cfg.project.name.trim().is_empty() {
                    w.advance(1);
                }
            }
            (_, KeyCode::Down | KeyCode::Char('j')) if n > 0 => w.cursor = (w.cursor + 1) % n,
            (_, KeyCode::Up | KeyCode::Char('k')) if n > 0 => w.cursor = (w.cursor + n - 1) % n,
            (_, KeyCode::Char(' ')) if n > 0 => w.choose(),
            (_, KeyCode::Backspace | KeyCode::Left) => w.advance(-1),
            (_, KeyCode::Char('q')) => return Ok(false),
            _ => {}
        }
    }
}

fn draw(f: &mut Frame, w: &W) {
    let [head, body, foot] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Min(10),
        Constraint::Length(1),
    ])
    .areas(f.area());
    f.render_widget(
        Line::from(vec![
            Span::styled(
                " sixloom init ",
                Style::new()
                    .fg(ratatui::style::Color::Black)
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(format!("  {}", w.path.display()), Style::new().fg(MUTED)),
        ]),
        head,
    );
    let [side, main] =
        Layout::horizontal([Constraint::Length(22), Constraint::Min(40)]).areas(body);
    let steps: Vec<Line> = STEPS
        .iter()
        .enumerate()
        .map(|(i, (_, name))| {
            let (mark, st) = if i < w.step {
                ("✓", Style::new().fg(OK))
            } else if i == w.step {
                ("▸", Style::new().fg(ACCENT).add_modifier(Modifier::BOLD))
            } else {
                ("·", Style::new().fg(MUTED))
            };
            Line::styled(format!(" {mark} {name}"), st)
        })
        .collect();
    f.render_widget(
        Paragraph::new(steps).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(MUTED)),
        ),
        side,
    );

    let (q, help) = match w.step() {
        Step::Mode => (
            "What are we setting up?",
            "Adopt keeps your code; it adds sixloom.toml, skills, and instruction blocks.",
        ),
        Step::Name => ("Project name", "Used in the spec and the docs site title."),
        Step::Agents => (
            "Which coding agents will you use?",
            "Skills are copied into each agent's skills directory. Space toggles.",
        ),
        Step::Frontend => (
            "Which interfaces does the product have?",
            "Web, TUI, CLI, mobile or desktop; pick one or more (space toggles). The preview is built in each and becomes the real interface. Folders can be changed later with `sixloom layout`.",
        ),
        Step::UiKit => (
            "UI component kit",
            "shadcn/ui gives accessible primitives; tokens still come from DESIGN.md.",
        ),
        Step::Docs => (
            "Docs engine",
            "All three render MDX + Mermaid and generate the API reference from OpenAPI.",
        ),
        Step::Design => (
            "Design tool for the UI design loom",
            "Stitch uses Google's stitch CLI (you sign in once with `stitch login`).",
        ),
        Step::Bank => (
            "Keep other skills available?",
            "A bank keeps them out of context; other-skill injects them only when relevant.",
        ),
        Step::Review => (
            "Ready to create",
            "Enter creates everything below. Backspace goes back.",
        ),
    };
    let mut lines = vec![
        Line::styled(q, Style::new().add_modifier(Modifier::BOLD)),
        Line::styled(help, Style::new().fg(MUTED)),
        Line::raw(""),
    ];
    match w.step() {
        Step::Name => lines.push(Line::from(vec![
            Span::raw("  › "),
            Span::styled(format!("{}▏", w.cfg.project.name), Style::new().fg(ACCENT)),
        ])),
        Step::Review => {
            let agents: Vec<_> = harness::ALL
                .iter()
                .zip(&w.agents)
                .filter(|(_, on)| **on)
                .map(|(h, _)| h.id)
                .collect();
            for (k, v) in [
                (
                    "mode",
                    if w.mode == Mode::New {
                        "new project"
                    } else {
                        "adopt existing"
                    }
                    .to_string(),
                ),
                ("name", w.cfg.project.name.clone()),
                ("agents", agents.join(", ")),
                (
                    "interfaces",
                    w.cfg
                        .interfaces
                        .iter()
                        .map(|i| format!("{} → {}/", i.label(), i.path))
                        .collect::<Vec<_>>()
                        .join(", "),
                ),
                ("services", "decided in the system-design loom".into()),
                (
                    "docs",
                    format!("{} ({})", w.cfg.docs.engine, w.cfg.docs.content_dir),
                ),
                (
                    "design",
                    format!("{} · fidelity {}", w.cfg.design.tool, w.cfg.design.fidelity),
                ),
                (
                    "skill bank",
                    if w.cfg.skills.bank { "yes" } else { "no" }.into(),
                ),
            ] {
                lines.push(Line::from(vec![
                    Span::styled(format!("  {k:<12}"), Style::new().fg(MUTED)),
                    Span::raw(v),
                ]));
            }
        }
        _ => {
            let multi = matches!(w.step(), Step::Agents | Step::Frontend);
            for (i, (id, desc, on)) in w.options().iter().enumerate() {
                let mark = match (multi, on) {
                    (true, true) => "[x]",
                    (true, false) => "[ ]",
                    (false, true) => "(•)",
                    (false, false) => "( )",
                };
                let st = if i == w.cursor {
                    Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)
                } else {
                    Style::new()
                };
                lines.push(Line::from(vec![
                    Span::styled(
                        format!("{} {mark} {id:<12}", if i == w.cursor { "›" } else { " " }),
                        st,
                    ),
                    Span::styled(desc.clone(), Style::new().fg(MUTED)),
                ]));
            }
        }
    }
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(MUTED))
                .title(title(STEPS[w.step].1)),
        ),
        main,
    );
    f.render_widget(
        keys(&[
            ("↑↓", "move"),
            ("space", "toggle"),
            ("enter", "next"),
            ("⌫", "back"),
            ("esc", "cancel"),
        ]),
        foot,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn walks_every_step() {
        let p = std::path::PathBuf::from("/tmp/x");
        let agents = harness::ALL.iter().map(|h| h.id == "claude").collect();
        let mut w = W {
            cfg: crate::init::default_config("x"),
            mode: Mode::New,
            path: &p,
            step: 0,
            cursor: 0,
            agents,
        };
        let mut t = Terminal::new(TestBackend::new(100, 30)).unwrap();
        // Swap React for Vue and add a Ratatui TUI: no React left, so the UI-kit step is skipped.
        while w.step() != Step::Review {
            if w.step() == Step::Frontend {
                for pick in ["web:react-vite", "web:vue", "tui:ratatui"] {
                    w.cursor = w.options().iter().position(|o| o.0 == pick).unwrap();
                    w.choose();
                }
            }
            t.draw(|f| draw(f, &w)).unwrap();
            w.advance(1);
            assert!(w.step() != Step::UiKit);
        }
        t.draw(|f| draw(f, &w)).unwrap();
        let text: String = t
            .backend()
            .buffer()
            .content()
            .iter()
            .map(|c| c.symbol())
            .collect();
        assert!(text.contains("web (web · vue) → apps/web/"));
        assert!(text.contains("tui (tui · ratatui) → apps/tui/"));
        assert_eq!(w.cfg.interfaces.len(), 2);
    }
}
