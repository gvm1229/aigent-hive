# Aigent Hive 활성 계획

> Revision: 245
> 기준일: 2026-08-22
> Product version: `0.9.5` stable
> 다음 target: `0.10.0-test`
> 현재 milestone: `0.10.0` 관계·검색 범위 결정

## 목표

- Hive-native Markdown 관계 graph와 Graphify code-only 제한 채택 범위 확정
- host-owned 프로젝트 Skill 경로 세션 예약 계약 정합화
- 등록된 nested project의 안전한 knowledge scan 복구
- pre-`0.10.0` 지식·프로젝트 무손실 upgrade
- 번호 공개 시험판과 같은 product bytes의 안정판 출시

## 완료 조건

- 대체 제품 범위와 수락 기준의 사용자 확인
- Upgrade 전후 canonical Markdown·프로젝트 설정 byte 보존
- 기존 SQLite 직접 검색 결과 저하 `0건`
- 등록 project root 밖 sibling read·write와 전역 Git 설정 mutation `0건`
- Rust·Python·문서·보안·upgrade·rollback 전체 gate 통과
- 공개 번호 시험판의 세 운영체제 수용 뒤 같은 product bytes의 안정판 게시

## 중지 경계

- Graphify 전면 지식 graph 실패 뒤 제한 채택 범위의 사용자 확인
- 승인 범위 밖 제품·dependency 추가
- protected `main` 통합과 stable publication 환경 승인

## Completion index

측정 정본: 아래 `Active fragments`의 checklist. Backlog와 archive 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| `0.10.0` 범위 확정·출시 | 0 | 9 | 0% |
| **합계** | **0** | **9** | **0%** |

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
| [`active/release-0.10.0.md`](active/release-0.10.0.md) | `SCP10-*`, `REL10-*` | 범위 확정·번호 시험판·안정판 출시 |

## 실행 순서

1. `SCP10-001` 관계·검색 범위 사용자 확인
2. `SCP10-002` host-owned Skill 경로 예약과 `SCP10-003` nested project scan 구현
3. 승인한 Graphify·관계 검색 범위 구현과 pre-`0.10.0` upgrade·rollback 검증
4. `REL10-*` 공개 시험판·세 운영체제 수용·안정판 출시

## 비활성 자료

- 버전 비종속 후보: [`backlog/README.md`](backlog/README.md)
- 완료·대체 기록: [`../archive/README.md`](../archive/README.md)
- 외부 참고 자료: [`references.md`](references.md)
