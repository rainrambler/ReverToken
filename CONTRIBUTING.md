# Contributing to ReverToken

Thanks for taking the time to contribute! This document explains how to set up the
project, run the checks, and submit changes.

By participating in this project, you agree to follow the
[Code of Conduct](CODE_OF_CONDUCT.md).

## Ways to contribute

- Report reproducible bugs or request features through GitHub Issues.
- Improve documentation, examples, or error messages.
- Add test coverage or support for another binary format.
- Review pull requests and reproduce reported issues.

Please do **not** attach proprietary, sensitive, or malware samples to public
issues. Describe the file and, if possible, provide a minimized synthetic fixture.

Security vulnerabilities must be reported privately following
[SECURITY.md](SECURITY.md), not through public issues.

## Development setup

ReverToken requires Rust **1.85 or newer** and uses edition 2024.

```sh
git clone <repository-url>
cd <repository-directory>
cargo build
cargo test
```

No external binary-format library is required at build time; parsing is
implemented in Rust with `goblin`.

## Before submitting a pull request

Run the same checks as CI:

```sh
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test
```

For documentation-only changes, at least run `cargo test` if you touched code
blocks or examples.

## Coding guidelines

- Keep the parser and output layers separate. `src/formats/` returns data; it does
  not print.
- Avoid panics on untrusted input. Return an error or skip an unsupported file.
- Preserve the raw symbol name whenever a demangled name is added.
- Escape control characters before writing names or paths to a terminal.
- Prefer small, focused functions and add comments only where the binary format
  logic is not obvious.
- Keep user-visible behavior documented in both `README.md` and
  `README.zh-CN.md` when applicable.
- Add a `CHANGELOG.md` entry under `[Unreleased]` for user-visible changes.

## Tests and fixtures

Run the full suite with:

```sh
cargo test
```

Integration tests live in `tests/cli.rs` and invoke the compiled binary. Unit tests
cover demangling, search semantics, and output escaping.

Small binary fixtures are checked in under `tests/fixtures/`. They must be:

- small (a few kilobytes where possible),
- reproducible from the sources in `tests/fixtures/src/`,
- free of third-party or proprietary code, and
- generated with `tests/fixtures/build.sh`.

If you add or replace a fixture, update `tests/fixtures/README.md` and the tests
that depend on it.

## Adding a new binary format

1. Add a parser module under `src/formats/` that produces `BinaryReport` values.
2. Teach `src/formats/mod.rs` to detect and dispatch the format.
3. Reuse `demangle::demangle` for symbol names and `model::SearchQuery` for
   filtering.
4. Add a fixture, unit/integration tests, and documentation.
5. Update both READMEs and the changelog.

## Commit messages

Clear commit messages are appreciated. Conventional Commits are welcome but not
required, for example:

```text
feat: support COFF import libraries
fix: avoid panic on truncated PE export tables
docs: clarify ELF .dynsym filtering
test: add 32-bit PE fixture
```

Keep each commit logically coherent.

## Pull request checklist

- [ ] The change is focused and described in the PR.
- [ ] `cargo fmt --all -- --check` passes.
- [ ] `cargo clippy --all-targets -- -D warnings` passes.
- [ ] `cargo test` passes.
- [ ] New behavior has tests (or a clear reason why tests are not practical).
- [ ] User-visible changes are in `CHANGELOG.md`.
- [ ] Relevant documentation and fixtures are updated.
- [ ] No sensitive binaries or credentials are included.

## License

By contributing, you agree that your contributions are licensed under the
[MIT License](LICENSE), without additional terms or conditions.
