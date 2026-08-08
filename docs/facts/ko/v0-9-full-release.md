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
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:4a57fee408818d98f6b0bba20f8487743e9965d816f45b50a4d0d0d1a3915ea0"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:4c8f3d48f1157625f9e766b5525c4feb28b8f95eff5cabdee576082e1ff2bd15"
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
