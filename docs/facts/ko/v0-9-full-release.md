---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "test.12 Windows 수용 완료. 무인 기본 profile 수용·측정 기반 test lane 대장 완료, 다음 numbered candidate 대기."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:9a4853952266b6a234ecf88bda90eebbf16148b8e529aa7739c9321c45866b91"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:036a7ea7282cf0ed6ffe0bef403331b73249cc566b2e381f8f43efc13e4097e3"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.12`: candidate `31391084832`, OIDC publication
`31392103115`, exact `c98add0`, 여섯 npm package의 `test=0.9.0-test.12`, `latest=0.8.0` 유지.
Windows actual user root 보존형 제거·clean reinstall·`dry-run → apply → validate`·install validate,
새 Codex session `hive` 탐색, Discord 실제 전달 수용 완료.

product-owned 신속 기본 profile: contributor preference·setup 대화 없는 clean install·보존형 재설치 수용,
Hive-owned user projection 자동 복원. 모든 Python 적합성 module: 측정된 하나의 lane 배정. CI: lane 분리 실행.
다음 gate: numbered candidate·public test 수용 뒤 `main` 진행.
