---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Optional bootstrap preserves scope; Korean setup keeps exact terms, user context, and all built-in Skills by default."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:072bfc2c939e2a2e2e26f897b4cca9a876bd9d4be28adc8db14bafe7e5bb941b"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:35f3cace4b6297a298b8b59db208b3d8ecfd82331758fb6bd34dd1ec03aa8ec7"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:12fbe0128457b6c9d0a4f32744eb3eb678c715129bb04bfc64d6f8cef5c073bc"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:1b205aeec354bb90bc5921111227c6a00cee586ac3bd23a37f8f0aeb0c004237"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:d211300dea66781251306e376e43bf9e798504ef"
status: active
---

# Global Onboarding

Manual order: CLI installation, host activation, global setup, explicit project setup. The optional
one-prompt path starts global setup without project inspection.

Supported legacy recovery requires matching saved-preference and live-file evidence; other bytes
remain unchanged. Korean setup retains `Skill` and `Wiki` exactly with canonical regression samples.

Global profiles retain nonexclusive user context only. Project workflow, technical choices, and
work priority belong to project scope.

Global setup enables all built-in Skills by default. Per-Skill toggles replace profile-bound
recommended suites; existing recorded closures remain until an approved preview changes them.
