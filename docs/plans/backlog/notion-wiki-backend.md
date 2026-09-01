# Notion Wiki backend

- 상태: `idea`
- 마지막 검토일: 2026-08-20
- 관련 결정: [`ADR-0018`](../../decisions/ADR-0018-notion-wiki-backend.md)

## 문제

Notion을 지식 정본으로 선택한 사용자의 편집 경험과 Hive 검색·인용 계약 연결 필요.

## 기대 효과

- Notion 편집과 Hive 검색의 결합
- 기존 SQLite ranking·citation 재사용

## 현재 제외 이유

- host별 plugin·MCP·OAuth 동작 변화 가능성
- scope·부분 fetch·write confirmation의 실제 수용 근거 부재
- `0.10.0`의 Graphify·문서·시험 범위 집중

## 선행 조건

- 공식 host 연결 기능 재검증
- 사용자 선택 scope의 revision·read·create·update receipt
- OAuth token·host 설정의 Hive 저장·변경 `0건`
- Markdown mode upgrade·rollback 무손실 증거

## 승격 조건

세 지원 host의 연결·실패 복구 설계와 최소 한 host 실제 E2E 증거 확보.
