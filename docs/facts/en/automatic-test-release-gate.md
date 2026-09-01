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
  - "repo:.agents/directives/03-workflow.md#sha256:629d32bb289108bbc782e295e4ffda6a4a4d5006fbf151212db0cc79457391f0"
  - "repo:.github/workflows/release.yml#sha256:2f3760d989da12d1b07bfe706b9e7f1cd1e3121d3a53b18843e7825b56d86cac"
  - "repo:docs/public-test-product.json#sha256:127030c1f2d45cce3fa84861eedcefdc6454fceaca888f51663cb19272d10721"
  - "repo:scripts/check-test-release-gate.py#sha256:669dd6cb700c9a169babf8ddf530c8ae4a7114a01096c6d1ae1d0cb63351c54d"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:97928e522edbad00c2fc5c137f246c15fcad06a5"
status: active
---

# Automatic Numbered-Test Release Gate

No separate approval prompt for a numbered test. At milestone completion,
the agent writes the next package number, checked plan IDs, and product digest to
`docs/test-release-intent.json`. `check-test-release-gate.py` compares that intent and the accepted
product tree with the candidate. New product bytes proceed through candidate,
publication, and public acceptance automatically. Identical product trees, docs, plans, facts,
source-only Skills/directives, tests, CI, and notices are refused. Stable approval stays explicit.
