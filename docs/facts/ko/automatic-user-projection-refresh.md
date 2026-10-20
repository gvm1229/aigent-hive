---
schema_version: 1
pair_id: automatic-user-projection-refresh
topic_slug: automatic-user-projection-refresh
language: ko
counterpart: ../en/automatic-user-projection-refresh.md
title: "사용자 투영 자동 갱신"
summary: "0.9.5 직접 설치 갱신의 mode별 사용자 설치 action 검증과 공개 갱신 버전 전이 기록"
tags: [installation, migration, projection, update]
aliases: ["갱신 뒤 투영 refresh"]
sources:
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:d62d3065eb447fe5653b11fbe396490c15471839cb4553c6a3fca1abb368baec"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:d16fe3cf14d23b5781bbb960884146b17c3770916a5572215e4922a4423f28d1"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:ccf59c00e326034d6d12ef9a9c8a52c91d5ff8cc"
status: active
---

# 사용자 투영 자동 갱신

- 인증된 기존 호스트: `update --apply`와 `UpdateHiveUser`, 빈 설치의 초기 설정: `install --apply`와 `InstallHiveUser`
- 두 경로의 검증 결과: `ValidateHiveUser`
- M2 macOS 공개 `test.14 → test.15`: 버전 전환·설정·설치·검증·갱신 확인·갱신·최종 검증 성공
-0.13.1은 새 사용자 갱신 실행 파일에서 활성 감시도 교체. 선택·기록 보존, 미완료 전환의 성공 표시 금지
