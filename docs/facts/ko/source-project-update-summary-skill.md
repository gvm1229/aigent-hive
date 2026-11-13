---
schema_version: 1
pair_id: source-project-update-summary-skill
topic_slug: source-project-update-summary-skill
language: ko
counterpart: ../en/source-project-update-summary-skill.md
title: "Source 프로젝트 전용 업데이트 요약 Skill"
summary: "소스 전용 update-summary의 검증 기반 제품 홍보: 새 기능·개선 구분, 핵심 기술 이름과 사용 이점 강조, 개발자 전용 변경 제외"
tags: [development, release-notes, skill]
aliases: ["update-summary"]
sources:
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:37f8bc71e3ef0e02ccaa82fc2d430dfb61be148f26be1709ac439c0424b8daf8"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
  - "repo:docs/decisions/subscriber-approval-via-main-2026-11-12.md#sha256:7686a84a596fff628fa03c8229666571c8bc2c88bc4cf6489d6a99befd8ec9fd"
  - "repo:docs/releases/0.10.0.subscriber.ko.md#sha256:ce658d7a5addabc93d69c99d3bea80fd0137c61d3141c9880c05fa1e50d4e426"
  - "repo:tests/results/discord-sectioned-notification-0.12.0.md#sha256:40aaa2a04ecd3d673e28b09be83c2fb4a81f82f0297d931cd9eea407131077d1"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:0bcae67aa54b63c6d52b58ee959bdb4425fcbe82"
status: active
---

# 소스 전용 업데이트 요약 스킬

소비자 배포 제외. 요청 버전의 검증된 이점·기술명·예시·선택·비용·한계 안내, 실패는 내부 기록. 명사형 문구, 기계적 종결 변환 제외. 0.11.1 말투·승인된 0.10.0 설명 참고.
새 기능→수정→개선, 구분선·빈 범주 생략·변경당 한 번 분류. 전체 2,000자, 초과 때만 정리. 미출시는 초안.
원문·SHA-256 파일의 검토된 main PR 승인, 최초 1회 환경 변수로 전송 허용. 출시 소스·파일명·바이트 검사, 변경 문구는 새 PR·출시 소스 필요. 출시별 등록 폐기, PR만으로 출시·전송 권한 발생 없음. 상세: 출시 안내·2026-11-12 결정.
