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
  - "repo:crates/hive-cli/build.rs#sha256:a3b83a223dd02e759bdee84a5e1848f01f614834177d6a91de8d23d073feebff"
  - "repo:crates/hive-cli/src/main.rs#sha256:e71b5c908435cda30208af5119f184e80d84b9a9d44934cec5ae7b64af95eee7"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:4795665ae984b818ed5781b43884bec10088ddb64287465dcbfafbed81b965a0"
  - "repo:scripts/dev-install.sh#sha256:6b9454df5ac9abecff21c3a8a45c3c840d32dcee5d00d1f2c87170e2fd5d7b33"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
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
