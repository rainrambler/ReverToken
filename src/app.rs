use std::io::{self, BufWriter, Write};
use std::path::PathBuf;

use anyhow::{Context, Result, bail};

use crate::formats;
use crate::model::SearchQuery;
use crate::output;
use crate::scan;

pub struct Config {
    pub path: PathBuf,
    pub search: Vec<String>,
    pub match_all: bool,
    pub case_sensitive: bool,
    pub no_demangle: bool,
    pub json: bool,
    pub follow_links: bool,
    pub verbose: bool,
}

pub fn run(config: Config) -> Result<()> {
    let query = SearchQuery::new(config.search, config.case_sensitive, config.match_all);
    let discovery = scan::discover(&config.path, config.follow_links)?;

    if !discovery.warnings.is_empty() {
        if config.verbose {
            for warning in &discovery.warnings {
                eprintln!("warning: {warning}");
            }
        } else {
            eprintln!(
                "warning: {} path(s) could not be traversed; use --verbose for details",
                discovery.warnings.len()
            );
        }
    }

    let mut reports = Vec::new();
    let mut failed = 0usize;

    for file in &discovery.files {
        match formats::inspect(file, !config.no_demangle) {
            Ok(found) => reports.extend(found),
            Err(error) => {
                if discovery.input_is_file {
                    return Err(error);
                }
                failed += 1;
                if config.verbose {
                    eprintln!("warning: {error:#}");
                }
            }
        }
    }

    if discovery.input_is_file && reports.is_empty() {
        bail!(
            "{} is not a supported PE, ELF, or Mach-O executable/shared library",
            config.path.display()
        );
    }

    if failed > 0 {
        eprintln!(
            "warning: skipped {} file(s) that could not be parsed; use --verbose for details",
            failed
        );
    }

    let stdout = io::stdout();
    let mut writer = BufWriter::new(stdout.lock());

    if config.json {
        output::write_json(&mut writer, &reports, &query).context("failed to write JSON output")?;
    } else {
        output::write_human(&mut writer, &reports, &query)
            .context("failed to write terminal output")?;
    }

    writer.flush().context("failed to flush output")?;
    Ok(())
}
