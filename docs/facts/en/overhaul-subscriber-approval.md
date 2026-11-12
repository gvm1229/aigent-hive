---
schema_version: 1
pair_id: overhaul-subscriber-approval
topic_slug: overhaul-subscriber-approval
language: en
counterpart: ../ko/overhaul-subscriber-approval.md
title: "Overhaul subscriber notice approval"
summary: "Subscriber notices are preapproved until the whole overhaul plan is complete."
tags: [overhaul, release, subscriber]
aliases: [overhaul-subscriber-approval]
sources:
  - "repo:docs/plans/overhaul/README.md#sha256:05851d46a0eca9c392d44b6766cc4f2f2106fc9338abf985f0cddcf0e140dd02"
links: [stable-release-discord-subscriber-notification]
reviewed_revision: "git:88ecb7a13736634afca601fe354e53c88b0dc58e"
status: active
---

# Overhaul subscriber notice approval

On 2026-11-12, the maintainer approved all subscriber notices until the whole overhaul plan is complete. Notice and committed SHA-256 file reach main through the reviewed develop-to-main PR. Set `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr` once in `release-publication`; no per-release secret. Stable publication with `send_subscriber_update=true` needs no further notice approval. Release verification remains required.
