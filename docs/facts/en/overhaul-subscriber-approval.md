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
  - "repo:docs/plans/overhaul/README.md#sha256:6377a80d8cdce79b25ca5d69ccdf3805a02acda8f962c8cca4811285491d18b7"
links: [stable-release-discord-subscriber-notification]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Overhaul subscriber notice approval

On 2026-11-12, the maintainer approved all subscriber notices until the whole overhaul plan is complete. Notice and committed SHA-256 file reach main through the reviewed develop-to-main PR. Set `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr` once in `release-publication`; no per-release secret. Stable publication with `send_subscriber_update=true` needs no further notice approval. Release verification remains required.
