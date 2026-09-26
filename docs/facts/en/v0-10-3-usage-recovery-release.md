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
  - "repo:docs/plans/active/usage-recovery-0.10.2.md#sha256:6419915d2b1149efa2613fb0f84c4659f787e1d7f59f3489103b205d7f0a2691"
  - "repo:docs/releases/0.10.3.md#sha256:94a75051e50352ff04de8e649b8382db81ee5b6b2ea95276203bdddeedcc2ea8"
links: [automatic-test-release-gate, release-verification]
reviewed_revision: "git:a1b96c88b4938c8771ccf635ffef0e8dcafa9ff1"
status: active
---

# 0.10.3 Usage Guard Recovery Stable Release

Historical 0.10.3 behavior; not the later quota-reset acknowledgement contract.

- A legacy, prior-process, or damaged regular `halt.json` triggers a fresh usage measurement.
- An allowed measurement clears the marker at the same path.
- A limited or unknown measurement atomically replaces the marker at the same path with current evidence.
- Symlinks, path escape, and unsupported future formats produce a safety error distinct from usage exhaustion.
- Public acceptance: Windows, macOS, and Linux for `0.10.3-test.1`, plus an existing installed target.
- Stable publication: npm `latest=0.10.3` and the GitHub `v0.10.3` normal Release.
