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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:a757180da3c34992858923db154f6d1f7b8de2d5c353b6bf81a48e32331c19eb"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:359e033f6bad6a6145820efb0a079a6643d4774a6d9b8e1b560d9d4e156df5be"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:64e7ee1eb9aaafd399fe971ca35e5df6aee68285029a9b84fa6b928a3324ffdc"
  - "repo:scripts/accept-public-hive.py#sha256:b951e079d0974d4bf2a80e37337f2acf95d03e2e42a4bc428dd9fbde89a538a3"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:838842805e453e0508d054e4aa67d7a59b3aa53f"
status: active
---

# 사용자 투영 자동 갱신

- 직접 설치 갱신 투영 재검증 action: `--apply`는 `InstallHiveUser`, `--validate`는 `ValidateHiveUser`
- M2 macOS 공개 `test.14 → test.15`: version 전이와 setup·install·validate·update-check·update·final validate 성공
