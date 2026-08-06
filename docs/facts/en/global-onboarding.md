---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "A numbered host-activation, global-preference, and project-setup flow preserves explicit scope routing."
tags: [onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:5d76a2698ec20d359181c065e44105cf91264d943aaf748077971da14613173c"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:da6bf1991539dadff877be04330aaf96b61f2112bb0acf8d2f69cba2cdc2a692"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:1fa7abad6925bcf17c8b253458e024733e5de1f6"
status: active
---

# Global Onboarding

The first-setup sequence is CLI installation, terminal host activation, global user-scope setup,
then explicit project setup. The global prompt never inspects an ambient project; the project
prompt acts only on a named repository or absolute path after a write preview. Known prior
ownership snapshots may update; unknown or modified manifests remain blocked before preview or
mutation.
