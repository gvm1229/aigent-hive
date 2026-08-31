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
  - "repo:.github/workflows/release-publish.yml#sha256:6d9b351dfbe99fef461d642285a5bc37730ef6ba29d3c62d38c800bdd8e6220f"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:82db6eddc542a4e618f073469d5456d30173b3d16961e2cfb074988180e193d5"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:a0c3a87868199f81c144ed0895f4b564f3113f8b"
status: active
---

# Stable Release Discord Subscriber Notification

The stable-only workflow validates the Korean `update-summary`, banner, and protected environment.
After GitHub Release success, it sends the banner, then the summary. Test releases send nothing and
the notifier never prints the webhook URL. Before the banner, the summary bytes must match both its
versioned `.sha256` file and protected `AIGENT_HIVE_SUBSCRIBER_SUMMARY_DIGEST`. Main and child
bullets remain intact; the 2,000-character limit applies.
