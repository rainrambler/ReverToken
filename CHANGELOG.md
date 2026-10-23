# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- GitHub Actions CI for Linux, Windows, and macOS, including an MSRV check.
- Release workflow for tagged builds on Linux, Windows, and macOS.
- English and Simplified Chinese documentation.
- Contribution, security, and code-of-conduct policies.

## [0.1.0] - 2026-10-23

### Added

- Initial release of the `revertoken` command-line tool.
- Recursive file and directory scanning with format detection by magic bytes.
- PE32/PE32+ export directory parsing, including ordinals, RVAs, virtual
  addresses, forwarders, and function/data classification.
- ELF32/ELF64 dynamic symbol (`.dynsym`) export parsing.
- Mach-O export trie parsing, including universal (fat) binaries.
- Itanium C++ demangling via `cpp_demangle`.
- MSVC C++ demangling via `msvc-demangler`.
- Case-insensitive keyword search over raw and demangled names, with repeatable
  keywords and optional AND matching.
- Human-readable tables and JSON output.
- Integration tests with small PE, ELF, and Mach-O fixtures.
