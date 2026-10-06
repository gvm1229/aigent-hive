---
schema_version: 1
pair_id: stable-release-discord-subscriber-notification
topic_slug: stable-release-discord-subscriber-notification
language: en
counterpart: ../ko/stable-release-discord-subscriber-notification.md
title: "Stable Release Discord Subscriber Notification"
summary: "Explicit stable opt-in sends the approved Korean banner and subscriber update after GitHub Release success; publication alone sends no message."
tags: [discord, release, subscriber]
aliases: [stable-release-discord]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:a90ceb2d2eb2a176057e7afd20bd34e8bef8e9c176a1e80ddbe16876f0d9309c"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:82db6eddc542a4e618f073469d5456d30173b3d16961e2cfb074988180e193d5"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:bb224faca9a37b022f5eafe1e8d36c87182836ae"
status: active
---

# Stable Release Discord Subscriber Notification

The stable-only workflow validates Korean copy and banner before publication. After GitHub
Release success, explicit `send_subscriber_update=true` sends banner then summary. Default false
permits publication without sending. Tests send nothing; webhook URLs
are never printed. Delivery checks the versioned file, sidecar, and external approval digest,
preserving main/child bullets and the 2,000-character limit. After explicit wording approval,
`register-stable-summary-approval.py` checks the supplied digest and files before registering it
with existing `gh` access. No manual GitHub setup per release, rewriting, or sending during
registration. Retry the same approval on failure; changed wording needs new approval.
