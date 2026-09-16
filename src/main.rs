use clap::{Args, Parser, Subcommand, ValueEnum};
use openspec_reviewer::build::{attach_state, build_review};
use openspec_reviewer::render::{json, markdown, text, tui};
use openspec_reviewer::source::{ChangeSource, DiffSource, GhSource, GitSource, Source};
use std::io::IsTerminal;
use std::path::PathBuf;
use std::process::ExitCode;

/// Review an OpenSpec change as the semantic diff it is.
#[derive(Parser)]
#[command(name = "openspec-reviewer", version, arg_required_else_help = true)]
struct Cli {
    #[command(flatten)]
    output: OutputFlags,
    #[command(subcommand)]
    source: Option<Command>,
}

#[derive(Args, Clone, Copy)]
struct OutputFlags {
    /// Plain text even when stdout is a terminal.
    #[arg(long, global = true)]
    plain: bool,
    /// Output format; any value implies plain output.
    #[arg(long, global = true, value_enum)]
    format: Option<Format>,
    /// Only the findings and the summary, one finding per line.
    #[arg(long, global = true)]
    findings_only: bool,
    /// ANSI colour in plain text output.
    #[arg(long, global = true)]
    color: bool,
    /// Neither read nor write approvals and notes.
    #[arg(long, global = true)]
    no_state: bool,
    /// Ask the configured agent for hints on every pairing before printing.
    #[arg(long, global = true)]
    assist: bool,
    /// Include hints in --findings-only output.
    #[arg(long, global = true)]
    hints: bool,
}

#[derive(Clone, Copy, PartialEq, Eq, ValueEnum)]
enum Format {
    Text,
    Json,
    Markdown,
}

#[derive(Subcommand)]
enum Command {
    /// The change as it is in the working tree.
    Change { name: String },
    /// A unified diff from a file, or stdin when the path is `-` or absent.
    Diff { path: Option<PathBuf> },
    /// The files of <REF> against --base, which defaults to main, then master.
    Git {
        #[arg(value_name = "REF")]
        reference: String,
        #[arg(long, value_name = "REF")]
        base: Option<String>,
    },
    /// A GitHub pull request by number or URL through the gh CLI.
    Gh { pr: String },
    /// Check every citation in specs and source against the repository.
    Lint {
        /// Print the per-requirement ledger of citing source files.
        #[arg(long)]
        coverage: bool,
        #[command(subcommand)]
        action: Option<LintAction>,
    },
    /// Agent skills that call the reviewer: install them or list their state.
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
    },
    /// The agent the reviewer hands a pairing to.
    Assist {
        #[command(subcommand)]
        action: AssistAction,
    },
}

#[derive(Subcommand)]
enum AssistAction {
    /// Write the built-in prompt templates to openspec/reviewer/prompts/.
    Prompts,
}

#[derive(Subcommand)]
enum SkillsAction {
    /// Write the skills into .claude/skills/, and .agents/skills/ when it exists.
    Install,
    /// Name every skill and command with whether it is installed, up to date or edited.
    List,
}

#[derive(Subcommand)]
enum LintAction {
    /// Write openspec/reviewer.toml from what the repository contains.
    Init,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("openspec-reviewer: {e}");
            ExitCode::from(2)
        }
    }
}

fn run(cli: Cli) -> Result<u8, Box<dyn std::error::Error>> {
    let root = std::env::current_dir()?;
    let Some(command) = cli.source else {
        return Ok(2);
    };
    if let Command::Lint { coverage, action } = command {
        return match action {
            Some(LintAction::Init) => {
                let path = openspec_reviewer::citations::write_init(
                    &root,
                    &openspec_reviewer::source::survey(&root),
                )?;
                println!("{}", path.display());
                Ok(0)
            }
            None => run_lint(&root, coverage, cli.output.format),
        };
    }
    if let Command::Skills { action } = command {
        return run_skills(&root, action);
    }
    if let Command::Assist { action } = command {
        return run_assist(&root, action);
    }
    let source: Box<dyn Source> = match command {
        Command::Change { name } => Box::new(ChangeSource {
            root: root.clone(),
            name,
        }),
        Command::Diff { path } => Box::new(DiffSource {
            root: root.clone(),
            path,
        }),
        Command::Git { reference, base } => Box::new(GitSource {
            root: root.clone(),
            reference,
            base,
        }),
        Command::Gh { pr } => Box::new(GhSource {
            root: root.clone(),
            pr,
        }),
        Command::Lint { .. } | Command::Skills { .. } | Command::Assist { .. } => {
            unreachable!("handled above")
        }
    };
    let snapshot = source.fetch()?;
    let review = build_review(&root, &snapshot)?;
    let state_home = if cli.output.no_state {
        None
    } else {
        openspec_reviewer::state::state_home()
    };
    let mut built = attach_state(&root, review, state_home.as_deref())?;
    let session = assist_session(&root, state_home.as_deref(), &built.review);

    let interactive = std::io::stdout().is_terminal()
        && !cli.output.plain
        && cli.output.format.is_none()
        && !cli.output.findings_only;
    if interactive {
        tui::run(built.review, built.stores, Some(session))?;
        return Ok(0);
    }

    if cli.output.assist {
        batch_review(&session, &mut built)?;
    }

    let code = built.review.summary.exit_code() as u8;
    let out = match cli.output.format.unwrap_or(Format::Text) {
        Format::Json => json::render(&built.review),
        Format::Markdown => markdown::render(&built.review),
        Format::Text => text::render(
            &built.review,
            text::TextOptions {
                colour: cli.output.color,
                findings_only: cli.output.findings_only,
                hints: cli.output.hints,
            },
        ),
    };
    print!("{out}");
    Ok(code)
}

fn run_lint(
    root: &std::path::Path,
    coverage: bool,
    format: Option<Format>,
) -> Result<u8, Box<dyn std::error::Error>> {
    use openspec_reviewer::citations::{coverage::coverage as ledger, lint, require_config};
    use openspec_reviewer::render::lint as render;
    use openspec_reviewer::source::Workspace;
    use std::io::Write;

    let config = require_config(root)?;
    let workspace = Workspace::load(root, &config.lint)?;
    let report = lint(&workspace.input(), &config)?;
    let ignored = config.ignored_requirements();
    let ledger = coverage.then(|| ledger(&report.index, &report.register, &ignored));
    match format {
        Some(Format::Json) => print!("{}", render::render_json(&report, ledger.as_ref())),
        _ => {
            let out = render::render_text(&report, ledger.as_ref());
            std::io::stderr().write_all(out.stderr.as_bytes())?;
            print!("{}", out.stdout);
        }
    }
    Ok(report.exit_code() as u8)
}

/// `--assist`: one agent call per pairing without a valid cache, in list
/// order, before anything prints. Sequential, like the view's batch run.
fn batch_review(
    session: &openspec_reviewer::assist::Session,
    built: &mut openspec_reviewer::build::Built,
) -> Result<(), Box<dyn std::error::Error>> {
    let agent = session.assistant()?;
    for change in &mut built.review.changes {
        let Some(store) = built.stores.get_mut(&change.name) else {
            continue;
        };
        for p in change.pairings_mut() {
            let hash = p.text_hash();
            if !p.hints.is_empty() {
                continue;
            }
            let hints = session.review_pairing(agent.as_ref(), p)?;
            let state = store.set_hints(&p.key(), hash, hints)?;
            p.hints = state.hints_for(hash);
            p.state = state;
        }
    }
    built.review.recount();
    Ok(())
}

/// Prompt files live beside the approvals, under the state directory, so
/// they survive the run and travel with nothing. Without a state home they
/// go to a temporary directory.
fn assist_session(
    root: &std::path::Path,
    state_home: Option<&std::path::Path>,
    review: &openspec_reviewer::review::Review,
) -> openspec_reviewer::assist::Session {
    let prompt_dir = match state_home {
        Some(home) => home
            .join("openspec-reviewer")
            .join(openspec_reviewer::source::repo_key(root))
            .join("prompts"),
        None => std::env::temp_dir().join("openspec-reviewer-prompts"),
    };
    openspec_reviewer::assist::Session::open(root, prompt_dir)
        .with_glossary(review.glossary.clone())
}

fn run_assist(
    root: &std::path::Path,
    action: AssistAction,
) -> Result<u8, Box<dyn std::error::Error>> {
    match action {
        AssistAction::Prompts => {
            let export = openspec_reviewer::assist::export_prompts(root)?;
            for path in &export.written {
                println!("wrote {path}");
            }
            for path in &export.skipped {
                println!("kept  {path} (already there)");
            }
        }
    }
    Ok(0)
}

fn run_skills(
    root: &std::path::Path,
    action: SkillsAction,
) -> Result<u8, Box<dyn std::error::Error>> {
    use openspec_reviewer::skills::{FileKind, FileState};
    use openspec_reviewer::source::skills;

    match action {
        SkillsAction::Install => {
            for p in skills::install(root)? {
                match p.state {
                    FileState::Edited => {
                        println!(
                            "kept  {} (edited by hand; differs from what the tool wrote)",
                            p.path
                        )
                    }
                    _ => println!("wrote {}", p.path),
                }
            }
        }
        SkillsAction::List => {
            for p in skills::listed(root)? {
                let kind = match p.kind {
                    FileKind::Skill => "skill",
                    FileKind::Command => "command",
                };
                let first_sentence = p
                    .skill
                    .description
                    .split_once(". ")
                    .map_or(p.skill.description, |(head, _)| head);
                println!(
                    "{kind:<7} {:<14} {}\n        {first_sentence}",
                    p.state, p.path
                );
            }
        }
    }
    Ok(0)
}
