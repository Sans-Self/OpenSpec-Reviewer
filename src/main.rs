use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::env::{Bash, Fish, Shells, Zsh};
use clap_complete::{ArgValueCandidates, CompleteEnv, CompletionCandidate};
use openspec_reviewer::build::{attach_state, build_review};
use openspec_reviewer::render::colour::{Background, Palette, PaletteChoice};
use openspec_reviewer::render::terminal::detect_background;
use openspec_reviewer::render::{json, markdown, text, tui};
use openspec_reviewer::source::complete;
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
    /// ANSI colour in plain text output even when stdout is not a terminal.
    #[arg(long, global = true)]
    color: bool,
    /// Neither read nor write approvals and notes.
    #[arg(long, global = true)]
    no_state: bool,
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
    Change {
        #[arg(add = ArgValueCandidates::new(|| here(complete::change_names)))]
        name: String,
    },
    /// A unified diff from a file, or stdin when the path is `-` or absent.
    Diff {
        #[arg(value_hint = clap::ValueHint::FilePath)]
        path: Option<PathBuf>,
    },
    /// The files of <REF> against --base, which defaults to main, then master.
    Git {
        #[arg(value_name = "REF", add = ArgValueCandidates::new(|| here(complete::git_refs)))]
        reference: String,
        #[arg(long, value_name = "REF", add = ArgValueCandidates::new(|| here(complete::git_refs)))]
        base: Option<String>,
    },
    /// A GitHub pull request by number or URL through the gh CLI.
    Gh {
        #[arg(add = ArgValueCandidates::new(|| here(complete::pull_requests)))]
        pr: String,
    },
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

/// Candidates for the directory the shell is in; a lookup that cannot find
/// the directory answers for the relative one, which is the same place.
fn here(lookup: fn(&std::path::Path) -> Vec<CompletionCandidate>) -> Vec<CompletionCandidate> {
    lookup(&std::env::current_dir().unwrap_or_default())
}

fn main() -> ExitCode {
    // Before `parse`, and before anything writes to stdout: with COMPLETE set
    // the process belongs to the shell, not to the user.
    CompleteEnv::with_factory(Cli::command)
        .shells(Shells(&[&Bash, &Zsh, &Fish]))
        .complete();

    let cli = Cli::parse();
    match run(cli) {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("openspec-reviewer: {e}");
            ExitCode::from(2)
        }
    }
}

/// The palette for plain output: the user's choice, resolved against
/// where stdout goes and `--color`, on the background `COLORFGBG` names.
fn plain_palette(choice: PaletteChoice, force: bool) -> Palette {
    let choice = choice.resolve(std::io::stdout().is_terminal(), force);
    let background = match choice {
        PaletteChoice::None => Background::Dark,
        _ => detect_background(false),
    };
    Palette::new(choice, background)
}

fn run(cli: Cli) -> Result<u8, Box<dyn std::error::Error>> {
    let root = std::env::current_dir()?;
    let user = openspec_reviewer::config::user::read_user_config()?;
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
            None => run_lint(
                &root,
                coverage,
                cli.output.format,
                plain_palette(user.palette, cli.output.color),
            ),
        };
    }
    if let Command::Skills { action } = command {
        return run_skills(&root, action);
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
        Command::Lint { .. } | Command::Skills { .. } => unreachable!("handled above"),
    };
    let snapshot = source.fetch()?;
    let review = build_review(&root, &snapshot)?;
    let state_home = if cli.output.no_state {
        None
    } else {
        openspec_reviewer::state::state_home()
    };
    let built = attach_state(&root, review, state_home.as_deref())?;

    let interactive = std::io::stdout().is_terminal()
        && !cli.output.plain
        && cli.output.format.is_none()
        && !cli.output.findings_only;
    if interactive {
        tui::run(&root, built.review, built.stores, user.palette)?;
        return Ok(0);
    }

    let code = built.review.summary.exit_code() as u8;
    let out = match cli.output.format.unwrap_or(Format::Text) {
        Format::Json => json::render(&built.review),
        Format::Markdown => markdown::render(&built.review),
        Format::Text => text::render(
            &built.review,
            text::TextOptions {
                palette: plain_palette(user.palette, cli.output.color),
                findings_only: cli.output.findings_only,
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
    palette: Palette,
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
            let out = render::render_text(&report, ledger.as_ref(), palette);
            std::io::stderr().write_all(out.stderr.as_bytes())?;
            print!("{}", out.stdout);
        }
    }
    Ok(report.exit_code() as u8)
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
