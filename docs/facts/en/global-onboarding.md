---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Global setup stores multiple user contexts without choosing project workflows; Korean product terms and all built-in Skill defaults remain fixed."
tags: [bootstrap, onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:30e7d1dece221c145e4a75fe9e05ec9520ca3ab58b7d1311088b9c4ad72759ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:128edc67999108258248cd5d1c356666931bbc7a6d9a747eaf108bc0cf5125f3"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:2e064212050d755bf101322fdcc94f8a737db7b59204b75bb6bfcd64d8e32ceb"
  - "repo:harness/skills/setup-hive/SKILL.md#sha256:cb996a8698314710ce527c2c1d5bf41c0895bead8e7d52f9b1c4052b8d6666f6"
  - "repo:harness/user-setup/catalog.yml#sha256:af1147b8468f48eb81ec77ed4a14d5eba2fd31a4302e5459544fec3b2e22b595"
  - "repo:schemas/user-setup.schema.json#sha256:680009cadc1d41add4b16331bde37509cf636c845644a3923094a281110fb786"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:0c0a3fd18bd4b3746202c5a38aa7cb03d4b94908"
status: active
---

# Global Onboarding

Manual order: CLI installation, host activation, global setup, explicit project setup. The optional
one-prompt path starts global setup without project inspection.

Supported legacy recovery requires matching saved-preference and live-file evidence; other bytes
remain unchanged. See `global-user-contexts` for contexts, Skill selection, and Korean product terms.
