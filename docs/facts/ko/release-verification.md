---
schema_version: 1
pair_id: release-verification
topic_slug: release-verification
language: ko
counterpart: ../en/release-verification.md
title: "Release 검증"
summary: "Npm·GitHub 출처 증거와 local bundle 무결성·transactional activation의 분리 검증."
tags: [release, security, verification]
aliases: ["Release integrity"]
sources:
  - "repo:docs/decisions/ADR-0008-release-integrity.md#sha256:bace760d9be892a1e4f1f0554d2d55bbbaae85065125e9fae19a994f60f27410"
links: [judge-verification, update-transaction]
reviewed_revision: "git:567c7000e56699b7fa82163164e0cc4a9dc1bd0b"
status: active
---

# Release 검증

- 획득 출처: npm registry integrity 또는 GitHub exact tag attestation
- Local 검증: artifact path·length·SHA-256
- Update: transactional activation
- 거부: downgrade·같은 sequence의 다른 manifest
- Stable 비필수 항목: release private key·platform certificate
