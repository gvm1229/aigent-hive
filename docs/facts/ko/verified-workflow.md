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
  - "repo:docs/decisions/ADR-0020-0.10.0-product-scope.md#sha256:7d48651ed01e4a694f8a699b43d589ab4d85342d1eb37e59c3ead322d8599868"
  - "repo:docs/decisions/product-release-decisions.md#sha256:e89ac8584204a7e52ed157e9b29d523f870b8ac387fc1e4a044f7f7333d17af5"
  - "repo:docs/plans/active/verified-workflow-0.10.0.md#sha256:6b2ee66b721493ad6c69f2f22d56ed16a6b47886e7609a4be42bff9aea768c57"
  - "repo:scripts/accept-verified-workflow.py#sha256:ad4bfb4f5c2b477a5900f0e28161ce1baee155af1b96cb73e93a0ec871a149a5"
links: [host-neutral-continuation, v0-10-product-scope]
reviewed_revision: "git:f050bb65eeed570541346af6dc22c52cdc6dbaf9"
status: active
---

# Verified workflow Skill

`verified-workflow`: evidence graph·bounded retry·독립 검증·exact recovery 결합. 자연어
continuation 자동 선택 조건: workflow signal 2개 이상. 작업 길이와 bare `continue`는 선택
근거에서 제외. Disposable 수용 범위: 정규화 route·canonical run 생성·의도적 실패 뒤 성공
retry·별도 host-owned Judge receipt·새 process/session 복구·terminal 취소의 단일 receipt
검증. 증명 범위는 CLI process 복구이며 Codex desktop 재시작과 단일 Judge의 인증 quorum
완료 권한은 제외.
