# 계획 안내

## 탐색

| 목적 | 위치 | 자동 선행 확인 |
| --- | --- | --- |
| 현재 실행 | [`PLAN.md`](PLAN.md) | 예 |
| 버전 미지정 후보 | [`backlog/README.md`](backlog/README.md) | 아니요 |
| 완료·대체 기록 | [`../archive/README.md`](../archive/README.md) | 아니요 |
| 외부 참고 | [`references.md`](references.md) | 아니요 |

기본 확인 순서: `PLAN.md` → `../state/CURRENT.md` → 현재 작업의 active fragment.

## Active checklist

- `PLAN.md`의 `Active fragments` 등록 문서만 집계
- 모든 실행 항목의 고유 ID와 단일 소유자 필수
- `PLAN.md` 내부 checkbox 금지
- Backlog·Archive 항목의 완료율 집계 금지

## Revision

계획 변경 횟수의 단조 증가 정수. 이전 revision 복구: Git history와 Archive 명세.
