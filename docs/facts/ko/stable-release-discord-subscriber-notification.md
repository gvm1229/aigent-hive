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
  - "repo:scripts/publish-stable-discord-update.py#sha256:04e76513d36ede4c84e57d01b4a22335ac19ba0bab2bc604c9af1f7cab5f348c"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
  - "repo:tests/results/discord-notification-0.12.0.md#sha256:48eb0e36599335dab851fb2631e0338e4581392baf256615f5778e6053e5eabf"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:2d59ff6bd00e1f7c3f44d0744700a736bfdb7a3d"
status: active
---

# 안정판 Discord 구독자 알림

안정판게시전 승인문구·외부지문검증필수, 전송생략/false의게시거부. GitHub정식출시 성공뒤배너·그다음안내문전송. 시험판전송·webhook값출력 제외.
버전별원문·승인파일·외부지문대조, 전체2,000자한계·주/하위목록 보존. 기존승인목록형식과 순서맞는새기능/수정/개선의비어있지않은섹션·구분선 허용, 본문재작성 없음.
명시적문구승인뒤 기존gh로지문등록만수행, 재작성·출시·전송 없음. 동일지문실패재시도, 변경문구는새승인필수.
복구는 공개안정판·성공게시·기존전송제외를확인, 중복영수증거부·제품/태그재게시 제외. 0.12.0의두전송수락 확인.
