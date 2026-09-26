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
  - "repo:.agents/directives/references/ci-and-candidates.md#sha256:0de7ded9fe08a1e1e2ff78c4653c3781a8b492b59e6f9498ed07f89a5a56c18a"
  - "repo:.github/workflows/release.yml#sha256:993bf1709b27d5f6f5c18df46dab392fbb44570ecb3fcc9e0c4bd321fc5dc664"
  - "repo:docs/public-test-product.json#sha256:5c8c2bf9f0e1b3d5134ee61ba4d8312ec2deaec369bf13e45d56384d97640dd2"
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
