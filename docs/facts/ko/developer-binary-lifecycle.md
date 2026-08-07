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
  - "repo:crates/hive-cli/build.rs#sha256:870578d55ee86e6414ff823c929b9eebe70b9ea4f829d4b6ce3d8d1f922c1991"
  - "repo:crates/hive-cli/src/main.rs#sha256:afe80f6416d7d9f1d8c9599a9306c396b6c5ada2730c9b60174906626e06e87a"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:282c641531d7a72009ee3973801ee8d20ea1305fd7e6f836f202e6a4e22508d4"
  - "repo:scripts/dev-install.sh#sha256:675d29e359a127a994d3b7904d3c842b3dafd884b8e28659a0d2b21ef3fc2a79"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:63628b43f1995b8245dcb1236e5fb6b515faea76"
status: active
---

# 개발 binary lifecycle

`scripts/dev-install.sh --sandbox`: source local `product-dev` binary build. `--global`: 기존
active Hive executable backup 뒤 atomic 교체. `--rollback`: active developer digest가 일치할 때만
그 executable 복구. 세 경로 모두 canonical user preference·knowledge·index·directive·Skill과
project `.hive`를 초기화·삭제·migration·변경하지 않는다. Local binary: `local developer build`
출력, public `developer test build` identity 미사용. Local `-dev` binary만 internally reproducible
prior user manifest와 live managed byte 일치 조건의 projection refresh base 사용. Public stable·test
release는 signed historical base 부재 시 fail-closed 유지.
