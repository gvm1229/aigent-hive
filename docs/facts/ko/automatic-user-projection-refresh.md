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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:88a29477d0b59f362df545687e6939267545980d9f78eb7e10d8f3322f81a94c"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:b8bb2ace9f509cf8f48cf703971069e7ca73ada3704a8c7dc18adfd03a27e9ef"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# 사용자 투영 자동 갱신

`0.9.5` 계획: bare `hive update`의 authenticated binary replacement 뒤 user projection 자동 갱신.
Semantic host scope 정본: valid saved setup과 host별 install manifest. Default host·untrusted command
history 사용 금지. scope 부재·invalid 상태: projection mutation 없는 binary-only outcome·명확한 recovery 안내.
