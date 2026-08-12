---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.x Test and Stable Releases"
summary: "v0.9.2 releases completed usage-guard work with all public docs updated; v0.9.3 requires later explicit approval."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.2 scope", "0.9.3 scope", "0.9.x release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:4e93f1bb01339ed05f69cdb773c27ba83b704de8b24465f761e08e201955eb39"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:53314be9705bd61590992cae77cfcf96a9d823e7142821399e6411492de76e00"
  - "repo:docs/guides/release-update.md#sha256:f046e838fa7f44c6fa336fd089d4740c6f3f2a8ab8fb8a010e748f7b1d4bcd10"
  - "repo:docs/plans/active/release-0.9.2-test-qualification.md#sha256:2917125fb3746ba29660ce9b0a8b3a6c3e753c4e07930f755f42b6229db88f9a"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:d08ec9aa109b55c30ca9d3c455185c6e5cb3f08e"
status: active
---

# Aigent Hive 0.9.x Test and Stable Releases

Stable `v0.9.1` is published from exact source `1e5e7b3`. Version `0.9.2` releases the completed
installed usage-guard convergence through `2cec037`, plus release-only metadata and qualification.
Every public README, installation guide, HTML guide, npm README, plugin metadata, documentation
index, command, and version example is updated before publication. Native orchestration and
custom-subagent work is excluded. Version `0.9.3` stays frozen until the QA-contributor instruction
and a later explicit maintainer approval.
