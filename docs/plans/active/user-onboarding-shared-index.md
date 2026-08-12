# Global onboarding과 shared index 계획

> Checklist owner: `UOS-*`
> Load condition: user setup, preference projection, project setup mode, shared knowledge index
> Decision: [`ADR-0012`](../../decisions/ADR-0012-global-onboarding-shared-index.md)
> Target: `0.8.0`

## 현재 구현 audit

| 요청 여정 | 현재 상태 | 판정 |
| --- | --- | --- |
| User plugin install | 세 host minimal bootstrap·setup-required gate·update | 충족 |
| 설치 후 mandatory global setup | Signed answer schema와 `setup-hive`·user-scope CLI | 충족 |
| Language·identity·persona·multi-host 질문 | Global catalog 기반 one-question sequence | 충족 |
| Recommended 또는 개별 Skill 선택 | Dependency closure preview와 selected projection | 충족 |
| Wiki default-on opt-out | 기본 활성화, setup·agent intent disable, Markdown 보존 | 충족 |
| Usage guard 권장·20% | 활성화 권장과 신속 기본 `20%`, fallback은 실제 native 실패 뒤 별도 consent | 보정 필요 |
| User guidance marker | `AIGENT-HIVE:USER:START|END` append·own-block replace | 충족 |
| User `.agents` projection | Provider-neutral directive·selected Skill과 host mirror | 충족 |
| User Wiki + SQLite | `~/.hive/knowledge` + disposable root SQLite 구현 | 충족 |
| Project expedited/custom | Global inherit 또는 bounded custom override | 충족 |
| Project kind 필수 질문 | Project identity·domain profile 선행 | 충족 |
| Project Wiki, project DB 없음 | Project Markdown Wiki + user-root 단일 SQLite | 충족 |
| Initial global expedited | English·English Wiki·strict·all built-ins 고정 기본값 | 충족 |
| Project auto onboarding | Global 상속·canonical evidence 추론·unresolved-only 질문 | 충족 |

## 구현

- [x] [UOS-001] User installation의 `bootstrap|setup-required|operational` state machine,
  operational route gate와 non-destructive reconfigure contract
- [x] [UOS-002] `user-setup` schema에 interface language, Wiki language, user profile,
  persona, selected hosts, selected Skills, Wiki enabled, usage guard enabled·threshold 추가
- [x] [UOS-003] Signed user profile·persona·recommended Skill suite catalog와
  unknown/custom value validation
- [x] [UOS-004] One-question-at-a-time `setup-hive` Skill과
  `hive setup --scope user --answers ... --dry-run|--apply|--validate` 구현
- [x] [UOS-005] Minimal bootstrap install, setup 완료 전 setup·doctor·update·recover 외
  Hive Skill activation 차단
- [x] [UOS-006] 복수 selected host의 native plugin activation과
  `AIGENT-HIVE:USER` marker foreign-byte 보존
- [x] [UOS-007] `~/.agents/directives`·`~/.agents/skills` provider-neutral projection,
  host별 selected Skill mirror와 ownership ledger
- [x] [UOS-008] Recommended suite 또는 개별 Skill 선택, dependency closure preview,
  optional third-party capability consent와 deselection cleanup
- [x] [UOS-009] Global Wiki default-on, `en|ko|both`, setup·agent intent 기반
  disable/enable, data preservation과 explicit delete 분리
- [x] [UOS-010] Usage guard 활성화 권장, 신속 기본 threshold `20%`,
  native-first sensor와 CodexBar fallback-only consent 연결
- [x] [UOS-011] `setup-harness`의 `expedited|custom` mode와 mode 무관 필수
  project kind 질문
- [x] [UOS-012] Expedited global preference 상속과 custom language·Wiki·persona·Skill
  override, global disable 경계 검증
- [x] [UOS-013] `~/.hive/config/projects.yml` registration과 user-root 단일 SQLite의
  user Wiki + enabled project Wiki 통합 rebuild
- [x] [UOS-014] Project SQLite 생성 제거, source project·language·digest·visibility
  provenance와 project-private/confidential cross-project query 차단
- [x] [UOS-015] `0.7.0 → 0.8.0` user/project migration: connected preference 보존,
  unconnected setup-required fail-closed, project SQLite derived cleanup과 setup review
- [x] [UOS-016] Targeted Rust/Python contract, setup matrix, rebuild equivalence,
  Codex·Antigravity local install→global setup→project expedited/custom E2E
- [x] [UOS-017] Initial `Expedited — set everything to default`: English interface,
  enabled English Wiki, general custom profile, strict persona, active host, all built-in Skills,
  usage guard enabled·stored threshold 20·CodexBar fallback disabled
- [x] [UOS-018] `auto-setup-harness`의 global preference 상속, canonical project
  evidence와 confidence record, unresolved-only one-question sequence, zero-question apply gate,
  promotion·third-party Skill·fallback hook의 추론 승인 금지
- [x] [UOS-019] Wiki enabled 상태의 agent-reviewed task-fact autocapture completion gate:
  결과·사용 도구 또는 project·작성 기준·원 요청 요약의 bounded 기록, user-root·project
  범위 분리, disable 시 capture 0건, raw transcript·hook·tool output·runtime ingestion 금지.
  `0.9.1` Windows fresh Codex session의 ordinary prompt 자동 기록·canonical Markdown·derived
  receipt와 다음 session 자동 회수로 확인; 보정 owner: `KAC-*`
- [x] [UOS-020] Current projection과 authenticated historical inventory·`retired-names.yml`을
  대조해 폐기·개명된 Hive-owned Skill file과 중첩 빈 directory를 setup·update 때 수렴 제거.
  Modified·foreign path: 보존과 conflict
- [x] [UOS-021] `hive uninstall`의 Hive-owned transient tree 정리를 leaf-to-root 반복 prune로
  보정. Empty install transaction·backup·marketplace·projection shell 제거, knowledge·saved
  preference·foreign byte 보존. `dev-install` rollback state: 별도 developer lifecycle 유지
- [x] [UOS-022] `0.9.0-test.19` Mac 증거 재현 fixture와 actual user-root acceptance. 현재 관찰값
  22개 active Skill·44개 active file·44개 retired empty leaf를 보정 전 fixture로 고정하고,
  setup·uninstall·reinstall 뒤 retired Skill·owned empty shell `0건`. Exact 44-leaf 회귀 fixture와
  `0.9.1` Windows user-root preserving reinstall에서 retired empty `agents/`·empty transaction `0건`

## 완료 evidence

- Minimal bootstrap 뒤 operational route 차단과 재실행 가능한 global setup
- 복수 host native qualification, selected Skill projection, foreign marker byte 보존
- Wiki disable 시 knowledge Skill 제거·shared operation 차단·Markdown 보존
- Usage guard 기본 활성화·신속 threshold `20%`, explicit disable 시 sensor·CodexBar 호출 0회
- 정상 native sensor에서 CodexBar 이름·질문·호출 `0회`
- Retired Skill·Hive-owned empty shell `0건`, knowledge·saved preference·foreign byte 보존
- Project activation과 user-root registry·index의 연결 실패 rollback
- 인증된 `0.7.0` 이하에만 허용되는 legacy project SQLite 호환 경로
- 동결 `0.7.0` unconnected install의 `0.8.0` 전환 거부와 전체 install tree 무변경
- Codex·Antigravity expedited/custom connected matrix 4/4
- Shared index 동일 입력 재실행 byte-exact no-op와 `changed_paths=[]`
- `auto-setup-harness` canonical·plugin·source·Codex·Claude projection parity
- Skill validator PASS, `hive-cli` 221/221, `hive-projection` 26/26,
  setup·projection·documentation Python 114개 PASS
- Full pre-push Rust workspace 477/477, Python conformance 576개 실행,
  575 PASS와 Windows `pwsh` 전용 1개 expected skip
- 독립 final blocker review의 severity 전체 finding 0건
- Product source version `0.7.0`; signed `0.8.0` release activation은 Phase 7 외부 gate

## 실행 순서

1. UOS-001–004: state·schema·catalog·CLI/Skill contract
2. UOS-005–010: user install·projection·Wiki·usage preference
3. UOS-011–014: project mode와 shared index
4. UOS-015: compatible migration
5. UOS-016: local qualification
6. UOS-017–019: initial defaults, automatic project inference와 task-fact autocapture
7. Existing Claude·signing·publication external gate

## 검증 범위

- Work loop: changed crate + direct Python contract
- Pre-commit: affected crate + nearest setup/index/projection regression
- Pre-push: full Rust + full Python 1회
- Release: clean clone, 세 OS target, hostile/security, signing·provenance
- First public release 전 신규 hostile 범위: install·canonical data·credential·external
  path·rollback/recovery·changed regression 보호만 허용
