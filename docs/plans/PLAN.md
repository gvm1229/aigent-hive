# Aigent Hive 활성 계획

> Revision: 244
> 기준일: 2026-08-20
> Product version: `0.9.5` stable
> 다음 target: `0.10.0-test`
> 현재 milestone: Graphify 실패 뒤 `0.10.0` 대체 범위 결정

## 목표

- Graphify 실패를 대체할 `0.10.0` 제품 범위 확정
- host-owned 프로젝트 Skill 경로 세션 예약 계약 정합화
- pre-`0.10.0` 지식·프로젝트 무손실 upgrade
- 번호 공개 시험판과 같은 product bytes의 안정판 출시

## 완료 조건

- 대체 제품 범위와 수락 기준의 사용자 확인
- Upgrade 전후 canonical Markdown·프로젝트 설정 byte 보존
- 기존 SQLite 직접 검색 결과 저하 `0건`
- Rust·Python·문서·보안·upgrade·rollback 전체 gate 통과
- 공개 번호 시험판의 세 운영체제 수용 뒤 같은 product bytes의 안정판 게시

## 중지 경계

- Graphify 하드 게이트 실패 뒤 대체 범위의 사용자 확인
- 승인 범위 밖 제품·dependency 추가
- protected `main` 통합과 stable publication 환경 승인

## Completion index

측정 정본: 아래 `Active fragments`의 checklist. Backlog와 archive 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| `0.10.0` 범위 확정·출시 | 0 | 8 | 0% |
| **합계** | **0** | **8** | **0%** |

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

1. `SCP10-001` 대체 범위 사용자 확인과 `SCP10-002` host-owned Skill 경로 세션 예약 계약 구현
2. 승인한 제품 범위 구현과 pre-`0.10.0` upgrade·rollback 검증
3. `REL10-*` 공개 시험판·세 운영체제 수용·안정판 출시

## 비활성 자료

- 버전 비종속 후보: [`backlog/README.md`](backlog/README.md)
- 완료·대체 기록: [`../archive/README.md`](../archive/README.md)
- 외부 참고 자료: [`references.md`](references.md)
