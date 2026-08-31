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
  - "repo:.github/workflows/public-test-acceptance.yml#sha256:eb42ccf9ecd466efd4c9693df8a58dace4c7e541ea05fbbc24193b7d4844b841"
  - "repo:crates/hive-core/src/korean.rs#sha256:bb575d5e73f1567755656c7e6be98cca871416a052e83e920d95b91e77186188"
  - "repo:docs/architecture/korean-language-core.md#sha256:3b97a9ba4e09ea2c68e2094ff57b383e255ecd7e50d85facf50b9f3ea3c56fa3"
  - "repo:docs/plans/active/korean-language-core-0.10.0.md#sha256:d84549268a83748e23da88c1e9c1d51163776e9511b258feb2b79c3318239e09"
  - "repo:harness/language-packs/im-not-ai/2.3.2/manifest.json#sha256:50e8bec5fb4c7a479f9e0800f262d49c3e01258ba3c7b9066aab65ba3f7ca34e"
  - "repo:harness/skills/humanize-kor/SKILL.md#sha256:b356691df025bb30def279528450be5c5c9085adf11457efcda87834ef452f67"
  - "repo:scripts/qualify-korean-public-test.py#sha256:f65f27b409d902b3d44beb1fd7f30f843eacbcb7f3acf5c9288bc04bef659a0c"
links: [language-consistency, public-skill-identity, v0-10-product-scope]
reviewed_revision: "git:eaed3203ce3fea062acab325a9ce0892348aff02"
status: active
---

# Korean Language Core for 0.10.0

Five profiles and humanize-kor share the validated active pack selected by the consumer target.
Strict rule parsing, protected spans, change limits and negative-clause checks reject invalid
rewrites; complete semantic equivalence still requires host review. The host owns rewriting,
with at most one automatic retry and exact-source fallback. Pinned im-not-ai 2.3.2 supports
consented staged activation and hash-verified rollback, including recovery from current-pack
corruption. Public Windows/macOS/Linux receipts cover the recorded corpus, not all future text.
