# Aigent Hive 활성 계획

> Revision: 270
> 기준일: 2026-08-22
> Product version: `0.9.5` stable
> 다음 target: `0.10.0-test`
> 현재 milestone: Agent 지침 단일 정본·경량화
> 확정 범위: [`ADR-0020`](../decisions/ADR-0020-0.10.0-product-scope.md)

## 목표

- Hive-native Markdown 관계 graph와 Graphify code-only 제한 채택 구현
- FTS·vector·graph hybrid 검색의 hard gate와 조건부 구현
- host-owned 프로젝트 Skill 경로 세션 예약 계약 정합화
- 등록된 nested project의 안전한 knowledge scan 복구
- pre-`0.10.0` 지식·프로젝트 무손실 upgrade
- Host-neutral 연속 실행 closure gate와 조건부 hook adapter
- 자연어 routing 기반 `verified-workflow`와 명시적 `adversarial-judge`
- 모든 지원 predecessor의 Skill rename·폐기 artifact 완전 cleanup
- 번호 공개 시험판과 같은 product bytes의 안정판 출시

## 완료 조건

- 승인된 관계·검색 범위의 checklist·수락 기준 정합성
- Upgrade 전후 canonical Markdown·프로젝트 설정 byte 보존
- 기존 SQLite 직접 검색 결과 저하 `0건`
- Vector hard gate 실패의 product dependency·release 차단 `0건`
- 등록 project root 밖 sibling read·write와 전역 Git 설정 mutation `0건`
- Rust·Python·문서·보안·upgrade·rollback 전체 gate 통과
- 공개 번호 시험판의 세 운영체제 수용과 유지보수자의 명시적 승인 뒤 같은 product bytes의 안정판 게시

## 중지 경계

- 승인 범위 밖 제품·dependency 추가
- protected `main` 통합과 stable publication 환경 승인

## 현재 연속 실행 경계

- 유지보수자 권한: `0.10.0`의 구현·시험·commit·`feature/0.10.0` push·CI 관찰·번호 공개 시험판
- Verified workflow 대상: dependency·evidence·retry·독립 검증이 필요한 미완료 구현·검증 항목
- 제외: `REL10-005–007` — protected `main` 안정판 후보, 안정판 게시·설치, 유지보수자 안정판 승인
- 종료 조건: 제외 항목 외 Agent 소유 checklist `0건`과 해당 검증 증거

## 기본 출시 권한

- 모든 요청의 기본값: 구현·검증·번호 공개 시험판 범위
- 안정판 `tag`·protected `main` 통합·게시·설치: 유지보수자의 현재 요청 안 버전명 포함 명시 승인 전 금지
- `release`, `ship`, `continue`, `all todos`만으로 안정판 승인 추론 금지
- 번호 공개 시험판 수용 보고 뒤 안정판 여부: 유지보수자 별도 결정

## Completion index

측정 정본: 아래 `Active fragments`의 checklist. Backlog와 archive 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 관계·검색 graph | 3 | 13 | 18.8% |
| Hybrid vector search | 0 | 13 | 0% |
| Host-owned Skill 예약 | 1 | 0 | 100% |
| Nested project scan | 1 | 0 | 100% |
| Agent 지침 경량화 | 0 | 7 | 0% |
| Host-neutral 연속 실행 | 6 | 4 | 60.0% |
| Verified workflow | 5 | 1 | 83.3% |
| Adversarial judge | 4 | 4 | 50.0% |
| Skill migration cleanup | 3 | 7 | 30.0% |
| `0.10.0` 출시 | 0 | 7 | 0% |
| **합계** | **23** | **56** | **29.1%** |

## Required load order

1. 설치 product usage guard
2. `docs/plans/PLAN.md`
3. `docs/state/CURRENT.md`
4. 현재 작업을 소유한 active fragment
5. 직접 관련 architecture·decision·guide

Archive·backlog·완료 history의 자동 선행 load 금지.

## Active fragments

| Fragment | Checklist | 범위 |
| --- | --- | --- |
| [`active/knowledge-relationship-graph-0.10.0.md`](active/knowledge-relationship-graph-0.10.0.md) | `SCP10-001`, `KRG10-*` | Markdown·Graphify 관계 검색 |
| [`active/hybrid-vector-search-0.10.0.md`](active/hybrid-vector-search-0.10.0.md) | `KRG10-014`, `VEC10-*` | 조건부 semantic vector 검색 |
| [`active/host-owned-skill-reservations-0.10.0.md`](active/host-owned-skill-reservations-0.10.0.md) | `SCP10-002` | Host-owned Skill 세션 예약 |
| [`active/nested-project-knowledge-scan-0.10.0.md`](active/nested-project-knowledge-scan-0.10.0.md) | `SCP10-003` | Nested project scan |
| [`active/agent-directive-optimization-0.10.0.md`](active/agent-directive-optimization-0.10.0.md) | `DIR10-*` | Source·소비자 Agent 지침 경량화 |
| [`active/host-neutral-continuation-0.10.0.md`](active/host-neutral-continuation-0.10.0.md) | `CON10-*` | Goal·closure·선택형 hook |
| [`active/verified-workflow-0.10.0.md`](active/verified-workflow-0.10.0.md) | `VWF10-*` | 자연어 routing·실행 graph |
| [`active/adversarial-judge-0.10.0.md`](active/adversarial-judge-0.10.0.md) | `JDG10-*` | 명시적 독립 adversarial Judge |
| [`active/skill-retirement-migration-0.10.0.md`](active/skill-retirement-migration-0.10.0.md) | `SKM10-*` | Rename·폐기 artifact cleanup |
| [`active/release-0.10.0.md`](active/release-0.10.0.md) | `REL10-*` | 번호 시험판·안정판 출시 |

## 실행 순서

1. `DIR10-001–007` source·소비자 지침 단일 정본·경량화·upgrade 수용
2. `CON10-002–010`, `VWF10-002–006` closure·natural routing·bounded three-host adapter 구현·수용
3. `SKM10-002–010` 모든 stable registry·predecessor Skill cleanup과 direct jump upgrade
4. `JDG10-002–008` explicit adversarial Judge·`judge-evidence`·host launch·quorum 결합
5. `KRG10-001–007`, `VEC10-001–007` native 관계·vector feasibility·adopt|defer
6. `KRG10-008–013`, `KRG10-015–016`과 통과 시 `VEC10-008–012` 구현·수용
7. `REL10-*` 공개 시험판·세 운영체제 수용·안정판 출시

## 비활성 자료

- 버전 비종속 후보: [`backlog/README.md`](backlog/README.md)
- 완료·대체 기록: [`../archive/README.md`](../archive/README.md)
- 외부 참고 자료: [`references.md`](references.md)
