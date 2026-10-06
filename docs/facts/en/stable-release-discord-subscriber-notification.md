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
  - "repo:.github/workflows/release-discord-notification.yml#sha256:0f7dfee71bb06b4af4ccf2ca26122fec42e02b21d5158e939561120017da3d68"
  - "repo:.github/workflows/release-publish.yml#sha256:240f5376bc7ea26109a1c34be965c6e74f9dc840e400af78bb1e55a2bac01263"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:04e76513d36ede4c84e57d01b4a22335ac19ba0bab2bc604c9af1f7cab5f348c"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
  - "repo:tests/results/discord-notification-0.12.0.md#sha256:48eb0e36599335dab851fb2631e0338e4581392baf256615f5778e6053e5eabf"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:db4abad72eba0d76187c8abff5f7791f455cf88b"
status: active
---

# Stable Release Discord Subscriber Notification

Stable publication requires approved copy/external digest before uploads; disabled notification blocks publication. After GitHub Release success, send banner then summary. Tests send nothing; never print webhook URLs. Validate versioned copy/sidecar/external digest and the entire 2,000-character limit without reformatting. Accept historic flat lists and ordered nonempty new-feature/fix/improvement sections with separators. Registration needs wording approval/existing gh; it never rewrites, publishes or sends. Retry the same digest; changed copy needs new approval. Recovery binds stable publication. Approved main/develop revisions use digest-scoped duplicate guards; no republishing. 0.12.0 recovery accepted both requests.
