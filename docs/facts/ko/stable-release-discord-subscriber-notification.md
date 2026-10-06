---
schema_version: 1
pair_id: stable-release-discord-subscriber-notification
topic_slug: stable-release-discord-subscriber-notification
language: ko
counterpart: ../en/stable-release-discord-subscriber-notification.md
title: "안정판 Discord 구독자 알림"
summary: "안정판의 명시적 전송 선택 후 승인한 한국어 배너·안내문 전송, 안정판 게시와 별도인 기본 비활성 전송."
tags: [discord, release, subscriber]
aliases: [stable-release-discord]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:a90ceb2d2eb2a176057e7afd20bd34e8bef8e9c176a1e80ddbe16876f0d9309c"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:82db6eddc542a4e618f073469d5456d30173b3d16961e2cfb074988180e193d5"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:bb224faca9a37b022f5eafe1e8d36c87182836ae"
status: active
---

# 안정판 Discord 구독자 알림

안정판 게시 전 한국어 안내·배너 검사. 기본false의 `send_subscriber_update`로 게시와 전송 분리.
명시적true의 안정판만 GitHub Release 성공 뒤 배너, 그 성공 뒤 요약 전송.
시험판 전송·webhook URL 출력 없음. 원문·버전별 sidecar·외부 지문 대조, 주·하위 목록과 2,000자 제한 보존.
문구 승인 뒤 `register-stable-summary-approval.py`에서 승인 지문·파일 검증 후 기존 `gh` 인증으로 자동 등록.
출시마다 GitHub 수동 설정 불필요. 등록 중 안내문 재작성·출시·메시지 전송 없음.
등록 실패는 동일 승인으로 재시도, 변경 문구의 자동 승인·발송 중 지문 갱신 금지.
