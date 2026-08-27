---
schema_version: 1
pair_id: source-graph-public-qualification
topic_slug: source-graph-public-qualification
language: en
counterpart: ../ko/source-graph-public-qualification.md
title: "Source Graph Public Qualification"
summary: "The 0.10.0 source graph combines source Wiki FTS with grounded Markdown edges and gates release candidates with 30 exact and 30 relationship questions."
tags: [graph, knowledge, qualification, v0-10]
aliases: ["source graph acceptance", "source relationship qualification"]
sources:
  - "repo:.github/workflows/release.yml#sha256:88394c81a55cb27a5fea46cc1adddd6877e0a3006c24567a1865e34b2bef26bb"
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:f8920322c1f918b16e9b2df7c1b3a29867cbd4c6cc95b82caa33016d63faab47"
  - "repo:crates/hive-wiki/src/source.rs#sha256:334881a8ed13d2e960d95d924c71391f029be131b67f8315c54a7385f1205a0f"
  - "repo:scripts/qualify-source-graph.py#sha256:10896324388562ef86b75c443f463c734d1323327a3631957c0aa86e88649a79"
links: [graphify-0-10-adoption, hybrid-vector-search-0-10]
reviewed_revision: "git:56db5d7f6b1fd49f4ed817617d2bc635fd0bbf63"
status: active
---

# Source Graph Public Qualification

`hive source-wiki graph` keeps source relations under `.agents/work`, joins an English FTS hit
to bounded `EXTRACTED` edges, and leaves canonical fact bytes unchanged. Every numbered release
candidate runs the shipped target binary against 30 exact and 30 relationship questions. The
gate requires exact Recall@10 of 100%, grounded relationship Recall@10 of at least 90%, cold CLI
p95 at most two seconds, and no provider API, API key, or query log use.
