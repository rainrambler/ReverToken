use std::borrow::Cow;
use std::io::Write;

use anyhow::Result;
use serde::Serialize;

use crate::model::{BinaryReport, ExportSymbol, FileFormat, SearchQuery};

#[derive(Debug, Clone, Copy, Default)]
pub struct Stats {
    pub binaries: usize,
    pub total_exports: usize,
    pub matched_exports: usize,
}

pub fn write_human<W: Write>(
    writer: &mut W,
    reports: &[BinaryReport],
    query: &SearchQuery,
) -> Result<Stats> {
    let mut stats = Stats {
        binaries: reports.len(),
        ..Stats::default()
    };
    let mut wrote_file = false;

    for report in reports {
        stats.total_exports += report.exports.len();

        let selected: Vec<&ExportSymbol> = report
            .exports
            .iter()
            .filter(|symbol| symbol.matches(query))
            .collect();
        stats.matched_exports += selected.len();

        // In search mode, behave like grep: do not print files without a hit.
        if query.is_active() && selected.is_empty() {
            continue;
        }

        if wrote_file {
            writeln!(writer)?;
        }
        wrote_file = true;

        let path = printable(&report.path);
        writeln!(writer, "{path}")?;

        let mut facts = vec![
            report.format.display_name(report.bits),
            report.arch.clone(),
            report.endianness.label().to_owned(),
            report.kind.clone(),
        ];
        if query.is_active() {
            facts.push(format!(
                "{} {}, {} {}",
                format_count(report.exports.len()),
                plural(report.exports.len(), "export", "exports"),
                format_count(selected.len()),
                plural(selected.len(), "match", "matches")
            ));
        } else {
            facts.push(format!(
                "{} {}",
                format_count(report.exports.len()),
                plural(report.exports.len(), "export", "exports")
            ));
        }
        writeln!(writer, "  {}", facts.join(" | "))?;

        if selected.is_empty() {
            writeln!(writer, "  (no exports)")?;
            continue;
        }

        write_table(writer, &selected)?;
    }

    if wrote_file {
        writeln!(writer)?;
    }

    if stats.binaries == 0 {
        writeln!(
            writer,
            "No supported PE/ELF/Mach-O executables or libraries found."
        )?;
    } else if query.is_active() {
        writeln!(
            writer,
            "Scanned {} {}, {} {}; {} matched.",
            format_count(stats.binaries),
            plural(stats.binaries, "binary", "binaries"),
            format_count(stats.total_exports),
            plural(stats.total_exports, "export", "exports"),
            format_count(stats.matched_exports)
        )?;
    } else {
        writeln!(
            writer,
            "Scanned {} {}, {} {}.",
            format_count(stats.binaries),
            plural(stats.binaries, "binary", "binaries"),
            format_count(stats.total_exports),
            plural(stats.total_exports, "export", "exports")
        )?;
    }

    Ok(stats)
}

fn write_table<W: Write>(writer: &mut W, symbols: &[&ExportSymbol]) -> Result<()> {
    struct Row {
        address: String,
        ordinal: String,
        kind: &'static str,
        name: String,
        mangled: String,
        note: String,
    }

    let show_mangled = symbols.iter().any(|symbol| symbol.demangled.is_some());
    let show_note = symbols.iter().any(|symbol| symbol.forwarder.is_some());

    let rows: Vec<Row> = symbols
        .iter()
        .map(|symbol| Row {
            address: if symbol.forwarder.is_some() {
                "-".to_owned()
            } else {
                format!("0x{:016x}", symbol.address)
            },
            ordinal: symbol
                .ordinal
                .map(|ordinal| ordinal.to_string())
                .unwrap_or_else(|| "-".to_owned()),
            kind: symbol.kind.label(),
            name: printable(symbol.display_name()).into_owned(),
            mangled: if show_mangled && symbol.demangled.is_some() {
                printable(&symbol.name).into_owned()
            } else {
                String::new()
            },
            note: symbol
                .forwarder
                .as_deref()
                .map(|target| format!("-> {}", printable(target)))
                .unwrap_or_default(),
        })
        .collect();

    let ord_width = rows
        .iter()
        .map(|row| row.ordinal.chars().count())
        .max()
        .unwrap_or(1)
        .max("ORD".len());
    let name_width = rows
        .iter()
        .map(|row| row.name.chars().count())
        .max()
        .unwrap_or(1)
        .max("NAME".len());
    let mangled_width = rows
        .iter()
        .map(|row| row.mangled.chars().count())
        .max()
        .unwrap_or(1)
        .max("MANGLED".len());

    let mut header = format!(
        "  {:<18}  {:>ord_width$}  {:<4}  {:<name_width$}",
        "ADDRESS", "ORD", "TYPE", "NAME"
    );
    let mut rule = format!(
        "  {:<18}  {:>ord_width$}  {:<4}  {:<name_width$}",
        "-".repeat(18),
        "-".repeat(ord_width),
        "----",
        "-".repeat(name_width)
    );
    if show_mangled {
        header.push_str(&format!("  {:<mangled_width$}", "MANGLED"));
        rule.push_str(&format!("  {:<mangled_width$}", "-".repeat(mangled_width)));
    }
    if show_note {
        header.push_str("  NOTE");
        rule.push_str("  ----");
    }
    writeln!(writer, "{}", header.trim_end())?;
    writeln!(writer, "{}", rule.trim_end())?;

    for row in &rows {
        let mut line = format!(
            "  {:<18}  {:>ord_width$}  {:<4}  {:<name_width$}",
            row.address, row.ordinal, row.kind, row.name
        );
        if show_mangled {
            line.push_str(&format!("  {:<mangled_width$}", row.mangled));
        }
        if show_note {
            line.push_str("  ");
            line.push_str(&row.note);
        }
        writeln!(writer, "{}", line.trim_end())?;
    }

    Ok(())
}

#[derive(Serialize)]
struct JsonReport<'a> {
    scanned_binaries: usize,
    total_exports: usize,
    matched_exports: usize,
    files: Vec<JsonFile<'a>>,
}

#[derive(Serialize)]
struct JsonFile<'a> {
    path: &'a str,
    format: FileFormat,
    arch: &'a str,
    bits: u8,
    endianness: crate::model::Endianness,
    kind: &'a str,
    export_count: usize,
    exports: Vec<&'a ExportSymbol>,
}

pub fn write_json<W: Write>(
    writer: &mut W,
    reports: &[BinaryReport],
    query: &SearchQuery,
) -> Result<Stats> {
    let total_exports = reports.iter().map(|report| report.exports.len()).sum();
    let mut matched_exports = 0;

    let files = reports
        .iter()
        .map(|report| {
            let exports: Vec<&ExportSymbol> = report
                .exports
                .iter()
                .filter(|symbol| symbol.matches(query))
                .collect();
            matched_exports += exports.len();

            JsonFile {
                path: &report.path,
                format: report.format,
                arch: &report.arch,
                bits: report.bits,
                endianness: report.endianness,
                kind: &report.kind,
                export_count: report.exports.len(),
                exports,
            }
        })
        .collect();

    let stats = Stats {
        binaries: reports.len(),
        total_exports,
        matched_exports,
    };
    let report = JsonReport {
        scanned_binaries: stats.binaries,
        total_exports: stats.total_exports,
        matched_exports: stats.matched_exports,
        files,
    };

    let json = serde_json::to_string_pretty(&report)?;
    writeln!(writer, "{json}")?;
    Ok(stats)
}

fn plural<'a>(count: usize, singular: &'a str, plural: &'a str) -> &'a str {
    if count == 1 { singular } else { plural }
}

#[allow(clippy::manual_is_multiple_of)]
fn format_count(value: usize) -> String {
    let digits = value.to_string();
    let mut result = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, ch) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index) % 3 == 0 {
            result.push(',');
        }
        result.push(ch);
    }
    result
}

/// Keep terminal control characters from a malicious binary from corrupting
/// the output (or the user's terminal).
fn printable(value: &str) -> Cow<'_, str> {
    if !value.chars().any(char::is_control) {
        return Cow::Borrowed(value);
    }

    let mut output = String::with_capacity(value.len() + 8);
    for ch in value.chars() {
        match ch {
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            ch if ch.is_control() => {
                use std::fmt::Write as _;
                let _ = write!(output, "\\u{{{:x}}}", ch as u32);
            }
            ch => output.push(ch),
        }
    }
    Cow::Owned(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_numbers_with_commas() {
        assert_eq!(format_count(0), "0");
        assert_eq!(format_count(999), "999");
        assert_eq!(format_count(1_000), "1,000");
        assert_eq!(format_count(1_234_567), "1,234,567");
    }

    #[test]
    fn escapes_control_characters() {
        assert_eq!(printable("normal"), "normal");
        assert_eq!(printable("evil\nname"), "evil\\nname");
        assert_eq!(printable("esc\u{1b}[31m"), "esc\\u{1b}[31m");
    }
}
