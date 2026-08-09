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
  - "repo:docs/plans/active/release-0.9.0.md#sha256:378c59f5e55241f4d50037965f8a0fe865255f15dd0ce814462048d6a2c3d770"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:f356500621c21702abb8c21746cf138078a9d9fc"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

- Stable `v0.9.0`: 미게시
- 최신 공개 시험판: `0.9.0-test.6`; 여섯 npm package `test=0.9.0-test.6`, `latest=0.8.0` 유지
- 후보 `31294665865`와 OIDC 게시 `31295045199`: exact `f356500`, annotated GitHub prerelease 공개
- `REL9-011` local 증거: Codex CLI `0.147.0`, macOS 격리 user root에서 host의 physical path
  표기 재현. no-follow 확인 뒤 경로 정규화, marketplace→plugin→user-setup dry-run·apply·validate,
  rollback·foreign byte Rust 회귀 PASS
- `REL9-011` 잔여 출시 조건: 수정 numbered 시험판의 Windows clean install·fresh session 수용
- 단일 `release-publish.yml` OIDC 실제 게시, Copier·Rust Discord 기본 필드 parity 완료
