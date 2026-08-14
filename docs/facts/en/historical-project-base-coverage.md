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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:8943d5559309ea5b084f211a4bda523bc88e1e5f6afdd23b6b1226e85a652bf5"
  - "repo:crates/hive-render/src/lib.rs#sha256:019c4b9187834d210c659a1ade13f9a30d5b04c45088e5184e04d0340797712e"
  - "repo:harness/release/0.9.4/migration-table.json#sha256:96fad7ef16ba8404447130124cdb21ac3bd4350492b438dbac88891f1ca1c3b3"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# Historical Project Base Coverage

Every version declared as a project-upgrade source must have an exact full historical project base
in the release bundle and must be authenticated before mutation. The migration-table range is valid
only when the packaged binary and release test matrix prove that coverage. Missing or tampered bases
must stop before apply and preserve all project bytes.
