---
schema_version: 1
pair_id: upgrade
topic_slug: upgrade
language: ko
counterpart: ../en/upgrade.md
title: "Update, Upgrade와 Migration"
summary: "Version policy, signed root update, project-local three-way upgrade와 recovery 경계."
tags: [migration, update, versioning]
aliases: ["업데이트와 마이그레이션"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:ac9005e2003713a5c2e6bbaba3ecb50121a4ea370bb42204bee68326fc9133dd"
  - "repo:crates/hive-update/src/lib.rs#sha256:46a5cc32939b251d8f866df24c701356bfa645a24ca63bdbf531deaa8ea221da"
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:b314d07c19558eb0de0b629250ea19c4ede782f4afc66d92078e9660f75eb26e"
links: [boundaries, plugin-lifecycle, security-release]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Update, Upgrade와 Migration

Hive source, release bundle과 installed harness의 공통 product version: `X.Y.Z`. Compatible
feature는 minor 증가, compatible correction은 patch 증가, breaking change는 explicit major
authority 필요. Plan revision은 별도 lifecycle.

Root Hive update: signed release metadata, compatibility, provenance, backup, activation과
recovery 검증. Project upgrade: recorded base, current local bytes와 incoming release-generated
directive·Skill 비교. Unmodified file은 incoming version 우선. Locally modified file은 local
conflicting hunk 우선과 non-conflicting incoming change 추가.

공통 안전 경계: pinned filesystem capability, preflight validation, journal과 interrupted
mutation의 explicit recovery. Canonical project document, preference, knowledge, role, run, user
Markdown과 foreign bytes 보존. SQLite와 runtime cache는 migration authority가 아닌 rebuildable
output.
