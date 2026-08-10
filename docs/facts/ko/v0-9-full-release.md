---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "test.12 Windows 수용 완료. stable `0.9.0` 전 기본 profile 무인 설치 수용·측정 기반 테스트 체계 정리 필요."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9a4853952266b6a234ecf88bda90eebbf16148b8e529aa7739c9321c45866b91"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:629c28cdace1188fa43c8129dc38d293c4f9806f752a529171b67c492fd96d2e"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.12`: candidate `31391084832`, OIDC publication
`31392103115`, exact `c98add0`, 여섯 npm package의 `test=0.9.0-test.12`, `latest=0.8.0` 유지.
Windows actual user root 보존형 제거·clean reinstall·`dry-run → apply → validate`·install validate,
새 Codex session `hive` 탐색, Discord 실제 전달 수용 완료.

main 전 내부 gate: contributor preference·setup 대화 없는 product-owned 신속 기본 profile의 clean install·
보존형 재설치 자동 수용, replacement coverage 기반 테스트 체계 대장·정리. disposable consumer fixture:
ignore `tests/work/` 경계.
