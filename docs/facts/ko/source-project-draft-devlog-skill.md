---
schema_version: 1
pair_id: source-project-draft-devlog-skill
topic_slug: source-project-draft-devlog-skill
language: ko
counterpart: ../en/source-project-draft-devlog-skill.md
title: "Source 프로젝트 전용 Draft Devlog Skill"
summary: "사용자 제공 임시 token과 명시 발행 권한으로 PortareFolium MCP에 일반화된 한국어 기술 글을 생성·수정하는 비출하 source-project Skill draft-devlog"
tags: [blog, development, mcp, skill]
aliases: ["draft-devlog"]
sources:
  - "repo:.agents/skills/draft-devlog/SKILL.md#sha256:aeab2ab8f8745790c13283a82e1e85ed07c74bc05d3a2be52d4e7ed3cb284e64"
  - "repo:.agents/skills/draft-devlog/scripts/portfolio_mcp.py#sha256:8562b9628443dbf4e366f31f9de83da1501480969ff4121a05c5a20e3a72dd0c"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
links: [source-development, source-project-update-summary-skill]
reviewed_revision: "git:f1c89f0998447f3bc53fbe0560521874efc65323"
status: active
---

# Source 프로젝트 전용 Draft Devlog Skill

`draft-devlog`: 명시 유지보수자 승인 기반 비출하 source workspace Skill. `tools/list`·`get_schema`
확인 뒤 PortareFolium MCP 사용. 새 글 기본값 `published=false`, 발행·공개 글 수정은 현재 요청의
명시 권한 필요. 작업별 사용자 제공 임시 token만 사용하며 파일·receipt·fact·log 기록 금지.
검증된 방법·수치는 유지하고 Hive version·branch·commit·plan·checklist ID·경로·credential 등
source 내부 문맥 제거. 제품 projection `0건`.
