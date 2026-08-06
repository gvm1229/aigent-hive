---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: en
counterpart: ../ko/global-onboarding.md
title: "Global Onboarding"
summary: "Distinct global and project prompts preserve explicit scope routing after authenticated host activation."
tags: [onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:a8326c3d7cf53451e09dcca9bc54f34b00b0428cf3d606e8e0c40ff3adf7b845"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:da6bf1991539dadff877be04330aaf96b61f2112bb0acf8d2f69cba2cdc2a692"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:982e5ccacf83856fed20ccf9ed9920e9635e70f0"
status: active
---

# Global Onboarding

After terminal host activation, the global prompt routes only to user-scope setup and never
inspects an ambient project. A separate prompt starts a local harness only for an explicit
project. Known prior ownership snapshots may update; unknown or modified manifests remain
blocked before preview or mutation.
