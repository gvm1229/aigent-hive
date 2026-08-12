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
  - "repo:README.md#sha256:3c390ad3b1a884c49a15304b0a0799299384e2e319e626ff7a752ecf4d700d94"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:53314be9705bd61590992cae77cfcf96a9d823e7142821399e6411492de76e00"
  - "repo:docs/guides/release-update.md#sha256:f046e838fa7f44c6fa336fd089d4740c6f3f2a8ab8fb8a010e748f7b1d4bcd10"
  - "repo:docs/guides/release-verification-builds.md#sha256:e9490fbcdd337f9935957e641d73f834bdf602030d28c8c0808699a1606eb9d9"
  - "repo:docs/plans/active/release-0.9.2-test-qualification.md#sha256:d2cc4469f1ca4aff0439f4eae9a063652ed971faf6d42a4136d62a0f64b8fdaf"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:d08ec9aa109b55c30ca9d3c455185c6e5cb3f08e"
status: active
---

# Aigent Hive 0.9.x Test and Stable Releases

Stable `v0.9.1` is published from exact source `1e5e7b3`. Version `0.9.2` releases the completed
installed usage-guard convergence through `2cec037`, plus release-only metadata and qualification.
Every public README, installation guide, HTML guide, npm README, plugin metadata, documentation
index, command, and version example is updated before publication. Public READMEs expose only the
stable installation; one neutral maintainer link leads to a separate test-build guide. Native orchestration and
custom-subagent work is excluded. Version `0.9.3` stays frozen until the QA-contributor instruction
and a later explicit maintainer approval.
