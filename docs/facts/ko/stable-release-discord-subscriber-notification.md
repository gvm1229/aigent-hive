---
schema_version: 1
pair_id: stable-release-discord-subscriber-notification
topic_slug: stable-release-discord-subscriber-notification
language: ko
counterpart: ../en/stable-release-discord-subscriber-notification.md
title: "안정판 Discord 구독자 알림"
summary: "안정판 게시 전 구독자 안내문 승인 필수, 누락 공지는 제품 재게시 없는 전용 복구."
tags: [discord, release, subscriber]
aliases: [stable-release-discord]
sources:
  - "repo:.github/workflows/release-discord-notification.yml#sha256:e4d2b64ca1d25902b8c6701b6e5395cd47049be8169ec49d03d6666df3f3930c"
  - "repo:.github/workflows/release-publish.yml#sha256:240f5376bc7ea26109a1c34be965c6e74f9dc840e400af78bb1e55a2bac01263"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:82db6eddc542a4e618f073469d5456d30173b3d16961e2cfb074988180e193d5"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
  - "repo:tests/results/discord-notification-0.12.0.md#sha256:48eb0e36599335dab851fb2631e0338e4581392baf256615f5778e6053e5eabf"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:604b352d2d2b34c3e62e6b0b6578fd3ff18cf8ff"
status: active
---

# 안정판 Discord 구독자 알림

안정판 게시 전 승인한 한국어 안내문·배너·외부 승인 지문 검증 필수. 전송 생략·false의 안정판 게시 거부. GitHub 정식출시 성공 뒤 배너, 그 성공 뒤 안내문 전송.
시험판 전송·webhook값 출력 없음. 버전별 원문·승인파일·외부지문 대조, 주·하위 목록과2,000자 한계 유지.
등록 도구는 명시적 문구 승인 뒤 기존gh 인증으로 지문 등록만 수행, 재작성·출시·전송 없음. 실패는 동일지문 재시도, 변경문구는 새승인 필요.
공지전용 복구는 공개 안정판·성공게시·기존전송 제외를 확인하고 중복영수증 거부. 승인문구만 전송, 제품·정식태그 재게시 제외.

0.12.0: 두전송 수락.
