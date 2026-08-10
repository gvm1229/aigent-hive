---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.0 Test and Stable Release"
summary: "The 0.9.0 stable release remains blocked: test.11 proved the Windows user setup, while test.12 must prove a preserving uninstall, clean reinstall, and new Codex-session discovery."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:026e00b1386bdd2364e8befa7830a94b654dae96e7d4cae0411aa21613eb798b"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 Test and Stable Release

Stable `v0.9.0` remains absent. Candidate `31372510565` and OIDC publication `31373214154`
released `0.9.0-test.11` from `b0e41f58bd6b73b56cbe92c2b054fb5cefcc9f03`; all six npm packages
have `test=0.9.0-test.11` and `latest=0.8.0`, with an annotated GitHub prerelease. Its isolated
Windows installation and actual user-root `dry-run → apply → validate` passed, including silent
Codex marketplace recovery, the product-only Skill catalog, Korean/bilingual Wiki, usage guard,
and persisted Discord test delivery.

`REL9-011` remains open only for test.12: preserving `hive uninstall`, clean reinstall with saved
preferences, and automatic CLI discovery in a newly opened Codex session on Windows 11.
