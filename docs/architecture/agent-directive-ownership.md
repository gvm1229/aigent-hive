# Agent 지침 ownership

## 목표

- Entry file: 짧은 router와 항상 필요한 안전 경계
- 상세 규칙: 작업별 단일 canonical directive
- Rust renderer·host projection: canonical source의 생성 결과
- Historical base: 검사 전용 immutable bytes

## Source 규칙

| Rule ID | Canonical owner | Entry 요약 허용 |
| --- | --- | --- |
| `behavior` | `.agents/directives/01-behavior.md` | 언어·활성 version·continuation 중단 3조건 |
| `architecture` | `.agents/directives/02-architecture.md` | Provider API·credential 금지 |
| `git-release` | `.agents/directives/03-workflow.md` | 안정판 version별 명시 승인 |
| `documentation-state` | `.agents/directives/04-documentation-state.md` | 계획·fact route |
| `security` | `.agents/directives/05-security-safety.md` | 사용자·외부 byte 보존 |
| `session` | `.agents/directives/06-session-coordination.md` | 자동 편집 route |
| `usage` | `.agents/directives/07-installed-usage-guard.md` | source task preflight |
| `human-style` | `.agents/directives/08-human-documentation-style.md` | 사람용 문서 route |

## 소비자 규칙

| Rule ID | Canonical owner | Generated projection |
| --- | --- | --- |
| `project-behavior` | `harness/directives/00-project-harness.md` | `.agents/directives/00-project-harness.md` |
| `project-knowledge` | `harness/directives/01-project-knowledge.md` | `.agents/directives/01-project-knowledge.md` |
| `project-upgrade` | `harness/directives/02-project-upgrade.md` | `.agents/directives/02-project-upgrade.md` |
| `project-session` | `harness/directives/03-session-coordination.md` | `.agents/directives/03-session-coordination.md` |
| `user-completion` | `crates/hive-cli/src/user_directives.rs` | Codex·Claude·Antigravity 사용자 marker |

`harness/template/AGENTS.md.jinja`: route·identity·항상 필요한 안전 요약만 소유.
`verified-workflow`: 실행 생성·검증·재시도·Judge 절차 소유. 소스는 `01-behavior`·`04-documentation-state`, 소비자는 `00-project-harness`의 연속 실행·종료 정책 참조.

- 소스 종료 기록: 실제 실행 대상·식별자·지문·작업 기준 연결과 `run closure` 결과 확인
- 소비자 종료 기록: 요청 전체와 실행 결과 대조, 명령 성공과 작업 완료 구분
- 소스 루트의 소비자 상태 생성 금지 유지. 격리 수용 성공을 실제 작업 완료 근거로 대체 금지

## 중복 허용

- Entry router의 한 문장 안전 요약과 canonical owner
- 영어·한국어 사용자 marker의 동일 의미 projection
- Canonical Skill과 host별 byte-identical projection
- Historical `harness/project-bases/**`·`harness/user-bases/**`

그 밖의 활성 surface normalized 규범 문장 중복: 허용 `0건`.

## 크기 기준

| Surface | 기준 |
| --- | ---: |
| Source `AGENTS.md` | 8KiB 이하 |
| 소비자 `AGENTS.md` Hive block | baseline 13,856 bytes 대비 50% 이상 축소 |
| Source `.agents/directives/*.md` 합계 | baseline 66,849 bytes 대비 25% 이상 축소 |
