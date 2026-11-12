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
  - "repo:crates/hive-cli/build.rs#sha256:dacee7913c94de791fe4ad4b7532e3ee40f3b44fb0982df8962e54a9f3bdb56f"
  - "repo:crates/hive-cli/src/main.rs#sha256:02cb26b5329922f7344a35ef9450ed450895df65836bacdec1d091303fa63bfb"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d47f8ba8f5e3e88783cfc992cb88475d7cf47fa3459baecb434610e7f91c2d45"
  - "repo:docs/research/developer-install-recovery.md#sha256:552390190deb914cf2dfd84106461b994b7ee3587a5ce954102cfa62f74d9925"
  - "repo:scripts/dev-install.sh#sha256:49d7e923900b37bf30b8678e150bfb7f21ad4427876da5031b46aac04df8560e"
  - "repo:scripts/recover-dev-install.py#sha256:128ee7d1aa5dbd883f963df2ec9eb1449771d4343bda1deef1e94161d7bf964b"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# Developer Binary Lifecycle

Source development uses a sandbox binary and an isolated user root. Global developer replacement and its legacy monitor deployment are refused before writes. Existing binary-only rollback cannot restore changed user projections. The Windows recovery tool authenticates the source CLI and stages only generated package files and ownership records in a reversible backup. Official CLIs recreate the projection and update the npm package. Actual 0.13.0 validation and interactive update passed; knowledge, settings and records were preserved. Public releases retain ownership checks. Same-version local dev projection authentication remains limited to matching live managed bytes.
