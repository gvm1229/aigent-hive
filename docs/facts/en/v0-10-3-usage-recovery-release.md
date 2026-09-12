---
schema_version: 1
pair_id: v0-10-3-usage-recovery-release
topic_slug: v0-10-3-usage-recovery-release
language: en
counterpart: ../ko/v0-10-3-usage-recovery-release.md
title: "0.10.3 Usage Guard Recovery Stable Release"
summary: "0.10.3 replaces or clears a legacy halt.json in place after a fresh measurement, so an old marker alone cannot block work."
tags: [release, usage-guard, v0-10-3]
aliases: ["0.10.3 stable"]
sources:
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:881ae77507817b888cf66ff0da2ee52fbc4048fbb9355641a093dcf6f3d69fc1"
  - "repo:docs/plans/active/usage-recovery-0.10.2.md#sha256:6419915d2b1149efa2613fb0f84c4659f787e1d7f59f3489103b205d7f0a2691"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
links: [release-verification, automatic-test-release-gate]
reviewed_revision: "git:8afe730759325bc003475f81545da7afc88cfb18"
status: active
---

# 0.10.3 Usage Guard Recovery Stable Release

- A legacy, prior-process, or damaged regular `halt.json` triggers a fresh usage measurement.
- An allowed measurement clears the marker at the same path.
- A limited or unknown measurement atomically replaces the marker at the same path with current evidence.
- Symlinks, path escape, and unsupported future formats produce a safety error distinct from usage exhaustion.
- Public acceptance: Windows, macOS, and Linux for `0.10.3-test.1`, plus an existing installed target.
- Stable publication: npm `latest=0.10.3` and the GitHub `v0.10.3` normal Release.
