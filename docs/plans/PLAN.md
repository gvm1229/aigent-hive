# Aigent Hive 활성 계획

> Revision: 242
> 기준일: 2026-08-20
> Product version: `0.9.5` stable
> 다음 target: `0.10.0-test`
> 현재 milestone: 작업 자동 분담 가능성 조사

## 목표

- 현재 계획·상태 문서의 8KiB 이하 축소
- 완료 기록 archive와 버전 비종속 backlog 분리
- phase 중심 시험·fixture의 목적 중심 재편
- Graphify 조사 판정과 실패 후보의 backlog 이전
- 작업 자동 분담 기능의 공식·실제 가능성 조사
- pre-`0.10.0` 지식·프로젝트 무손실 upgrade

## 완료 조건

- 현재 계획·상태의 과거 연대기·완료 누적표 제거
- 모든 Python 시험의 목적별 단일 lane 소유
- 안정성 회귀의 근거 없는 삭제 `0건`
- Markdown 정본·SQLite 직접 검색 유지
- Graphify의 provider API·API key·query log·background 동작 `0건`
- 공개 번호 시험판의 세 운영체제 수용 뒤 같은 product bytes의 안정판 게시

## 중지 경계

- Graphify 하드 게이트 실패 뒤 `0.10.0` 대체 범위 결정
- private·confidential graph의 collection별 명시적 동의
- protected `main` 통합과 stable publication 환경 승인

## Completion index

측정 정본: 아래 `Active fragments`의 checklist. Backlog와 archive 제외.

| 범위 | 완료 | 미완료 | 진행률 |
| --- | ---: | ---: | ---: |
| `0.9.5` 출시 마감 | 5 | 1 | 83.3% |
| 문서 구조 정리 | 8 | 0 | 100% |
| 시험 구조 재편 | 9 | 0 | 100% |
| Graphify 조사 | 6 | 0 | 100% |
| 작업 자동 분담 조사 | 4 | 0 | 100% |
| `0.10.0` 출시 | 0 | 6 | 0% |
| **합계** | **32** | **7** | **82.1%** |

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
| [`active/release-0.9.5-stable-publication.md`](active/release-0.9.5-stable-publication.md) | `REL95-*` | 안정판 Windows 설치 마감 |
| [`active/documentation-structure-0.10.0.md`](active/documentation-structure-0.10.0.md) | `DOC10-*` | Archive·Backlog·현재 정본 축소 |
| [`active/test-organization-0.10.0.md`](active/test-organization-0.10.0.md) | `TST10-*` | 시험·fixture 목적별 재편 |
| [`active/graphify-knowledge-graph-0.10.0.md`](active/graphify-knowledge-graph-0.10.0.md) | `GPH10-*` | Graphify 조사·조건부 제품 도입 |
| [`active/host-work-delegation-research-0.10.0.md`](active/host-work-delegation-research-0.10.0.md) | `HWD10-*` | 작업 자동 분담 조사 |
| [`active/release-0.10.0.md`](active/release-0.10.0.md) | `REL10-*` | 번호 시험판·안정판 출시 |

## 실행 순서

1. `REL95-006` 증거 재조정과 Windows 안정판 설치 마감
2. `DOC10-*` 문서 archive·backlog·현재 정본 축소
3. `TST10-*` 시험·fixture 재편과 전체 기준 복구
4. `GPH10-001–006` Graphify 격리 조사와 채택 판정
5. 실패 후보 backlog 이전과 `0.10.0` 대체 범위 결정
6. `HWD10-*` 작업 자동 분담 조사
7. 대체 범위 승인 뒤 `REL10-*` 공개 시험판·안정판 출시

## 비활성 자료

- 버전 비종속 후보: [`backlog/README.md`](backlog/README.md)
- 완료·대체 기록: [`../archive/README.md`](../archive/README.md)
- 외부 참고 자료: [`references.md`](references.md)
