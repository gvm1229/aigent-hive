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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:8d58b21e0a57a82908a5f6f59e489ec6e17d8e73191b17f9794f3dba16e9aef1"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:7a5c873834ba9a77e6efdedc60a5eed953fa40102dfcf88c084db5b591f465c3"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:acd4022de5697806003207634ac0b7cb874baeb802af491f28d39ec048daf830"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# 사용자 투영 자동 갱신

- 직접 설치 갱신 투영 재검증 action: `--apply`는 `InstallHiveUser`, `--validate`는 `ValidateHiveUser`
- M2 macOS 공개 `test.14 → test.15`: version 전이와 setup·install·validate·update-check·update·final validate 성공
