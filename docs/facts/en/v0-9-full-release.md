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
  - "repo:.github/workflows/release-publish.yml#sha256:903c4fd819d0d09afdbc379ac874a22d592274b495aab6de82ee15166381bcbb"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:0d2220c5b07d579fd7c54d958380b2482c1e44c9b738651840d119c79692b5be"
  - "repo:docs/guides/release-update.md#sha256:785e83d497c4f39ea683ac280adf8e071b27fda02b19c4c086573782a70bcadb"
  - "repo:docs/plans/active/release-0.9.2-test-qualification.md#sha256:2c2d6a00e695dc549649d5eb0c8416986dc5962e88c15a2f4836ee715eee821f"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:3f6e8607d62a8905abb68aabc599f88a573c08f1"
status: active
---

# Aigent Hive 0.9.x Test and Stable Releases

Stable `v0.9.1` is published from exact source `1e5e7b3`. Version `0.9.2` releases the completed
installed usage-guard convergence through `2cec037`, plus release-only metadata and qualification.
Native orchestration and custom-subagent work from `c777da1` onward moves to `0.9.3` on a separate
branch. Stable publication follows an accepted numbered public test and never serves as testing.
The `0.9.2` gate updates every public README, installation guide, HTML guide, npm README, plugin
metadata, documentation index, command, and version example before publication. Existing `develop`
history remains intact. Version `0.9.3` stays frozen until the QA-contributor instruction and a
later explicit maintainer approval.
