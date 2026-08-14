---
schema_version: 1
pair_id: stable-release-discord-subscriber-notification
topic_slug: stable-release-discord-subscriber-notification
language: ko
counterpart: ../en/stable-release-discord-subscriber-notification.md
title: "안정판 Discord 구독자 알림"
summary: "Aigent Hive 안정판 GitHub Release 성공 뒤 보호된 출시 환경을 통해 한국어 배너를 먼저 보내고 검증된 한국어 구독자 업데이트를 이어서 전송."
tags: [discord, release, subscriber]
aliases: [stable-release-discord]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:40594864c88b2ab2ddce13ee5f858167f717ad9649ba2e447bd75236d0494247"
  - "repo:docs/plans/active/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:9b1fe57e0141e59523edae80e910ad537ade2a0b105678b608ad0101b47c9da9"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:e1af8adfa30cd07e45496fb2491b7018e14b3ad9"
status: active
---

# 안정판 Discord 구독자 알림

안정판 전용 출시 흐름에서 `update-summary` 한국어 메시지·배너·보호된 환경 비밀 값 사전 검사. GitHub
Release 성공 뒤 배너 먼저 전송. 이 요청 성공 뒤에만 한국어 구독자 요약 전송. 시험판 Discord 요청 없음.
알림 도구의 webhook URL 출력 없음. 보호된 환경의 `0.9.3`·`0.9.4` 실제 전달 시험 성공,
안정판·npm 게시 없음. 유지보수자 실제 Discord 화면에서 각 배너 아래 해당 한국어 요약 표시 수용.
