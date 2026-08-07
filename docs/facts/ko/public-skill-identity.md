---
schema_version: 1
pair_id: public-skill-identity
topic_slug: public-skill-identity
language: ko
counterpart: ../en/public-skill-identity.md
title: "Public Skill identity"
summary: "Consumer Skill의 짧은 동작 이름, aigent-hive plugin namespace, 선택 언어 descriptor, fail-closed legacy migration 계약."
tags: [localization, migration, plugin, skill]
aliases: ["Skill naming"]
sources:
  - "repo:docs/plans/PLAN.md#sha256:7da60789ebf4f03df4fbdf3b970878e186230ef62d2e32c0d8bc403d0e2e91d9"
  - "repo:docs/plans/active/skill-identity-localization.md#sha256:143bbe51fdd932b352471e586a873420d0037af4950050f01cab1647277f2f0d"
links: [global-onboarding, skill-routing]
reviewed_revision: "git:90624108d8774fea2ed71efe64a5263cbb14fbe5"
status: active
---

# Public Skill identity

Consumer Skill: 짧은 동작 이름과 `aigent-hive:<name>` namespace. 저장된 retired ID: current ID 이관,
새 projection: current ID만 출력. Canonical retired-ID ledger: 이름 해석·예약 전용. old projection 삭제
권한: frozen historical release inventory 또는 installed ownership manifest의 exact byte 검증만 허용.
변조·unknown·foreign path: write 없이 conflict. Hive-owned display name·description: 선택 `en|ko`
interface language 적용.
