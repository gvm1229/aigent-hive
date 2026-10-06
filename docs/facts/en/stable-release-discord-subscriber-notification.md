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
  - "repo:.github/workflows/release-discord-notification.yml#sha256:e4d2b64ca1d25902b8c6701b6e5395cd47049be8169ec49d03d6666df3f3930c"
  - "repo:.github/workflows/release-publish.yml#sha256:240f5376bc7ea26109a1c34be965c6e74f9dc840e400af78bb1e55a2bac01263"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:82db6eddc542a4e618f073469d5456d30173b3d16961e2cfb074988180e193d5"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:ac0ce696cb993750ca9a7406d151036d75387b9c"
status: active
---

# Stable Release Discord Subscriber Notification

Stable publication requires approved Korean copy and banner; the external approval digest is checked before uploads. Missing or disabled notification blocks stable publication. After GitHub Release success, send banner then summary. Tests send nothing; never print webhook URLs. Versioned copy/sidecar/external digest checks retain nested bullets and the 2,000-character limit. The registrar uses existing gh access after explicit wording approval; it never rewrites, publishes or sends. Retry the same digest; changed copy needs new approval. A separate recovery workflow binds a published stable and skipped successful publication, rejects duplicate receipts, and sends approved copy without republishing.
