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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:1bbaf363f78a69358c7184ea4fdac804568200926de169ad939da77999d99c85"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:f5c90bf5b90baef8d7a5ec2228d0d23338ea95aef2c8078b7656c4df7d6ff600"
  - "repo:crates/hive-render/src/lib.rs#sha256:c2abca0c0461baebddc1fe16992eac0e8ec6bdf0dbcf74daf2343a93b9824cea"
  - "repo:docs/archive/plans/releases/0.9.5/release-0.9.5-stable-publication.md#sha256:70ed823701fa0ae8be728d97b8705846f0eaa50e6e8758425d439bfee4d1334c"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
  - "repo:scripts/check-project-base-coverage.py#sha256:9fb5bf18a2bc89f0f990c89d5dda633455e22fa5e3fbf527f83f08ae0519dff2"
  - "repo:scripts/qualify-project-predecessors.py#sha256:80f839a04103dc25d74259acd5b1daa7d933fa58e98eb605206eb1a046c67f58"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
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
