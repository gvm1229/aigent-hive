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
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:1229cfa84e1fb0357c943fd0ef2910f3cdb5dd7e70f67879f0832db0ea26c800"
  - "repo:crates/hive-wiki/src/source.rs#sha256:f9d3cae67f645e63b1483e83303fd0ecce41d50a9f66d3cd9540fc6e6f824f52"
  - "repo:scripts/qualify-source-graph.py#sha256:62e74cb2994404d7607f33a38da73b1973592609fc1b6af3686a7920c2086710"
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
Fixed questions use same-revision facts and cited files in a disposable snapshot, never
historical executables. Current index, graph and lint are checked separately. Both fact trees
must stay unchanged and both lint reports must have zero errors and warnings.
