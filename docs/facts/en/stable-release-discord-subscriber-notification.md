---
schema_version: 1
pair_id: stable-release-discord-subscriber-notification
topic_slug: stable-release-discord-subscriber-notification
language: en
counterpart: ../ko/stable-release-discord-subscriber-notification.md
title: "Stable Release Discord Subscriber Notification"
summary: "Approved subscriber copy is required before stable publication; skipped announcements use a publication-free recovery."
tags: [discord, release, subscriber]
aliases: [stable-release-discord]
sources:
  - "repo:.github/workflows/release-discord-notification.yml#sha256:c0824f4bc8fa52cca8020b01d9c9b533279dd159cac82fc492ceb4f535345e2d"
  - "repo:.github/workflows/release-publish.yml#sha256:241b1121f2b95690858514f9bcc8af27f6bd9084d125b14ff0e5df31cd401a3d"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/decisions/subscriber-approval-via-main-2026-11-12.md#sha256:0e919445e53ecef2030dfc422679f20a22b519a37dc64ada56494ceb871e92d9"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:de31d59272ef49533a99da9421da6d72bb1d9ae467d3424f1207db1aba6a5f36"
  - "repo:tests/results/discord-notification-0.12.0.md#sha256:48eb0e36599335dab851fb2631e0338e4581392baf256615f5778e6053e5eabf"
  - "repo:tests/results/discord-sectioned-notification-0.12.0.md#sha256:40aaa2a04ecd3d673e28b09be83c2fb4a81f82f0297d931cd9eea407131077d1"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:84ce26fa4163718ec88f9ee63c8e298f0dfa7aaa"
status: active
---

# Stable Release Discord Subscriber Notification

Reviewed main PR approves copy/digest; AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr is required. Missing/other values block validation/send. No per-release settings; old secret removable. Validate name, bytes, one-line hash, 2,000 characters and ordered lists/sections. Stable Release sends banner then copy; no test notices/webhook disclosure. Normal recovery keeps released-source bindings and duplicate guard. Approved 0.15.0 correction uses stable 0.15.1 reviewed copy: ID edit/delete-replace; no ID means marked post/manual deletion. Save new ID/safe receipt, no body/token. PR review alone never publishes/sends. Local tests do not prove Discord delivery. Details: the 2026-11-12 decision.
