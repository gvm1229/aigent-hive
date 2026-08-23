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
  - "repo:crates/hive-cli/build.rs#sha256:a900257ee09c03ad6043903e9d4dec4feb2a7bd1966f330840d8573ea7a62b8c"
  - "repo:crates/hive-cli/src/main.rs#sha256:a76209fd83892c171590fc2c84d9bbe294eafc0158083e0da635e381ecf6c65e"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:359e033f6bad6a6145820efb0a079a6643d4774a6d9b8e1b560d9d4e156df5be"
  - "repo:scripts/dev-install.sh#sha256:675d29e359a127a994d3b7904d3c842b3dafd884b8e28659a0d2b21ef3fc2a79"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:838842805e453e0508d054e4aa67d7a59b3aa53f"
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
