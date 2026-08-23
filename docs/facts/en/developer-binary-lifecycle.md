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
  - "repo:crates/hive-cli/build.rs#sha256:26e622183b275a72d3145413763dd908eac72547866664f0a35886763475e991"
  - "repo:crates/hive-cli/src/main.rs#sha256:dec08219ab5f05c7f5b071083b64511b385004159a153f903e6ac8cd8793baa4"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:85b13d22add18756fa11e29fcc1ebcf84b18d143385991143a8453c29e3d0328"
  - "repo:scripts/dev-install.sh#sha256:675d29e359a127a994d3b7904d3c842b3dafd884b8e28659a0d2b21ef3fc2a79"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:f91816a46d44d57929cb0b580ca32ff4caa95053"
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
