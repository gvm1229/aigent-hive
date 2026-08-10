---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "게시된 0.9.0-test.8의 Codex 활성화·setup 수용 전 0.9.0 stable 출시 차단."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:cea7b974c4edd12f496ed095b416879dbc0db8f8337081c0455acdbb0ed09e12"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.8`: candidate `31359482328`, OIDC publication
`31360092061`, exact `f7debf4`, 여섯 npm package의 `test`, `latest=0.8.0` 유지,
annotated GitHub prerelease. Windows 격리 npm prefix의 test.8 CLI identity 확인.

`REL9-011`: maintainer의 실제 Windows 11 machine에서 clean install·fresh Codex session 수용 필요.
macOS install·cross-compile은 대체 증거 아님.
