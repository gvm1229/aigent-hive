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
  - "repo:crates/hive-cli/build.rs#sha256:fd4fe508be6d9abb3a412a3163544a154802d335ca92f23f28783096026752da"
  - "repo:crates/hive-cli/src/main.rs#sha256:ca5d0af23e3719732dec1a6d3a38dcde959a7dfa1426ef7f7be9edc2623b0a4d"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b57e2ac894503cf91d6efc41940284e79cc11e2ca70f6718d5b88926a917ec67"
  - "repo:scripts/dev-install.sh#sha256:675d29e359a127a994d3b7904d3c842b3dafd884b8e28659a0d2b21ef3fc2a79"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:0340f8a26d14ebcc134c14e421d605f8022912f8"
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
