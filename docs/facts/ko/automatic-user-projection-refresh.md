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
  - "repo:crates/hive-cli/src/update_activation.rs#sha256:5286f271a2601e90572b4d216c3e6b65f40bfd7e15401ea2f2d3a48069fbdf18"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:db91b9841c057a3f9b964185fb2a2f3c2f8701908cf6439e26bf05d389a7243d"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:73d95136c28b6742d76d7aca0874144808500168a67fc97accdee9bc2b387481"
links: [interactive-binary-update, multi-host-user-install, projection-upgrade-purge]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# 사용자 투영 자동 갱신

`0.9.5` 계획: bare `hive update`의 authenticated binary replacement 뒤 user projection 자동 갱신.
Semantic host scope 정본: valid saved setup과 host별 install manifest. Default host·untrusted command
history 사용 금지. scope 부재·invalid 상태: projection mutation 없는 binary-only outcome·명확한 recovery 안내.
