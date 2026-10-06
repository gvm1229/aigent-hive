---
schema_version: 1
pair_id: project-refresh-routing
topic_slug: project-refresh-routing
language: ko
counterpart: ../en/project-refresh-routing.md
title: "프로젝트 갱신 스킬 연결"
summary: "선택한 전역 프로젝트 갱신 스킬의 자연어 요청 지원."
tags: [project, routing, skill]
aliases: ["project-refresh"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade/skill_merge.rs#sha256:9c3362827e588d15401dd0f11a4853ea02155749864615a9083ebdfdd8126ae3"
  - "repo:harness/skills/project-refresh/SKILL.md#sha256:8c252fa5ef5c4c40647cc11127a404f0bac7c096648e8b3ca9f8af9655203067"
  - "repo:harness/skills/project-refresh/agents/openai.yaml#sha256:b2563a605a8a14b629efb04dc36c7f4b4e4c556f91b5ea9c6cdb454bc92fccf8"
  - "repo:schemas/project-skill-merge.schema.json#sha256:d0e320be45bbc7873261a9b125359d87e883c54903b2a11ab13939c56404a599"
links: [project-onboarding, skill-routing]
reviewed_revision: "git:5857c29c72341a0eb8a9fc55e360259144432dee"
status: active
---

# 프로젝트 갱신 스킬 연결

선택한 전역·프로젝트 스킬의 자연어 갱신 요청 지원. 미리보기의 실제 적용 금지. 인증된 갱신·정확 경로 예약·원본 보존·최종 검증을 스킬에서 수행, 사용자 명령 입력 불필요.

새 Hive 개선과 사용자 스킬의 검토 결합 지원. 공식 원문·지문 조회 뒤 본문과 연결 설정을 함께 검토. 대상·계획의 정확한 승인 지문 확인 후 적용, 현재·새 원문·결합 내용 변경 시 재검토. 실제 모델의 스킬 선택은 별도 호스트 검증 대상.
