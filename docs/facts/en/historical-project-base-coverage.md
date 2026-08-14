---
schema_version: 1
pair_id: historical-project-base-coverage
topic_slug: historical-project-base-coverage
language: en
counterpart: ../ko/historical-project-base-coverage.md
title: "Historical Project Base Coverage"
summary: "A declared project upgrade source range requires exact authenticable full bases and matrix acceptance."
tags: [migration, project-upgrade, regression, release]
aliases: ["Historical base parity"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:162954ace665a9f30166cf241abe18b5e1168ebd8e862c106819a142d496bd46"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:0aa8c272002f64443b8204e80f5744c02474e4621ca807d28cfe36ff3bdb49f6"
  - "repo:crates/hive-render/src/lib.rs#sha256:9ac7b87b5dde4f582027a219d4695c9158115e99041f10e304089cce4f55a30e"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# Historical Project Base Coverage

For the future `0.9.5` candidate, the complete project-base source set is `0.9.1` through `0.9.4`.
The coverage checker derives a digest-bound report and rejects a same-major source range below `0.9.1`.
The compiled CLI matrix passed scan, dry-run, rollback, apply, and validation for `0.9.1` through
`0.9.3`, while the `0.9.4` frozen base is byte-identical to its release tag. Missing or tampered
bases stop before apply and preserve project and foreign bytes.
