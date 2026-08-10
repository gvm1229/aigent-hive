---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "test.15 최신 공개 수용 build. Windows 보존형 재설치 수용·develop 전체 CI 통과, 정식 서명·external TUF authorization 필요."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:5c00353f7683e586ada9ccfec9e80dd7504d2f464d88309ea8d9786f916219d5"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:00aacb0a11b5595075096985ce3872bda492799b24ecbc726025e3b558a75080"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a39b88112f5582a836e0c5848668407190d4a616"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. `0.9.0-test.15`: candidate `31407585364`, OIDC publication `31409030152`,
exact `6f809a27`, 여섯 npm package의 `test=0.9.0-test.15`, `latest=0.8.0` 유지. Windows 보존형 재설치·
setup 검증·새 Codex session 탐색·Discord 실제 전달, `develop` CI `31410354787` 19개 작업 전체 PASS.
Stable publication 선행 조건: macOS·Windows 서명과 external TUF authorization·rollback floor 검증. 두 release workflow의 해당 단계 부재.
