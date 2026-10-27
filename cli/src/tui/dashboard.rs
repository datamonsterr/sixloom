//! `sixloom` with no arguments: the project dashboard.

use anyhow::Result;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Cell, Clear, List, ListItem, ListState, Paragraph, Row, Table, Tabs, Wrap,
};
use ratatui::{DefaultTerminal, Frame};
use std::path::Path;

use super::{keys, title, view_style, ACCENT, BAD, MUTED, OK, WARN};
use crate::change::{self, Change};
use crate::config::Config;
use crate::state::{self, State, View, LOOMS};
use crate::{catalog, loom, skills, trace};

const TABS: &[&str] = &["Looms", "Trace", "Changes", "Skills"];
const TRACKS: &[(&str, &[&str])] = &[
    ("product", &["spec", "design", "preview"]),
    ("engineering", &["data", "system", "api"]),
    ("build", &["tests", "build", "validate"]),
];

struct App<'a> {
    root: &'a Path,
    cfg: &'a Config,
    tab: usize,
    sel: ListState,
    state: State,
    views: Vec<View>,
    trace: Option<trace::Trace>,
    changes: Vec<Change>,
    skills: Vec<(String, String, bool, String)>,
    msg: Option<(String, bool)>,
    confirm: Option<String>,
}

impl<'a> App<'a> {
    fn new(root: &'a Path, cfg: &'a Config) -> Self {
        let mut a = App {
            root,
            cfg,
            tab: 0,
            sel: ListState::default(),
            state: State::default(),
            views: vec![],
            trace: None,
            changes: vec![],
            skills: vec![],
            msg: None,
            confirm: None,
        };
        a.refresh();
        let first = a
            .views
            .iter()
            .position(|v| {
                matches!(
                    v,
                    View::Stale | View::Review | View::InProgress | View::Ready
                )
            })
            .unwrap_or(0);
        a.sel.select(Some(first));
        a
    }

    fn refresh(&mut self) {
        self.state = State::load(self.root).unwrap_or_default();
        self.views = LOOMS
            .iter()
            .map(|l| self.state.view(self.root, self.cfg, l))
            .collect();
        self.trace = trace::build(self.root, &self.cfg.docs.content_dir, None).ok();
        self.changes = change::list(self.root).unwrap_or_default();
        if let Ok(cat) = catalog::load() {
            let chosen: Vec<String> = skills::selected(&cat, self.cfg)
                .iter()
                .map(|s| s.name.clone())
                .collect();
            self.skills = cat
                .skill
                .iter()
                .map(|s| {
                    (
                        s.pack.clone(),
                        s.name.clone(),
                        chosen.contains(&s.name),
                        s.source(),
                    )
                })
                .collect();
        }
    }

    fn selected(&self) -> &'static state::Loom {
        &LOOMS[self.sel.selected().unwrap_or(0).min(LOOMS.len() - 1)]
    }

    fn act(&mut self, key: char) {
        let l = self.selected();
        let res = match key {
            's' => loom::start(self.root, self.cfg, l.id, false).map(|p| {
                format!(
                    "{} started · context pack {} (~{} tokens)",
                    l.id, p.path, p.tokens
                )
            }),
            'e' => {
                let opts = loom::EndOpts {
                    commit: true,
                    ..Default::default()
                };
                loom::end(self.root, self.cfg, l.id, opts).map(|r| {
                    r.commit
                        .map(|m| format!("committed: {m}"))
                        .unwrap_or_else(|| format!("{} in review", l.id))
                })
            }
            'a' => loom::approve_and_commit(self.root, self.cfg, l.id)
                .map(|by| format!("{} approved by {by}", l.id)),
            _ => return,
        };
        self.msg = Some(match res {
            Ok(m) => (m, true),
            Err(e) => (format!("{e:#}").replace('\n', " "), false),
        });
        self.refresh();
    }
}

pub fn run(root: &Path, cfg: &Config) -> Result<()> {
    let mut terminal = ratatui::init();
    let res = event_loop(&mut terminal, App::new(root, cfg));
    ratatui::restore();
    res
}

fn event_loop(t: &mut DefaultTerminal, mut app: App) -> Result<()> {
    loop {
        t.draw(|f| draw(f, &mut app))?;
        let Event::Key(k) = event::read()? else {
            continue;
        };
        if k.kind != KeyEventKind::Press {
            continue;
        }
        if let Some(action) = app.confirm.take() {
            if matches!(k.code, KeyCode::Char('y') | KeyCode::Enter) {
                app.act(action.chars().next().unwrap_or(' '));
            } else {
                app.msg = Some(("cancelled".into(), true));
            }
            continue;
        }
        match k.code {
            KeyCode::Char('q') | KeyCode::Esc => return Ok(()),
            KeyCode::Char('c') if k.modifiers.contains(KeyModifiers::CONTROL) => return Ok(()),
            KeyCode::Tab | KeyCode::Right => app.tab = (app.tab + 1) % TABS.len(),
            KeyCode::BackTab | KeyCode::Left => app.tab = (app.tab + TABS.len() - 1) % TABS.len(),
            KeyCode::Char(c @ '1'..='4') => app.tab = c as usize - '1' as usize,
            KeyCode::Down | KeyCode::Char('j') => app.sel.select(Some(
                (app.sel.selected().unwrap_or(0) + 1).min(LOOMS.len() - 1),
            )),
            KeyCode::Up | KeyCode::Char('k') => app
                .sel
                .select(Some(app.sel.selected().unwrap_or(0).saturating_sub(1))),
            KeyCode::Char('r') => {
                app.refresh();
                app.msg = Some(("refreshed".into(), true));
            }
            KeyCode::Char(c @ ('s' | 'e' | 'a')) if app.tab == 0 => {
                let verb = match c {
                    's' => "Start",
                    'e' => "End and commit",
                    _ => "Record human approval of",
                };
                app.confirm = Some(format!("{c}{verb} the {} loom? [y/N]", app.selected().id));
            }
            _ => {}
        }
    }
}

fn draw(f: &mut Frame, app: &mut App) {
    let [head, pipe, tabs, body, foot] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(5),
        Constraint::Length(1),
        Constraint::Min(8),
        Constraint::Length(2),
    ])
    .areas(f.area());

    let c = app.cfg;
    f.render_widget(
        Line::from(vec![
            Span::styled(
                " sixloom ",
                Style::new()
                    .fg(ratatui::style::Color::Black)
                    .bg(ACCENT)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!("  {}  ", c.project.name),
                Style::new().add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                format!(
                    "{} · {} · docs {} · design {} · agents {}",
                    c.frontend.framework,
                    c.frontend.ui_kit,
                    c.docs.engine,
                    c.design.tool,
                    c.agents.harnesses.join(",")
                ),
                Style::new().fg(MUTED),
            ),
        ]),
        head,
    );
    draw_pipeline(f, app, pipe);
    f.render_widget(
        Tabs::new(
            TABS.iter()
                .enumerate()
                .map(|(i, t)| format!("{} {t}", i + 1)),
        )
        .select(app.tab)
        .highlight_style(
            Style::new()
                .fg(ACCENT)
                .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        )
        .style(Style::new().fg(MUTED)),
        tabs,
    );
    match app.tab {
        0 => draw_looms(f, app, body),
        1 => draw_trace(f, app, body),
        2 => draw_changes(f, app, body),
        _ => draw_skills(f, app, body),
    }
    let [msg, k] = Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(foot);
    if let Some((m, ok)) = &app.msg {
        f.render_widget(
            Line::styled(format!(" {m}"), Style::new().fg(if *ok { OK } else { BAD })),
            msg,
        );
    }
    f.render_widget(
        keys(&[
            ("↑↓", "loom"),
            ("←→ 1-4", "tab"),
            ("s", "start"),
            ("e", "end"),
            ("a", "approve"),
            ("r", "refresh"),
            ("q", "quit"),
        ]),
        k,
    );
    if let Some(q) = &app.confirm {
        let area = centered(f.area(), 56, 5);
        f.render_widget(Clear, area);
        f.render_widget(
            Paragraph::new(Line::from(q[1..].to_string()))
                .wrap(Wrap { trim: true })
                .block(
                    Block::bordered()
                        .border_type(BorderType::Rounded)
                        .border_style(Style::new().fg(ACCENT))
                        .title(title("confirm")),
                ),
            area,
        );
    }
}

fn draw_pipeline(f: &mut Frame, app: &App, area: Rect) {
    let mut lines = vec![];
    for (name, ids) in TRACKS {
        let mut spans = vec![Span::styled(format!(" {name:<12}"), Style::new().fg(MUTED))];
        for (i, id) in ids.iter().enumerate() {
            let idx = LOOMS.iter().position(|l| l.id == *id).unwrap();
            let v = app.views[idx];
            let mut st = view_style(v);
            if app.sel.selected() == Some(idx) {
                st = st.add_modifier(Modifier::REVERSED);
            }
            spans.push(Span::styled(format!(" {} {:<9}", v.glyph(), id), st));
            if i + 1 < ids.len() {
                spans.push(Span::styled(" ─▶", Style::new().fg(MUTED)));
            }
        }
        lines.push(Line::from(spans));
    }
    f.render_widget(
        Paragraph::new(lines).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(MUTED))
                .title(title("pipeline")),
        ),
        area,
    );
}

fn draw_looms(f: &mut Frame, app: &mut App, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Length(30), Constraint::Min(30)]).areas(area);
    let items: Vec<ListItem> = LOOMS
        .iter()
        .zip(&app.views)
        .map(|(l, v)| {
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {} ", v.glyph()), view_style(*v)),
                Span::raw(format!("{:<14}", l.title)),
                Span::styled(v.label(), view_style(*v)),
            ]))
        })
        .collect();
    let list = List::new(items)
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(MUTED))
                .title(title("looms")),
        )
        .highlight_style(
            Style::new()
                .bg(ratatui::style::Color::Rgb(40, 40, 52))
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("▌");
    f.render_stateful_widget(list, left, &mut app.sel);

    let l = app.selected();
    let idx = app.sel.selected().unwrap_or(0);
    let v = app.views[idx];
    let s = app.state.get(l.id);
    let mut lines = vec![
        Line::from(vec![
            Span::styled(l.title, Style::new().add_modifier(Modifier::BOLD)),
            Span::styled(format!("   {}", v.label()), view_style(v)),
        ]),
        Line::styled(
            format!("skill {} · reviewer {}", l.skill, l.reviewer),
            Style::new().fg(MUTED),
        ),
        Line::raw(""),
    ];
    match v {
        View::Blocked => lines.push(Line::styled(
            format!("Waiting for: {}", app.state.deps_approved(l).join(", ")),
            Style::new().fg(MUTED),
        )),
        View::Stale => {
            let ch = state::changed_inputs(app.root, app.cfg, l, &app.state);
            lines.push(Line::styled(
                format!("Inputs changed since approval: {}", ch.join(", ")),
                Style::new().fg(WARN),
            ));
            lines.push(Line::styled(
                "Update only what changed, then end and approve again.",
                Style::new().fg(WARN),
            ));
        }
        View::Approved => lines.push(Line::styled(
            format!(
                "Approved by {} at {}",
                s.approved_by.unwrap_or_default(),
                s.approved_at.unwrap_or_default().get(..16).unwrap_or("")
            ),
            Style::new().fg(OK),
        )),
        _ => {}
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Context pack (reads only)",
        Style::new().fg(ACCENT),
    ));
    if l.inputs.is_empty() {
        lines.push(Line::styled(
            "  the user's own material",
            Style::new().fg(MUTED),
        ));
    }
    for i in l.inputs {
        let p = state::expand(i, app.cfg);
        let there = !state::files_under(app.root, &p).is_empty();
        lines.push(Line::from(vec![
            Span::styled(
                if there { "  ✓ " } else { "  · " },
                Style::new().fg(if there { OK } else { MUTED }),
            ),
            Span::raw(p),
        ]));
    }
    lines.push(Line::styled("Produces", Style::new().fg(ACCENT)));
    for o in l.outputs {
        lines.push(Line::raw(format!("  {}", state::expand(o, app.cfg))));
    }
    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "Ask your agent (new session)",
        Style::new().fg(ACCENT),
    ));
    for inv in loom::invocations(app.cfg, l.skill) {
        lines.push(Line::raw(format!("  {inv}")));
    }
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: false }).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(MUTED))
                .title(title("detail")),
        ),
        right,
    );
}

fn draw_trace(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(MUTED))
        .title(title("trace"));
    let Some(t) = &app.trace else {
        f.render_widget(
            Paragraph::new("No spec/SPEC.md yet. Run the spec loom first.")
                .style(Style::new().fg(MUTED))
                .block(block),
            area,
        );
        return;
    };
    let cell = |v: &Vec<String>| {
        if v.is_empty() {
            Cell::from("—").style(Style::new().fg(MUTED))
        } else {
            Cell::from(v.len().to_string()).style(Style::new().fg(OK))
        }
    };
    let rows: Vec<Row> = t
        .rows
        .iter()
        .map(|r| {
            Row::new(vec![
                Cell::from(r.ac.clone()),
                Cell::from(r.priority.clone()),
                Cell::from(r.screens.join(",")),
                cell(&r.ui),
                Cell::from(r.operations.join(",")),
                cell(&r.docs),
                cell(&r.tests),
            ])
        })
        .collect();
    let [tbl, gaps] = Layout::vertical([
        Constraint::Min(4),
        Constraint::Length((t.gaps.len() as u16 + 2).min(8)),
    ])
    .areas(area);
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(12),
                Constraint::Length(7),
                Constraint::Length(14),
                Constraint::Length(4),
                Constraint::Min(16),
                Constraint::Length(5),
                Constraint::Length(6),
            ],
        )
        .header(
            Row::new(["AC", "prio", "screens", "ui", "operations", "docs", "tests"])
                .style(Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        )
        .block(block),
        tbl,
    );
    let g: Vec<Line> = if t.gaps.is_empty() {
        vec![Line::styled(" no gaps", Style::new().fg(OK))]
    } else {
        t.gaps
            .iter()
            .map(|g| Line::styled(format!(" ! {g}"), Style::new().fg(WARN)))
            .collect()
    };
    f.render_widget(
        Paragraph::new(g).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(MUTED))
                .title(title("gaps")),
        ),
        gaps,
    );
}

fn draw_changes(f: &mut Frame, app: &App, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(MUTED))
        .title(title("changelog stack"));
    if app.changes.is_empty() {
        f.render_widget(
            Paragraph::new("No change records. Feedback goes through the sxl-patch skill.")
                .style(Style::new().fg(MUTED))
                .block(block),
            area,
        );
        return;
    }
    let rows = app.changes.iter().map(|c| {
        let st = match c.status.as_str() {
            "built" => Style::new().fg(OK),
            "rejected" => Style::new().fg(MUTED),
            "proposed" => Style::new().fg(WARN),
            _ => Style::new().fg(super::INFO),
        };
        Row::new(vec![
            Cell::from(c.id.clone()),
            Cell::from(c.status.clone()).style(st),
            Cell::from(c.kind.clone()),
            Cell::from(c.title.clone()),
            Cell::from(c.affects.clone()).style(Style::new().fg(MUTED)),
        ])
    });
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(8),
                Constraint::Length(10),
                Constraint::Length(9),
                Constraint::Min(20),
                Constraint::Length(24),
            ],
        )
        .header(
            Row::new(["ID", "status", "kind", "title", "affects"])
                .style(Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        )
        .block(block),
        area,
    );
}

fn draw_skills(f: &mut Frame, app: &App, area: Rect) {
    let rows = app.skills.iter().map(|(pack, name, on, src)| {
        Row::new(vec![
            Cell::from(if *on { "●" } else { "○" }).style(Style::new().fg(if *on {
                OK
            } else {
                MUTED
            })),
            Cell::from(pack.clone()).style(Style::new().fg(MUTED)),
            Cell::from(name.clone()),
            Cell::from(src.clone()).style(Style::new().fg(MUTED)),
        ])
    });
    f.render_widget(
        Table::new(
            rows,
            [
                Constraint::Length(2),
                Constraint::Length(12),
                Constraint::Length(30),
                Constraint::Min(20),
            ],
        )
        .header(
            Row::new(["", "pack", "skill", "source"])
                .style(Style::new().fg(ACCENT).add_modifier(Modifier::BOLD)),
        )
        .block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(MUTED))
                .title(title("skills  ● installed  ○ available"))
                .title_bottom(Line::styled(
                    " add your own: sixloom skills add <dir> ",
                    Style::new().fg(MUTED),
                )),
        ),
        area,
    );
}

pub fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect {
        x: area.x + (area.width - w) / 2,
        y: area.y + (area.height - h) / 2,
        width: w,
        height: h,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    #[test]
    fn renders_every_tab() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let mut cfg = crate::init::default_config("demo");
        cfg.agents.harnesses = vec!["claude".into(), "codex".into()];
        crate::init::run(root, &cfg, crate::init::Mode::Adopt, false).unwrap();
        std::fs::write(
            root.join("spec/SPEC.md"),
            "### F-001 A\n- **F-001.AC1** Given x, then y.\n",
        )
        .unwrap();
        let mut app = App::new(root, &cfg);
        let mut t = Terminal::new(TestBackend::new(120, 40)).unwrap();
        for tab in 0..TABS.len() {
            app.tab = tab;
            t.draw(|f| draw(f, &mut app)).unwrap();
            let text: String = t
                .backend()
                .buffer()
                .content()
                .iter()
                .map(|c| c.symbol())
                .collect();
            assert!(text.contains("pipeline"));
            match tab {
                0 => assert!(text.contains("/sxl-specify") && text.contains("$sxl-specify")),
                1 => assert!(text.contains("F-001.AC1")),
                2 => assert!(text.contains("No change records")),
                _ => assert!(text.contains("sxl-tdd")),
            }
        }
        app.confirm = Some("sStart the spec loom? [y/N]".into());
        t.draw(|f| draw(f, &mut app)).unwrap();
        // Small terminals must not panic.
        let mut small = Terminal::new(TestBackend::new(40, 12)).unwrap();
        small.draw(|f| draw(f, &mut app)).unwrap();
    }
}
