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
  - "repo:.github/workflows/release-publish.yml#sha256:6d9b351dfbe99fef461d642285a5bc37730ef6ba29d3c62d38c800bdd8e6220f"
  - "repo:.github/workflows/release.yml#sha256:2f3760d989da12d1b07bfe706b9e7f1cd1e3121d3a53b18843e7825b56d86cac"
  - "repo:README.md#sha256:ac7b9cc92c876e73c7731f685482a15ef8ba9bc4a1ec9c1ff081e8dc2d14e089"
  - "repo:docs/public-stable-release.json#sha256:5a004d43808985a6652eaf8a7967352992a7313089cc2597d3fd69fc0ec65a1f"
  - "repo:scripts/check-public-stable-docs.py#sha256:39d8b26827f208a20c7785d8727c920fdef1e04fc23804c1ed21dd33aaee616a"
links: [product-purpose, release-verification]
reviewed_revision: "git:8a45250106590f065df639132298b840940a3a35"
status: active
---

# 안정판 공개 문서

공개 안정판 대장이 일반 사용자용 version·배포일·문서 범위·release note coverage 소유.
README·설치 HTML·제품 개요·문서 색인은 해당 안정판만 안내.
번호 시험판은 npm·GitHub·유지보수자 검증 기록에 보존, 일반 설치 안내 노출 제외.
test 후보는 대장 안정판 유지, stable 후보는 build·게시 전 요청 version·배포일 일치 필수.
