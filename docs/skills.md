# 스킬 모음

설치 제품용 Skill의 이름·기능·사용 예시 정본. Aigent Hive source 개발도 같은 제품 Skill 사용.
명시 유지보수자 요청의 비출하 source-project `update-summary|draft-devlog`는 이 제품 목록과 분리.
이름 이관 구현: [`SIL-008–015`](archive/plans/foundations/skill-identity-localization.md). 호출 형식:
`$aigent-hive:<Skill 이름>`.

## 이름 원칙

- Active Skill: 설치 제품의 단일 목록만 유지
- Source 전용 Skill: 비출하 `update-summary|draft-devlog|autonomous-workflow` 3건. 경로: `.agents/skills/<name>/`; 제품 catalog·release bundle·consumer projection 제외. `autonomous-workflow`는 0.17.0부터 같은 본문의 제품 Skill도 존재(본문 동일 계약 시험)
- 제품 이름: 사용자가 인식할 기능 중심
- 모든 Hive Skill: 선택 언어·host와 무관하게 설명 첫머리에 `(정본 영문 ID)` 표기
- 폐기 ID: scope가 지정된 one-to-one·merge·split 이관 입력으로만 허용
- Historical release byte: 변경 금지

## 제품 Skill 목록

| Skill 이름 | 기능 | 사용 예시 |
| --- | --- | --- |
| `quick-answer` | 별도 조사 없는 독립 질문 응답 | `rehash` 명령 의미 설명 |
| `project-setup` | 저장소 근거를 활용한 안내형·자동 프로젝트 설정 | 저장소 최소 질문 설정 |
| `code-polish` | 동작·회귀 시험을 보존하는 생성 코드 정리 | 변경 파일의 중복 wrapper 정리 |
| `humanize-kor` | 의미·수치·인용·링크를 보존하는 기존 한국어 text 윤문 | 번역투가 있는 한국어 문서의 보수적 윤문 |
| `autonomous-workflow` | 중간 질문 없이 끝까지 진행, 진짜 질문은 마지막에 한 번 | 자리를 비운 동안 작업 끝내기 |
| `team-execution` | Mailbox·장벽·경로 lease·취소를 적용한 제한된 팀 조율 | 분리된 구현·검증 lane 실행 |
| `multi-goal` | 집계 규칙·중첩 예산·종료 검증을 적용한 목표 graph 실행 | AND 기준을 가진 복수 목표 실행 |
| `custom-subagent-create` | Codex·Claude custom subagent profile 추천·동의·생성 | 구현·검증 역할의 제한된 profile 생성 |
| `knowledge-manage` | 기존 Hive 지식의 검사·색인 재생성·명시 정리, 검토된 지식의 전역 공유, 컴퓨터 간 이전 | Wiki link 검사, 배포 규칙 전역 공유, 새 컴퓨터로 지식 이전 |
| `knowledge-capture` | 대화 종료 전 나중 작업에 도움이 되는 사실·선호·방식의 안전한 기록, 사용자가 고른 저장소·폴더의 검토된 지식 추출 | PR 전 Clippy 실행 규칙 기록, 저장소 규칙·결정의 Wiki 반입 |
| `prompt-refine` | 실행 전 승인용 prompt 정리·개선 | Codex 실행용 prompt 개선 |
| `research-best-practices` | 공식 자료 중심의 최신 모범 사례 조사 | Rust 자동 update 모범 사례 조사 |
| `knowledge-recall` | 현재 질문·작업에 도움 되는 Hive 지식의 제한 조회 | 기존 출시 결정 검색 |
| `usage-guard` | 전역·project별 사용량 보호 상태·사용자 선택 기준 관리 | 특정 project의 조기 중지 한도 설정 |
| `ship` | 저장소 규칙을 읽고 변경을 독립 관심사별로 검증·commit·선택적 push | 큰 변경을 기능·문서·release commit으로 분리 |
| `amend-directive` | 전역·project·Hive source의 사용자 수정 가능 agent behavior 변경 | Setup의 사소한 승인 질문 금지 |
| `user-setup` | 언어·Wiki·host·Skill·사용량 보호 전역 설정 | 전역 Aigent Hive 설정 변경 |
| `run` | 실행 상태 저장, 역할 인계 기록, 저장된 실행의 새 session 재개 | Context 정리 전 진행 상태 저장, `RUN-42` 재개 |
| `project-transition` | 구조·major version이 다른 프로젝트 이관 | Project의 다음 major 형식 이관 |
| `product-update` | 설치된 Aigent Hive 자체 갱신 | Aigent Hive 최신 stable update |
| `project-refresh` | 사용자 수정을 보존하는 project Hive 파일 갱신 | Project의 current Hive format refresh |
| `judge` | 제한된 clean-context 독립 Judge 요청 준비, package 출처·무결성·서명 증거 확인 | 명시적 반례 검토, release candidate의 독립 review 준비 확인 |
| `athena` | 작업 계획과 완료 기준을 구현 전에 저장하고 결과를 별도로 검토 | 아테나 호출 |
| `artemis` | 코드 변경 없이 자료를 조사하고 문제 원인을 진단 | 아르테미스 호출 |
| `hephaistos` | 저장한 계획을 구현하고 결과 코드를 정리 | 헤파이스토스 호출 |
| `apollon` | 글을 쓰고 다듬으며 요청문과 프롬프트를 정리 | 아폴론 호출 |
| `eileithyia` | 사용자나 프로젝트의 Hive를 설정하고 업데이트를 설치 | 에일레이티이아 호출 |
| `poseidon` | 여러 작업의 역할을 배정하고 작업 간 인계를 기록 | 포세이돈 호출 |
| `demeter` | Hive 지식을 저장·검색·관리하고 다른 컴퓨터로 이전 | 데메테르 호출 |
| `aither` | 사용량을 확인하고 사용량 보호 기준을 관리 | 아이테르 호출 |
| `hades` | Hive 저장 공간을 측정하고 정리할 파일을 미리 보여 줌 | 하데스 호출 |

9신 입구: 정본 이름·별칭 명시 요청 전용, 일반 작업 문구는 기존 전문 Skill 소속. `harness/gods.toml`에서 위 신 입구 행 생성, 기존 기능·승인 경계 유지. 소스 개발 브랜치의 추가 항목이며 공개 안정판의 설치 범위는 README 안내 기준. [기능 소속·역할 이전](architecture/god-ownership.md) 참조.

## Source Skill 폐기 경로

| 폐기 source 이름 | 제품 경로 |
| --- | --- |
| `source-answer` | `quick-answer` |
| `source-project-setup` | `project-setup` |
| `source-code-polish` | `code-polish` |
| `source-ralph-loop` | `autonomous-workflow` |
| `source-knowledge-import` | `knowledge-capture` |
| `source-knowledge-maintain` | `knowledge-manage` |
| `source-knowledge-capture` | `knowledge-capture` |
| `source-prompt-refine` | `prompt-refine` |
| `source-research-best-practices` | `research-best-practices` |
| `source-knowledge-recall` | `knowledge-recall` |
| `source-commit-work` | `ship` |
| `source-amend-directive` | `amend-directive` |
| `source-review` | Wiki 질문은 `knowledge-recall`; code·Git 근거 확인은 기본 읽기 도구 |
| `source-knowledge` | 조회 `knowledge-recall`; 관리 `knowledge-manage`; 기록 `knowledge-capture` |

## 병합 결정

- `auto-setup-project` + `setup-project` → `project-setup`: 동일 목적의 질문 방식 통합
- `manage-wiki` + `maintain-knowledge` → `knowledge-manage`: thin router와 실제 관리 작업 통합(`0.16.0` 이전 이름 `knowledge-maintain`)
- `hive-usage-guard` + `manage-usage` → `usage-guard`: 단일 설정·정책 resolver
- `hive-commit` + source Git workflow → `ship`: universal workflow와 저장소별 규칙 분리
- `hive-directive-amend` → `amend-directive`: 사용자 수정 가능 behavior와 immutable safety 경계 분리
- `source-review` 제거: Wiki 조회와 기본 read-only repository inspection으로 분리
- `source-knowledge` 제거: 기존 세 knowledge Skill과 `hive source-wiki` CLI route로 분리

## 유지할 분리

- `knowledge-recall` / `knowledge-capture` / `knowledge-manage`: 읽기 전용 조회, 새 지식 기록, 기존 지식 관리의 범위와 권한 차이
- `project-refresh` / `project-transition`: 일반 갱신과 구조 이관의 복구 위험 차이

## 범용화 경계

- `ship`: current repository의 `AGENTS.md`·Git guide·branch protection·검증 명령을 읽어 적용.
  Aigent Hive의 `develop`·`main` 규칙은 제품 Skill에 포함하지 않고 이 저장소 문서에만 유지
- `amend-directive`: 사용자 또는 repository가 소유한 directive와 Hive-owned marker만 preview 뒤
  수정. 서명된 release·plugin cache 직접 수정 금지. Aigent Hive source root에서는 tracked
  `.agents/directives/`와 canonical `harness/`를 repository 지침에 따라 함께 수정 가능
- Rust 강제 경계: path ownership, signature, credential, provider API 금지, foreign byte 보존 같은
  안전 규칙. 대화 말투·setup 질문·workflow preference 같은 behavior는 directive로 수정 가능
- Source Wiki: `hive-source.json`을 발견한 제품 knowledge Skill이 consumer command 대신
  `hive source-wiki` CLI로 route. 별도 source Skill 필요 없음

## `hive-loop-engineering` 계보

`hive-loop-engineering` → current `engineer-run` → product `verified-workflow`(0.10.0–0.16.x) → `autonomous-workflow`(0.17.0). 반복 상태 기계는 `hive loop`·`hive orchestration` CLI에 유지

## `0.10.0` 이름 이관

- `knowledge-import` → `knowledge-scan`; 기존 지식의 컴퓨터 간 이전은 `knowledge-transfer`

- `ralph-loop`와 `iterative-execution` → `verified-workflow`(0.17.0부터 `autonomous-workflow`)
- `package-review` → `judge-evidence`
- Retired 이름: routing compatibility만 제공
- Successful upgrade: authenticated retired Skill artifact·projection `0건`

## `0.16.0` 통합

| 옛 이름 | 새 이름 | 새 Skill 안 위치 |
| --- | --- | --- |
| `knowledge-scan` | `knowledge-capture` | 저장소 스캔 `references/scan.md` |
| `knowledge-maintain` | `knowledge-manage` | 정비 `references/maintenance.md` |
| `knowledge-promote` | `knowledge-manage` | 공유 `references/promote.md` |
| `knowledge-transfer` | `knowledge-manage` | 이전 `references/transfer.md` |
| `run-checkpoint` | `run` | 저장 `references/checkpoint.md` |
| `run-handoff` | `run` | 인계 `references/handoff.md` |
| `run-resume` | `run` | 재개 `references/resume.md` |
| `adversarial-judge` | `judge` | 반대 검토 `references/adversarial.md` |
| `judge-evidence` | `judge` | 증거 검증 `references/evidence.md` |

- 기준: 같은 대상·권한 단위로 묶음. 읽기 전용 조회, 새 지식 기록, 기존 지식 관리를 분리
- 옛 이름 호출: 새 Skill 설명과 본문의 옛 이름 안내, 새 이름과 위치 한 번 안내 뒤 계속
- 이전 단계 기록: `0.10.0`의 `knowledge-import`·`package-review` 이관도 최종 이름으로 직접 연결
- 갱신: 인증된 옛 Skill 폴더 제거, 사용자 수정·외부 파일 보존과 충돌 보고

## `0.17.0` 이름 이관

- `verified-workflow` → `autonomous-workflow`(이름 변경, 유지관리자 2026-11-28 결정: 두 달간 사용 기록 없음)
