# Source project-only `update-summary` Skill

- 상태: 완료
- 제품 version: 변경 없음
- 범위: `aigent-hive` source workspace만 사용. `harness/`, release bundle, consumer harness와 설치 product inventory 변경 없음

## Checklist

- [x] [SUS-001] 명시 유지보수자 요청 기반 source-only 예외·비출하 경계의 정본 기록
  - Evidence: `AGENTS.md`, `.agents/directives/02-architecture.md`, source layout·product decision·ADR-0009 current truth
- [x] [SUS-002] 현재·직전 안정판 비교, 구독자 대상 한국어 제목·개선 항목·검증된 사실만 사용하는 `update-summary` workflow 작성
  - Evidence: `.agents/skills/update-summary/SKILL.md`의 verified source evidence·Korean output·nonshipping boundary
- [x] [SUS-003] `SKILL.md`·`agents/openai.yaml` 초기화와 구조 검증, human-documentation·Markdown link·Source Wiki gate
  - Evidence: `quick_validate.py` PASS, human documentation style finding `0`, Markdown link `666` PASS, Source Wiki index `128` pages·lint `0 error`·Korean query hit `1`
- [x] [SUS-004] 구독자가 체감하거나 활용할 수 없는 개발자·기여자 전용 변경을 update summary에서 명시적으로 제외
  - Evidence: `SKILL.md`의 end-user relevance filter. GitHub Release 언어 순서·형식, CI·검증 절차, 저장소 계획·기여자 workflow처럼 설치 product 또는 사용자 작업 방식에 영향을 주지 않는 항목의 출력 대상 제외

## Acceptance

- `$update-summary` 호출 또는 구독자용 버전 개선 내역 요청에서 source workspace 전용 workflow 선택
- 제품 harness·catalog·release artifact·consumer projection 변경 `0건`
- 새 Skill의 front matter·UI metadata 구조 검증 통과
- 모든 개선 항목은 구독자가 새로 할 수 있는 일·피할 수 있는 일·다르게 이해할 수 있는 일을 설명
