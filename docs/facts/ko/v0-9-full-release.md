---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "현재 Codex plugin 활성화·setup 수정 numbered 시험판 수용 전 0.9.0 stable 출시 차단."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:27ef86823b582e9435e08fe3adfd45203c5b0328751c5b3b130465662f48ebd0"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.6`: 여섯 npm package의 `test`, `latest=0.8.0` 유지,
annotated GitHub prerelease. macOS 격리 검증: user-root 경로 정규화와
marketplace→plugin→setup dry-run·apply·validate, rollback·foreign byte 회귀 통과.

추가 stable gate: product-only 22개 Skill과 전역·project 한도를 쓰는 product `usage-guard` 하나.
`REL9-011`: maintainer의 실제 Windows 11 machine에서 수정 numbered 시험판 clean install·fresh
session 수용 필요. macOS install·cross-compile은 대체 증거 아님.
