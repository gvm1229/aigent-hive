# Aigent Hive 문서 홈

Source 개발자와 agent를 위한 공통 문서 진입점.

## 목적별 시작

| 목적 | 시작 문서 |
| --- | --- |
| 제품 이해 | [제품 개요](overview/product.md) |
| 전체 문서 탐색 | [전체 색인](01-index.md) |
| 현재 상태 확인 | [CURRENT](state/CURRENT.md) |
| 다음 작업 확인 | [Active plan](plans/PLAN.md) |
| Source 구조 이해 | [Architecture 안내](architecture/README.md) |
| 결정 근거 확인 | [Decision 안내](decisions/README.md) |
| 개발·검증 실행 | [Development guide](guides/development.md) |
| 운영 절차 확인 | [Guide 안내](guides/README.md) |
| 외부 조사 확인 | [Research 안내](research/README.md) |
| 원자 knowledge 검색 | [Fact 안내](facts/README.md) |
| README 언어 선택 | [English](../README.md) · [한국어](readme/README.ko.md) |

## 문서 종류

```text
overview      제품의 목적·지원 범위·기능
architecture  현재 동작 구조와 trust boundary
decisions     채택한 선택과 배제한 대안
guides        사람이 실행하는 절차와 명령
research      외부 자료·version·확인일 기반 조사
facts         한 문서에 한 가지 reusable fact
plans         아직 완료되지 않은 목표와 acceptance
state         현재 handoff와 artifact record
readme        언어별 간결한 입구
```

## 정본 우선순위

1. Current source·schema·test
2. Accepted ADR·current architecture
3. `docs/state/CURRENT.md`
4. Active plan
5. Atomic fact
6. Historical Git revision

Fact와 source 불일치 시 source·ADR 우선. 유효한 knowledge의 문서 간 이동은
[지식 보존 규칙](decisions/ADR-0014-docs-wiki-architecture.md) 적용.

## 현재와 과거

- Current truth: `overview/`, `architecture/`, `decisions/`, `guides/`, `facts/`, `state/`
- Future work: `plans/`
- Dated external evidence: `research/`
- Ordinary history: Git
- Secret·legal erasure: 별도 승인된 exceptional history purge
