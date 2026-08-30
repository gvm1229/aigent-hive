---
schema_version: 1
pair_id: projection-upgrade-purge
topic_slug: projection-upgrade-purge
language: ko
counterpart: ../en/projection-upgrade-purge.md
title: "인증된 projection 갱신 정리"
summary: "Hive는 이전 Hive projection을 인증한 뒤 retired Skill과 직접 충돌하는 안전·소유권 규칙만 갱신"
tags: [consumer-harness, preservation, skills, upgrade]
aliases: ["PUG93"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:17b4e24061b7214faa292fa50e65e9b0d9902270bdbe86fdc06ae53b7970bf05"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:85b13d22add18756fa11e29fcc1ebcf84b18d143385991143a8453c29e3d0328"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:a50a29d628ce2e15e20b21fd74964ae96c493b259ffeabe0eade38cde54991aa"
  - "repo:crates/hive-update/src/merge.rs#sha256:4dc96d4c159d55be6664fa565dbb0eb77c1df532330f8a539f028ce51a9fcaaa"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:acb330569b20bdfe3aa993ade2a07e0142e1fe5f981074b5bb506f647e8e97c6"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:a6aea1ed5b977bc818bace5c9d712d2da01328f59753e9b93136c17b1a8f24d3"
  - "repo:tests/conformance/contracts/test_static_contracts.py#sha256:8818a6eb6d47a571477ec7beae8ecb3b7c70610944124bd5aea764e3a960d021"
links: [consumer-session-coordination, hive-preserving-uninstall]
reviewed_revision: "git:15128a22d61452bb22fd8d9e9168acd9d26340f8"
status: active
---

# 인증된 projection 갱신 정리

전역 설정은 retired-name ledger와 배포된 과거 Hive digest가 active byte와 모두 일치할 때만
`.agents/skills/<name>/SKILL.md`를 제거. 프로젝트 갱신은 인증된 project base inventory 사용.
incoming projection에 없는 미수정 retired 경로는 삭제, 수정·foreign byte는 보존.

Hive directive와 `AGENTS.md`의 Hive-owned marker는 safety·ownership 내용을 가진 incoming rule이
기존 Hive rule과 겹칠 때만 incoming 우선. 분리된 사용자 추가, foreign block, 안전과 무관한 겹침은
local 우선 유지. 모든 갱신에 preview·digest·atomic apply·rollback·빈 owned directory 정리 경계 적용.
