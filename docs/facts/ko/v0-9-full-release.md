---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "test.13 Windows 보존형 재설치 수용 완료. 무인 기본 profile 수용·측정 기반 test lane 대장 완료, main stable candidate 대기."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:5c00353f7683e586ada9ccfec9e80dd7504d2f464d88309ea8d9786f916219d5"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:00aacb0a11b5595075096985ce3872bda492799b24ecbc726025e3b558a75080"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:3e960b5185f637d7606eb01126d2543519138608"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.13`: candidate `31403054797`, OIDC publication
`31404195752`, exact `03a16676`, 여섯 npm package의 `test=0.9.0-test.13`, `latest=0.8.0` 유지.
Windows actual user root 보존형 제거·clean reinstall·`dry-run → apply → validate`·install validate,
새 Codex session `hive` 탐색, Discord 실제 전달 수용 완료.

product-owned 신속 기본 profile: contributor preference·setup 대화 없는 clean install·보존형 재설치 수용,
Hive-owned user projection 자동 복원. 모든 Python 적합성 module: 측정된 하나의 lane 배정. CI: lane 분리 실행.
다음 gate: non-force `develop → main` merge와 exact main stable candidate.
