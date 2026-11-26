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
  - "repo:.github/workflows/release-publish.yml#sha256:3f4998b2889cfd7cb3ada0b94a7a1648ef6bbd9c8cd7e848676742d13bc17a42"
  - "repo:.github/workflows/release.yml#sha256:6fa640b5ce08594bdcbccd28530846ff7a34f7dfd6f918f4b62a860555ed7567"
  - "repo:README.md#sha256:05c2f90b0096663c3c8ded051eec7d7632df4b99442d90a77faca4f26b5bf80b"
  - "repo:docs/public-stable-release.json#sha256:45913c2779432631e5873c9f9f221c76673fea90a878a05cb5fee827aaf46474"
  - "repo:scripts/check-public-stable-docs.py#sha256:1b0ff65b6c7b7d37fd45554d2465885515ab5f9f06c6f3f46a0c08b99d5a89f9"
  - "repo:tests/conformance/release/test_release_notes.py#sha256:0e6e4201c7c3369ea787c956b7022add3aae86f1331894787331e955a4c9a63c"
links: [product-purpose, release-verification]
reviewed_revision: "git:c5b13bc26c4b2485dcad7509bc8590f97d0831fe"
status: active
---

# 안정판 공개 문서

공개 안정판 대장이 일반 사용자용 version·배포일·문서 범위·release note coverage 소유.
README·설치 HTML·제품 개요·문서 색인은 해당 안정판만 안내.
README에는 출시 이력 없이 GitHub Releases 링크만 유지, release note coverage는 출시 설명 문서 기준.
번호 시험판은 npm·GitHub·유지보수자 검증 기록에 보존, 일반 설치 안내 노출 제외.
test 후보는 대장 안정판 유지, stable 후보는 build·게시 전 요청 version·배포일 일치 필수.

후보 생성 전 CI에서 현재 출시 설명·실제 소스 버전 검증부 실행. 합성 시험만으로 실제 출시 문서의 검증을 대체하는 방식 제외.
