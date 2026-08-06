---
schema_version: 1
pair_id: global-onboarding
topic_slug: global-onboarding
language: ko
counterpart: ../en/global-onboarding.md
title: "Global onboarding"
summary: "공통 provider-neutral prompt 기반 active host activation과 guided project setup."
tags: [onboarding, setup]
aliases: ["User setup"]
sources:
  - "repo:README.md#sha256:c671da37fa77443069d1799719bce28ea3cae6dc6f532cce11c96b169c121d10"
  - "repo:docs/decisions/ADR-0012-global-onboarding-shared-index.md#sha256:44401a82ba3bd9f2bc4048876f5480157720bc5fce005a7c0b63f4d960f63bf1"
links: [project-onboarding, test-distribution]
reviewed_revision: "git:1144b25b9653cbb3e2a39bc6716acd13239f4ac7"
status: active
---

# Global onboarding

CLI 설치 후 Codex·Claude Code·Gemini Antigravity 공통 provider-neutral prompt 기반
active host activation과 guided setup. 명시적 사용자 선택 유지: 언어, daily-update
opt-in, write preview, unresolved preference, optional capability.
