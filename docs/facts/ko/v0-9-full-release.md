---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "test.9 게시 성공 뒤에도 보존된 test.8 Codex marketplace 실패로 Windows user-setup dry-run 중단, 0.9.0 stable 출시 차단."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:6f3521bbf939c70b51f3ebcb31c3019e174b558f19b2658e0d9cfb563bed02e0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:72e809923518c17689bf12ce990f87cba3ab1eaa28770b08b817afc6f20a01ab"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:7dd812e81a6e4e2771c783fc65835a3387bbd7ca"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.9`: candidate `31367482147`, OIDC publication
`31368361218`, exact `f88b0e5`, 여섯 npm package의 `test=0.9.0-test.9`,
`latest=0.8.0` 유지, annotated GitHub prerelease. Windows 격리 npm install의 test build #9 확인.
저장 답안 dry-run: test.8 transaction이 보존한 manifest 없는 Codex marketplace entry로 중단;
host mutation 0건.

`REL9-011`: Hive-managed recovery 또는 clean user host state 뒤 maintainer의 실제 Windows 11 machine에서
clean install·fresh Codex session 수용 필요. macOS install·cross-compile은 대체 증거 아님.
