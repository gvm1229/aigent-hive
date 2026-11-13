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
  - "repo:.github/workflows/release-discord-notification.yml#sha256:c0824f4bc8fa52cca8020b01d9c9b533279dd159cac82fc492ceb4f535345e2d"
  - "repo:.github/workflows/release-publish.yml#sha256:241b1121f2b95690858514f9bcc8af27f6bd9084d125b14ff0e5df31cd401a3d"
  - "repo:docs/archive/plans/foundations/stable-release-discord-notification.md#sha256:a502d4265210ff29e64b25364381c6ad17aecf1ce4bf90f35e08ac240efb6f63"
  - "repo:docs/decisions/subscriber-approval-via-main-2026-11-12.md#sha256:0e919445e53ecef2030dfc422679f20a22b519a37dc64ada56494ceb871e92d9"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:de31d59272ef49533a99da9421da6d72bb1d9ae467d3424f1207db1aba6a5f36"
  - "repo:tests/results/discord-notification-0.12.0.md#sha256:48eb0e36599335dab851fb2631e0338e4581392baf256615f5778e6053e5eabf"
  - "repo:tests/results/discord-sectioned-notification-0.12.0.md#sha256:40aaa2a04ecd3d673e28b09be83c2fb4a81f82f0297d931cd9eea407131077d1"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:84ce26fa4163718ec88f9ee63c8e298f0dfa7aaa"
status: active
---

# 안정판 Discord 구독자 알림

main 검토 PR의 원문·지문 승인, AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr 필수. 누락·다른 값의 검사·전송 거부. 출시별 설정 불필요, 옛 비밀값 삭제 가능. 파일명·바이트·한 줄 지문·2,000자·목록·섹션 검증. 정식 Release 뒤 배너→본문, 시험판 알림·웹훅 노출 제외. 일반 복구의 출시 소스 연결·중복 차단 유지. 승인된 0.15.0 정정은 정식 0.15.1 검토 문구 사용: ID 수정·삭제교체, ID 없으면 명시 정정·수동 삭제 기록. 새 ID·안전한 수신 기록 보존, 본문·토큰 기록 제외. PR 승인과 발송 권한, 로컬 검사와 실제 수신의 구분. 상세: 2026-11-12 결정.
