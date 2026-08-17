---
schema_version: 1
pair_id: automatic-user-projection-refresh
topic_slug: automatic-user-projection-refresh
language: ko
counterpart: ../en/automatic-user-projection-refresh.md
title: "사용자 투영 자동 갱신"
summary: "0.9.5 계획의 bare binary update 뒤 authenticated saved user-install scope 한정 투영 갱신"
tags: [installation, migration, projection, update]
aliases: ["갱신 뒤 투영 refresh"]
sources:
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:c685f83d115dc2764780f7cf52a8c949268cde0b2fa7efd36ad99b173c883b25"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
  - "repo:scripts/accept-public-hive.py#sha256:59a78bea773c38e18248fb6cdefe6e612a69d8f46ae0139eeff7a7b30fa455f2"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:5f3bb933d6df91864f005041ad13d223ecdfbc9c"
status: active
---

# 사용자 투영 자동 갱신

- bare `hive update`: stable channel 기본값 유지
- `--channel test`: explicit prerelease 선택
- `--user-root`·`--confirm`: 전용 수용 root 지원
- activated binary: 인증된 saved host scope만 갱신
- 공개 `test.4 → test.5`: user projection 수용 증거 대기
