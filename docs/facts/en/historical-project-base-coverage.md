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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:af09aadf2ddfabc082dfac9ae6c8233c2fe48f964db8996063848838f04f68c5"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:5d1ded97d4dfa1fcc3bbac149ededed530ce4d384eb8b87b360c441fbbce8deb"
  - "repo:crates/hive-render/src/lib.rs#sha256:644c0b46c68ceaeb9cb798f2c076f301ed12be889121cd4c086f23ecd50e69ae"
  - "repo:scripts/accept-public-hive.py#sha256:59a78bea773c38e18248fb6cdefe6e612a69d8f46ae0139eeff7a7b30fa455f2"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:32bf5dfd2cd2663070174a4efebee39d7fa98935"
status: active
---

# Historical Project Base Coverage

The `0.9.2` historical marker now renders its stored Markdown backend, and an upgrade records
current local overrides in one apply. A read-only PortareFolium copy passed scan, dry-run, apply,
validate, local-marker preservation, foreign-byte preservation, and tampered-ledger no-mutation.
Public-artifact acceptance remains pending `0.9.5-test.4`.
