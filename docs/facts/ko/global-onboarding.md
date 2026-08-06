---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "인증 host activation 이후 global·project 별도 prompt 기반 명시 scope routing."
tags: [onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:a8326c3d7cf53451e09dcca9bc54f34b00b0428cf3d606e8e0c40ff3adf7b845"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:da6bf1991539dadff877be04330aaf96b61f2112bb0acf8d2f69cba2cdc2a692"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:982e5ccacf83856fed20ccf9ed9920e9635e70f0"
status: active
---

# Global onboarding

Terminal host activation 뒤 global prompt는 user-scope setup만 route하며 ambient project
inspection 없음. 별도 prompt는 명시 project에만 local harness 시작. Known prior ownership
snapshot은 update 가능, unknown·modified manifest는 preview·mutation 전 차단.
