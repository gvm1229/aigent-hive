---
schema_version: 1
pair_id: verification-result-clarity
topic_slug: verification-result-clarity
language: en
counterpart: ../ko/verification-result-clarity.md
title: "Verification Result Clarity"
summary: "Hive reports whether each verification scope ran, why it did not run, and what the result does and does not establish."
tags: [communication, reporting, verification]
aliases: ["skip reporting", "verification qualifiers"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:2532c785b59f23a099b9e4a6eb71798f696dc4b79103600cf7c245582afa9f26"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c5cb31b7cf39c02be926e38ee529e023aabe45870b84a75b711f4f84c424e282"
  - "repo:docs/guidance-schema.md#sha256:881e80dda2f29dbc6239177aa66f518eff5bfb67dd93db017d6c280d914a7452"
  - "repo:harness/template/AGENTS.md.jinja#sha256:e9545c960f609ad7369e2d5e0cc9f48f79fdc7cd20836cf6199f19eb4ca4f301"
links: [language-consistency, release-verification]
reviewed_revision: "git:c5d7b90c0b2e126f73fdfd6da850d5eed07b4d61"
status: active
---

# Verification Result Clarity

Hive source agents and installed consumer harnesses qualify every passed,
failed, skipped, deferred, unverified, and unsupported result with the affected
scope, exact reason, current host or platform relationship, execution status,
and what the result proves or leaves unverified. Platform labels alone never
stand in for whether a check actually ran. Acceptance: source, consumer, and
project guidance agree and their projection tests pass. Origin: maintainer
request after an ambiguous Windows skip summary could be read as either prior
Windows verification or mistaken non-Windows detection.
