---
schema_version: 1
pair_id: stable-public-documentation
topic_slug: stable-public-documentation
language: ko
counterpart: ../en/stable-public-documentation.md
title: "안정판 공개 문서"
summary: "일반 사용자 공개 문서는 현재 안정판만 안내하고 번호 시험판은 유지보수자 검증 경로에만 남기는 대장·출시 gate"
tags: [documentation, release, stable]
aliases: ["공개 안정판 문서"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:35420bffac94da9392c605c6512edffa879458e177e892e407d9a979feffc693"
  - "repo:.github/workflows/release.yml#sha256:a15f748db5a727188a90c8836fe1a80235a5221f3896dff6f088b3dfaa3b28a4"
  - "repo:README.md#sha256:27679c3c338ef2f82b352800ccb882c2536bcc2c7dbfd18b93df52e3349554b0"
  - "repo:docs/public-stable-release.json#sha256:d06e22bccdbd8dc6b359be7e827e6bb2a2d981777f42d4fe8f600d92244c203c"
  - "repo:scripts/check-public-stable-docs.py#sha256:fecedad7d9cde787974550b0d754ceedfd0f432f9f035bddb044fbb986b6d6b6"
links: [product-purpose, release-verification]
reviewed_revision: "git:8a45250106590f065df639132298b840940a3a35"
status: active
---

# 안정판 공개 문서

공개 안정판 대장이 일반 사용자용 version·배포일·문서 범위·release note coverage 소유.
README·설치 HTML·제품 개요·문서 색인은 해당 안정판만 안내.
번호 시험판은 npm·GitHub·유지보수자 검증 기록에 보존, 일반 설치 안내 노출 제외.
test 후보는 대장 안정판 유지, stable 후보는 build·게시 전 요청 version·배포일 일치 필수.
