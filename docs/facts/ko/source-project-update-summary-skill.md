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
  - "repo:.agents/skills/update-summary/SKILL.md#sha256:39d78386d806ec055193c0e27b1e074f85655b752624eb756afdac3d82f4aa95"
  - "repo:docs/archive/plans/foundations/source-update-summary-skill.md#sha256:4c2eb48e174ddacef78f3b1d576db2f703f4807632feac925458128da4dd9039"
  - "repo:docs/releases/0.10.0.subscriber.ko.md#sha256:ce658d7a5addabc93d69c99d3bea80fd0137c61d3141c9880c05fa1e50d4e426"
  - "repo:scripts/register-stable-summary-approval.py#sha256:8cd05c881ecadb7324bb144b0ff20e9c1a3629e6386bcce4d31a99d86c8e6c10"
  - "repo:tests/results/discord-sectioned-notification-0.12.0.md#sha256:40aaa2a04ecd3d673e28b09be83c2fb4a81f82f0297d931cd9eea407131077d1"
links: [public-skill-identity, source-development, v0-9-full-release]
reviewed_revision: "git:0bcae67aa54b63c6d52b58ee959bdb4425fcbe82"
status: active
---

# Source 프로젝트 전용 업데이트 요약 Skill

`update-summary`는 소스전용·소비자배포 제외. 요청버전대비완료·검증된이점·핵심기술명·예시·선택·비용·한계를Hive초심자에게안내, 실패내부기록.
새기능→수정→개선 섹션·구분선·빈범주생략·변경당1회분류. 기본말투는명사형, 습니다종결·기계적함/음변환 제외.0.11.1말투·0.10.0기능설명 참고.
전체2,000자한계·초과때만자동정리. 미출시는초안, 문구승인과출시권한별도. gh지문등록만·수동설정불필요, 출시/전송·지문자동갱신 제외. 변경문구새승인·동일지문재시도.
0.12.0승인870자발송이력 보존. 명사형652자수정초안은미승인·미전송.
