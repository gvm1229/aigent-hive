---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Optional one-prompt bootstrap and numbered setup preserve global/project scope while safely recovering supported legacy user projections."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:072bfc2c939e2a2e2e26f897b4cca9a876bd9d4be28adc8db14bafe7e5bb941b"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:35f3cace4b6297a298b8b59db208b3d8ecfd82331758fb6bd34dd1ec03aa8ec7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:12fbe0128457b6c9d0a4f32744eb3eb678c715129bb04bfc64d6f8cef5c073bc"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2f653de651b5a7b540efab9522e5808156cc527ac6b3cda20df4a3f943b66c07"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:d211300dea66781251306e376e43bf9e798504ef"
status: active
---

# Global Onboarding

Manual order: CLI installation, terminal host activation, global user-scope setup, explicit
project setup. The optional one-prompt path selects one exact release, checks Node.js/npm,
activates one host, and starts global setup without project inspection.

Schema-1 `0.7.0` recovery requires matching saved-preference digest, legacy inventory, and live
file digests. It adds later Codex metadata and records a schema-2 base. Legacy local edits or an
unknown inventory block the migration and preserve active bytes. `0.9.0-test.3` host recovery
requires its frozen `setup-hive` digest plus the current selected projection.
