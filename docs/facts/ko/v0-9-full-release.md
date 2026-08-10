---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "test.12 Windows 보존형 제거·clean reinstall 수용 완료. stable `0.9.0` 전 새 Codex session CLI 탐색·webhook 환경값 outbound 증거 필요."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:52abb3afc4d52549b1d3b701ad91bed84a28d95f0331f5768aa76a2dfab4572a"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.12`: candidate `31391084832`, OIDC publication
`31392103115`, exact `c98add0`, 여섯 npm package의 `test=0.9.0-test.12`, `latest=0.8.0` 유지,
annotated GitHub prerelease. Windows actual user root 보존형 제거·clean reinstall·`dry-run → apply →
validate`·install validate PASS. saved preference·knowledge digest 보존, Hive active Skill 22개·retired
ID `0건`, usage guard `20%`, persisted Discord 설정, home temporary answer `0건`. webhook environment value 부재.

`REL9-011`: 실제 Windows 11의 새 Codex session 자동 CLI 탐색, configured webhook environment value가
있는 Discord outbound test 수용 필요.
