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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:d5e36f5d1cb6080fa7952b1cf4354e7d54f0df12bc0799d758ced53d7f083b84"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:d3bbe15942fc1ebdd38a81f62b37e9da2e47ac5db74259ca846fd1e1f08a3ac8"
  - "repo:crates/hive-render/src/lib.rs#sha256:9b9fd4e3a9734087d452f85e02a21904f60d6233de8bbc566459d4949bf13202"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:78f471770ca909180c73ac1e65783f0a9ebd762a12ad70411575197578459ce3"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Historical Project Base Coverage

The `0.9.2` historical marker now renders its stored Markdown backend, and an upgrade records
current local overrides in one apply. A read-only PortareFolium copy passed scan, dry-run, apply,
validate, local-marker preservation, foreign-byte preservation, and tampered-ledger no-mutation.
Public-artifact acceptance remains pending `0.9.5-test.4`.
