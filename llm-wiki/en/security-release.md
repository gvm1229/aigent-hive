---
schema_version: 1
pair_id: security-release
topic_slug: security-release
language: en
counterpart: ../ko/security-release.md
title: "Security and Release Trust"
summary: "Verifier-only trust roots, authenticated judge evidence, and protected release publication."
tags: [release, security, trust]
aliases: ["release trust"]
sources:
  - "repo:docs/architecture/release-update-trust-boundary.md#sha256:3f90ab4526cd1ca556a057af529fada637a2a476374927408098e759ae6deb8f"
  - "repo:docs/decisions/ADR-0007-ed25519-judge-trust.md#sha256:5a17bcd8f6869437a9e37c025c4fa2da285ff03af16a7144552162bac5a09a1a"
  - "repo:docs/decisions/ADR-0008-verifier-only-tuf-updates.md#sha256:97989993dba9959f24117f0e4917954a3e67b215cfe659942172e9f22c6ff709"
links: [boundaries, upgrade, usage-hosts]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Security and Release Trust

Hive is verifier-only at the trust boundary. It does not generate, read, store, or use private
signing material. Judge and human authority derives from external, agent-write-denied Ed25519 public
trust roots and detached attestations bound to canonical digests.

Release authorization uses a TUF-compatible Ed25519 metadata chain with threshold roles, expiry,
snapshot consistency, rollback protection, root rotation, exact target lengths, and SHA-256
digests. The running binary accepts only compiled migration routes; signed releases cannot deliver
shell, dynamic library, WebAssembly, or other executable migration code.

Candidate creation and public publication are distinct protected workflows. External signers own
private credentials. Publication independently verifies TUF, source commit, provenance, platform
signing evidence, offline attestation bundles, and exact candidate bytes before creating public
tags or releases.
