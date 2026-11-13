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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:945222ab2da1310babab674c81b28304d03015271720795afde44abd2324e0e1"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:70713e7d4c2de637a09b089dbd64716330e3e1e4859161e3f868c5f260d0960c"
  - "repo:crates/hive-render/src/lib.rs#sha256:334534e638f8266e8f4aa3b98e1092dd657bbe47f0d7c8ae40b932a64d5b1df9"
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:9fb5bf18a2bc89f0f990c89d5dda633455e22fa5e3fbf527f83f08ae0519dff2"
  - "repo:scripts/qualify-project-predecessors.py#sha256:4b40b7a8fd7800b0294db2d4addedc825443a9e10db87eea0d376d29ed591918"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:319ac812a6905a9810fdec56f81f0dc605b80895"
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
