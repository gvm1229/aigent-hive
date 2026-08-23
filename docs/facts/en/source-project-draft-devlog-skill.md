---
schema_version: 1
pair_id: source-project-draft-devlog-skill
topic_slug: source-project-draft-devlog-skill
language: en
counterpart: ../ko/source-project-draft-devlog-skill.md
title: "Source Project-Only Draft Devlog Skill"
summary: "The nonshipping source-project Skill draft-devlog creates or revises generalized Korean technical posts through PortareFolium MCP with user-supplied temporary tokens and explicit publication authority."
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

# Source Project-Only Draft Devlog Skill

`draft-devlog` is an explicit maintainer-authorized nonshipping Skill for this source workspace. It
uses the PortareFolium MCP only after `tools/list` and `get_schema`, defaults new posts to unpublished,
and requires same-request authority for publication or public-post edits. The user supplies a
temporary token for each task; the token never enters files, receipts, facts, or logs. Blog content
keeps verified methods and measurements while removing Hive versions, branches, commits, plans,
checklist ids, paths, credentials, and other source-only context. No product projection exists.
Production acceptance verified public-post authority, an authorized identifier-only update,
matching metadata and content digest, and zero read-back policy findings.
