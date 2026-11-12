---
schema_version: 1
pair_id: overhaul-subscriber-approval
topic_slug: overhaul-subscriber-approval
language: ko
counterpart: ../en/overhaul-subscriber-approval.md
title: "재정비 구독자 안내 자동 승인"
summary: "재정비 전체 완료까지 구독자 안내 자동 승인."
tags: [overhaul, release, subscriber]
aliases: [overhaul-subscriber-approval]
sources:
  - "repo:docs/plans/overhaul/README.md#sha256:f5f479392e6cb05d85a0657d4cfe1f8fc5606b73b0681f248f1757c6a3cf948d"
links: [stable-release-discord-subscriber-notification]
reviewed_revision: "git:88ecb7a13736634afca601fe354e53c88b0dc58e"
status: active
---

# 재정비 구독자 안내 자동 승인

2026-11-12 유지관리자 결정: 재정비 전체 완료까지 모든 구독자 안내 자동 승인. 안내 원문·커밋된 SHA-256 파일의 develop→main 검토 PR 승인과 `release-publication` 환경 변수 `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr`의 최초 1회 설정. 버전별 비밀값 없이 `send_subscriber_update=true` 정식 게시 허용. 출시 검증 절차 유지.
