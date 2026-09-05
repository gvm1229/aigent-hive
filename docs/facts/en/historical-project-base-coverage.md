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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:a9535c1ae9e207b08dfce0d71fe9293e38168d8c7a38a4187ce8c6752be90ce4"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:d3bbe15942fc1ebdd38a81f62b37e9da2e47ac5db74259ca846fd1e1f08a3ac8"
  - "repo:crates/hive-render/src/lib.rs#sha256:4ce1a5feac500ede6f71c6b1b2ba0764e189ab48b6cae971122e8d5e538eee42"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:107433e7cf842a5c1034f669d6afc36a074e23a728f27052c1ffe7c96da9bb02"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Historical Project Base Coverage

The `0.9.2` historical marker now renders its stored Markdown backend, and an upgrade records
current local overrides in one apply. A read-only PortareFolium copy passed scan, dry-run, apply,
validate, local-marker preservation, foreign-byte preservation, and tampered-ledger no-mutation.
Public-artifact acceptance remains pending `0.9.5-test.4`.
