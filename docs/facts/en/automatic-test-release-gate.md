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
  - "repo:.agents/directives/references/ci-and-candidates.md#sha256:b1b2a41cfcad009d6561bcc57e94bdd4ebcaee771931ed3369d85a6935dcd52c"
  - "repo:.github/workflows/release.yml#sha256:6fa640b5ce08594bdcbccd28530846ff7a34f7dfd6f918f4b62a860555ed7567"
  - "repo:docs/public-test-product.json#sha256:cfd7fc3e8e8e13af996c104384d890704a857972e0837bb74656f2f1acef4648"
  - "repo:scripts/check-test-release-gate.py#sha256:75a37fd28d2aaf302c7079088b54c4cedb4060bd4497f4aa9219198ff024ce95"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:140913559ab6a428546e25fba7c7efbb775e5eef"
status: active
---

# Automatic Numbered-Test Release Gate

No separate approval prompt for a numbered test. At milestone completion,
the agent writes the next package number, checked plan IDs, and product digest to
`docs/test-release-intent.json`. `check-test-release-gate.py` compares that intent and the accepted
product tree with the candidate. New product bytes proceed through candidate,
publication, and public acceptance automatically. Identical product trees, docs, plans, facts,
source-only Skills/directives, tests, CI, and notices are refused. Stable approval stays explicit.

Required CI must pass for the exact candidate SHA. Record changed retry inputs; investigate
all failed jobs and OS-gated forms after a second same-family failure. No unchanged retry.
