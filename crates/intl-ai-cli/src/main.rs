mod commands;

use clap::{Args, Parser, Subcommand, ValueEnum};
use intl_ai_core::error::Error;
use intl_ai_core::lockfile::Origin;
use std::path::PathBuf;
use std::process::ExitCode;

/// intl-ai: AI-powered build-time i18n translation as a single binary.
#[derive(Parser)]
#[command(name = "intl-ai", version, about)]
struct Cli {
    /// Config file path. '-' reads TOML from stdin.
    #[arg(long, global = true, value_name = "PATH")]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scaffold intl-ai.toml for the current project.
    Init(InitArgs),
    /// Translate missing keys (additive by default; never overwrites
    /// existing values unless --regenerate/--stale say so).
    Fill(FillArgs),
    /// Report missing/stale/modified/unreviewed findings without writing.
    Check(CheckArgs),
    /// Set origin on lockfile entries (escape hatch for the positional rule).
    Mark(MarkArgs),
    /// Set reviewed=true on lockfile entries.
    Review(SpecArgs),
    /// Set reviewed=false on lockfile entries.
    Unreview(SpecArgs),
    /// Lockfile maintenance.
    Lockfile(LockfileArgs),
    /// Config tooling.
    Config(ConfigArgs),
    /// Import a 0.4.x intl-ai.lock.json into shards.
    Migrate,
}

#[derive(Args)]
struct InitArgs {
    /// Locale directory to record in the generated config.
    #[arg(long)]
    locale_dir: Option<PathBuf>,
    /// Source locale. Default: en.
    #[arg(long)]
    source: Option<String>,
    /// Target locales (repeatable or comma-separated).
    #[arg(long, value_delimiter = ',')]
    targets: Vec<String>,
}

#[derive(Args)]
struct FillArgs {
    /// Target locales (repeatable or comma-separated). Default: config targets.
    #[arg(long, value_delimiter = ',')]
    locale: Vec<String>,
    /// Key globs (repeatable or comma-separated), e.g. "auth.*".
    #[arg(long, value_delimiter = ',')]
    keys: Vec<String>,
    /// File with one key glob per line.
    #[arg(long, value_name = "PATH")]
    keys_file: Option<PathBuf>,
    /// Re-fill AI-owned keys whose source hash changed.
    #[arg(long)]
    stale: bool,
    /// Regenerate AI-owned existing values.
    #[arg(long)]
    regenerate: bool,
    /// With --regenerate: also overwrite human-owned values (prints count).
    #[arg(long, requires = "regenerate")]
    include_human: bool,
    /// Compute and report without writing locale files or lockfile shards.
    #[arg(long)]
    dry_run: bool,
    /// Output format for the run report.
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
}

#[derive(Args)]
struct CheckArgs {
    /// Target locales (repeatable or comma-separated). Default: config targets.
    #[arg(long, value_delimiter = ',')]
    locale: Vec<String>,
    /// Scope stale/modified/unreviewed findings to one effective origin.
    #[arg(long, value_enum)]
    origin: Option<OriginArg>,
    /// Finding kinds that fail the run (comma-separated). Default: config
    /// check.fail_on.
    #[arg(long, value_delimiter = ',')]
    fail_on: Vec<String>,
    /// Output format for the findings report.
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
}

#[derive(Args)]
struct MarkArgs {
    /// Key globs to mark, e.g. "auth.*".
    #[arg(required = true)]
    spec: Vec<String>,
    /// Target locale(s) (required; repeatable or comma-separated).
    #[arg(long, value_delimiter = ',', required = true)]
    locale: Vec<String>,
    /// New origin. 'ai' requires an existing entry; 'human' upserts one.
    #[arg(long, value_enum, required = true)]
    origin: OriginArg,
}

#[derive(Args)]
struct SpecArgs {
    /// Key globs to affect, e.g. "auth.*".
    #[arg(required = true)]
    spec: Vec<String>,
    /// Target locale(s) (required; repeatable or comma-separated).
    #[arg(long, value_delimiter = ',', required = true)]
    locale: Vec<String>,
}

#[derive(Args)]
struct LockfileArgs {
    #[command(subcommand)]
    command: LockfileCommand,
}

#[derive(Subcommand)]
enum LockfileCommand {
    /// Validate every shard (parse, version, entry types).
    Check,
    /// Rewrite every shard to canonical form (idempotent).
    Fmt,
    /// Resolve merge conflicts in shards.
    Merge,
}

#[derive(Args)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Subcommand)]
enum ConfigCommand {
    /// Validate the resolved config and print it.
    Validate(ValidateArgs),
}

#[derive(Args)]
struct ValidateArgs {
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OutFormat {
    Human,
    Json,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
enum OriginArg {
    Ai,
    Human,
    All,
}

impl OriginArg {
    fn to_origin(self) -> Option<Origin> {
        match self {
            Self::Ai => Some(Origin::Ai),
            Self::Human => Some(Origin::Human),
            Self::All => None,
        }
    }
}

fn exit_code(err: &anyhow::Error) -> u8 {
    match err.downcast_ref::<Error>() {
        Some(e) if e.is_hard() => 10,
        _ => 10,
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match &cli.command {
        Command::Init(args) => commands::init::run(args),
        Command::Fill(args) => commands::fill::run(&cli, args),
        Command::Check(args) => commands::check::run(&cli, args),
        Command::Mark(args) => commands::spec::mark(&cli, args),
        Command::Review(args) => commands::spec::review(&cli, args, true),
        Command::Unreview(args) => commands::spec::review(&cli, args, false),
        Command::Lockfile(args) => commands::lockfile::run(&cli, args),
        Command::Config(args) => commands::config::run(&cli, args),
        Command::Migrate => commands::migrate::run(),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            eprintln!("intl-ai: {e:#}");
            ExitCode::from(exit_code(&e))
        }
    }
}
