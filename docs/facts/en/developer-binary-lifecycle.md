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
  - "repo:crates/hive-cli/src/main.rs#sha256:729c7f4664b1395df0cdeb15c36edc610f59ee200bded3481698b54da6260b1d"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9a2ca4c65c5b7880464c7804d224cad91de28493eaf189aa63b1ed439806dde1"
  - "repo:scripts/dev-install.sh#sha256:c2c301aca92316edd613d4548ed041baa421958a126c182386856979aeef0016"
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
