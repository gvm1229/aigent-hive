# Aigent Hive

> A provider-neutral local harness for Codex, Claude Code, and Gemini Antigravity.

[![Version](https://img.shields.io/badge/version-0.9.0-4C1)](Cargo.toml)
[![Rust](https://img.shields.io/badge/Rust-stable-000000?logo=rust)](rust-toolchain.toml)
[![License](https://img.shields.io/badge/license-Apache--2.0-green)](LICENSE)

[English](./README.md) · [한국어](./docs/readme/README.ko.md)

Hive gives subscription-authenticated agent hosts one consistent setup, Skill routing,
project knowledge, durable role/run state, usage safeguards, and safe update contracts.
It never asks for model-provider API keys, calls model-provider APIs, or replaces the
host's own model runtime.

Stable `0.8.0` remains the npm `latest` release. Developer test build `0.9.0-test.4`
is published only on npm `test` and as a GitHub prerelease.

## Install 0.8.0

`0.8.0` is published on npm as `latest` for installation validation. This publication
does not create a GitHub Release or Git release tag.

```console
npm install -g aigent-hive
```

Or pin the exact version:

```console
npm install -g aigent-hive@0.8.0
```

The npm installer requires Node.js and npm. The installed `hive` runtime is a native
Rust binary and does not require Node.js.

### Developer test build 0.9.0-test.4

For developers and contributors testing the next release:

```console
npm install -g aigent-hive@0.9.0-test.4
hive --version
```

Expected version label:

```text
AIgent Hive v0.9.0-test #4 · developer test build (released 2026-08-07)
```

This explicit version never changes npm `latest`.

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

## Optional one-prompt setup

If you want Codex, Claude Code, or Gemini Antigravity to guide the entire user-level
installation, paste the following prompt instead of manually following the first three
setup steps below. It is optional: the four-step setup remains the predictable manual path.

```text
I want the optional one-prompt Aigent Hive setup. Work only at user scope; do not inspect,
initialize, or change any project, repository, folder, or current working directory.

First ask whether I want the stable release 0.8.0 (recommended) or the developer test build
0.9.0-test.4. The stable install guidance is https://github.com/gvm1229/aigent-hive#install-080
and the test-build release notes are https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.0-test.4.
Detect my operating system and active host (Codex, Claude Code, or Gemini Antigravity), asking
me if either is unclear. Check whether Node.js and npm are available. If they are missing,
give me the official OS-specific Node.js installation command and request any approval the host
requires before installing it. Then install the exact Hive release I selected using the official
method in the linked guidance, verify `hive --version`, and activate only my host with
`hive install --scope user --host <detected-host> --apply --output json`.

Then begin interactive global setup in this conversation. For a first setup, ask only whether I
want English or Korean first; continue one question at a time. For existing settings, first ask
whether I want to change one setting or review everything. Do not start project setup afterward:
offer the separate project-setup prompt instead. Never ask for provider API credentials or install
an optional third-party Skill.
```

This option installs only the release you choose. A test build stays on npm's `test` tag and does
not change `latest`.

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

Follow these four steps in order. Repeat step 2 for each host, step 4 for each project,
and step 3 whenever global preferences change.

### 1. Install the Hive CLI

Use one command from [Install 0.8.0](#install-080) above. The npm installation provides the
`hive` command; it does not yet activate Hive inside a host.

### 2. Activate Hive for this host

In a terminal, activate the host projection:

```console
hive install --scope user --host codex --apply --output json
```

Replace `codex` with `claude` or `antigravity` for that host. This operation restores an
authenticated known prior user installation before updating it to the current projection;
it still refuses unknown or modified ownership manifests.

### 3. Configure global preferences

Open Codex, Claude Code, or Gemini Antigravity and paste this shared prompt:

```text
Configure or reconfigure my global Aigent Hive preferences for this host. Do not inspect or configure a project, repository, folder, or current working directory. Start the interactive user-scope setup.
```

Use this prompt for the first setup and later preference changes. It configures only your
user-scope language, Wiki, user contexts, persona, Skills, and update preferences; it never inspects the
current folder or creates a project harness.

All built-in Skills are active by default. If you prefer a smaller set, choose Skills one by
one during setup; `setup-hive` always remains active. You may select multiple user contexts and
add a short description. They help Hive understand you globally, but never choose a project
workflow, implementation approach, delivery priority, or active Skill set. Your persona and
selected host also never change the active Skill set. Users with an earlier recommended-suite
setting keep its exact existing Skill set until they review and approve a new preview.

### 4. Configure one project

Open the exact project in the host and paste this separate prompt:

```text
Configure the local Aigent Hive harness for this project. Use my existing global Hive preferences, inspect only this project, show the exact write preview, and ask me only about choices that require my approval.
```

Use this prompt once for each repository. It inherits your global preferences and only changes
the named project after showing its exact write preview. If the host is not open in the project,
name the project with an absolute path instead:

```text
Configure the local Aigent Hive harness for the project at /absolute/path/to/project. Use my existing global Hive preferences, inspect only that project, show the exact write preview, and ask me only about choices that require my approval.
```

Do not use the project prompt from your home directory without a project path. A request that
includes both scopes completes global setup first and asks before inspecting or changing a project.

Neither prompt authorizes an update, optional third-party Skill, or provider-credential
access.

Hive previews its exact owned write set, preserves foreign guidance bytes, and keeps
canonical knowledge in Markdown.

## Updating

```console
hive update
```

This performs an immediate version check. If a newer version exists, Hive explains the
exact update and asks before invoking the authenticated install owner. Declining,
closing stdin, or running non-interactively causes no installation.
An existing `0.9.0-test.N` installation keeps its owner evidence and may update to exact
stable `0.9.0` through the same confirmation flow when that stable release is published.

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
