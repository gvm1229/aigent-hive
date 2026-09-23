---
schema_version: 1
pair_id: automatic-test-release-gate
topic_slug: automatic-test-release-gate
language: en
counterpart: ../ko/automatic-test-release-gate.md
title: "Automatic Numbered-Test Release Gate"
summary: "A completed authorized product milestone publishes and accepts one numbered test automatically; source-only or identical product changes cannot create one."
tags: [automation, product, release]
aliases: ["numbered public test gate"]
sources:
  - "repo:.agents/directives/03-workflow.md#sha256:fc79be9eea10702b0770715b5fe154e41a2e707557171be3819d493ad5a85ed5"
  - "repo:.github/workflows/release.yml#sha256:993bf1709b27d5f6f5c18df46dab392fbb44570ecb3fcc9e0c4bd321fc5dc664"
  - "repo:docs/public-test-product.json#sha256:68d42560c632a7d3b5344664b4cf4ac0aab2d22ca65cf322600fb1e780520d98"
  - "repo:scripts/check-test-release-gate.py#sha256:75a37fd28d2aaf302c7079088b54c4cedb4060bd4497f4aa9219198ff024ce95"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:dd63333a702a7a89585d101d2b9d043ebd0987d8"
status: active
---

# Automatic Numbered-Test Release Gate

No separate approval prompt for a numbered test. At milestone completion,
the agent writes the next package number, checked plan IDs, and product digest to
`docs/test-release-intent.json`. `check-test-release-gate.py` compares that intent and the accepted
product tree with the candidate. New product bytes proceed through candidate,
publication, and public acceptance automatically. Identical product trees, docs, plans, facts,
source-only Skills/directives, tests, CI, and notices are refused. Stable approval stays explicit.
