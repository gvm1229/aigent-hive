---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.0 시험·정식 릴리스"
summary: "v0.9.0: GitHub attestation·SHA-256·npm OIDC provenance의 최소 trust와 same-byte publication."
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.0 release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:903c4fd819d0d09afdbc379ac874a22d592274b495aab6de82ee15166381bcbb"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:f8c457200b2d02aafd77e71981e82af120aa2b91e3a23e877c2011fed38eabef"
  - "repo:docs/guides/release-update.md#sha256:785e83d497c4f39ea683ac280adf8e071b27fda02b19c4c086573782a70bcadb"
  - "repo:docs/plans/active/release-0.9.0-stable-publication.md#sha256:e3062418251fe301c275214b1f35a54d057648e7bfc7ac582b154b68506d7089"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:085ecd5d61f590106f651f929c33c21ac4b87d296f4a603f430f605dba6d1805"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:1b7ea99554fcf2e475cc77dcb1a3452a7805315f"
status: active
---

# Aigent Hive 0.9.0 시험·정식 릴리스

Stable `v0.9.0`: 미게시. 필수 trust: protected `main` exact tag, same-candidate GitHub Release,
SHA-256 sidecar, GitHub artifact attestation, npm Trusted Publishing OIDC·provenance. Human gate:
GitHub stable environment 한 번. macOS ad-hoc·Windows unsigned 상태 공개. 외부 release trust
ceremony·platform certificate gate: 제거 대상. Transactional backup·rollback·recovery 유지.
Replacement candidate 전 usage guard 권장 기본·failure-only CodexBar·projection purge 보정 필수.
