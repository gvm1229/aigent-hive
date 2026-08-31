---
schema_version: 1
pair_id: stable-release-discord-subscriber-notification
topic_slug: stable-release-discord-subscriber-notification
language: en
counterpart: ../ko/stable-release-discord-subscriber-notification.md
title: "Stable Release Discord Subscriber Notification"
summary: "After a successful stable GitHub Release, Aigent Hive sends its Korean banner first and then its verified Korean subscriber update through the protected release environment."
tags: [discord, release, subscriber]
aliases: [stable-release-discord]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:35420bffac94da9392c605c6512edffa879458e177e892e407d9a979feffc693"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:70d18c327fbd4886f11e54d01b3ca7c5f8d9582e471fa2954feabe1d5e140c99"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:349d2468d79f81708a3586a5fdde49d78aec2736"
status: active
---

# Stable Release Discord Subscriber Notification

The stable-only release workflow validates the Korean `update-summary` payload, the banner, and
the protected environment secret before publication. After the GitHub Release succeeds, it sends
the banner first. It sends the Korean subscriber summary only after that request succeeds. Test
releases send no Discord request. The notifier never prints the webhook URL. Protected-environment
delivery tests for `0.9.3` and `0.9.4` succeeded without a stable release or npm publication. The
maintainer accepted the Discord display. Each Korean summary appeared below its banner.
A versioned approval digest must match before the banner; main bullets, child bullets, and the
2,000-character message limit remain preserved.
