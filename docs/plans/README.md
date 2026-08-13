# Plan entrypoint

정본 entrypoint: [`PLAN.md`](PLAN.md)

기본 load: `PLAN.md`, `../state/CURRENT.md`, `PLAN.md`의 active fragment.
완료 [`phases/`](phases/), unrelated [`stages/`](stages/), stable [`contracts/`](contracts/), reference fragment의 선행 load 금지.

Checklist 정본: `PLAN.md`의 `Active fragments`에 등록된 문서. 모든 actionable item의 unique ID 필수. `PLAN.md`에는 checklist 배치 금지.
Plan revision: 계획 변경 횟수를 나타내는 단조 증가 정수. 과거 `1.99` 다음의 `2.00`은
새 세대가 아닌 `100`번째 변경 표기였으므로, 이후 정본은 `185` 같은 단일 정수 사용.
이전 revision 복구: Git history.
