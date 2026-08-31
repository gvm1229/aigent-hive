---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord v0.9·Notion v0.10 host integration"
summary: "Discord v0.9은 현지화한 구역형 Markdown 사용량 알림을 실제 형식으로 시험·전송하고, Notion 연결·host OAuth는 첫 v0.10 시험판 보류."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:crates/hive-cli/src/discord.rs#sha256:8e46be8e49884c9fbfacee0b17c2588bd637ff08118e4d98465dbc7b45ccba77"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:6c5febe7ae1ac1a892f7ac412c40d1b8d9ae339fe73fa8153faf9bb22051e1c0"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:8ffb5878b47033d6756f32b270f43b5c8df19243df499d0668ba31678b35672d"
  - "repo:docs/archive/plans/foundations/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/archive/plans/releases/0.9.0/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:dff20a5844cfaa8a4958ea1755392f0c598c77fc6679e72e275de7249deb3c87"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:914cca3de8883e2b1be0dfbea92da3dd2c856cdca53ed24d3bd45d9ff75b6cd2"
  - "repo:schemas/user-setup.schema.json#sha256:daee52c6535601606bc39d67800ed2e6ad248828ac73383cc7d8ded015c95652"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:47d4663f1e1f263276f9ce54b7c69a3ff95d2170"
status: active
---

# Discord v0.9·Notion v0.10 host integration

Discord outbound 알림:

- 선택 인터페이스 언어 하나와 제한된 안전 항목·순서 사용
- 시험·실제 알림 공통 renderer, 시험 고지 구역만 실제 내용 앞에 추가
- 빈 줄로 구분한 Markdown 구역: 이모지·굵은 제목의 사용량·작업 정보·작업 계속 요청
- 밑줄 표기: `0건`
- 실제 중단 알림: 원문 prompt·session ID·절대 경로·credential 없이 project·run 제목·checklist 진행 상태·host·남은 사용량 표시
- 전역 설정: webhook 환경 변수 이름과 재개 가능한 비밀 없는 답변만 기록
- Notion: v0.10 내부 후보 유지, v0.9의 OAuth token·webhook URL·원문 prompt·절대 경로 제외
