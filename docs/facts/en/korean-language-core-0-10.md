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
  - "repo:.github/workflows/public-test-acceptance.yml#sha256:31283b71f7c1a61e23658d2de67d119c5fb5886990ed3b788489a0a80ea14d12"
  - "repo:crates/hive-core/src/korean.rs#sha256:16037a43c32e9fd1c777c6f7aabb7fa7bcf0fb265086fe84fc0bc35a93f07bda"
  - "repo:docs/architecture/korean-language-core.md#sha256:2ae475f2f4c701f42fe28bff62cb37e60ef516526b2f147d1ec1544e2b32bfa4"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:f93dba421edc980e3a9ca8b5a8ce2ee978806094ae360203371948a27bcadaec"
  - "repo:harness/language-packs/im-not-ai/2.3.2/manifest.json#sha256:50e8bec5fb4c7a479f9e0800f262d49c3e01258ba3c7b9066aab65ba3f7ca34e"
  - "repo:harness/skills/humanize-kor/SKILL.md#sha256:8805da50d3370fa953a1325a0a6c5294247ab037cb173ab09266c67d09aa659a"
  - "repo:scripts/qualify-korean-public-test.py#sha256:96fda477a4c490aa15ad704f2a5117cfa1d63c00252c282408134de6219f498d"
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
