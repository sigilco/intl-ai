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
    /// Per-locale inventory counts (read-only; cheap enough for agents to
    /// call before deciding what to do).
    Status(StatusArgs),
    /// Import a 0.4.x intl-ai.lock.json into shards.
    #[command(hide = true)]
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
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
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
    #[arg(long, conflicts_with = "regenerate")]
    stale: bool,
    /// Regenerate AI-owned existing values. Destructive: needs a --keys
    /// scope, --keys-file, or --yes to confirm a whole-locale rewrite.
    #[arg(long)]
    regenerate: bool,
    /// Confirm a whole-locale --regenerate (no key scope given).
    #[arg(long)]
    yes: bool,
    /// Skip the stat-cache read/write for this run.
    #[arg(long)]
    no_cache: bool,
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
    /// Key globs (repeatable or comma-separated), e.g. "auth.*".
    #[arg(long, value_delimiter = ',')]
    keys: Vec<String>,
    /// File with one key glob per line.
    #[arg(long, value_name = "PATH")]
    keys_file: Option<PathBuf>,
    /// Scope stale/modified/unreviewed findings to one effective origin.
    #[arg(long, value_enum)]
    origin: Option<OriginArg>,
    /// Finding kinds that fail the run (comma-separated). Default: config
    /// check.fail_on; `--fail-on none` clears the gate entirely.
    #[arg(long, value_delimiter = ',')]
    fail_on: Vec<String>,
    /// Skip the stat-cache read/write for this run.
    #[arg(long)]
    no_cache: bool,
    /// Output format for the findings report.
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
    /// Run each configured spec check's self_test fixtures instead of
    /// checking locales (plan 5.2).
    #[arg(long)]
    self_test: bool,
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
    /// Re-arming a tombstoned key clears its absent flag.
    #[arg(long, value_enum, required_unless_present_any = ["absent", "present"])]
    origin: Option<OriginArg>,
    /// Tombstone the keys: deliberately untranslated — fill skips them,
    /// check doesn't report them missing. Mutually exclusive with --origin.
    #[arg(long, conflicts_with = "origin")]
    absent: bool,
    /// Clear a tombstone (key becomes an ordinary missing key again).
    #[arg(long, conflicts_with = "absent")]
    present: bool,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
}

#[derive(Args)]
struct SpecArgs {
    /// Key globs to affect, e.g. "auth.*".
    #[arg(required = true)]
    spec: Vec<String>,
    /// Target locale(s) (required; repeatable or comma-separated).
    #[arg(long, value_delimiter = ',', required = true)]
    locale: Vec<String>,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
}

#[derive(Args)]
struct LockfileArgs {
    #[command(subcommand)]
    command: LockfileCommand,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
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
    /// Print the JSON Schema for the config contract (same types
    /// `config validate` checks against; the schema file is also
    /// committed at docs/public/schema/intl-ai.schema.json).
    Schema,
}

#[derive(Args)]
struct StatusArgs {
    /// Target locales (repeatable or comma-separated). Default: config targets.
    #[arg(long, value_delimiter = ',')]
    locale: Vec<String>,
    /// Output format.
    #[arg(long, value_enum, default_value_t = OutFormat::Human)]
    format: OutFormat,
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
        Command::Status(args) => commands::status::run(&cli, args),
        Command::Migrate => commands::migrate::run(),
    };
    match result {
        Ok(code) => ExitCode::from(code),
        Err(e) => {
            let code = exit_code(&e);
            if wants_json(&cli.command) {
                // JSON error envelope: agent-facing consumers get the same
                // shape on failure as on success (M7).
                println!(
                    "{}",
                    serde_json::json!({"error": {"message": format!("{e:#}"), "code": code}})
                );
            } else {
                eprintln!("intl-ai: {e:#}");
            }
            ExitCode::from(code)
        }
    }
}

fn wants_json(cmd: &Command) -> bool {
    let f = match cmd {
        Command::Init(a) => a.format,
        Command::Fill(a) => a.format,
        Command::Check(a) => a.format,
        Command::Mark(a) => a.format,
        Command::Review(a) | Command::Unreview(a) => a.format,
        Command::Lockfile(a) => a.format,
        Command::Config(a) => match &a.command {
            ConfigCommand::Validate(v) => v.format,
            ConfigCommand::Schema => OutFormat::Human,
        },
        Command::Status(a) => a.format,
        Command::Migrate => OutFormat::Human,
    };
    matches!(f, OutFormat::Json)
}
