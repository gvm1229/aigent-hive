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
  - "repo:docs/decisions/subscriber-approval-via-main-2026-11-12.md#sha256:7686a84a596fff628fa03c8229666571c8bc2c88bc4cf6489d6a99befd8ec9fd"
  - "repo:docs/releases/0.9.4.subscriber.ko.md#sha256:6c8e438046a01dd5882040fbd9216cb8ebce68ba83bedb1c28b70cb58b559be8"
  - "repo:scripts/publish-stable-discord-update.py#sha256:de31d59272ef49533a99da9421da6d72bb1d9ae467d3424f1207db1aba6a5f36"
  - "repo:tests/results/discord-notification-0.12.0.md#sha256:48eb0e36599335dab851fb2631e0338e4581392baf256615f5778e6053e5eabf"
  - "repo:tests/results/discord-sectioned-notification-0.12.0.md#sha256:40aaa2a04ecd3d673e28b09be83c2fb4a81f82f0297d931cd9eea407131077d1"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:84ce26fa4163718ec88f9ee63c8e298f0dfa7aaa"
status: active
---

# 안정판 Discord 구독자 알림

검토된 develop→main PR로 원문·한 줄 SHA-256 파일 승인. `release-publication` 변수 `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr` 필요, 누락·다른 값은 게시 전 검사·전송 거부. 출시별 설정 불필요, 옛 지문 비밀값 삭제 가능.
출시 소스만 사용, main·후보·공개 출시 결합과 버전별 중복 차단 유지. 일반 누락 복구의 다른 커밋 재전송 폐기. 파일명·바이트·전체 2,000자·목록·섹션 검증 유지. Release 성공 뒤 배너→안내 전송, 시험판·webhook 노출 제외. 로컬 검사와 실제 Discord 수신 구분. 상세: 2026-11-12 결정.

명시 승인된 0.15.0 정정은 정식 0.15.1의 검토 원문·지문 사용. 일반 누락 복구의 원래 문구 제한 유지. 메시지 ID와 버전 확인 후 수정, 한정된 삭제·재전송 대체. ID 부재 시 명시 정정 새 글과 이전 글의 유지관리자 수동 삭제 필요 기록. 새 ID·안전한 수신 기록 보존, 웹훅·메시지 본문 기록 제외. 현재 요청의 실제 정정 전송 제외.
