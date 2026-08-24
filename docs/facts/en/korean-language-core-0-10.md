---
schema_version: 1
pair_id: korean-language-core-0-10
topic_slug: korean-language-core-0-10
language: en
counterpart: ../ko/korean-language-core-0-10.md
title: "Korean Language Core for 0.10.0"
summary: "The Korean language core uses a pinned im-not-ai-derived rule pack, deterministic preservation gates, host-owned local rewrites, humanize-kor, and approved pack rollback."
tags: [korean, language, skill, v0-10]
aliases: ["Korean output gate", "humanize-kor"]
sources:
  - "repo:.github/workflows/public-test-acceptance.yml#sha256:c99d8ed6497c2eff9a54ccf89e81bf631b6e88736ac1d9b163033334766ee3d7"
  - "repo:crates/hive-core/src/korean.rs#sha256:16037a43c32e9fd1c777c6f7aabb7fa7bcf0fb265086fe84fc0bc35a93f07bda"
  - "repo:docs/architecture/korean-language-core.md#sha256:2ae475f2f4c701f42fe28bff62cb37e60ef516526b2f147d1ec1544e2b32bfa4"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:aaf1355c1b691a83f047164caed5923bcc5a9769ffb44aecfe8b4c3d247af46c"
  - "repo:harness/language-packs/im-not-ai/2.3.2/manifest.json#sha256:50e8bec5fb4c7a479f9e0800f262d49c3e01258ba3c7b9066aab65ba3f7ca34e"
  - "repo:harness/skills/humanize-kor/SKILL.md#sha256:8805da50d3370fa953a1325a0a6c5294247ab037cb173ab09266c67d09aa659a"
  - "repo:scripts/qualify-korean-public-test.py#sha256:66d1b195a843159196b4753c2c6018a2e00873ee0b9c086bc772db110fcae715"
links: [language-consistency, public-skill-identity, v0-10-product-scope]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# Korean Language Core for 0.10.0

Hive inspects finished Korean text with five profiles and asks the active host for at most one
local rewrite. Deterministic verification preserves modality, numbers, quotations, links, code,
commands, paths, and attribution; failure selects the exact draft. `humanize-kor` applies the same
contract to user-selected text. The pinned `im-not-ai 2.3.2` transformation supports preview,
exact consent, staged activation, and rollback without raw or floating upstream installation.
Numbered public tests install exact npm bytes on Windows x64, macOS arm64, and Linux musl x64,
then rerun the gold corpus, preservation rejection, sanitization, update preview, and rollback.
