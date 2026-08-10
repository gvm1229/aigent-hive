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
  - "repo:.agents/directives/01-behavior.md#sha256:20c7359fc81cde6dfb49abe8782a7d41b29e534422b035c85ca71263b9d0c00e"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:f351f82fe27b3458b25eda8d74f94032206e3ab0a295db901157fa5f14c5e03a"
  - "repo:docs/guidance-schema.md#sha256:f5fc6aa2c36274d78d9703693a362c2f8d8eb81204d37f8a224434c14d1b196b"
  - "repo:harness/template/AGENTS.md.jinja#sha256:ea732dcaed4b7342f497c6b1268acce269627f07cc1fd596083c30ab300e8fa6"
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
