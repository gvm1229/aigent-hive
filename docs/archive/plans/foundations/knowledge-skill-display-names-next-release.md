# 다음 버전 지식 Skill 표시 이름

- 상태: 완료
- 대상: 다음 버전 미정
- 범위: `knowledge-capture`·`knowledge-recall`·`knowledge-promote`·`knowledge-maintain`·`knowledge-import`의 한국어 사용자 표시 이름
- 경계: 영문 정본 ID는 설명 첫머리만 유지. `v0.9.4` release·tag·package·시험판 변경 없음

## 목표

- 한국어 Skill 목록: 기능명만 표시
- 한국어 설명: 각 `(knowledge-*)` 정본 ID 첫머리 표시
- Codex·Claude·Antigravity 투영: 동일 표시·설명 분리

## Checklist

- [x] [KDN-001] 지식 Skill 다섯 개의 한국어 표시 이름에서 괄호 속 영문 정본 ID 제거
  - Evidence: `crates/hive-projection/src/lib.rs`의 한국어 `localized_skill_text` 표시 이름 다섯 개
- [x] [KDN-002] 세 host 한국어 투영에서 표시 이름 ID `0건`·설명 첫머리 ID 유지 회귀 검증
  - Evidence: `korean_knowledge_skill_labels_keep_ids_only_in_descriptions_on_every_host`, `cargo test -p hive-projection --locked` focused 검사
- [x] [KDN-003] 다음 버전 미정·release 변경 없음의 계획·현재 상태·bilingual fact·Source Wiki 증거 기록
  - Evidence: `PLAN.md`·`CURRENT.md`·`public-skill-identity` fact pair, documentation lane·Source Wiki index·lint

## 수락 기준

- 한국어 Skill 목록: `지식 찾아보기`처럼 기능명만 표시
- 한국어 설명: `(knowledge-recall)`처럼 정본 ID를 첫머리에 한 번만 표시
- 영어 표시명·정본 ID·선택 저장값·기존 `v0.9.4` artifact 불변
