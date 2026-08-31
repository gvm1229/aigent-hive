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
  - "repo:.github/workflows/release-publish.yml#sha256:6d9b351dfbe99fef461d642285a5bc37730ef6ba29d3c62d38c800bdd8e6220f"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:82db6eddc542a4e618f073469d5456d30173b3d16961e2cfb074988180e193d5"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:a0c3a87868199f81c144ed0895f4b564f3113f8b"
status: active
---

# 안정판 Discord 구독자 알림

안정판 전용 출시 흐름: 한국어 `update-summary`·배너·보호 환경 사전 검사. GitHub Release 성공 뒤 배너,
그 성공 뒤 요약 전송. 시험판 Discord 요청·webhook URL 출력 없음. 배너 전 원문 바이트와 버전별 `.sha256`,
보호 환경 `AIGENT_HIVE_SUBSCRIBER_SUMMARY_DIGEST`의 동일 지문 필수. 주 목록·두 칸 하위 목록 원문과
메시지 2,000자 제한 보존.
