# Aigent Hive 활성 계획

> Revision: 246
> 기준일: 2026-08-22
> Product version: `0.9.5` stable
> 다음 target: `0.10.0-test`
> 현재 milestone: `0.10.0` 관계·검색·scan 구현

## 목표

- Hive-native Markdown 관계 graph와 Graphify code-only 제한 채택 구현
- host-owned 프로젝트 Skill 경로 세션 예약 계약 정합화
- 등록된 nested project의 안전한 knowledge scan 복구
- pre-`0.10.0` 지식·프로젝트 무손실 upgrade
- 번호 공개 시험판과 같은 product bytes의 안정판 출시

## 완료 조건

- 승인된 관계·검색 범위의 checklist·수락 기준 정합성
- Upgrade 전후 canonical Markdown·프로젝트 설정 byte 보존
- 기존 SQLite 직접 검색 결과 저하 `0건`
- 등록 project root 밖 sibling read·write와 전역 Git 설정 mutation `0건`
- Rust·Python·문서·보안·upgrade·rollback 전체 gate 통과
- 공개 번호 시험판의 세 운영체제 수용 뒤 같은 product bytes의 안정판 게시

## 중지 경계

- 승인 범위 밖 제품·dependency 추가
- protected `main` 통합과 stable publication 환경 승인

## Completion index

측정 정본: 아래 `Active fragments`의 checklist. Backlog와 archive 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| 관계·검색 graph | 1 | 16 | 5.9% |
| Host-owned Skill 예약 | 0 | 1 | 0% |
| Nested project scan | 0 | 1 | 0% |
| `0.10.0` 출시 | 0 | 6 | 0% |
| **합계** | **1** | **24** | **4.0%** |

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
| [`active/host-owned-skill-reservations-0.10.0.md`](active/host-owned-skill-reservations-0.10.0.md) | `SCP10-002` | Host-owned Skill 세션 예약 |
| [`active/nested-project-knowledge-scan-0.10.0.md`](active/nested-project-knowledge-scan-0.10.0.md) | `SCP10-003` | Nested project scan |
| [`active/release-0.10.0.md`](active/release-0.10.0.md) | `REL10-*` | 번호 시험판·안정판 출시 |

## 실행 순서

1. `KRG10-001–007` native 관계 graph·query planner·metadata 검색
2. `SCP10-002–003` host-owned Skill 예약과 nested project scan
3. `KRG10-008–016` Graphify adapter·drift·fallback·upgrade·수용
4. `REL10-*` 공개 시험판·세 운영체제 수용·안정판 출시

## 비활성 자료

- 버전 비종속 후보: [`backlog/README.md`](backlog/README.md)
- 완료·대체 기록: [`../archive/README.md`](../archive/README.md)
- 외부 참고 자료: [`references.md`](references.md)
