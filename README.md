# ReverToken

[English](README.md) | [简体中文](README.zh-CN.md)

[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust 1.85+](https://img.shields.io/badge/rust-1.85%2B-orange.svg)](https://www.rust-lang.org)
[![Platforms](https://img.shields.io/badge/platforms-Linux%20%7C%20Windows%20%7C%20macOS-lightgrey.svg)](#supported-formats)

ReverToken is a cross-platform command-line tool for inspecting the **export tables** of
executables and shared libraries. It recursively scans a file or directory, extracts
exported symbols from **PE (EXE/DLL)**, **ELF (executable/`.so`)**, and **Mach-O**
(executable/dylib), demangles C++ names, and lets you search the results by keyword.

## Contents

- [Why ReverToken?](#why-revertoken)
- [Features](#features)
- [Supported formats](#supported-formats)
- [Installation](#installation)
- [Quick start](#quick-start)
- [CLI reference](#cli-reference)
- [Search semantics](#search-semantics)
- [Output](#output)
- [JSON output](#json-output)
- [How it works](#how-it-works)
- [Limitations](#limitations)
- [Development](#development)
- [Contributing](#contributing)
- [Security](#security)
- [License](#license)

## Why ReverToken?

Reverse engineers and developers often need a quick answer to questions such as:

- Which functions does this DLL or `.so` export?
- What is the readable name behind `?add@demo@@YAHHH@Z` or `_ZN4demo3addEii`?
- Which exports mention `crypto`, `SSL`, or `CreateFile`?

ReverToken answers these questions with one command and a stable, machine-readable
JSON mode. It is not a disassembler or decompiler; it focuses on export metadata.

## Features

- **File or directory input** — directories are scanned recursively. Formats are
  detected by magic bytes, not by file extension.
- **Export extraction**
  - PE: Export Directory, including ordinals, RVA/VA, forwarders, and function/data
    classification based on the containing section.
  - ELF: `.dynsym` entries that are defined, exported, and have default/protected
    visibility.
  - Mach-O: export trie, including thin and universal (fat) binaries.
- **C++ demangling**
  - Itanium ABI (GCC/Clang on Linux, MinGW, macOS): `_ZN4demo3addEii` →
    `demo::add(int, int)`
  - MSVC (Windows): `?add@demo@@YAHHH@Z` → `int __cdecl demo::add(int, int)`
  - C symbols and unrecognised names are left unchanged; the raw name is always kept.
- **Keyword search** — matches both raw and demangled names. Case-insensitive by
  default; repeat `-s` to search for multiple keywords.
- **Human and JSON output** — aligned terminal tables or structured JSON for scripts.
- **Terminal-safe** — control characters in symbol names are escaped.
- **Small and portable** — pure Rust, no system toolchain needed at runtime.

## Supported formats

| Format | Variants | Export source | Notes |
| --- | --- | --- | --- |
| PE | PE32, PE32+ (`.exe`, `.dll`) | Export Directory | Named exports, ordinals, forwarders |
| ELF | ELF32, ELF64 (`ET_EXEC`, `ET_DYN`) | `.dynsym` | Exported dynamic symbols only |
| Mach-O | 32/64-bit, thin and fat | Export trie | Executables, dylibs, bundles |

## Installation

### Prebuilt binaries

Tagged releases publish archives for Linux (x86-64), Windows (x86-64), and macOS
(Apple Silicon and Intel) through GitHub Actions. Download the archive for your
platform from the repository's **Releases** page and put `revertoken` (or
`revertoken.exe` on Windows) on your `PATH`.

The Linux archives target glibc 2.35 (Ubuntu 22.04) or newer. The macOS archives
are unsigned and not notarized, so Gatekeeper may require you to allow the binary
in **System Settings → Privacy & Security** on first launch.

### Build from source

Rust **1.85 or newer** is required (the project uses edition 2024).

```sh
git clone <repository-url>
cd <repository-directory>
cargo install --path .
```

Or build without installing:

```sh
cargo build --release
./target/release/revertoken --help
```

## Quick start

```sh
revertoken <PATH> [OPTIONS]
```

Scan a directory and search for `add` in raw and demangled names:

```sh
revertoken ./tests/fixtures -s add
```

```text
tests/fixtures/demo.dll
  PE32+ | X86_64 | LE | DLL | 2 exports, 1 match
  ADDRESS             ORD  TYPE  NAME                             MANGLED
  ------------------  ---  ----  -------------------------------  ------------------
  0x0000000180001000    1  FUNC  int __cdecl demo::add(int, int)  ?add@demo@@YAHHH@Z

tests/fixtures/libdemo.so
  ELF64 | x86_64 | LE | shared library | 2 exports, 1 match
  ADDRESS             ORD  TYPE  NAME                 MANGLED
  ------------------  ---  ----  -------------------  ---------------
  0x00000000000012e0    -  FUNC  demo::add(int, int)  _ZN4demo3addEii

tests/fixtures/libdemo.dylib
  Mach-O 64 | arm64 | LE | dylib | 2 exports, 1 match
  ADDRESS             ORD  TYPE  NAME                MANGLED
  ------------------  ---  ----  ------------------  --------------
  0x0000000000000320    -  SYM   demo_add(int, int)  __Z8demo_addii

Scanned 3 binaries, 6 exports; 3 matched.
```

More examples:

```sh
# Windows
revertoken C:\Windows\System32\kernel32.dll -s CreateFile

# Linux
revertoken /usr/lib/libssl.so -s SSL_CTX

# Require every keyword (AND instead of OR)
revertoken ./plugins -s crypto -s tls --match-all

# Machine-readable output
revertoken ./bin --json > exports.json
```

## CLI reference

| Option | Description |
| --- | --- |
| `<PATH>` | File or directory to scan |
| `-s, --search <TEXT>` | Search keyword; repeatable. Matches raw and demangled names |
| `--match-all` | Require all `--search` keywords to match (default: any) |
| `-c, --case-sensitive` | Match keywords case-sensitively |
| `--no-demangle` | Keep raw mangled names; do not demangle C++ symbols |
| `--json` | Emit JSON instead of a table |
| `-L, --follow-links` | Follow symbolic links while walking a directory |
| `-v, --verbose` | Show per-file parse and traversal warnings |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

Exit codes:

- `0` — scan completed, including when no symbols matched.
- `1` — I/O error, unsupported input file, or a malformed binary.
- `2` — invalid command-line arguments (handled by clap).

## Search semantics

- The query is matched against **both** the raw symbol name and the demangled name.
- Matching is substring-based and **case-insensitive by default**; use
  `--case-sensitive` to change that.
- Multiple `--search` values are **OR** by default. Add `--match-all` to require
  every keyword.
- Searching does not change exit status: no matches still exits `0`.

## Output

Each table contains:

| Column | Meaning |
| --- | --- |
| `ADDRESS` | PE: ImageBase + RVA; ELF: `st_value`; Mach-O: export address. Forwarders show `-` |
| `ORD` | PE export ordinal; `-` for other formats |
| `TYPE` | `FUNC`, `DATA`, or `SYM` when the type is unknown |
| `NAME` | Demangled name when available, otherwise the raw name |
| `MANGLED` | Raw symbol name, shown only when it differs from `NAME` |
| `NOTE` | Forwarder target, when present |

## JSON output

```sh
revertoken tests/fixtures/demo.dll --json -s demo
```

```json
{
  "scanned_binaries": 1,
  "total_exports": 2,
  "matched_exports": 1,
  "files": [
    {
      "path": "tests/fixtures/demo.dll",
      "format": "pe",
      "arch": "X86_64",
      "bits": 64,
      "endianness": "little",
      "kind": "DLL",
      "export_count": 2,
      "exports": [
        {
          "name": "?add@demo@@YAHHH@Z",
          "demangled": "int __cdecl demo::add(int, int)",
          "kind": "function",
          "address": 6442455072,
          "rva": 4128,
          "ordinal": 1
        }
      ]
    }
  ]
}
```

`exports` is the filtered list; `export_count` is the total number of exports in
that file. When a search is active, every scanned binary still appears in `files`,
even if its filtered `exports` array is empty.

## How it works

1. The input path is expanded with [`walkdir`](https://crates.io/crates/walkdir)
   when it is a directory.
2. The first 16 bytes are inspected to skip files that are not PE, ELF, or Mach-O.
3. [`goblin`](https://crates.io/crates/goblin) parses the container and its export
   table.
4. Exported names are optionally passed through
   [`cpp_demangle`](https://crates.io/crates/cpp_demangle) (Itanium ABI) or
   [`msvc-demangler`](https://crates.io/crates/msvc-demangler) (MSVC).
5. Filters and output formatting are applied. Parsing and demangling never modify
   the input files.

## Limitations

- This tool focuses on **export tables**, not import tables.
- PE support includes only **named** exports. Ordinal-only exports are not listed.
- ELF support reads `.dynsym` only; static `.symtab` symbols are not listed, and
  symbol versions such as `GLIBC_2.2.5` are not shown separately.
- Mach-O C symbol names keep the platform's leading underscore.
- Object files, static archives, and unknown formats are skipped during directory
  scans. Passing one explicitly is an error.
- Rust symbol mangling (v0) is not demangled.
- This is not a disassembler; it does not recover function signatures from machine
  code or inspect function bodies.

## Development

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

The test suite includes tiny PE, ELF, and Mach-O fixtures under
[`tests/fixtures`](tests/fixtures); see
[`tests/fixtures/README.md`](tests/fixtures/README.md) for how they are generated.

CI runs formatting, Clippy, and tests on Linux, Windows, and macOS, plus a check
against the minimum supported Rust version.

## Contributing

Issues and pull requests are welcome. Please read
[`CONTRIBUTING.md`](CONTRIBUTING.md) and
[`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md) before contributing.

- Use GitHub Issues for reproducible bugs and feature requests.
- Report security issues privately as described in [`SECURITY.md`](SECURITY.md).
- Keep changes focused and include tests for new behavior.
- Update [`CHANGELOG.md`](CHANGELOG.md) for user-visible changes.

## Security

ReverToken parses untrusted binary files. Please do not attach sensitive or
proprietary binaries to public issues. See [`SECURITY.md`](SECURITY.md) for the
reporting process.

## License

Released under the [MIT License](LICENSE).

## Acknowledgements

ReverToken stands on the shoulders of the Rust ecosystem, in particular
[`goblin`](https://crates.io/crates/goblin),
[`cpp_demangle`](https://crates.io/crates/cpp_demangle),
[`msvc-demangler`](https://crates.io/crates/msvc-demangler),
[`clap`](https://crates.io/crates/clap), and
[`walkdir`](https://crates.io/crates/walkdir).
