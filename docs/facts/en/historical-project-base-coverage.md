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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:f2ff844d6567f78dde41ec78d1f030cd76a050cdd6fb977a4d8bf8401b2dffd7"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:35f36abe56eaa66ef5047a2425602ddc9d2b926b37c5c3ebf0a38729d3dc45c8"
  - "repo:crates/hive-render/src/lib.rs#sha256:388ee4c000ff000246d04a626d3b2018d284bf79f85e645d6145a2bdac99254e"
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:6c90b2a4b1f84507f56a80ed540f6e97c9938b6f91a7ab087cc8347c8cfadf26"
  - "repo:scripts/qualify-project-predecessors.py#sha256:c4e75d248a201b433c01423436645e32920543d58360dbd2b098f9fc3d909278"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:fc288bed8f925b89bfd0ed67b808cfdd0722a70b"
status: active
---

# Historical Project Base Coverage

The 0.9.2 marker uses its stored Markdown backend; one apply records local overrides.
A read-only PortareFolium copy passed scan, preview, apply, validate, preservation and
tampered-ledger rejection. REL95-004 records 0.9.5-test.15 acceptance; test.4 is not pending.
That historical evidence does not qualify 0.11.0.

Every public stable predecessor since 0.9.1 must have a migration route and an authentic
old-CLI fixture. Missing releases fail candidate gates. Windows verified all nine predecessors
through 0.10.3 for upgrade, local preservation, rollback and interrupted recovery to 0.11.0.
This does not prove every historical setting combination. Public binaries exclude failure injection.
