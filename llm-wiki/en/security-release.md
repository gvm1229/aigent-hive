---
schema_version: 1
pair_id: security-release
topic_slug: security-release
language: en
counterpart: ../ko/security-release.md
title: "Security and Release Trust"
summary: "Preview provenance, verifier-only hardened trust, and protected release publication."
tags: [release, security, trust]
aliases: ["release trust"]
sources:
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
  - "repo:docs/decisions/ADR-0013-preview-release-scope.md#sha256:1ba89150ac521f638f686cd4fa9ff6d8cf256a0e561c0c3b80616fd1a3989f2f"
  - "repo:docs/guides/signed-update-and-release.md#sha256:e457b425ef3f8bf88599ad1ba576a9ab1a27d60c4ade9115665557586d6cf8e8"
  - "repo:docs/plans/active/preview-release.md#sha256:6464dfd503c88c116f5e8e09aadf92f6ee3b6e2e0f2158075c868852ac238c22"
  - "repo:docs/plans/active/windows-shell-install.md#sha256:0c79a70672c69438c85c9b3f9406036f6d90d616959e5030179147737eaed0f7"
  - "repo:docs/state/CURRENT.md#sha256:82abf3d38297cc460f830f0cc05a51e8e186ce099bd0aeeda0901fef6816fe5c"
links: [boundaries, product-intent, upgrade, usage-hosts]
reviewed_revision: "git:cb22a76995f7f1b17f826d521c26546ecd674f93"
status: active
---

# Security and Release Trust

The `0.8.0` public label is `Claude-unverified preview`. Codex and Antigravity have live host
evidence; Claude has package, fixture, and projection conformance without a subscription-backed
session. Windows 11 x86_64 acceptance now covers Codex user install, global setup, automatic
project onboarding, shared indexing, repeat update, rollback, and revalidation.

Current source `d39ce7f` passes all seven clean-clone CI jobs and all three unsigned native
runtime jobs: macOS arm64, macOS Intel, and Windows x86_64.

The preview trust baseline is a protected exact tag, release-asset SHA-256, GitHub artifact
attestation, source provenance, and package-manager or digest-pinned manual update. Network
self-update is disabled. Developer ID, notarization, Authenticode, and external TUF 2-of-3
authorization are deferred rather than falsely claimed.

Hive remains verifier-only at the hardened trust boundary. It never generates, reads, stores, or
uses private signing material. The existing TUF-compatible verifier, compiled-only migrations,
rollback protection, and external signer design remain available for a future hardened update
channel. Candidate creation and public publication stay separate protected workflows.

The existing candidate and publication workflows still target the hardened trust path: Developer
ID, notarization, Azure signing, external TUF authorization, and platform signer evidence. The
repository currently has no environments, secrets, or variables for those paths. An ADR-0013
preview workflow for exact `0.8.0` archives, SHA-256, and GitHub artifact attestation remains the
next candidate-trust task. Public release publication still requires a protected workflow and
final user confirmation.

The Windows consumer boundary has no PowerShell 7 dependency. `hive.exe` and the installed
harness do not detect, prompt for, install, update, or uninstall PowerShell 7. The direct
installer supports built-in Windows PowerShell 5.1, and `cmd.exe` invokes that same exact-version
bootstrap path.

PowerShell 7.6.4 LTS is a source development and release dependency only. The optional source
helper previews the exact WinGet command, package, version, and user scope; mutates nothing
without explicit consent; delegates installation to Microsoft's package; and requalifies the
result. PowerShell installation, update, and removal remain owned by Microsoft or the selected
package manager.
