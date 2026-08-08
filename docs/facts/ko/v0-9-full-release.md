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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:ee293a5b839fb7af3b7f4ebefc9be662f9ab595242e37cf31e6b143c6c69cb20"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:bf7fa4f0f5d2639358490df5e7978e9756cfe633e82ef84251ba4dc179101a05"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:27816088abbcfca7233e0e006f8b1e6cdec7aa55"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

- Stable `v0.9.0`: 미게시
- 최신 공개 시험판: `0.9.0-test.5`; `latest=0.8.0` 유지
- 후보 `31254605322`: 5개 native target·npm packaging·attestation PASS, 게시 2회는
  `0.9.0-test.6` artifact 생성 전 중단
- `REL9-011`: Codex CLI `0.146.1` 현재 JSON adapter·parser 검증, 격리
  marketplace→plugin→user-setup, 실패 되돌리기·foreign byte 보존, fresh-session discovery,
  수정 numbered 시험판 수용의 필수 출시 gate
- 단일 `release-publish.yml` OIDC와 Copier·Rust Discord 기본 필드 parity 완료
