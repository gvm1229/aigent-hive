---
schema_version: 1
pair_id: developer-binary-lifecycle
topic_slug: developer-binary-lifecycle
language: ko
counterpart: ../en/developer-binary-lifecycle.md
title: "개발 binary lifecycle"
summary: "Source local dev binary는 canonical user data를 바꾸지 않고 active executable을 임시 교체할 수 있다."
tags: [development, installation, version]
aliases: ["Dev install", "Local developer build"]
sources:
  - "repo:crates/hive-cli/build.rs#sha256:870578d55ee86e6414ff823c929b9eebe70b9ea4f829d4b6ce3d8d1f922c1991"
  - "repo:crates/hive-cli/src/main.rs#sha256:afe80f6416d7d9f1d8c9599a9306c396b6c5ada2730c9b60174906626e06e87a"
  - "repo:scripts/dev-install.sh#sha256:4e78ac1c159ce03be44374268de3ebfd53af3826029af88180186599490bd22f"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:b93e3e14950a2373fd99bfcf98daf71b1e562d3e"
status: active
---

# 개발 binary lifecycle

`scripts/dev-install.sh --sandbox`: source local `product-dev` binary build. `--global`: 기존
active Hive executable backup 뒤 atomic 교체. `--rollback`: active developer digest가 일치할 때만
그 executable 복구. 세 경로 모두 canonical user preference·knowledge·index·directive·Skill과
project `.hive`를 초기화·삭제·migration·변경하지 않는다. Local binary: `local developer build`
출력, public `developer test build` identity 미사용.
