---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: en
counterpart: ../ko/v0-9-full-release.md
title: "Aigent Hive 0.9.x Test and Stable Releases"
summary: "Stable v0.9.2 published the completed usage-guard scope and updated public docs; QA contributor 안희준 covers Windows x64 installation and configuration; v0.9.3 requires later explicit approval."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.2 scope", "0.9.3 scope", "0.9.x release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:4e93f1bb01339ed05f69cdb773c27ba83b704de8b24465f761e08e201955eb39"
  - "repo:README.md#sha256:f82fd4b6fda33025116d6a26e7d7823affd75911e7c21ff8926b258518895d33"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:b8209a7d3233f92d7174cb26ab08d9fbaa2831945f15dc9dee5a8e7e045cbe1c"
  - "repo:docs/guides/release-update.md#sha256:f046e838fa7f44c6fa336fd089d4740c6f3f2a8ab8fb8a010e748f7b1d4bcd10"
  - "repo:docs/guides/release-verification-builds.md#sha256:e9490fbcdd337f9935957e641d73f834bdf602030d28c8c0808699a1606eb9d9"
  - "repo:docs/plans/active/release-0.9.2-test-qualification.md#sha256:cd46512616a3c0be755319e226eea5a2184a904fd829e679d2baf44016bd837d"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a502867e6b20e8f22bc014af05ca678f211f40ed"
status: active
---

# Aigent Hive 0.9.x Test and Stable Releases

Stable `v0.9.2` publishes the completed installed usage-guard scope through `2cec037` and synchronized
public documentation. QA contributor 안희준 ([No-Jyun](https://github.com/No-Jyun)) covers Windows
x64 installation and configuration verification. Native orchestration and custom-subagent work remain
excluded; `0.9.3` needs later explicit maintainer approval.
