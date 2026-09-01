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
- [x] [DIR10-009] 설치 사용자 지침의 다섯 살 이해 수준 설명·핵심 용어 풀이·정확성 보존
- [x] [DIR10-010] 영어·한국어·이중 언어 투영과 직접 갱신·사용자 byte 보존
- [x] [DIR10-011] `update-summary`의 2,000자 초과 때만 자동 축약, 제한 안 원문 불변
- [x] [DIR10-012] 승인된 설명 개선 문구 추가·2,000자 제한·새 승인 지문 등록
- [x] [DIR10-013] 관련 Rust·Python·문서·upgrade 회귀 검사
- [x] [DIR10-014] 다음 번호 공개 시험판과 세 운영체제 설치 지침 수용
- [x] [DIR10-015] 모든 변경 branch의 develop 우선 통합과 develop 전용 main PR 계약

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

- 쉬운 설명 후속: Hive CLI Rust 448개 통과, 설정 회귀 73개 통과·Windows 조건부 5개 제외, 안내·정적 계약 43개 통과
- 승인 안내문: 1,989자, SHA-256 `ce658d7a5addabc93d69c99d3bea80fd0137c61d3141c9880c05fa1e50d4e426`, 외부 지문 등록 성공
- 공개 수용: `0.10.0-test.11` 공개 바이너리의 영어·한국어 지침 포함과 설치 미리보기, 세 운영체제 통과
- branch 통합: `release/` 포함 모든 변경 branch → develop → main, main PR head=develop 정적 계약 추가

- 구현: `8388428`, `47d4663`, `630c783`, `64125db`, `f34c524d`
- Fact 정합화: `a9d9bd6`, `a09ed6a`, `e2c3dd6`
- 정적 예산: source `AGENTS.md` 4,442 byte, 활성 source directive 29.5% 축소, 소비자 router 80.0% 축소
- 중복·경로·투영 gate: `scripts/check-agent-directives.py` failure `0건`
- Python lane: documentation 45, security 103, contract 372, integration 84, release 55 통과
- Rust: workspace 전 범위 통과; Windows 파일 잠금 1건은 격리 재실행 통과
- Upgrade: 지원하는 모든 historical full base의 direct jump·foreign byte·rollback 검사 통과
- Historical base 변경: `harness/project-bases/**`, `harness/user-bases/**` `0건`
- Source Wiki: 156 page, error·warning `0건`
