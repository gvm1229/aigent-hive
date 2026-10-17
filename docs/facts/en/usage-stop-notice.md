---
schema_version: 1
pair_id: usage-stop-notice
topic_slug: usage-stop-notice
language: en
counterpart: ../ko/usage-stop-notice.md
title: "Usage Protection Stop Notice"
summary: "Official logo and close-only Windows notification."
tags: [notice, usage, windows]
aliases: []
sources:
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:58148052d858dec443c5d8231c0ad91c7748dfd8e54e2a5b5939e6fc5790cffd"
  - "repo:tests/results/usage-stop-notice-only-windows-0.12.1.json#sha256:89acbc9c15cf5c0988ff69f823cdee69894f07957898d4f80d689609b64e8c4c"
  - "repo:tests/results/usage-stop-notice-windows-0.12.1.json#sha256:12746751e84b05243501dab68a86f2ba228424ac13d00fb12f9768c0ceefadf0"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:da2797509ea7e0ef94783877e3c2cb69c6bfd490"
status: active
---

# Usage Protection Stop Notice

The 0.12.1 Windows notice is notification only. The canonical Hive plugin logo supplies its title icon and body image. Cause, confirmation state, app preservation, detection time and direct Codex resume instructions remain visible. Only close actions exist; there is no command copy, acknowledgement, unlock or execution. Closing preserves the event and never resumes work. Reset, 60s sensor gap, partial confirmation and read failure remain distinct. Current Windows 144 DPI, 560/460 widths, 13 state/UI cases, actual entry and accessible close pass with the original Codex app preserved. No new live halt, physical keyboard, 96/192 DPI or other OS is claimed.
