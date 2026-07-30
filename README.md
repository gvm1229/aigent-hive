# Aigent Hive

> A provider-neutral local harness for Codex, Claude Code, and Gemini Antigravity.

[![Version](https://img.shields.io/badge/version-0.7.0-4C1)](Cargo.toml)
[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust)](rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](LICENSE)

[English](./README.md) · [한국어](./docs/readme/README.ko.md)

Hive gives subscription-authenticated agent hosts one consistent setup, Skill routing,
project knowledge, durable role/run state, usage safeguards, and safe update contracts.
It never asks for model-provider API keys, calls model-provider APIs, or replaces the
host's own model runtime.

## 0.8.0 test distribution

`0.8.0` is an installation and update test, not the stable public release. It will be
published to npm under the exact version and `test` tag only. It will not create a
GitHub Release or move npm's `latest` tag.

After the test distribution is published:

```console
npm install -g aigent-hive@0.8.0
```

or:

```console
npm install -g aigent-hive@test
```

The npm installer requires Node.js and npm. The installed `hive` runtime is a native
Rust binary and does not require Node.js.

### macOS and Linux with curl

```sh
curl --proto '=https' --tlsv1.2 -LsSf \
  https://unpkg.com/aigent-hive@0.8.0/install.sh | sh
```

### Windows PowerShell 5.1+

```powershell
irm https://unpkg.com/aigent-hive@0.8.0/install.ps1 | iex
```

### Windows Command Prompt

```bat
curl.exe -fLo install-aigent-hive.cmd https://unpkg.com/aigent-hive@0.8.0/install.cmd && install-aigent-hive.cmd
```

The direct installers fetch the same native package bytes from npm, verify the
embedded exact-version SHA-256, and record direct-install ownership. They do not
require npm, Node.js, or PowerShell 7.

## Supported targets

| Platform | Native target | 0.8.0 gate |
| --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | Candidate runtime qualified |
| macOS Intel | `x86_64-apple-darwin` | Candidate runtime qualified |
| Linux x86_64 | `x86_64-unknown-linux-musl` | Release qualification in progress |
| Linux arm64 | `aarch64-unknown-linux-musl` | Release qualification in progress |
| Windows x86_64 | `x86_64-pc-windows-msvc` | Candidate runtime qualified |

Codex and Antigravity have real-host qualification evidence. Claude Code packaging and
projection are covered by fixtures, but a real subscription-backed session remains
unverified. macOS notarization and Windows code signing are deferred until a later
stability release.

## First setup

Install Hive for the host you use:

```console
hive install --scope user --host codex --apply --output json
```

Replace `codex` with `claude` or `antigravity` when appropriate. Then ask the host to
set up Aigent Hive. The first choice is `English` or `한국어`; every later setup question
and the global Hive guidance use that language.

Setup also asks whether Hive may check for updates once per day. This is opt-in.
Automatic checks only report a newer version. They never install one.

For a project, ask Hive to set up the current repository. Hive previews its exact owned
write set, preserves foreign guidance bytes, and keeps canonical knowledge in Markdown.

## Updating

```console
hive update
```

This performs an immediate version check. If a newer version exists, Hive explains the
exact update and asks before invoking the authenticated install owner. Declining,
closing stdin, or running non-interactively causes no installation.

When daily checks are enabled, a successful check is throttled for 24 hours. An offline
or failed check is not recorded as successful, so the next Codex, Claude Code, or
Antigravity session retries it.

Hive never installs an update silently.

## Automatic dispatch safeguard

When enabled, Hive checks subscription usage immediately before a new automatic
dispatch:

```console
hive usage enforce --target <project> --session-id <id> --process-id <pid> --output json
hive run resume --dispatch-intent automatic --target <project> --run <run-id> --capabilities <json> --output json
```

The first command is only a preflight; it never authorizes dispatch by itself. External
runtime cancellation is auxiliary evidence and never replaces durable goal/task state.
Ordinary answers and manual work do not run this automatic-dispatch gate.

## What Hive owns

- Hive-owned marker blocks and manifest-listed files only
- Provider-neutral Skills and thin host projections
- Canonical Markdown/YAML/TOML state
- Disposable SQLite indexes rebuilt from canonical text
- Verified direct-install receipts

Hive does not own provider credentials, model sessions, foreign guidance, OMX/OMC
state, Homebrew/WinGet installations, or optional third-party Skills without explicit
approval.

## Architecture and maintainer docs

- [Documentation home](./docs/00-home.md)
- [Complete document index](./docs/01-index.md)
- [Product overview](./docs/overview/product.md)
- [Development and verification](./docs/guides/development.md)
- [Active plan](./docs/plans/PLAN.md)
- [Current project state](./docs/state/CURRENT.md)
- [Source layout](./docs/architecture/source-layout.md)
- [Release and update trust boundary](./docs/architecture/release-update-trust-boundary.md)
- [Product decisions](./docs/decisions/product-release-decisions.md)

Development requires Rust stable, Python 3.13 for conformance tests, and PowerShell 7
for Windows development/release workflows. Consumer installations do not require
Python or PowerShell 7.

```console
python scripts/dev-check.py pre-push
```

## QA Contributors

| Name | GitHub | Tested platform or area |
| --- | --- | --- |

## License

Apache-2.0. See [LICENSE](./LICENSE).
