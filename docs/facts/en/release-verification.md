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
  - "repo:.agents/directives/references/documentation-verification.md#sha256:310146e429ce94f0c091b4514d9bf2d900779205d52831dc10bc03d1ed72b281"
  - "repo:.agents/directives/references/release-qualification.md#sha256:36968bb8a9bc012cfb7b72343ae9f8551cc0c0536232283f2794b822853d856e"
  - "repo:docs/decisions/ADR-0008-release-integrity.md#sha256:bace760d9be892a1e4f1f0554d2d55bbbaae85065125e9fae19a994f60f27410"
links: [judge-verification, update-transaction]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
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
