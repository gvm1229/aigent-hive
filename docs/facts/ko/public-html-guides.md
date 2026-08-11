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
  - "repo:docs/guides/public-html-design-principles.md#sha256:e7a60f611bd80581b5852d17a9ef58e050504188b9551fe1533e1fdbc8b365b2"
  - "repo:docs/hive-core-features.ko.html#sha256:99c818208dcdda45552a0f4962f7853222a57706463c2dd13a70f2f49863177e"
  - "repo:docs/hive-install-guide.ko.html#sha256:bf1124b8259e33c56fa3d070a69f1fa1be0f1f3ae8873bb25bd8fe3a5b99418a"
  - "repo:docs/plans/active/public-html-guides.md#sha256:8bddcbc123282fba063c067a0fe869775eb69417fca6d95f138da1d2421cb167"
links: [global-onboarding, product-purpose]
reviewed_revision: "git:ff1a28ae30369e839bfc8a1933b8283da7abab3a"
status: active
---

# 공개 한국어 HTML 안내

Repository의 독립 한국어 페이지 2개: Hive 핵심 기능과 간단한 stable 설치 경로.
공통 기준: 정본 logo, 960 px 정보 구조, 벌집 금색 `#F5A623`.
디자인 원칙 범위: 브랜드 token·정보 계층·구성 요소·반응형 경계·접근성·명령 정확성.
설치 3단계: 단일 `--host`, 쉼표 구분 `--hosts`, 반복 `--host`, 따옴표로 묶은 CSV 공백, 전체 preflight와 부분 실패 JSON.
README branding: 기존 commit `245ae80`에서 완료.
