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
  - "repo:.agents/directives/01-behavior.md#sha256:dd66d053a9edd60c2f04e96283f4f95e5429dbf24e6b2d98c025bbf89039d5df"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6ee84b034e6a23171889dc33e8be8f839594edd080168ad7601e8c5fd9e5c9cc"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:33c0da7ba5156ea1aa0ccc08a8e4f88343cf5f6f896994a7d8b830ac0ad6bb74"
links: [language-consistency, release-verification]
reviewed_revision: "git:536f5076534cedcdb9ea3d118830792fe61cd75e"
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
