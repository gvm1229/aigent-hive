# Global onboarding과 shared index 계획

> Checklist owner: `UOS-*`
> Load condition: user setup, preference projection, project setup mode, shared knowledge index
> Decision: [`ADR-0012`](../../decisions/ADR-0012-global-onboarding-shared-index.md)
> Target: `0.8.0`

## 현재 구현 audit

| 요청 여정 | 현재 상태 | 판정 |
| --- | --- | --- |
| User plugin install | 세 host user-scope install·update 구현 | 충족 |
| 설치 후 mandatory global setup | Direct operational install, global answer schema 없음 | 미충족 |
| Language·identity·persona·multi-host 질문 | Project name/kind/primary host/role 질문만 제공 | 미충족 |
| Recommended 또는 개별 Skill 선택 | User plugin built-in 전체 projection, project optional Skill 개별 consent | 부분 |
| Wiki default-on opt-out | User/project Wiki 무조건 seed, enable flag 없음 | 미충족 |
| Usage guard opt-in·20% | Project threshold 필수, setup 문서 기본 `10%`, guard built-in | 미충족 |
| User guidance marker | `AIGENT-HIVE:USER:START|END` append·own-block replace | 충족 |
| User `.agents` projection | Host plugin package 중심, generic user `.agents` 없음 | 미충족 |
| User Wiki + SQLite | `~/.hive/knowledge` + disposable root SQLite 구현 | 충족 |
| Project expedited/custom | 단일 question sequence와 `general|custom` kind만 제공 | 미충족 |
| Project kind 필수 질문 | Project identity·domain profile 선행 | 충족 |
| Project Wiki, project DB 없음 | Project Wiki + project SQLite + root SQLite 독립 | 미충족 |

## 구현

- [ ] [UOS-001] User installation의 `bootstrap|setup-required|operational` state machine,
  operational route gate와 non-destructive reconfigure contract
- [ ] [UOS-002] `user-setup` schema에 interface language, Wiki language, user profile,
  persona, selected hosts, selected Skills, Wiki enabled, usage guard enabled·threshold 추가
- [ ] [UOS-003] Signed user profile·persona·recommended Skill suite catalog와
  unknown/custom value validation
- [ ] [UOS-004] One-question-at-a-time `setup-hive` Skill과
  `hive setup --scope user --answers ... --dry-run|--apply|--validate` 구현
- [ ] [UOS-005] Minimal bootstrap install, setup 완료 전 setup·doctor·update·recover 외
  Hive Skill activation 차단
- [ ] [UOS-006] 복수 selected host의 native plugin activation과
  `AIGENT-HIVE:USER` marker foreign-byte 보존
- [ ] [UOS-007] `~/.agents/directives`·`~/.agents/skills` provider-neutral projection,
  host별 selected Skill mirror와 ownership ledger
- [ ] [UOS-008] Recommended suite 또는 개별 Skill 선택, dependency closure preview,
  optional third-party capability consent와 deselection cleanup
- [ ] [UOS-009] Global Wiki default-on, `en|ko|both`, setup·agent intent 기반
  disable/enable, data preservation과 explicit delete 분리
- [ ] [UOS-010] Usage guard explicit opt-in, enabled 기본 threshold `20%`,
  native-first sensor와 CodexBar fallback-only consent 연결
- [ ] [UOS-011] `setup-harness`의 `expedited|custom` mode와 mode 무관 필수
  project kind 질문
- [ ] [UOS-012] Expedited global preference 상속과 custom language·Wiki·persona·Skill
  override, global disable 경계 검증
- [ ] [UOS-013] `~/.hive/config/projects.yml` registration과 user-root 단일 SQLite의
  user Wiki + enabled project Wiki 통합 rebuild
- [ ] [UOS-014] Project SQLite 생성 제거, source project·language·digest·visibility
  provenance와 project-private/confidential cross-project query 차단
- [ ] [UOS-015] `0.7.0 → 0.8.0` user/project migration: canonical Markdown·marker·Skill
  preference 보존, project SQLite derived cleanup과 setup review
- [ ] [UOS-016] Targeted Rust/Python contract, setup matrix, rebuild equivalence,
  Codex·Antigravity local install→global setup→project expedited/custom E2E

## 실행 순서

1. UOS-001–004: state·schema·catalog·CLI/Skill contract
2. UOS-005–010: user install·projection·Wiki·usage preference
3. UOS-011–014: project mode와 shared index
4. UOS-015: compatible migration
5. UOS-016: local qualification
6. Existing Claude·signing·publication external gate

## 검증 범위

- Work loop: changed crate + direct Python contract
- Pre-commit: affected crate + nearest setup/index/projection regression
- Pre-push: full Rust + full Python 1회
- Release: clean clone, 세 OS target, hostile/security, signing·provenance
- First public release 전 신규 hostile 범위: install·canonical data·credential·external
  path·rollback/recovery·changed regression 보호만 허용
