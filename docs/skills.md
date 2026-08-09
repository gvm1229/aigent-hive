# 스킬 모음

소스 개발용 Skill과 설치 제품용 Skill의 이름·기능·사용 예시 정본. 이름 이관 구현:
[`SIL-008`](plans/active/skill-identity-localization.md). 소비자 호출 형식:
`$aigent-hive:<제품 이름>`. 소스 호출 형식: `$<소스 이름>`.

## 이름 원칙

- 소스와 제품의 active Skill ID 중복 `0건`
- 같은 기능: 이름 계열 유지, exact 이름 분리
- 제품 이름: 사용자가 인식할 기능 중심
- 소스 이름: 제품과 구분 가능한 `source-*` 접두사 필수
- 폐기 ID: scope가 지정된 이관 입력으로만 허용
- Historical release byte: 변경 금지

## 전체 목록

| 현재 ID | 소스 이름 | 제품 이름 | 기능 | 사용 예시 |
| --- | --- | --- | --- | --- |
| `answer` | `source-answer` | `quick-answer` | 별도 조사 없는 독립 질문 응답 | `rehash` 명령 의미 설명 |
| `auto-setup-project`, `setup-project` | `source-project-setup` | `project-setup` | 저장소 근거를 활용한 안내형·자동 프로젝트 설정 | 저장소 최소 질문 설정 |
| `clean-ai-slop` | `source-code-polish` | `code-polish` | 동작·회귀 시험을 보존하는 생성 코드 정리 | 변경 파일의 중복 wrapper 정리 |
| `engineer-run`; historical `hive-loop-engineering` | `source-ralph-loop` | `ralph-loop` | 증거·재시도·완료 조건 기반 반복 실행 graph 설계 | 검증 가능한 Ralph loop 구성 |
| `import-repository-knowledge` | `source-knowledge-import` | `knowledge-import` | 저장소의 검토된 지식 후보 일괄 반입 | 저장소 규칙·결정의 Wiki 반입 |
| `manage-wiki`, `maintain-knowledge` | `source-knowledge-maintain` | `knowledge-maintain` | Wiki 검사·목록·색인·삭제·억제 관리 | Wiki link 검사와 색인 재구축 |
| `record-knowledge` | `source-knowledge-capture` | `knowledge-capture` | 하나의 검토된 사실·선호·작업 방식 기록 | PR 전 Clippy 실행 규칙 기록 |
| `refine-prompt` | `source-prompt-refine` | `prompt-refine` | 실행 전 승인용 prompt 정리·개선 | Codex 실행용 prompt 개선 |
| `research-practices` | `source-research-best-practices` | `research-best-practices` | 공식 자료 중심의 최신 모범 사례 조사 | Rust 자동 update 모범 사례 조사 |
| `search-knowledge` | `source-knowledge-recall` | `knowledge-recall` | 기존 지식에서 관련 결정·규칙 회수 | 기존 release 결정 검색 |
| `hive-usage-guard`, `manage-usage` | `source-usage-guard` | `usage-guard` | 현재 범위의 사용량 보호 상태·기준 관리 | 남은 사용량 20%의 자동 작업 중지 |
| `hive-commit` | `source-commit-work` | — | 소스 변경의 관심사별 검증·커밋 | 현재 변경의 독립 commit 분리 |
| `hive-directive-amend` | `source-amend-directive` | — | 소스·제품 agent directive 수정 | Setup의 사소한 승인 질문 금지 |
| `hive-editless-question` | `source-review` | — | 소스 저장소의 변경 없는 조사·상태 검토 | `v0.9.0` 계획 완료도 조사 |
| `hive-source-wiki` | `source-knowledge` | — | 소스 Wiki 조회·검사·색인·사실 기록 | Windows 설치 관련 source knowledge 검색 |
| `configure` | — | `user-setup` | 언어·Wiki·host·Skill·사용량 보호 전역 설정 | 전역 Aigent Hive 설정 변경 |
| `handoff-role` | — | `run-handoff` | 실행 역할과 남은 작업 인계 기록 | 검증 role과 남은 작업 인계 |
| `migrate-project` | — | `project-transition` | 구조·major version이 다른 프로젝트 이관 | Project의 다음 major 형식 이관 |
| `resume-work` | — | `run-resume` | 저장된 실행의 새 session 재개 | `RUN-42`의 마지막 checkpoint 재개 |
| `save-progress` | — | `run-checkpoint` | 현재 실행 상태와 다음 단계 저장 | Context 정리 전 진행 상태 저장 |
| `share-knowledge` | — | `knowledge-promote` | project 지식의 전역 재사용 지식 승격 | Deployment rule의 전역 knowledge 승격 |
| `update-hive` | — | `product-update` | 설치된 Aigent Hive 자체 갱신 | Aigent Hive 최신 stable update |
| `upgrade-project` | — | `project-refresh` | 사용자 수정을 보존하는 project Hive 파일 갱신 | Project의 current Hive format refresh |
| `verify-package` | — | `package-review` | package 출처·무결성·독립 검토 준비 확인 | Release candidate의 독립 review 준비 확인 |

## 병합 결정

- `auto-setup-project` + `setup-project` → `project-setup`: 동일 목적의 질문 방식 통합
- `manage-wiki` + `maintain-knowledge` → `knowledge-maintain`: thin router와 실제 관리 작업 통합
- `hive-usage-guard` + `manage-usage`: 동일 기능 계열, 이름은 source·product로 분리하고 상태·구현 공유 없음

## 유지할 분리

- `quick-answer` / `source-review`: 저장소 접근 경계 차이
- `knowledge-capture` / `knowledge-import` / `knowledge-promote`: 기록 규모·승격 권한 차이
- `source-knowledge` / `knowledge-maintain`: 자동 source gate·사실 기록과 명시적 관리 작업 차이
- `run-checkpoint` / `run-resume` / `run-handoff`: 저장·복구·인계 상태 변경 차이
- `project-refresh` / `project-transition`: 일반 갱신과 구조 이관의 복구 위험 차이

## `hive-loop-engineering` 계보

`hive-loop-engineering` → current `engineer-run` → approved source `source-ralph-loop`·product
`ralph-loop`. 기능 삭제 없음. Host가 실제 task를 실행하고 Skill은 반복 graph·증거·재시도·완료
조건만 소유하는 기존 경계 유지.
