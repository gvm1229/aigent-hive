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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:9488b87b8123e4cfbf702193a6c1ebf066fe143af522c51bea9ba5bdc5b3a6f5"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2527e7ceccba9293bc6f0ca7a7ca53ae986ec82066f1f4c62abc8426e4f18ab8"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:c1b10c7785570de5b92f85ef3a2b6a9f43efe12e03e008cc0950bbf156f00819"
  - "repo:docs/research/usage-upgrade-0.13.1.md#sha256:34eef7d725766ab1de07923d443623df1dd10b4e681950db813488249db2ace4"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:be1a9fc625b4bdb79597be8c3c081381d23e23d3"
status: active
---

# 사용자 투영 자동 갱신

- 인증된 기존 호스트: `update --apply`와 `UpdateHiveUser`, 빈 설치의 초기 설정: `install --apply`와 `InstallHiveUser`
- 두 경로의 검증 결과: `ValidateHiveUser`
- M2 macOS 공개 `test.14 → test.15`: 버전 전환·설정·설치·검증·갱신 확인·갱신·최종 검증 성공
-0.13.1은 새 사용자 갱신 실행 파일에서 활성 감시도 교체. 선택·기록 보존, 미완료 전환의 성공 표시 금지

- 공개0.13.0의 부분 선택도 알려진 묶음 전체로 인증, 미선택 Skill 요구 제외. 설정·지식·기록 보존 확인
- Windows 감시 갱신은 이전 감시기 종료 확인 뒤 기록 형식 전환, 현재 앱의 활성·비활성 보존 갱신 확인
