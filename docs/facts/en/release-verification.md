---
schema_version: 1
pair_id: release-verification
topic_slug: release-verification
language: en
counterpart: ../ko/release-verification.md
title: "Release Verification"
summary: "Hive separates release qualification from documentation-only repository integration and local bundle integrity."
tags: [release, security, verification]
aliases: ["Release integrity"]
sources:
  - "repo:.agents/directives/03-workflow.md#sha256:629d32bb289108bbc782e295e4ffda6a4a4d5006fbf151212db0cc79457391f0"
  - "repo:docs/decisions/ADR-0008-release-integrity.md#sha256:bace760d9be892a1e4f1f0554d2d55bbbaae85065125e9fae19a994f60f27410"
links: [judge-verification, update-transaction]
reviewed_revision: "git:567c7000e56699b7fa82163164e0cc4a9dc1bd0b"
status: active
---

# Release Verification

Npm registry integrity or GitHub exact-tag attestation establishes acquisition provenance. Hive
then verifies every local bundle artifact by path, length, and SHA-256 before transactional update.
Accepted release state rejects downgrade and same-sequence substitution. Release private keys and
platform certificates are not stable publication requirements. A Markdown-only repository change
may merge after its relevant local documentation, packaging, directive, and link gates pass without
waiting for unrelated full cross-platform CI. Such a follow-up never creates a new test or stable
release and unfinished CI is not reported as passed.
