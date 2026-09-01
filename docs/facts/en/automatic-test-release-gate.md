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
  - "repo:.agents/directives/03-workflow.md#sha256:9133a979df415b6df62b8669e3d0a1a6c069c9a441451948f16473bd5527878d"
  - "repo:.github/workflows/release.yml#sha256:b530af22eb2e6f932558e2f2699038d59c1bd8f2c48cedf37433417dac4a66bf"
  - "repo:docs/public-test-product.json#sha256:127030c1f2d45cce3fa84861eedcefdc6454fceaca888f51663cb19272d10721"
  - "repo:scripts/check-test-release-gate.py#sha256:06af753c2dc6a4568e5173676c455b3e618ab9daeea8aa91230e892613241c29"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:97928e522edbad00c2fc5c137f246c15fcad06a5"
status: active
---

# Automatic Numbered-Test Release Gate

No separate approval prompt for a numbered test. At milestone completion,
`check-test-release-gate.py` compares the accepted product tree with the candidate and requires
checked non-release implementation plan IDs. New product bytes proceed through candidate,
publication, and public acceptance automatically. Identical product trees, docs, plans, facts,
source-only Skills/directives, tests, CI, and notices are refused. Stable approval stays explicit.
