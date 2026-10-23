mod app;
mod demangle;
mod formats;
mod model;
mod output;
mod scan;

use std::io;
use std::path::PathBuf;
use std::process::ExitCode;

use clap::Parser;

/// Extract export tables from executables and shared libraries.
///
/// Scans a PE, ELF, or Mach-O binary (or a directory tree containing them),
/// prints each export with its raw and demangled C++ name, and optionally
/// filters the table by keyword.
#[derive(Debug, Parser)]
#[command(
    name = "revertoken",
    version,
    about = "Cross-platform executable/shared-library export table scanner",
    after_help = "EXAMPLES:\n  \
        revertoken ./bin\n  \
        revertoken C:\\Windows\\System32\\kernel32.dll -s CreateFile\n  \
        revertoken /usr/lib/libssl.so --search SSL_CTX --case-sensitive\n  \
        revertoken ./plugins --search crypto --search tls --match-all\n  \
        revertoken libfoo.so --json"
)]
struct Cli {
    /// File or directory to scan
    #[arg(value_name = "PATH")]
    path: PathBuf,

    /// Search keyword in raw and demangled names (repeatable; OR by default)
    #[arg(
        short,
        long,
        value_name = "TEXT",
        action = clap::ArgAction::Append,
        value_parser = clap::builder::NonEmptyStringValueParser::new()
    )]
    search: Vec<String>,

    /// Require all --search keywords to match (default: any keyword)
    #[arg(long, requires = "search")]
    match_all: bool,

    /// Match keywords case-sensitively (default: case-insensitive)
    #[arg(short = 'c', long)]
    case_sensitive: bool,

    /// Keep raw mangled names; do not demangle C++ symbols
    #[arg(long)]
    no_demangle: bool,

    /// Print machine-readable JSON instead of a table
    #[arg(long)]
    json: bool,

    /// Follow symbolic links while walking a directory
    #[arg(short = 'L', long)]
    follow_links: bool,

    /// Show per-file parse and traversal warnings
    #[arg(short, long)]
    verbose: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let config = app::Config {
        path: cli.path,
        search: cli.search,
        match_all: cli.match_all,
        case_sensitive: cli.case_sensitive,
        no_demangle: cli.no_demangle,
        json: cli.json,
        follow_links: cli.follow_links,
        verbose: cli.verbose,
    };

    match app::run(config) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) if is_broken_pipe(&error) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("error: {error:#}");
            ExitCode::FAILURE
        }
    }
}

fn is_broken_pipe(error: &anyhow::Error) -> bool {
    error
        .chain()
        .filter_map(|cause| cause.downcast_ref::<io::Error>())
        .any(|error| error.kind() == io::ErrorKind::BrokenPipe)
}
