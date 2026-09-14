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
        tui::run(built.review, built.stores)?;
        return Ok(0);
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
            },
        ),
    };
    print!("{out}");
    Ok(code)
}
