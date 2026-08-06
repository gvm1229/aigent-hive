---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Optional one-prompt bootstrap preserves global/project scope; Korean setup keeps product terms exact."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:072bfc2c939e2a2e2e26f897b4cca9a876bd9d4be28adc8db14bafe7e5bb941b"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:35f3cace4b6297a298b8b59db208b3d8ecfd82331758fb6bd34dd1ec03aa8ec7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:12fbe0128457b6c9d0a4f32744eb3eb678c715129bb04bfc64d6f8cef5c073bc"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:dc4ddb908ecef82f197b0055cea3adb36602f0b924f8a285add60c1d9b7f7ec7"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:d211300dea66781251306e376e43bf9e798504ef"
status: active
---

# Global Onboarding

Manual order: CLI installation, terminal host activation, global user-scope setup, explicit
project setup. The optional one-prompt path selects one exact release, checks Node.js/npm,
activates one host, and starts global setup without project inspection.

Schema-1 `0.7.0` recovery requires matching saved-preference digest, inventory, and live digests.
Local edits or an unknown inventory preserve active bytes. `0.9.0-test.3` recovery requires its
frozen `setup-hive` digest plus the selected projection.

Korean global setup retains product terms such as `Skill` and `Wiki` exactly. The canonical
`setup-hive` Skill owns exact Korean samples and source-to-projection regression coverage.
