# Security Policy

## Supported versions

| Version | Supported |
| --- | --- |
| 0.1.x | Yes |
| < 0.1 | No |

## Reporting a vulnerability

Please **do not open a public GitHub issue** for a security vulnerability.

Use GitHub's private vulnerability reporting for this repository:

1. Open the repository's **Security** tab.
2. Choose **Report a vulnerability**.
3. Include the affected version, platform, a minimal reproducer, and the impact.

If private reporting is not available, open a minimal public issue asking the
maintainers for a private channel; do not include exploit details or sensitive
files in that issue.

We aim to acknowledge reports within 7 days and will coordinate a fix and
disclosure timeline with the reporter. Please give us a reasonable opportunity to
release a fix before public disclosure.

## Scope

ReverToken parses untrusted binary files locally. Security-relevant issues include,
but are not limited to:

- panics, aborts, or unbounded memory use caused by a malformed PE, ELF, or Mach-O
  file;
- out-of-bounds reads or other memory-safety issues;
- infinite loops or pathological CPU consumption during parsing or demangling;
- terminal escape or control-character injection through symbol names, paths, or
  error messages;
- path traversal or unintended file access while walking a directory.

The tool does not execute scanned binaries and does not need network access.

## Safe sample handling

Do not attach proprietary, confidential, or live malware samples to public issues.
Prefer a minimal synthetic file that reproduces the problem, or ask for a private
upload channel in your report.

## No bug bounty

This project does not currently offer a bug bounty. Responsible reports and fixes
are still very welcome and will be credited when possible.
