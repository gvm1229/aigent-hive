# ADR-0014: `docs/` 기반 source Wiki

- 상태: accepted
- 날짜: 2026-07-31
- 범위: source 문서 구조, durable knowledge, source Wiki 색인
- 대체 대상: ADR-0011의 standalone source-Wiki 저장 경로 결정

## 문제

- 간결한 README 전환 과정에서 기존의 유효한 제품·개발 지식 누락
- 별도 source-fact directory로 인한 사람용 문서와 agent용 knowledge의 이중 구조
- 여러 사실과 서로 다른 주제를 한 page에 함께 둔 source Wiki
- `docs/`의 architecture·decision·guide·research·plan·state 사이를 연결하는 공통
  진입점과 전체 색인 부재

## 결정

### 문서 계층

- `docs/00-home.md`: 목적별 공통 진입점
- `docs/01-index.md`: 전체 문서의 상태·주제·한 줄 설명 색인
- 기존 topic directory: 사람이 읽는 설명, 결정, 절차, 연구, 계획, 현재 상태
- `docs/facts/en/`, `docs/facts/ko/`: 한 문서에 한 가지 reusable fact를 담는 exact pair
- Topic directory의 `README.md`: 해당 영역의 map of content
- Current fact, current human guide, historical record의 경로와 상태 구분

`facts/`는 AI 전용 문서함이 아니라 `docs/` Wiki의 원자 knowledge 계층. Human-readable
설명과 machine-searchable metadata를 같은 tracked Markdown에 결합.

### 지식 보존

- README·guide·overview 간소화 전 제거 후보의 유효성·정본·대체 위치 확인
- 유효한 knowledge는 삭제 대신 적절한 topic document 또는 atomic fact로 이동
- Deprecated·incorrect·superseded knowledge만 active tree에서 제거
- 제거 시 replacement locator 또는 제거 사유 확인
- Ordinary recovery는 Git history 사용, secret·legal erasure만 별도 history purge

### Source Wiki 계약

- Canonical fact root: `docs/facts/en/`, `docs/facts/ko/`
- Pair identity, language, reciprocal counterpart, source digest, reviewed revision 유지
- 한 page의 primary fact 1개와 직접 필요한 context만 허용
- Unrelated fact, workflow diary, raw transcript, tool output, runtime state 수집 금지
- Derived index와 advisory lock: ignored `.agents/work/source-wiki/`
- Explicit rebuild, fail-closed query, no-network clean-checkout rebuild 유지

### Migration

- Git의 간소화 직전 README와 현재 문서를 항목별 대조
- 중복 knowledge는 기존 current document로 연결
- 누락 knowledge는 topic document로 복원
- 기존 standalone source Wiki page는 atomic pair로 분할·이동
- CLI·Skill·test·directive의 canonical path를 `docs/facts/{en,ko}`로 전환
- 모든 old-path reference 제거와 lint·query·clean-copy 검증 뒤 standalone directory 삭제

## 결과

- 사람과 agent가 같은 `docs/` graph를 서로 다른 깊이로 탐색
- README 간소화와 durable knowledge 보존의 동시 달성
- 한 사실의 발견·갱신·폐기 범위 축소
- OMX·OMC와 독립적인 source knowledge ownership 유지
- SQLite 손실 뒤 tracked Markdown 기반 deterministic rebuild 유지
