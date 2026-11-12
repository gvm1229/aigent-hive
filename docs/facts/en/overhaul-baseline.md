---
schema_version: 1
pair_id: overhaul-baseline
topic_slug: overhaul-baseline
language: en
counterpart: ../ko/overhaul-baseline.md
title: "Overhaul baseline"
summary: "Hive 0.14.2 has a measured Windows project, retrieval and size baseline."
tags: [baseline, overhaul, retrieval]
aliases: [overhaul-baseline]
sources:
  - "repo:docs/research/overhaul/OVX-001.md#sha256:a915958a3ba29cede7f163a2bf3b7ddb1bb4a3225e856427d67c8fe89943e55b"
  - "repo:docs/research/overhaul/OVX-002.md#sha256:9581105e5f8a69db4b40591ca3c281559fde48d30e8c262d991f077f9290233d"
  - "repo:docs/research/overhaul/OVX-004.md#sha256:d919e893c1e21d2851dee03af64731a6b4263965a2edf945294f23fb9222849d"
links: []
reviewed_revision: "git:a0fa76c563d17083640e9ee32cc199fcbd4f4b7e"
status: active
---

# Overhaul baseline

On 2026-11-12, Hive 0.14.2 on Windows showed a general default in all 30 minimal project fixtures; no effective kind was selected. Fixed bilingual FTS queries returned the expected fact in the top five for 22/40 queries and 3/10 held-out queries. One new Codex project had 81 files. The 28 canonical Skills had 14,527 whitespace-separated words. Its synthetic user store used 198,235 bytes. These results do not qualify automatic project detection, semantic retrieval, other setups or other operating systems.
