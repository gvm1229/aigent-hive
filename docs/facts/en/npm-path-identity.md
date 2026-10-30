---
schema_version: 1
pair_id: npm-path-identity
topic_slug: npm-path-identity
language: en
counterpart: ../ko/npm-path-identity.md
title: "npm Path Identity and Arguments"
summary: "Canonical ownership checks keep the original npm argument spelling."
tags: [npm, preservation, uninstall, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/update_activation/removal.rs#sha256:282796ffeb06e2aa788402c0a950b8a35159a271a4ccde6daaf313df052b8451"
  - "repo:tests/results/runs/20261028T032255-d34a85916924.md#sha256:9838c31c45fc49982c643fcf1aa444eca433bcd61bb436c7f8bbf7d51cbe4bf1"
links: [hive-preserving-uninstall]
reviewed_revision: "git:4f4d621911ee0c15efab21a4e7d27b7022ea1101"
status: active
---

# npm Path Identity and Arguments

Compare canonical paths for npm removal ownership, but pass npm its original absolute path spelling. Do not introduce the internal Windows verbatim prefix into npm arguments.

The 0.14.0 preserving-removal request was verified with real npm in an isolated Windows VM: user data and an independent package remained unchanged. This does not qualify a personal installation, actual host/monitor control or public 0.14.0 artifacts.
