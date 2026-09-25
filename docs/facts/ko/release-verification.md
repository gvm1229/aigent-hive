---
schema_version: 1
pair_id: release-verification
topic_slug: release-verification
language: ko
counterpart: ../en/release-verification.md
title: "Release 검증"
summary: "출시 검증·Markdown 전용 repository 통합·local bundle 무결성의 분리."
tags: [release, security, verification]
aliases: ["Release integrity"]
sources:
  - "repo:.agents/directives/references/documentation-verification.md#sha256:310146e429ce94f0c091b4514d9bf2d900779205d52831dc10bc03d1ed72b281"
  - "repo:.agents/directives/references/release-qualification.md#sha256:36968bb8a9bc012cfb7b72343ae9f8551cc0c0536232283f2794b822853d856e"
  - "repo:docs/decisions/ADR-0008-release-integrity.md#sha256:bace760d9be892a1e4f1f0554d2d55bbbaae85065125e9fae19a994f60f27410"
links: [judge-verification, update-transaction]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Release 검증

- 획득 출처: npm registry integrity 또는 GitHub exact tag attestation
- Local 검증: artifact path·length·SHA-256
- Update: transactional activation
- 거부: downgrade·같은 sequence의 다른 manifest
- Stable 비필수 항목: release private key·platform certificate
- Markdown 전용 후속 변경: 관련 local 문서·packaging·directive·link gate PASS 뒤 전체 플랫폼
  CI 대기 생략 가능
- 후속 문서 통합의 새 test·stable release 생성 `0건`, 미완료 CI의 통과 보고 `0건`
