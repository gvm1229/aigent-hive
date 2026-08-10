---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "test.11 Windows user setup 수용 완료. stable `0.9.0` 전 test.12의 보존형 제거·clean reinstall·새 Codex session CLI 탐색 증거 필요."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:026e00b1386bdd2364e8befa7830a94b654dae96e7d4cae0411aa21613eb798b"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.11`: candidate `31372510565`, OIDC publication
`31373214154`, exact `b0e41f5`, 여섯 npm package의
`test=0.9.0-test.11`, `latest=0.8.0` 유지, annotated GitHub prerelease. Windows 격리 install·actual
user root `dry-run → apply → validate` PASS, 조용한 Codex marketplace recovery, product-only Skill,
Korean·bilingual Wiki, usage guard, persisted Discord test delivery 확인.

`REL9-011`: test.12의 보존형 `hive uninstall`, 저장 preference clean reinstall, 실제 Windows 11의
새 Codex session 자동 CLI 탐색 수용만 남음.
