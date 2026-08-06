---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Optional one-prompt bootstrap and numbered setup preserve global/project scope while safely refreshing user projections."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:072bfc2c939e2a2e2e26f897b4cca9a876bd9d4be28adc8db14bafe7e5bb941b"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ff6bd5eabfb169efa9e763628d3ae876564396b01d901807ededf18999b21b0d"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:183d58c69302e82b0cc911fab51a75137dad54439c3de4a46e38de13a13adf7b"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:16c09937d59f1906d7a77b8c722c6ffd9c69416a"
status: active
---

# Global Onboarding

The manual sequence is CLI installation, terminal host activation, global user-scope setup, then
explicit project setup. An optional one-prompt path chooses one exact stable or test release,
checks Node.js/npm when needed, activates one detected host, and starts global setup without
inspecting a project. First setup asks for interface language first; reconfiguration first offers
one-setting change or a full review.

User-projection refresh compares a release base, local bytes, and incoming bytes. A vanilla base
is exactly replaced; disjoint local changes merge; overlapping changes retain local text and
report omitted incoming hunks. An unavailable authenticated base stops before any write.
