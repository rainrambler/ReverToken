use std::path::Path;
use std::process::{Command, Output};

use serde_json::Value;

fn fixture(name: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
        .to_string_lossy()
        .into_owned()
}

fn fixtures_dir() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .to_string_lossy()
        .into_owned()
}

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_revertoken"))
        .args(args)
        .output()
        .expect("failed to run revertoken")
}

fn stdout_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr_str(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn scans_elf_and_demangles_itanium_names() {
    let output = run(&[&fixture("libdemo.so")]);
    assert!(output.status.success(), "{}", stderr_str(&output));

    let stdout = stdout_str(&output);
    assert!(stdout.contains("ELF64"), "{stdout}");
    assert!(stdout.contains("_ZN4demo3addEii"), "{stdout}");
    assert!(stdout.contains("demo::add(int, int)"), "{stdout}");
    assert!(stdout.contains("plain_c_export"), "{stdout}");
    assert!(stdout.contains("2 exports"), "{stdout}");
}

#[test]
fn scans_pe_and_demangles_msvc_names() {
    let output = run(&[&fixture("demo.dll")]);
    assert!(output.status.success(), "{}", stderr_str(&output));

    let stdout = stdout_str(&output);
    assert!(stdout.contains("PE32+"), "{stdout}");
    assert!(stdout.contains("?add@demo@@YAHHH@Z"), "{stdout}");
    assert!(stdout.contains("demo::add(int, int)"), "{stdout}");
    assert!(stdout.contains("plain_c_export"), "{stdout}");
}

#[test]
fn scans_macho_dylib() {
    let output = run(&[&fixture("libdemo.dylib")]);
    assert!(output.status.success(), "{}", stderr_str(&output));

    let stdout = stdout_str(&output);
    assert!(stdout.contains("Mach-O 64"), "{stdout}");
    assert!(stdout.contains("__Z8demo_addii"), "{stdout}");
    assert!(stdout.contains("demo_add(int, int)"), "{stdout}");
}

#[test]
fn search_matches_raw_and_demangled_names_case_insensitively() {
    let output = run(&[&fixture("libdemo.so"), "-s", "DEMO::ADD"]);
    assert!(output.status.success(), "{}", stderr_str(&output));

    let stdout = stdout_str(&output);
    assert!(stdout.contains("demo::add(int, int)"), "{stdout}");
    assert!(!stdout.contains("plain_c_export"), "{stdout}");
    assert!(stdout.contains("1 match"), "{stdout}");

    let output = run(&[
        &fixture("libdemo.so"),
        "-s",
        "DEMO::ADD",
        "--case-sensitive",
    ]);
    let stdout = stdout_str(&output);
    assert!(!stdout.contains("demo::add(int, int)"), "{stdout}");
    assert!(stdout.contains("0 matched"), "{stdout}");
}

#[test]
fn search_match_all_requires_every_keyword() {
    let output = run(&[
        &fixture("libdemo.so"),
        "-s",
        "demo",
        "-s",
        "add",
        "--match-all",
    ]);
    let stdout = stdout_str(&output);
    assert!(stdout.contains("demo::add(int, int)"), "{stdout}");

    let output = run(&[
        &fixture("libdemo.so"),
        "-s",
        "demo",
        "-s",
        "missing",
        "--match-all",
    ]);
    let stdout = stdout_str(&output);
    assert!(!stdout.contains("demo::add(int, int)"), "{stdout}");
    assert!(stdout.contains("0 matched"), "{stdout}");
}

#[test]
fn no_demangle_keeps_raw_names_only() {
    let output = run(&[&fixture("demo.dll"), "--no-demangle"]);
    assert!(output.status.success(), "{}", stderr_str(&output));

    let stdout = stdout_str(&output);
    assert!(stdout.contains("?add@demo@@YAHHH@Z"), "{stdout}");
    assert!(!stdout.contains("int __cdecl demo::add"), "{stdout}");
}

#[test]
fn json_output_contains_metadata_demangled_names_and_ordinals() {
    let output = run(&[&fixture("demo.dll"), "--json"]);
    assert!(output.status.success(), "{}", stderr_str(&output));

    let value: Value = serde_json::from_slice(&output.stdout).expect("valid JSON");
    assert_eq!(value["scanned_binaries"], 1);
    assert_eq!(value["total_exports"], 2);
    assert_eq!(value["files"][0]["format"], "pe");
    assert_eq!(value["files"][0]["arch"], "X86_64");

    let exports = value["files"][0]["exports"].as_array().unwrap();
    assert_eq!(exports.len(), 2);
    let add = exports
        .iter()
        .find(|export| export["ordinal"] == 1)
        .expect("ordinal 1 export");
    assert_eq!(add["name"], "?add@demo@@YAHHH@Z");
    assert_eq!(add["kind"], "function");
    assert!(add["demangled"].as_str().unwrap().contains("demo::add"));
}

#[test]
fn scans_a_directory_and_reports_every_format() {
    let output = run(&[&fixtures_dir()]);
    assert!(output.status.success(), "{}", stderr_str(&output));

    let stdout = stdout_str(&output);
    assert!(stdout.contains("demo.dll"), "{stdout}");
    assert!(stdout.contains("libdemo.so"), "{stdout}");
    assert!(stdout.contains("libdemo.dylib"), "{stdout}");
    assert!(stdout.contains("Scanned 3 binaries"), "{stdout}");
}

#[test]
fn explicit_unsupported_file_is_an_error() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml");
    let output = run(&[manifest.to_str().unwrap()]);
    assert!(!output.status.success());
    assert!(
        stderr_str(&output).contains("not a supported PE, ELF, or Mach-O"),
        "{}",
        stderr_str(&output)
    );
}

#[test]
fn empty_search_keyword_is_rejected_by_cli() {
    let output = run(&[&fixture("libdemo.so"), "-s", ""]);
    assert!(!output.status.success());
    assert!(
        stderr_str(&output).contains("value") || stderr_str(&output).contains("empty"),
        "{}",
        stderr_str(&output)
    );
}
