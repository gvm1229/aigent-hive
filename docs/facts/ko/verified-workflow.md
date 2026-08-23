---
schema_version: 1
pair_id: verified-workflow
topic_slug: verified-workflow
language: ko
counterpart: ../en/verified-workflow.md
title: "Verified workflow Skill"
summary: "복잡한 자연어 continuation을 verified-workflow로 routing하고 retry·Judge·복구·취소를 disposable 수용하는 0.10.0 결정"
tags: [orchestration, skills, v0-10]
aliases: ["ralph-loop"]
sources:
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:b88eaf08d187d6f83cfac8b9e3a186791f08b71d0d5287f5dafe4d2e7aaa8151"
  - "repo:docs/decisions/product-release-decisions.md#sha256:a56419242874c459f08f7575ec0b2b6c2249ac696e0efffb053706dfeb6c9f00"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:c2043678fc1e5ad2e8e2a9cb716e45ec44486b67f3a9af2349c8909e6f4b3a8b"
  - "repo:scripts/accept-verified-workflow.py#sha256:ad4bfb4f5c2b477a5900f0e28161ce1baee155af1b96cb73e93a0ec871a149a5"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:f050bb65eeed570541346af6dc22c52cdc6dbaf9"
status: active
---

# Verified workflow Skill

`verified-workflow`는 evidence graph·bounded retry·독립 검증·exact recovery를 결합합니다. 자연어
continuation은 workflow signal 2개 이상일 때만 자동 선택하며 작업 길이와 bare `continue`는
근거가 아닙니다. Disposable 수용은 정규화 route·canonical run 생성·의도적 실패 뒤 성공
retry·별도 host-owned Judge receipt·새 process/session 복구·terminal 취소를 한 receipt로
검증했습니다. 이는 CLI process 복구만 증명하며 Codex desktop 재시작이나 단일 Judge의
인증 quorum 완료 권한은 증명하지 않습니다.
