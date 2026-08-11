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
  - "repo:.github/workflows/release-publish.yml#sha256:505cc48a16b2ccc7ca7fe39fdaf47d7b851a19810cb75c784fdfe5a6717c5823"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:f8c457200b2d02aafd77e71981e82af120aa2b91e3a23e877c2011fed38eabef"
  - "repo:docs/guides/signed-update-and-release.md#sha256:41b38d004edd0a2305919b183b706d65705c3f0b8b3998ac63308f529ae7a549"
  - "repo:docs/plans/active/release-0.9.0-stable-publication.md#sha256:3da4edef672d721a8fc0b9a83100f5cc6076f35111c50551348e5ef357016cb7"
  - "repo:docs/plans/active/release-0.9.0.md#sha256:c40863c90f3b8947dfe52bfe43ef1f52ae5f1ed72150f6fcc2921e10bcfaa39f"
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
