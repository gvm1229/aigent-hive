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
  - "repo:.github/workflows/release.yml#sha256:98fc01b94dd0cc9c5fa839c4fc68a32c8398fd6a624e065c1a1631001173e777"
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:ee2e1628368de52fe46f08547c2866bcd271fac881b61127e0e00db67b297c1e"
  - "repo:crates/hive-wiki/src/source.rs#sha256:71bb10bdcb52766b4d79661dcd6fe3a9aec674d7ca3787e80dd6e840d31bb274"
  - "repo:scripts/qualify-source-graph.py#sha256:20181b16b7dee6e78992cab8c2e3411614153a6e7345ab3e928a7ce61e6a4478"
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
