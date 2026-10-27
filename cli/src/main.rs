mod catalog;
mod change;
mod config;
mod harness;
mod init;
mod lock;
mod loom;
mod skills;
mod state;
mod trace;
mod tui;
mod ui;

use anyhow::{bail, Context, Result};
use clap::{Args, Parser, Subcommand};
use std::io::IsTerminal;
use std::path::{Path, PathBuf};

use config::Config;
use ui::{bold, dim, green, red, yellow};

#[derive(Parser)]
#[command(
    name = "sixloom",
    version,
    about = "Spec → design → preview → data → system → API → tests → build: a loom workflow for coding agents",
    long_about = "sixloom turns a confirmed spec into a working product with coding agents.\n\
                  Humans review the things that are easy to review (spec, preview, data model, API docs);\n\
                  agents build the rest test-first. Run with no arguments in a terminal for the dashboard."
)]
struct Cli {
    /// Project directory (defaults to the nearest parent with sixloom.toml)
    #[arg(short = 'C', long, global = true, value_name = "DIR")]
    dir: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Option<Cmd>,
}

#[derive(Subcommand)]
enum Cmd {
    /// Create a new project, or add sixloom to an existing one (--adopt)
    Init(InitArgs),
    /// Show every loom's status (approved, stale, ready, …)
    Status {
        #[arg(long)]
        json: bool,
    },
    /// What to do next, with the exact skill invocation for each of your agents
    Next,
    /// Start, end or check one loom
    #[command(subcommand)]
    Loom(LoomCmd),
    /// Record human approval of a loom (`approve preview`) or a change (`approve change CH-0003`)
    Approve {
        /// Loom ID, or `change`
        target: String,
        /// Change ID when target is `change`
        id: Option<String>,
    },
    /// Change records: the changelog stack under spec/changes/
    #[command(subcommand)]
    Change(ChangeCmd),
    /// Feature → screen → UI → operation → docs → test matrix, with gaps
    Trace {
        /// Filter by feature or AC prefix, e.g. F-003
        id: Option<String>,
        #[arg(long)]
        json: bool,
        /// Exit 1 when there are gaps (for CI)
        #[arg(long)]
        strict: bool,
    },
    /// Lock or verify the acceptance tests
    #[command(subcommand)]
    Tests(TestsCmd),
    /// List, sync or add skills
    #[command(subcommand)]
    Skills(SkillsCmd),
    /// Re-install skills and instruction blocks after editing sixloom.toml or upgrading sixloom
    Update,
    /// Supported coding agents and whether they are installed
    Agents,
    /// Check tools, skills and project state
    Doctor,
    /// Check the done conditions; --confirm records your sign-off
    Done {
        #[arg(long)]
        confirm: bool,
    },
    /// Open the interactive dashboard
    Tui,
}

#[derive(Args, Default)]
struct InitArgs {
    /// Directory to create or adopt (default: current directory)
    path: Option<PathBuf>,
    /// Add sixloom to an existing codebase without touching application code
    #[arg(long)]
    adopt: bool,
    #[arg(long)]
    name: Option<String>,
    /// Comma-separated: claude,codex,opencode,gemini,kimi,antigravity,cursor,copilot,amp
    #[arg(long, value_delimiter = ',')]
    agents: Vec<String>,
    /// react-vite | nextjs | vue | nuxt | sveltekit
    #[arg(long)]
    frontend: Option<String>,
    /// shadcn | none
    #[arg(long)]
    ui_kit: Option<String>,
    /// fumadocs | docusaurus | starlight
    #[arg(long)]
    docs: Option<String>,
    /// claude | stitch | none
    #[arg(long)]
    design: Option<String>,
    /// key | all | none
    #[arg(long)]
    fidelity: Option<String>,
    /// Put the non-selected skills in an other-skill bank (.otherskill/bank)
    #[arg(long)]
    bank: bool,
    /// Accept defaults for anything not given; never open the wizard
    #[arg(short, long)]
    yes: bool,
    #[arg(long)]
    force: bool,
}

#[derive(Subcommand)]
enum LoomCmd {
    /// Check prerequisites, write the context pack, mark in progress
    Start {
        id: String,
        #[arg(long)]
        force: bool,
    },
    /// Run machine checks, mark in review, commit `sxl(<loom>): …`
    End {
        id: String,
        #[arg(short, long)]
        message: Option<String>,
        #[arg(long)]
        no_commit: bool,
        /// Also record approval (when the human already said yes to the gate)
        #[arg(long)]
        approve: bool,
        #[arg(long)]
        skip_checks: bool,
    },
    /// Run the machine checks only
    Check { id: String },
    /// Print the loom's context pack file list
    Pack { id: String },
}

#[derive(Subcommand)]
enum ChangeCmd {
    /// Create spec/changes/CH-xxxx-<slug>.md
    New {
        title: String,
        #[arg(long, default_value = "feedback")]
        kind: String,
        /// Affected IDs, comma-separated
        #[arg(long, value_delimiter = ',')]
        affects: Vec<String>,
    },
    List,
    /// Set a change's status (proposed, accepted, specced, tested, built, rejected)
    Status {
        id: String,
        status: String,
    },
    /// Regenerate spec/CHANGELOG.md
    Log,
}

#[derive(Subcommand)]
enum TestsCmd {
    /// Fingerprint tests/e2e, tests/contract, tests/integration into .sixloom/tests.lock
    Lock,
    /// Fail if a locked test was removed, disabled or had its assertions changed
    Verify,
}

#[derive(Subcommand)]
enum SkillsCmd {
    /// Every skill in the catalog, and which ones this project installs
    List,
    /// Re-install the selected skills into every agent's skills directory
    Sync,
    /// Add your own skill (a directory with SKILL.md) for every configured agent
    Add { path: PathBuf },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("{} {e:#}", red("error:"));
        std::process::exit(1);
    }
}

fn root_of(dir: &Option<PathBuf>) -> Result<PathBuf> {
    let start = match dir {
        Some(d) => d.clone(),
        None => std::env::current_dir()?,
    };
    config::find_root(&start).with_context(|| {
        format!(
            "no sixloom.toml at or above {}; run `sixloom init`",
            start.display()
        )
    })
}

fn project(dir: &Option<PathBuf>) -> Result<(PathBuf, Config)> {
    let root = root_of(dir)?;
    let cfg = Config::load(&root)?;
    Ok((root, cfg))
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let tty = std::io::stdout().is_terminal() && std::io::stdin().is_terminal();
    let Some(cmd) = cli.cmd else {
        if tty {
            if let Ok((root, cfg)) = project(&cli.dir) {
                return tui::dashboard::run(&root, &cfg);
            }
            println!(
                "No sixloom project here. Start one with {}.\n",
                bold("sixloom init")
            );
        }
        use clap::CommandFactory;
        Cli::command().print_help()?;
        return Ok(());
    };
    match cmd {
        Cmd::Init(a) => cmd_init(a, cli.dir, tty),
        Cmd::Status { json } => cmd_status(&cli.dir, json),
        Cmd::Next => cmd_next(&cli.dir),
        Cmd::Loom(l) => cmd_loom(&cli.dir, l),
        Cmd::Approve { target, id } => {
            let (root, cfg) = project(&cli.dir)?;
            if target == "change" {
                let id = id.context("usage: sixloom approve change CH-0001")?;
                let c = loom::approve_change(&root, &id)?;
                println!("{} {} accepted: {}", green("✓"), c.id, c.title);
                println!("  next: apply it with the sxl-patch skill (spec → looms → tests → code)");
            } else {
                let by = loom::approve(&root, &cfg, &target)?;
                println!("{} {target} approved by {by}", green("✓"));
                print_next(&root, &cfg)?;
            }
            Ok(())
        }
        Cmd::Change(c) => cmd_change(&cli.dir, c),
        Cmd::Trace { id, json, strict } => cmd_trace(&cli.dir, id, json, strict),
        Cmd::Tests(t) => {
            let root = root_of(&cli.dir)?;
            match t {
                TestsCmd::Lock => {
                    let (n, warnings) = lock::lock(&root)?;
                    for w in warnings {
                        println!("{} {w}", yellow("!"));
                    }
                    println!("{} locked {n} tests into {}", green("✓"), lock::LOCK);
                }
                TestsCmd::Verify => {
                    let problems = lock::verify(&root)?;
                    if problems.is_empty() {
                        println!("{} locked tests intact", green("✓"));
                    } else {
                        for p in &problems {
                            println!("{} {p}", red("✗"));
                        }
                        bail!(
                            "{} locked-test violation(s); see the sxl-tdd rules",
                            problems.len()
                        );
                    }
                }
            }
            Ok(())
        }
        Cmd::Skills(s) => cmd_skills(&cli.dir, s),
        Cmd::Update => {
            let (root, cfg) = project(&cli.dir)?;
            let rep = skills::sync(&root, &cfg)?;
            let files = init::write_instructions(&root, &cfg)?;
            print_skill_report(&rep);
            println!("{} refreshed {}", green("✓"), files.join(", "));
            Ok(())
        }
        Cmd::Agents => {
            println!("{}", bold("Supported coding agents"));
            for h in harness::ALL {
                let mark = if h.on_path() {
                    green("installed")
                } else {
                    dim("not found")
                };
                let v = if h.verified {
                    ""
                } else {
                    "  (paths from docs, not smoke-tested)"
                };
                println!(
                    "  {:<12} {:<20} skills: {:<16} {}{}",
                    h.id,
                    h.name,
                    h.skills_dir,
                    mark,
                    dim(v)
                );
            }
            Ok(())
        }
        Cmd::Doctor => cmd_doctor(&cli.dir),
        Cmd::Done { confirm } => {
            let (root, cfg) = project(&cli.dir)?;
            let checks = loom::done_checks(&root, &cfg)?;
            for c in &checks {
                let mark = if c.ok { green("✓") } else { red("✗") };
                println!(
                    "{mark} {}{}",
                    c.label,
                    if c.detail.is_empty() || c.ok {
                        String::new()
                    } else {
                        dim(&format!("\n    {}", c.detail))
                    }
                );
            }
            println!(
                "\n{}",
                bold("Also confirm yourself (from the validate report):")
            );
            for q in DONE_HUMAN {
                println!("  ☐ {q}");
            }
            if confirm {
                let by = loom::mark_done(&root, &cfg)?;
                println!("\n{} done, signed off by {by}. New feedback starts the next cycle with sxl-patch.", green("✓"));
            } else if checks.iter().all(|c| c.ok) {
                println!(
                    "\nAll machine checks pass. When the list above is true, run {}.",
                    bold("sixloom done --confirm")
                );
            }
            Ok(())
        }
        Cmd::Tui => {
            let (root, cfg) = project(&cli.dir)?;
            tui::dashboard::run(&root, &cfg)
        }
    }
}

pub const DONE_HUMAN: &[&str] = &[
    "CI is green on the main branch for the release commit",
    "Preview vs real screenshots match on every screen state (within tolerance)",
    "No serious accessibility violations (axe) and no console errors in real mode",
    "Docs 'Getting started' works on a clean checkout",
    "You clicked through every key flow in the real app and accept it",
];

fn cmd_init(a: InitArgs, dir: Option<PathBuf>, tty: bool) -> Result<()> {
    let base = dir.unwrap_or(std::env::current_dir()?);
    let path = match &a.path {
        Some(p) if p.is_absolute() => p.clone(),
        Some(p) => base.join(p),
        None => base,
    };
    let name = a.name.clone().unwrap_or_else(|| {
        path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "app".into())
    });
    let mut cfg = init::default_config(&name);
    let interactive = tty && !a.yes;
    let adopt = a.adopt || (!a.yes && path.join(".git").exists() && a.path.is_none());
    apply_flags(&mut cfg, &a)?;
    let mode = if adopt {
        init::Mode::Adopt
    } else {
        init::Mode::New
    };
    let (cfg, mode) = if interactive {
        match tui::wizard::run(cfg, mode, &path)? {
            Some(x) => x,
            None => {
                println!("Cancelled.");
                return Ok(());
            }
        }
    } else {
        (cfg, mode)
    };
    let out = init::run(&path, &cfg, mode, a.force)?;
    println!("{} sixloom initialised in {}", green("✓"), path.display());
    println!("  {}", dim(&format!("created {}", out.created.join(", "))));
    print_skill_report(&out.skills);
    println!();
    print_next(&path, &cfg)?;
    Ok(())
}

fn apply_flags(cfg: &mut Config, a: &InitArgs) -> Result<()> {
    if !a.agents.is_empty() {
        for h in &a.agents {
            harness::get(h)?;
        }
        cfg.agents.harnesses = a.agents.clone();
    }
    if let Some(v) = &a.frontend {
        cfg.frontend.framework = v.clone();
        if !matches!(v.as_str(), "react-vite" | "nextjs") && a.ui_kit.is_none() {
            cfg.frontend.ui_kit = "none".into();
        }
    }
    if let Some(v) = &a.ui_kit {
        cfg.frontend.ui_kit = v.clone();
    }
    if let Some(v) = &a.docs {
        cfg.docs.engine = v.clone();
        cfg.docs.content_dir = config::content_dir_for(v).into();
    }
    if let Some(v) = &a.design {
        cfg.design.tool = v.clone();
    }
    if let Some(v) = &a.fidelity {
        if !["key", "all", "none"].contains(&v.as_str()) {
            bail!("fidelity must be key, all or none");
        }
        cfg.design.fidelity = v.clone();
    }
    cfg.skills.bank = a.bank;
    cfg.validate()
}

fn print_skill_report(r: &skills::Report) {
    println!(
        "{} {} skills installed into {}",
        green("✓"),
        r.native.len(),
        r.dirs.join(", ")
    );
    if !r.banked.is_empty() {
        println!(
            "  {} more in the other-skill bank (.otherskill/bank)",
            r.banked.len()
        );
    }
    if !r.removed.is_empty() {
        println!("  removed {}", r.removed.join(", "));
    }
}

fn print_next(root: &Path, cfg: &Config) -> Result<()> {
    let next = loom::next(root, cfg)?;
    let Some((l, v)) = next.first() else {
        println!(
            "{}",
            bold("Nothing pending. Run `sixloom done` to check the done conditions.")
        );
        return Ok(());
    };
    let what = match v {
        state::View::Stale => "update the stale",
        state::View::Review => "get approval for the",
        state::View::InProgress => "continue the",
        _ => "start the",
    };
    println!(
        "{} {what} {} loom ({}) in a new agent session:",
        bold("Next:"),
        l.id,
        l.title
    );
    for line in loom::invocations(cfg, l.skill) {
        println!("    {line}");
    }
    if next.len() > 1 {
        let rest: Vec<_> = next[1..]
            .iter()
            .map(|(l, v)| format!("{} ({})", l.id, v.label()))
            .collect();
        println!("  {}", dim(&format!("also open: {}", rest.join(", "))));
    }
    Ok(())
}

fn cmd_status(dir: &Option<PathBuf>, json: bool) -> Result<()> {
    let (root, cfg) = project(dir)?;
    let st = state::State::load(&root)?;
    if json {
        let v: Vec<_> = state::LOOMS
            .iter()
            .map(|l| {
                let s = st.get(l.id);
                serde_json::json!({
                    "loom": l.id, "title": l.title, "skill": l.skill, "reviewer": l.reviewer,
                    "status": st.view(&root, &cfg, l).label(),
                    "approved_by": s.approved_by, "approved_at": s.approved_at,
                    "stale_inputs": state::changed_inputs(&root, &cfg, l, &st),
                })
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&v)?);
        return Ok(());
    }
    println!(
        "{}  {}",
        bold(&cfg.project.name),
        dim(&format!(
            "{} · {} · {} · design: {}",
            cfg.frontend.framework, cfg.frontend.ui_kit, cfg.docs.engine, cfg.design.tool
        ))
    );
    let track = |ids: &[&str]| -> String {
        ids.iter()
            .map(|id| {
                let l = state::loom(id).unwrap();
                ui::view_color(
                    st.view(&root, &cfg, l),
                    &format!("{} {}", st.view(&root, &cfg, l).glyph(), id),
                )
            })
            .collect::<Vec<_>>()
            .join(&dim(" → "))
    };
    println!("  product      {}", track(&["spec", "design", "preview"]));
    println!("  engineering  {}", track(&["data", "system", "api"]));
    println!(
        "  build        {}\n",
        track(&["tests", "build", "validate"])
    );
    for l in state::LOOMS {
        let v = st.view(&root, &cfg, l);
        let s = st.get(l.id);
        let extra = match v {
            state::View::Approved => s.approved_by.map(|b| format!("by {b}")).unwrap_or_default(),
            state::View::Stale => format!(
                "changed: {}",
                state::changed_inputs(&root, &cfg, l, &st).join(", ")
            ),
            state::View::Blocked => format!("needs {}", st.deps_approved(l).join(", ")),
            _ => String::new(),
        };
        println!(
            "  {}  {:<9} {:<14} {:<18} {}",
            ui::view_color(v, v.glyph()),
            l.id,
            l.title,
            ui::view_color(v, v.label()),
            dim(&extra)
        );
    }
    println!();
    print_next(&root, &cfg)
}

fn cmd_next(dir: &Option<PathBuf>) -> Result<()> {
    let (root, cfg) = project(dir)?;
    print_next(&root, &cfg)
}

fn cmd_loom(dir: &Option<PathBuf>, c: LoomCmd) -> Result<()> {
    let (root, cfg) = project(dir)?;
    match c {
        LoomCmd::Start { id, force } => {
            let pack = loom::start(&root, &cfg, &id, force)?;
            let l = state::loom(&id)?;
            println!(
                "{} {} loom started ({}; reviewer: {})",
                green("◐"),
                l.title,
                l.skill,
                l.reviewer
            );
            println!(
                "\n{} {} (~{} tokens). Read only these:",
                bold("Context pack:"),
                pack.path,
                pack.tokens
            );
            for (f, t) in &pack.files {
                println!("  {f} {}", dim(&format!("~{t}")));
            }
            if pack.files.is_empty() {
                println!(
                    "  {}",
                    dim("(no inputs: this loom starts from the user's own material)")
                );
            }
            println!(
                "\nWhen the gate checklist is all yes: {}",
                bold(&format!("sixloom loom end {id} --approve"))
            );
        }
        LoomCmd::End {
            id,
            message,
            no_commit,
            approve,
            skip_checks,
        } => {
            let msg = loom::end(
                &root,
                &cfg,
                &id,
                message.as_deref(),
                !no_commit,
                skip_checks,
            )?;
            match msg {
                Some(m) => println!("{} committed: {m}", green("✓")),
                None => println!(
                    "{} {id} in review {}",
                    green("✓"),
                    dim("(nothing committed)")
                ),
            }
            if approve {
                let by = loom::approve(&root, &cfg, &id)?;
                println!("{} approved by {by}", green("✓"));
                print_next(&root, &cfg)?;
            } else {
                println!(
                    "After the human review: {}",
                    bold(&format!("sixloom approve {id}"))
                );
            }
        }
        LoomCmd::Check { id } => {
            let p = loom::check(&root, &cfg, &id)?;
            if p.is_empty() {
                println!("{} {id}: all machine checks pass", green("✓"));
            } else {
                for x in &p {
                    println!("{} {x}", red("✗"));
                }
                bail!("{} problem(s)", p.len());
            }
        }
        LoomCmd::Pack { id } => {
            let pack = loom::write_pack(&root, &cfg, state::loom(&id)?)?;
            println!("{} (~{} tokens)", pack.path, pack.tokens);
        }
    }
    Ok(())
}

fn cmd_change(dir: &Option<PathBuf>, c: ChangeCmd) -> Result<()> {
    let root = root_of(dir)?;
    match c {
        ChangeCmd::New {
            title,
            kind,
            affects,
        } => {
            let p = change::create(&root, &title, &kind, &affects)?;
            change::render_log(&root)?;
            println!(
                "{} {}",
                green("✓"),
                p.strip_prefix(&root).unwrap_or(&p).display()
            );
            println!("  fill in Source and Proposal, then: sixloom approve change <id>  (bugs: set status accepted)");
        }
        ChangeCmd::List => {
            let all = change::list(&root)?;
            if all.is_empty() {
                println!("{}", dim("No change records yet."));
            }
            for c in all {
                println!(
                    "  {:<8} {:<10} {:<9} {}  {}",
                    c.id,
                    c.status,
                    c.kind,
                    c.title,
                    dim(&c.affects)
                );
            }
        }
        ChangeCmd::Status { id, status } => {
            let c = change::set_status(&root, &id, &status, None)?;
            change::render_log(&root)?;
            println!("{} {} → {}", green("✓"), c.id, c.status);
        }
        ChangeCmd::Log => println!(
            "{} {}",
            green("✓"),
            change::render_log(&root)?.strip_prefix(&root)?.display()
        ),
    }
    Ok(())
}

fn cmd_trace(dir: &Option<PathBuf>, id: Option<String>, json: bool, strict: bool) -> Result<()> {
    let (root, cfg) = project(dir)?;
    let t = trace::build(&root, &cfg.docs.content_dir, id.as_deref())?;
    if json {
        println!("{}", serde_json::to_string_pretty(&t)?);
    } else {
        println!(
            "{}",
            bold(&format!(
                "{:<11} {:<6} {:<10} {:<6} {:<16} {:<6} {}",
                "AC", "prio", "screens", "ui", "operations", "docs", "tests"
            ))
        );
        let n = |v: &Vec<String>| {
            if v.is_empty() {
                red("—")
            } else {
                v.len().to_string()
            }
        };
        for r in &t.rows {
            let ops = if r.operations.is_empty() {
                dim("—")
            } else {
                r.operations.join(",")
            };
            println!(
                "{:<11} {:<6} {:<10} {:<6} {:<16} {:<6} {}",
                r.ac,
                r.priority,
                if r.screens.is_empty() {
                    red("—")
                } else {
                    r.screens.join(",")
                },
                n(&r.ui),
                ops,
                n(&r.docs),
                n(&r.tests)
            );
        }
        if t.gaps.is_empty() {
            println!("\n{} no gaps", green("✓"));
        } else {
            println!("\n{}", bold("Gaps"));
            for g in &t.gaps {
                println!("  {} {g}", yellow("!"));
            }
        }
    }
    if strict && !t.gaps.is_empty() {
        bail!("{} trace gap(s)", t.gaps.len());
    }
    Ok(())
}

fn cmd_skills(dir: &Option<PathBuf>, s: SkillsCmd) -> Result<()> {
    match s {
        SkillsCmd::List => {
            let cat = catalog::load()?;
            let proj = project(dir).ok();
            let chosen: Vec<String> = proj
                .as_ref()
                .map(|(_, cfg)| {
                    skills::selected(&cat, cfg)
                        .iter()
                        .map(|s| s.name.clone())
                        .collect()
                })
                .unwrap_or_default();
            for p in &cat.pack {
                println!(
                    "{} {}",
                    bold(&p.title),
                    dim(&format!("[{}] {}", p.id, p.description))
                );
                for s in cat.skill.iter().filter(|s| s.pack == p.id) {
                    let mark = if chosen.contains(&s.name) {
                        green("●")
                    } else {
                        dim("○")
                    };
                    println!("  {mark} {:<30} {}", s.name, dim(&s.source()));
                }
            }
            if proj.is_some() {
                println!("\n{} installed for this project   {} available (bank, --extra or another choice)", green("●"), dim("○"));
            }
        }
        SkillsCmd::Sync => {
            let (root, cfg) = project(dir)?;
            print_skill_report(&skills::sync(&root, &cfg)?);
        }
        SkillsCmd::Add { path } => {
            let (root, cfg) = project(dir)?;
            let name = skills::add_local(&root, &cfg, &path)?;
            println!(
                "{} added your skill '{name}' for {}",
                green("✓"),
                cfg.agents.harnesses.join(", ")
            );
        }
    }
    Ok(())
}

fn cmd_doctor(dir: &Option<PathBuf>) -> Result<()> {
    let ok = |b: bool| if b { green("✓") } else { yellow("!") };
    println!("{}", bold("Tools"));
    for (bin, why) in [
        ("git", "commits per loom"),
        ("node", "preview, docs, tests"),
        ("just", "task runner used by CI"),
        ("gh", "PRs during the build"),
    ] {
        println!("  {} {:<6} {}", ok(harness::which(bin)), bin, dim(why));
    }
    let Ok((root, cfg)) = project(dir) else {
        println!("\n{} not inside a sixloom project", yellow("!"));
        return Ok(());
    };
    println!("{}", bold("Agents"));
    let cat = catalog::load()?;
    let wanted = skills::selected(&cat, &cfg);
    for id in &cfg.agents.harnesses {
        let h = harness::get(id)?;
        let missing: Vec<_> = wanted
            .iter()
            .filter(|s| {
                !root
                    .join(h.skills_dir)
                    .join(&s.name)
                    .join("SKILL.md")
                    .is_file()
            })
            .map(|s| s.name.as_str())
            .collect();
        let inst = root.join(h.instructions).is_file();
        println!(
            "  {} {:<12} {}{}",
            ok(missing.is_empty() && inst),
            h.id,
            if h.on_path() {
                ""
            } else {
                "binary not on PATH · "
            },
            if missing.is_empty() {
                format!("{} skills in {}", wanted.len(), h.skills_dir)
            } else {
                format!("missing {} (run sixloom update)", missing.join(", "))
            }
        );
    }
    println!("{}", bold("Project"));
    let spec = trace::load_spec(&root);
    match spec {
        Ok(s) if s.problems.is_empty() => {
            println!("  {} spec: {} features", green("✓"), s.features.len())
        }
        Ok(s) => println!("  {} spec: {}", yellow("!"), s.problems.join("; ")),
        Err(_) => println!("  {} spec: not written yet (sxl-specify)", dim("○")),
    }
    if root.join(lock::LOCK).exists() {
        let v = lock::verify(&root)?;
        println!(
            "  {} locked tests {}",
            ok(v.is_empty()),
            if v.is_empty() {
                "intact".into()
            } else {
                v.join("; ")
            }
        );
    }
    Ok(())
}
