---
schema_version: 1
pair_id: security-release
topic_slug: security-release
language: en
counterpart: ../ko/security-release.md
title: "Security and Release Trust"
summary: "Pre-1.0 provenance, verifier-only hardened trust, and protected release publication."
tags: [release, security, trust]
aliases: ["release trust"]
sources:
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
  - "repo:docs/decisions/ADR-0013-0.8-release-scope.md#sha256:3b5b29532bd3e353aaaea9b95637a6582bf6c6ab5dab01ebfc61bc7967ecd613"
  - "repo:docs/guides/signed-update-and-release.md#sha256:e457b425ef3f8bf88599ad1ba576a9ab1a27d60c4ade9115665557586d6cf8e8"
  - "repo:docs/plans/active/release-0.8.0.md#sha256:e6ae8f6c48b5018960533264670696f22a17b0a786c9b4b8087bbc11fe0f515d"
  - "repo:docs/plans/active/windows-shell-install.md#sha256:d6a7e05eae8d1a4328a9fd58414b087442d32a7423ff92cb9e0d158bbb4ae179"
  - "repo:docs/state/CURRENT.md#sha256:de64dfc9f37a949e805fae83aaab84878e61565ba0b0c7b67887dc2f67cb5eaf"
links: [boundaries, product-intent, upgrade, usage-hosts]
reviewed_revision: "git:51f40e24316e9f776626ddf73676f7719b020a42"
status: active
---

# Security and Release Trust

The public identity is `Aigent Hive 0.8.0`, without a preview label, GitHub prerelease flag, or
preview npm dist-tag. Pre-1.0 SemVer communicates evolving maturity. Codex and Antigravity have
live host evidence; Claude has package, fixture, and projection conformance without a
subscription-backed session.

Current source `9fb2552` passes all seven clean-clone CI jobs. Historical native runtime evidence
at `d39ce7f` covers macOS arm64, macOS Intel, and Windows x86_64. Linux musl x86_64 and arm64
release qualification remains open.

The release baseline is a protected exact tag, five native archives, SHA-256, GitHub artifact
attestation, source provenance, and byte identity across GitHub and npm packages. The intended
primary command is `npm install -g aigent-hive`; verified Unix, PowerShell, and CMD installers
remain Node-free alternatives. Network self-update is disabled.

Hive remains verifier-only at the hardened trust boundary. It never generates, reads, stores, or
uses private signing material. The existing TUF-compatible verifier, compiled-only migrations,
rollback protection, and external signer design remain available for a future hardened update
channel. Candidate creation and public publication stay separate protected workflows.

The `0.8.0` workflows must build five targets, stage npm platform packages, attest artifacts, and
publish platform packages before the `aigent-hive` umbrella package. GitHub uses a normal release
and npm uses `latest`. Developer ID, notarization, Authenticode, Azure signing, and external TUF
authorization are deferred opt-in hardening, not mandatory absent secrets. Public GitHub and npm
publication still require final user confirmation and registry ownership.

The Windows consumer boundary has no PowerShell 7 dependency. `hive.exe` and the installed
harness do not detect, prompt for, install, update, or uninstall PowerShell 7. The direct
installer supports built-in Windows PowerShell 5.1, and `cmd.exe` invokes that same exact-version
bootstrap path.

PowerShell 7.6.4 LTS is a source development and release dependency only. The optional source
helper previews the exact WinGet command, package, version, and user scope; mutates nothing
without explicit consent; delegates installation to Microsoft's package; and requalifies the
result. PowerShell installation, update, and removal remain owned by Microsoft or the selected
package manager.
