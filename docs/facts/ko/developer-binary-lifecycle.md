---
schema_version: 1
pair_id: developer-binary-lifecycle
topic_slug: developer-binary-lifecycle
language: ko
counterpart: ../en/developer-binary-lifecycle.md
title: "개발 binary lifecycle"
summary: "Source local dev binary의 active executable 임시 교체와 internally reproducible user projection 안전 갱신, public release 인증 경계 유지."
tags: [development, installation, version]
aliases: ["Dev install", "Local developer build"]
sources:
  - "repo:crates/hive-cli/build.rs#sha256:a3b83a223dd02e759bdee84a5e1848f01f614834177d6a91de8d23d073feebff"
  - "repo:crates/hive-cli/src/main.rs#sha256:729c7f4664b1395df0cdeb15c36edc610f59ee200bded3481698b54da6260b1d"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:9a2ca4c65c5b7880464c7804d224cad91de28493eaf189aa63b1ed439806dde1"
  - "repo:scripts/dev-install.sh#sha256:c2c301aca92316edd613d4548ed041baa421958a126c182386856979aeef0016"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:e22ff9036b04535dc9a9cc542f08b20143cfe40e"
status: active
---

# 개발 binary lifecycle

`scripts/dev-install.sh --sandbox`: source local `product-dev` binary build. `--global`: 기존
active Hive executable backup 뒤 atomic 교체. `--rollback`: active developer digest가 일치할 때만
그 executable 복구. 세 경로 모두 canonical user preference·knowledge·index·directive·Skill과
project `.hive` 초기화·삭제·migration·변경 금지. Local binary: `local developer build`
출력, public `developer test build` identity 미사용. Local `-dev` binary만 internally reproducible
prior user manifest와 live managed byte 일치 조건의 projection refresh base 사용. Public stable·test
release는 signed historical base 부재 시 fail-closed 유지.
