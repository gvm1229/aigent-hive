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
  - "repo:crates/hive-cli/build.rs#sha256:248e82fd5ea76c34fca96b454e8192c30e0f24cd470b0f609fa2df6f47bf2eed"
  - "repo:crates/hive-cli/src/main.rs#sha256:c7dfa1349f5239cd9df726f0fbf7d9a73a3da5a90218551cac420c0d7331a9bd"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d16fe3cf14d23b5781bbb960884146b17c3770916a5572215e4922a4423f28d1"
  - "repo:docs/research/developer-install-recovery.md#sha256:552390190deb914cf2dfd84106461b994b7ee3587a5ce954102cfa62f74d9925"
  - "repo:scripts/dev-install.sh#sha256:49d7e923900b37bf30b8678e150bfb7f21ad4427876da5031b46aac04df8560e"
  - "repo:scripts/recover-dev-install.py#sha256:128ee7d1aa5dbd883f963df2ec9eb1449771d4343bda1deef1e94161d7bf964b"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
status: active
---

# Developer Binary Lifecycle

Source development uses a sandbox binary and an isolated user root. Global developer replacement and its legacy monitor deployment are refused before writes. Existing binary-only rollback cannot restore changed user projections. The Windows recovery tool authenticates the source CLI and stages only generated package files and ownership records in a reversible backup. Official CLIs recreate the projection and update the npm package. Actual 0.13.0 validation and interactive update passed; knowledge, settings and records were preserved. Public releases retain ownership checks. Same-version local dev projection authentication remains limited to matching live managed bytes.
