---
schema_version: 1
pair_id: developer-binary-lifecycle
topic_slug: developer-binary-lifecycle
language: en
counterpart: ../ko/developer-binary-lifecycle.md
title: "Developer Binary Lifecycle"
summary: "A source-local dev binary can temporarily replace the active executable without changing canonical user data."
tags: [development, installation, version]
aliases: ["Dev install", "Local developer build"]
sources:
  - "repo:crates/hive-cli/build.rs#sha256:870578d55ee86e6414ff823c929b9eebe70b9ea4f829d4b6ce3d8d1f922c1991"
  - "repo:crates/hive-cli/src/main.rs#sha256:afe80f6416d7d9f1d8c9599a9306c396b6c5ada2730c9b60174906626e06e87a"
  - "repo:scripts/dev-install.sh#sha256:4e78ac1c159ce03be44374268de3ebfd53af3826029af88180186599490bd22f"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:b93e3e14950a2373fd99bfcf98daf71b1e562d3e"
status: active
---

# Developer Binary Lifecycle

`scripts/dev-install.sh --sandbox` builds a source-local `product-dev` binary. `--global`
backs up and atomically replaces the existing active Hive executable; `--rollback` restores that
executable only while the active developer digest still matches. These paths do not initialize,
delete, migrate, or otherwise change canonical user preferences, knowledge, index, directives,
Skills, or project `.hive` data. A local binary prints `local developer build`, never the public
`developer test build` identity.
