---
schema_version: 1
pair_id: automatic-user-projection-refresh
topic_slug: automatic-user-projection-refresh
language: en
counterpart: ../ko/automatic-user-projection-refresh.md
title: "Automatic User Projection Refresh"
summary: "The 0.9.5 plan requires a bare binary update to refresh only the authenticated saved user-install scope."
tags: [installation, migration, projection, update]
aliases: ["Post-update projection refresh"]
sources:
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:dcbb1b3571a08a9da251deb486ab5a7c1fab7e052139f222c42860c09e4c354a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:scripts/accept-public-hive.py#sha256:59a78bea773c38e18248fb6cdefe6e612a69d8f46ae0139eeff7a7b30fa455f2"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:b8e4c79437ea61cce0c012d37a8fed97860bf287"
status: active
---

# Automatic User Projection Refresh

Bare `hive update` keeps the stable channel by default. `--channel test` is an explicit prerelease
selection, and `--user-root` plus `--confirm` supports an isolated acceptance root. The direct
installer accepts its optional unsigned-signer fallback while still rejecting bound placeholders.
Public `test.6 → test.7` user-projection evidence remains required.
