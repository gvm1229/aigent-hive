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
  - "repo:docs/research/usage-stop-notice-0.12.1.md#sha256:c8b0b2678355ce54e64eed1d2e0100049c54bd59548947f6f1eb37d637455568"
  - "repo:tests/results/usage-stop-notice-windows-0.12.1.json#sha256:12746751e84b05243501dab68a86f2ba228424ac13d00fb12f9768c0ceefadf0"
links: [installed-usage-guard, quota-reset-session-control]
reviewed_revision: "git:d6c851d2dee4cea3e4d48c6f30c96d24f8c678c0"
status: active
---

# Usage Protection Stop Notice

The 0.12.1 product uses a separate Windows notice for Hive's cause, confirmation state, app preservation and manual resume. Production omits test labels. Resume instructions copy an exact command for external PowerShell; the window never runs it. Closing only dismisses the window; it never acknowledges, unlocks or resumes work. Reset, 60s observation gap, partial confirmation, read failure and acknowledgement remain distinct; the original cause persists. Windows 144 DPI, 560/460 widths, 844ms entry, accessible close, Esc event routing and unchanged state pass. Display-only evidence excludes live stops, installation, other OS, 96/192 DPI and physical keyboard input.
