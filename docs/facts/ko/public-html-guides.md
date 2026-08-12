---
schema_version: 1
pair_id: public-html-guides
topic_slug: public-html-guides
language: ko
counterpart: ../en/public-html-guides.md
title: "공개 한국어 HTML 안내"
summary: "Hive 기능·설치·브랜드 기준을 담은 한국어 HTML 2개와 디자인 원칙 안내."
tags: [branding, documentation, onboarding]
aliases: ["Hive 간단 설치 안내", "Hive 핵심 기능 안내"]
sources:
  - "repo:docs/guides/public-html-design-principles.md#sha256:6aae9ab9808ca927fe3736f3c4bde3e91a0e25a9700ae75f98592e1577ab01ec"
  - "repo:docs/hive-core-features.ko.html#sha256:8f77210359186752205a0b4dcffcefd5d1a0bb8530d3620463be073c81b33abf"
  - "repo:docs/hive-install-guide.ko.html#sha256:d4b0a063d3595b4af46b67f7ac1f6bd85b0b72652d85679fd25c1243c42bd1fe"
  - "repo:docs/plans/active/public-html-guides.md#sha256:ef7818fd4550419c585f2fae43c569fa2d1541fe4508e622f9ad738317e371bf"
links: [global-onboarding, product-purpose]
reviewed_revision: "git:0b3bbbbfcb5904262c5281a0415851b96779ab9e"
status: active
---

# 공개 한국어 HTML 안내

Repository의 독립 한국어 페이지 2개: Hive 핵심 기능과 간단한 stable 설치 경로.
공통 기준: 정본 logo, 960 px 정보 구조, 벌집 금색 `#F5A623`.
독립 공유: 각 HTML에 정본 PNG 원본 byte 1회 내장, system font 사용, network·file-relative resource reference 0건.
디자인 원칙 범위: 브랜드 token·정보 계층·구성 요소·반응형 경계·접근성·명령 정확성.
설치 3단계: 단일 `--host`, 쉼표 구분 `--hosts`, 반복 `--host`, 따옴표로 묶은 CSV 공백, 전체 preflight와 부분 실패 JSON.
README branding: 기존 commit `245ae80`에서 완료.
