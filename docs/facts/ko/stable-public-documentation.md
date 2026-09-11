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
  - "repo:.github/workflows/release-publish.yml#sha256:e664105a2734fc5ec7c35f93ddc5ce0362ad5e391ae881c63e326a8c25866bca"
  - "repo:.github/workflows/release.yml#sha256:0b800d9f74b331f34aad1507b57129fb319fdf49934815026c6352c6aa91a5d7"
  - "repo:README.md#sha256:ba515cb5d9ee4f825305a99d0807b19bf608cb792eb768644776e3c0b4522735"
  - "repo:docs/public-stable-release.json#sha256:d084bc14870482ec5d33fce97b63f5db6ed67efc88bc38325bd06e167076d658"
  - "repo:scripts/check-public-stable-docs.py#sha256:69b25685285621ee94a515748de03c56b9100ca0e2f9e283bdc35a2278cb9f04"
links: [product-purpose, release-verification]
reviewed_revision: "git:8a45250106590f065df639132298b840940a3a35"
status: active
---

# 안정판 공개 문서

공개 안정판 대장이 일반 사용자용 version·배포일·문서 범위·release note coverage 소유.
README·설치 HTML·제품 개요·문서 색인은 해당 안정판만 안내.
번호 시험판은 npm·GitHub·유지보수자 검증 기록에 보존, 일반 설치 안내 노출 제외.
test 후보는 대장 안정판 유지, stable 후보는 build·게시 전 요청 version·배포일 일치 필수.
