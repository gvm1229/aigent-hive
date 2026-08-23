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
  - "repo:.agents/directives/01-behavior.md#sha256:20905d49494df815461b4e9ffe6df89ee33ccb774510da2cfa10c98f0508b077"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:92700072141579de36f5c9e9405aec31bcac07047bd2a492e25362a6a709dce3"
  - "repo:docs/guidance-schema.md#sha256:eae385d284f448a27a5243d8e7846aa69d9568e0849d3457147fb814229416ad"
  - "repo:harness/template/AGENTS.md.jinja#sha256:f1170037b949896332fdb95f058fde810a00b0474b423e054899a74a5da3b200"
links: [language-consistency, release-verification]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
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
