---
schema_version: 1
pair_id: developer-binary-lifecycle
topic_slug: developer-binary-lifecycle
language: en
counterpart: ../ko/developer-binary-lifecycle.md
title: "Developer Binary Lifecycle"
summary: "A source-local dev binary can replace the active executable and safely refresh an internally reproducible user projection without relaxing public-release authentication."
tags: [development, installation, version]
aliases: ["Dev install", "Local developer build"]
sources:
  - "repo:crates/hive-cli/build.rs#sha256:870578d55ee86e6414ff823c929b9eebe70b9ea4f829d4b6ce3d8d1f922c1991"
  - "repo:crates/hive-cli/src/main.rs#sha256:f8ea20501bfcc0226a8f720c7e18c5b772389aa423d3796ed8c440d1759bc671"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2ac47f0ba3f6a05f76c1e524ad9945d695e150c5665ed77dfb496e86ebab82d9"
  - "repo:scripts/dev-install.sh#sha256:675d29e359a127a994d3b7904d3c842b3dafd884b8e28659a0d2b21ef3fc2a79"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:1b6536e688f448bfa6d4ce7593f271fbd8e255da"
status: active
---

# Developer Binary Lifecycle

`scripts/dev-install.sh --sandbox` builds a source-local `product-dev` binary. `--global`
backs up and atomically replaces the existing active Hive executable; `--rollback` restores that
executable only while the active developer digest still matches. These paths do not initialize,
delete, migrate, or otherwise change canonical user preferences, knowledge, index, directives,
Skills, or project `.hive` data. A local binary prints `local developer build`, never the public
`developer test build` identity. Only a local `-dev` binary may use an internally reproducible
prior user manifest plus matching live managed bytes as its projection-refresh base. Public stable
and test releases keep the signed historical-base requirement and fail closed when it is missing.
