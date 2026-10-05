---
schema_version: 1
pair_id: host-external-integrations
topic_slug: host-external-integrations
language: ko
counterpart: ../en/host-external-integrations.md
title: "Discord 연결과 Notion 후보 폐기"
summary: "Discord 알림 유지, Notion 정본 후보 폐기와 과거 설계의 참고 보존."
tags: [discord, integration, notion]
aliases: ["Host integration priority"]
sources:
  - "repo:crates/hive-cli/src/discord.rs#sha256:8e46be8e49884c9fbfacee0b17c2588bd637ff08118e4d98465dbc7b45ccba77"
  - "repo:crates/hive-cli/src/usage_control.rs#sha256:41e5712b85c91de0f0df5505e999b245e60a530d5764f8cf4a35c285b8c4f730"
  - "repo:crates/hive-cli/src/user_setup.rs#sha256:ffaa44da03bc2179a4b9d7e743fb42d556506338072fe7de2d73dc3100409971"
  - "repo:docs/archive/plans/foundations/v0.10.0-notion-candidate.md#sha256:f863a6c59dde7c117e9b4b294cb0974e051ffca5970d830cfa75e50d9799dc4f"
  - "repo:docs/archive/plans/releases/0.9.0/discord-onboarding-v09.md#sha256:91a27ed57ddd259ac0a3270ee9242243f0a567bdae3fc756b90f76303c01c037"
  - "repo:docs/decisions/ADR-0018-notion-wiki-backend.md#sha256:160bc8bc434f1547e1fb3dad23902b740304323c800b92ece62ccda10a61114e"
  - "repo:docs/research/discord-notion-host-integrations.md#sha256:5b26108090c75343964f5452c3b7fd20a1df6300feda8561847bad6feb1748b9"
  - "repo:harness/skills/user-setup/SKILL.md#sha256:cf32fd58324f630d383593776f6d04cd3f9af72b7c2f125fa572c65b8303e841"
  - "repo:schemas/user-setup.schema.json#sha256:9c4d51829f1c6bc8553ed566a9663907dd5746d08b94a3a76c7744e21b556548"
links: [knowledge-storage, orchestration-ownership]
reviewed_revision: "git:ad412a7b2aa6bba927f5f841e0e5161f4fb60488"
status: active
---

# Discord 연결과 Notion 후보 폐기

- Discord 시험·실제 사용량 알림: 같은 현지화 Markdown 구성, 시험 안내 뒤 실제 내용
- 사용량·안전한 작업 정보·재개 안내 구역, 빈 줄·이모지·굵은 제목 사용
- 실제 알림: 안전한 프로젝트·진행 정보만 제공, 원문 요청·세션 식별자·개인 경로·자격 증명 제외
- 설정: webhook 환경 변수 이름과 비밀 없는 재개 답변만 저장
- 2026-10-05 사용자 결정: Notion 정본 후보 폐기. 과거 설계 참고 보존, 0.10의 예정 기능 주장 종료
- 완료된 Discord 연결·기존 제품 데이터와 별도 결정
