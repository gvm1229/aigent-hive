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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:81f1199edfd9c19f5ef79fcdd391d72adc72a0567bdba058b1b3c3cb49ce9a44"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:f1b45ed3cfd4ae5feb40574c0825fbcc26efc67c95dc1032812656221a776f88"
  - "repo:crates/hive-render/src/lib.rs#sha256:54c93f6fdf51beda50d73eba8e3ea0a06e2441f69f93a8b40440a9d2fc37d767"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Historical Project Base Coverage

The `0.9.2` historical marker now renders its stored Markdown backend, and an upgrade records
current local overrides in one apply. A read-only PortareFolium copy passed scan, dry-run, apply,
validate, local-marker preservation, foreign-byte preservation, and tampered-ledger no-mutation.
Public-artifact acceptance remains pending `0.9.5-test.4`.
