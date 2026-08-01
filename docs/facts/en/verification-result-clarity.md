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
  - "repo:.agents/directives/01-behavior.md#sha256:69cad89a5e857e404f6d51106a8688623afd6d3ad1613ddc5a326ab7b998bb30"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:c3cc02dcd02afddbd583d51bd02bc113dc283a17e8244587e0bbf832450dd823"
  - "repo:docs/guidance-schema.md#sha256:99c034ed85314fa0f707f057e4e567cfb32159e9bd50e5f81388c37de740c2e6"
  - "repo:harness/template/AGENTS.md.jinja#sha256:71eeaf7aff5e21b8a7cf764daf6060cb44954f14218370585c3d72a6f25f14c7"
links: [language-consistency, release-verification]
reviewed_revision: "git:19eda4d7ef87fe3122c14c455df07758c3dc6ff1"
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
