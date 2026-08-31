# Agent 지침 경량화 `0.10.0`

> Checklist owner: `DIR10-*`
> 목표: 안전 의미를 유지한 단일 정본·조건부 load·중복 없는 projection

## Checklist

- [x] [DIR10-001] 활성 source·소비자 규칙 ownership 대장과 허용 entrypoint 요약·projection·historical 제외 경로 정의
- [x] [DIR10-002] Source `AGENTS.md` 8KiB 이하 router와 `01–08` directive 단일 소유권 정리, 활성 directive 총 byte 25% 이상 축소
- [x] [DIR10-003] 소비자 `AGENTS.md` Hive block 50% 이상 축소와 `00–03` 조건부 route, `verified-workflow`의 공통 continuation 복제 제거
- [x] [DIR10-004] `hive-render`의 preference 유무 공통 canonical template 경로와 legacy 거대 문자열·후처리 규칙 삽입 제거
- [x] [DIR10-005] `user_setup`·`user_install` 공통 완료·중단·안정판 block의 단일 내부 renderer와 영어·한국어 parity
- [x] [DIR10-006] 비허용 normalized 규범 중복 `0건`, size budget·router target·projection parity·중단 3조건 정적 gate
- [x] [DIR10-007] `0.7.0–0.9.5` direct upgrade·foreign byte·rollback·historical digest 불변과 Source Wiki error·stale source digest warning `0건`
- [x] [DIR10-008] 새 version 명시가 없는 개발 요청을 활성 `PLAN.md` product version·다음 번호 시험판에 귀속하고 임의의 미래 version 제안 차단 — `f34c524d`

## 규칙 소유권

| 범위 | 정본 |
| --- | --- |
| 응답·작업 선택·continuation | `.agents/directives/01-behavior.md` |
| Runtime·orchestration·artifact | `.agents/directives/02-architecture.md` |
| Git·worktree·시험판·안정판 | `.agents/directives/03-workflow.md` |
| 계획·상태·fact·Wiki·closure 절차 | `.agents/directives/04-documentation-state.md` |
| Filesystem·credential·외부 도구·파괴 작업 | `.agents/directives/05-security-safety.md` |
| Session manifest·경로 예약 | `.agents/directives/06-session-coordination.md` |
| 사용량 보호 | `.agents/directives/07-installed-usage-guard.md` |
| 사람용 문서 말투 | `.agents/directives/08-human-documentation-style.md` |

## 소스 설명 규칙 후속 정리

- 사용자 요청: 다섯 살 아이도 이해할 수준의 쉬운 설명을 대화·설명형 글 전체에 기본 적용
- 정본: `01-behavior.md`의 쉬운 말·용어 풀이·원인과 결과·정확성 보존, `08-human-documentation-style.md`에서 참조
- 검증: 기존 지침의 중복·크기·경로 검사, 문서·Source Wiki 정합성. 읽기 수준의 기계적 완전 보장 주장 금지
- 범위: 소스 지침·문서만 변경, 소비자 제품·승인된 업데이트 안내문·기존 출시 수용 불변
- 완료 근거: Windows 기존 정적 계약 22개 통과, 비허용 중복 0건·지침 크기 기준 유지

## 보존 경계

- `harness/project-bases/**`, `harness/user-bases/**`: byte 변경 `0건`
- 공개 CLI·schema·Skill ID 변경 `0건`
- Provider API·credential·외부 byte·stable 승인·continuation 중단 3조건 유지
- `REL10-005–007`: 실행 제외

## 완료 근거

- 구현: `8388428`, `47d4663`, `630c783`, `64125db`, `f34c524d`
- Fact 정합화: `a9d9bd6`, `a09ed6a`, `e2c3dd6`
- 정적 예산: source `AGENTS.md` 4,442 byte, 활성 source directive 29.5% 축소, 소비자 router 80.0% 축소
- 중복·경로·투영 gate: `scripts/check-agent-directives.py` failure `0건`
- Python lane: documentation 45, security 103, contract 372, integration 84, release 55 통과
- Rust: workspace 전 범위 통과; Windows 파일 잠금 1건은 격리 재실행 통과
- Upgrade: 지원하는 모든 historical full base의 direct jump·foreign byte·rollback 검사 통과
- Historical base 변경: `harness/project-bases/**`, `harness/user-bases/**` `0건`
- Source Wiki: 156 page, error·warning `0건`
