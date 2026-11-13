---
schema_version: 1
pair_id: developer-binary-lifecycle
topic_slug: developer-binary-lifecycle
language: en
counterpart: ../ko/developer-binary-lifecycle.md
title: "Developer Binary Lifecycle"
summary: "Global developer replacement is retired; authenticated Windows recovery preserves data while restoring ordinary stable updates."
tags: [development, installation, version]
aliases: ["Dev install", "Local developer build"]
sources:
  - "repo:crates/hive-cli/build.rs#sha256:c7738445caec1081e42e203eb5568eae0d997a0009037a1d74c8c4804e3de8b7"
  - "repo:crates/hive-cli/src/main.rs#sha256:3d1ea658e591f1b311e578d6f4f0d2fcc03de28361abf14cbd3dafc2c4e0151c"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9903f35f13ae28c9853cdf99ab17c42876e8a5b12c91100e7f4c51cc04147c09"
  - "repo:docs/research/developer-install-recovery.md#sha256:552390190deb914cf2dfd84106461b994b7ee3587a5ce954102cfa62f74d9925"
  - "repo:scripts/dev-install.sh#sha256:49d7e923900b37bf30b8678e150bfb7f21ad4427876da5031b46aac04df8560e"
  - "repo:scripts/recover-dev-install.py#sha256:128ee7d1aa5dbd883f963df2ec9eb1449771d4343bda1deef1e94161d7bf964b"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
status: active
---

# Developer Binary Lifecycle

Source development uses a sandbox binary and an isolated user root. Global developer replacement and its legacy monitor deployment are refused before writes. Existing binary-only rollback cannot restore changed user projections. The Windows recovery tool authenticates the source CLI and stages only generated package files and ownership records in a reversible backup. Official CLIs recreate the projection and update the npm package. Actual 0.13.0 validation and interactive update passed; knowledge, settings and records were preserved. Public releases retain ownership checks. Same-version local dev projection authentication remains limited to matching live managed bytes.
