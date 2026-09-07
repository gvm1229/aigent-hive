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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:c32316f67b6846b57cfd654b4abd8a518c74244cb6be1a9981e135190e203a07"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6d2138ea9d68803f4447e7295cc08fcf5b548c23df8b2d1e5826c0aa46bff668"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:633265271a82ee37ef07acd9e5a3d406ea80a3f17c7c9809d3d8d7619dd91260"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# 사용자 투영 자동 갱신

- 직접 설치 갱신 투영 재검증 action: `--apply`는 `InstallHiveUser`, `--validate`는 `ValidateHiveUser`
- M2 macOS 공개 `test.14 → test.15`: version 전이와 setup·install·validate·update-check·update·final validate 성공
