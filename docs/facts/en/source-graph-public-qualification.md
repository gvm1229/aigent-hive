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
  - "repo:.github/workflows/release.yml#sha256:fe8bb871aaa0710a655f41521b7fe7960c63ff5c660795a2ee09aed29cb92631"
  - "repo:crates/hive-cli/src/knowledge.rs#sha256:fb2a64a7cafaf921a5252a65b042ade24ab03b7dcee5eec52dc7b895b5b92df5"
  - "repo:crates/hive-wiki/src/source.rs#sha256:c9a37705e4dc5263646bed03ce55cd3b5ed28de2e20204c4ed1b0aa05df2413f"
  - "repo:scripts/qualify-source-graph.py#sha256:62e74cb2994404d7607f33a38da73b1973592609fc1b6af3686a7920c2086710"
links: [graphify-0-10-adoption, hybrid-vector-search-0-10]
reviewed_revision: "git:52e63238cb6063312240d1c9b5ed1006d8a23a7b"
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
