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
  - "repo:.agents/directives/01-behavior.md#sha256:a78fc02202dc5c3b934e28924dd86660d297151f4905606dc7a26f2179083eaa"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:f790e1c19261cd6367504b074eb5516d8f4486e4bc7316ec9776327847e40272"
  - "repo:docs/guidance-schema.md#sha256:8d81babc67179cf5170fbde10cdb73fcaeaa0735f2c98a116f61eda9b2ec86ec"
  - "repo:harness/template/AGENTS.md.jinja#sha256:65ff8ac38eebfea005dbde07fe1e4cf12148129a050ba1f83296867bcd98a0c5"
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
