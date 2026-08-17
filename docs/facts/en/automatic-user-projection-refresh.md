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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:c685f83d115dc2764780f7cf52a8c949268cde0b2fa7efd36ad99b173c883b25"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:scripts/accept-public-hive.py#sha256:59a78bea773c38e18248fb6cdefe6e612a69d8f46ae0139eeff7a7b30fa455f2"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:5f3bb933d6df91864f005041ad13d223ecdfbc9c"
status: active
---

# Automatic User Projection Refresh

Bare `hive update` keeps the stable channel by default. `--channel test` is an explicit prerelease
selection, and `--user-root` plus `--confirm` supports an isolated acceptance root. The activated
binary still refreshes only the authenticated saved host scope; public `test.4 → test.5` evidence
remains required.
