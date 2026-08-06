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
  - "repo:.agents/directives/01-behavior.md#sha256:b84d45bd3fb28b67581ff024fdfb8d0aaf0dfbbda580f712c96d1228685dadbb"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:39bc19a47799793c2f2e984f5d7d6edb4e18fbbd96ec33ac30e7c258fda66d0b"
  - "repo:docs/guidance-schema.md#sha256:fd8fffda818038ee48b66b0581787e2fd741404b0a9253ca34c0d55f15ad4d15"
  - "repo:harness/template/AGENTS.md.jinja#sha256:070d97440343d699565448c239efb55c905df79119df289525d41edc6e81581f"
links: [language-consistency, release-verification]
reviewed_revision: "git:33f365d3dbb1af51333a6dbb1834ce437a932ea0"
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
