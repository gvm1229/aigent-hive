---
schema_version: 1
pair_id: usage-stop-notice
topic_slug: usage-stop-notice
language: en
counterpart: ../ko/usage-stop-notice.md
title: "Usage Protection Stop Notice"
summary: "Production cause, state and manual resume copy with Windows display evidence."
tags: [notice, usage, windows]
aliases: []
sources:
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:e4bb6407c618423280f980006c64aa6c6c2eb17f52d2fee44df5abe8552af36b"
  - "repo:tests/results/usage-stop-notice-windows-0.12.1.json#sha256:406e5d0573cd42c27dcf980b5517e5f29ff324baaefa849c47ee6c6407b4b2c5"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:ee69dbe35105053232d9029e50007519f6052fbe"
status: active
---

# Usage Protection Stop Notice

The 0.12.1 product uses a separate Windows notice for Hive's cause, confirmation state, app preservation and manual resume. Production omits test labels. Closing only dismisses the window; it never acknowledges, unlocks or resumes work. Reset, 60s observation gap, partial confirmation, read failure and acknowledgement remain distinct; the original cause persists. Windows 144 DPI, 560/460 widths, 875ms entry, accessible close, Esc event routing and unchanged state pass. Display-only evidence excludes live stops, installation, other OS, 96/192 DPI and physical keyboard input.
