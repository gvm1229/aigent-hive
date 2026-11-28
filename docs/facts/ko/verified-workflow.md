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
  - "repo:docs/archive/plans/verified-workflow-0.10.0.md#sha256:6b2ee66b721493ad6c69f2f22d56ed16a6b47886e7609a4be42bff9aea768c57"
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:71a8c4e0f75fdc3f99aa7f4d1c4e2ace759af640dbc4dd013174639e71a3f01b"
  - "repo:docs/decisions/product-release-decisions.md#sha256:3620f38dc575abdd65508175d3a33b7e998b957d70b05c8c807337eafa5a0321"
  - "repo:scripts/accept-verified-workflow.py#sha256:2b2b43f8f98c0ce8fbc583a6cf5ef360d6c5d18c7f6890812a3bea0625a75d0a"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:c06e1d0e8077c4fb0a20bb209c2580570454f354"
status: active
---

# Verified workflow Skill

`verified-workflow`: evidence graph·bounded retry·독립 검증·exact recovery 결합. 자연어
continuation 자동 선택 조건: workflow signal 2개 이상. 작업 길이와 bare `continue`는 선택
근거에서 제외. Disposable 수용 범위: 정규화 route·canonical run 생성·의도적 실패 뒤 성공
retry·별도 host-owned Judge receipt·새 process/session 복구·terminal 취소의 단일 receipt
검증. 증명 범위는 CLI process 복구이며 Codex desktop 재시작과 단일 Judge의 인증 quorum
완료 권한은 제외.

0.17.0: 제품 Skill은 `autonomous-workflow`로 교체. 옛 이름 호출은 새 이름과 `renamed_from`으로 route, 반복 상태 기계는 `hive loop` CLI에 유지.
