# 0.16.0 재계획 — local 실패 뒤 개선판

- 기준일: 2026-11-21, 작성: local 세션. 진입 근거: [OV160-012 test.3 실패](OV160-012.md). 판: 3판(Fable 5.2 1·2차 검토 반영, [검토 기록](#7-fable-52-자문-검토))
- 유지관리자 요청(2026-11-21 원문 요약): 안전 장치가 지금은 과하게 제한적인지 저장소 규칙 밖에서도 검토, 위험 가능성이 있으면 항상 백업, Claude(사용한 양)·Codex(남은 양) 사용량 지표 방향 구분과 Antigravity 조사, 개선안과 실패 보고 기반의 새 0.16.0 계획, Fable 5.2 자문 검토
- 원문 인용: "The codebase is yours, so feel free to propose changes." / "I don't want to lose any files like ~/.hive/config/projects.yml ever again."
- 활성 버전: 0.16.0, 다음 번호 시험: 0.16.0-test.4. 부여된 권한: 구현·검증·번호 공개 시험·local 재확인 뒤 정식(자율 진행 기억 `overhaul-auto-release-approval`). 별도 승인 필요: 없음(정식 태그 조건은 local 재확인 통과)
- 제외: Gemini CLI 실제 수용(계정 지원 종료, 미검증 유지). 0.17.0 이동: [6절](#6-0170으로-옮길-것)
- 약어: `UI` = `crates/hive-cli/src/user_install.rs`. 계획 ID 대응: 014a = OV160-014, 014b = OV160-021

## 1. 안전 장치 검토 — 지킬 것과 풀 것

판단 기준: **사용자 자료를 지키는 장치는 강화, 작업 흐름만 막고 자료 보호 효과가 없는 장치는 완화.** 이번 실패의 공통점: 엄격한 검사가 실패를 안전하게 처리하지 못하고 더 위험한 우회 경로로 밀어냄(인증 실패 → 보존 재설치 → `projects.yml` 삭제).

| ID | 현재 장치 | 이번 문제 | 판정 | 개선 |
| --- | --- | --- | --- | --- |
| S-A | 이전 출시 설치 인증의 출시별 내장 지문 비교 | 고정 상수 1개(UI:4683)와 고정 범위 1개(UI:4644)로 0.15.2·0.15.3 사용자 전체 갱신 실패 | **수정 후 조건부 완화** | 먼저 상수·범위 수정(014a). 이후 "Hive 전용 위치 안 파일은 백업 뒤 교체 가능" 규칙(014b) |
| S-B | 보존 재설치 = `execute_uninstall` 뒤 설치 | 제거 단계의 `projects.yml`·`.hive/runtime`·`.hive/backups` 삭제, 백업 없음 | **강화** | 기존 설치 거래(백업·작업 기록·복구)로 통합, 삭제는 허용 목록 방식 |
| S-C | 완료 항목의 의존 대상 완료 필수 | 실패한 003·004·007 되돌리기 불가 | **완화 + 출시 차단 유지** | `reopened` 상태, 정식 단계에서는 `reopened` 존재 시 실패 |
| S-D | 계획 조각 8KiB·지침 읽기 예산 여유 0 | 한 줄 추가마다 다른 문장 삭제 | **완화** | 조각 32KiB(루트 파일은 8KiB 유지), 예산 10% 여유 |
| S-E | 정식 조건: 사람이 하는 local 확인 전부 통과 | 호스트 소유 제한까지 출시 차단 | **조정** | 사전 기록된 호스트 한계만 비차단, 이전 정식에서 통과한 확인의 실패는 항상 제품 결함 |
| S-F | 전 호스트 공통 "남은 %" 기준 | Claude 화면은 사용한 %, Antigravity 센서 1.1.18 고정 | **개선** | 호스트별 화면 방향으로 입력·표시, 저장은 남은 % 유지 |
| S-G | 세션 보호 끄기의 명시 의사 요구 | 없음 | **유지** | 사용량 소진 방지 핵심 |
| S-H | 전역 호스트 설정 직접 수정 금지 | Codex 훅 신뢰 대행 불가 | **유지** | 안내 강화 |
| S-I | 한국어 서술형 어미 검사 | 수정 시간 소요 | **유지** | 문서 일관성 효과 |
| S-J | 공개 갱신 CI 단계 | `if: inputs.product_version == '0.14.0'` 고정(`.github/workflows/public-test-acceptance.yml:94`, 비슷한 고정 122·146·166·174행)으로 0.14.0 뒤 계속 건너뜀 | **강화** | 매 시험 실행, 버전 고정 조건 금지 정적 시험 |

## 2. 호스트별 사용량 지표

| 호스트 | 앱 화면 방향 | Hive가 받는 값 | 근거 |
| --- | --- | --- | --- |
| Claude | 사용한 %(예: 주간 86% 사용) | 상태 표시줄 `rate_limits.five_hour/seven_day.used_percentage`, `resets_at` | [공식 상태 표시줄 문서](https://code.claude.com/docs/en/statusline), `usage_control.rs:1128–1141`의 `100 - used` 변환 |
| Codex | 남은 %(예: 남은 양 94%) | 내부 API `usedPercent`(`usage/codex_native.rs`), 남은 %로 변환 | 이 PC `hive usage show --host codex` |
| Antigravity | 남은 비율 막대(5시간·주간, 모델 묶음별) | 콜백 `quota.<pool>.remaining_fraction`(`usage_control/antigravity.rs`, 묶음 `gemini-weekly`·`3p-weekly`) | 센서 코드, Google 개발자 포럼 사용자 보고(공식 명세 없음, 주간 표시 누락·불일치 보고 다수) |

- 결정: 저장은 `*_stop_remaining_percent` 유지(설정 변환 불필요). 입력·표시만 호스트 방향
- 예: Claude "96% 사용에서 멈춤" → 저장 `weekly_stop_remaining_percent = 4`, 표시 "멈춤 기준: 96% 사용(남은 4%)"
- Antigravity: 이 PC 1.2.17, 센서는 `VERSION = "1.1.18"` 고정(antigravity.rs:59 콜백, :233 저장 기록 두 곳) → 현재 기록 거부 추정. 실제 콜백 확인 전 범위 확정 금지(OV160-017 조사 단계)

## 3. 순서

014a → 016 → 014b → 018 → 020. 015·017·019는 병행(017은 주로 `usage_control.rs`·`preferences.rs`, 015·019는 소스 전용). 같은 파일 작업 직렬: 014a·016·014b는 `UI`.

## 4. 단계 기록

### OV160-014a 갱신 인증 수정과 공개 갱신 CI 재가동(S-A 수정, S-J)

- 의존: 없음. 진입: develop 최신. 소유: cloud 작업자
- 변경 `UI`
  - `parse_three_part_version`(UI:6469)을 `pub(crate)`로 공개(017 재사용)
  - 새 `fn render_claude_marketplace_for(version: &str) -> Vec<u8>`(UI:3881 본문 이동, `version` 인자), 기존 `render_claude_marketplace()`는 `render_claude_marketplace_for(env!("CARGO_PKG_VERSION"))`
  - `historical_09x_user_inventory`(UI:4633)의 Claude `host-adapter-metadata` 분기(UI:4681–4688): 기대 지문 = `sha256_digest(&render_claude_marketplace_for(request.product_version))`, 단 이 계산은 **검증된 버전 목록** `VERIFIED_CLAUDE_MARKETPLACE_TEMPLATE_VERSIONS`(처음은 `["0.15.0","0.15.1","0.15.2","0.15.3"]`, 네 태그의 `render_claude_marketplace` 본문 동일 확인 완료) 안에서만, 그 밖 버전은 기존 상수 비교 유지. 목록 확장 절차: `git show v<버전>:crates/hive-cli/src/user_install.rs`에서 함수 본문 비교 후 동일하면 추가
  - Antigravity 범위(UI:4644–4646): `request.installed_host_version_range`가 `">=1.1.7 <1.2.0"` 또는 `UserHost::Antigravity.version_range()`이면 허용
  - UI:5003의 다른 목록은 변경 없음
- 조사(제한): 0.15.2에서 0.15.3 갱신이 이 PC에서 통과한 이유. 허용 행동: `docs/research/overhaul/OV153-010.md`·`OV153-016-request.md` 읽기, `git show v0.15.3:crates/hive-cli/src/user_install.rs`. 결정 규칙: 보존 재설치로 통과했다면 그때도 `projects.yml` 삭제 가능성 기록(016 등록 목록 재구성 비중 근거), 다른 경로면 그 경로를 014a 시험에 추가
- 시험 고침: `every_09_stable_user_inventory_authenticates_for_every_host`(UI 약 10466–10560)·`seed_historical_09x_user_install`(약 10835–10920)이 0.15.3에 실제 마켓플레이스 바이트와 실제 범위(0.15.2 이상 `<2.0.0`) 사용
- 새 시험(시험 안에서 고정물 생성, 실제 PC 파일 미사용): `harness/user-bases/0.15.3` + `render_claude_marketplace_for("0.15.3")`로 0.15.3 사용자 설치 재구성 → `authenticated_saved_projection_hosts`가 codex·claude·antigravity 모두 인증 → `hive update` 경로 성공. 이름 `prior_0_15_3_user_install_updates_without_reinstall`
- CI: `public-test-acceptance.yml`의 공개 갱신 단계(`tests/fixtures/interactive-cli/public-update-ci.py`)에서 `if: inputs.product_version == '0.14.0'` 제거, 이전 정식 출처는 `docs/public-stable-release.json`의 `stable_version`. 스크립트 수정 필수: 57행 `--previous-version`의 `choices` 제거, 63행 패키지 정규식을 `<제품버전>-test.N` 일반형으로, 44–50행 신원 출처 `harness/setup/legacy-update-images.json`에 0.15.x 항목 추가. 180행은 `antigravity`+`agy-stub.py`만 선택하므로 CI가 잡는 것은 Antigravity 범위 결함까지, Claude·Codex 갱신 인증은 단위 시험(`prior_0_15_3_user_install_updates_without_reinstall`)이 담당. 122·146·166·174행 고정 조건도 같은 방식 검토, 의미상 특정 버전 전용이면 허용 표시 주석 `# version-pin-allowed: <이유>`. 정적 시험 `tests/conformance/contracts/test_acceptance_workflow_has_no_version_pins.py`: 수용 워크플로의 `inputs.product_version ==`와 `startsWith(inputs.product_version` 금지, 허용 표시 주석이 바로 위 줄에 있으면 예외
- 확인: `cargo test -p hive-cli prior_0_15_3` 통과, 수정 전 코드에서 같은 시험 실패(음성 확인), 정적 시험 통과
- 증명 못 하는 것: 0.15.2 이하 실제 공개 바이트(014b와 CI 행렬이 대응)

### OV160-016 설치 거래 통합과 삭제 허용 목록(S-B)

- 의존: 014a(같은 파일). 소유: cloud 작업자
- 결정: 새 백업 체계를 만들지 않고 기존 설치 거래 확장. 기존 요소: `apply_plan`(UI:5281)의 `.hive/backups/user-install/<host>/<unix>-<pid>-<nonce>/files/…`, `UserBackupManifest`(UI:451), 작업 기록 `.hive/install-transactions/<host>.json`(UI:5766), `recover`(UI:5619), `rollback_after_failure`(UI:6336), `execute_apply`(UI:2741)의 호스트 활성화 이전 상태 포착·되돌리기
- 변경 0 판정 함수(016 소유, 014b 재사용): `fn hive_exclusive_entry(host: UserHost, entry: &UserOwnershipEntry, current_skills: &BTreeSet<String>) -> bool`. 참 조건 전부:
  - 경로가 `.hive/marketplaces/<host>/`·`.hive/plugins/`·`ANTIGRAVITY_SOURCE_RELATIVE` 아래, 또는 `.gemini/config/skills/<name>/` 아래이면서 `<name>`이 현재 Skill 목록 또는 `harness/skills/retired-names.yml` 이름
  - 경로에 `..` 없음, 절대 경로 아님
  - 소유 종류가 실제 이름 집합 안(`immutable-plugin-package`·`host-skill-projection`·`host-adapter-metadata`·`shared-migration-state`·`user-install-manifest`, UI:3582 참조)
  - 그 밖 종류 경계: `shared-marker`는 경로가 호스트 안내 파일(`historical_guidance_path`)과 같을 때만, `canonical-data-protected`는 `historical_canonical_user_path` 통과 때만(UI:4692–4704와 동일), 둘 다 교체·퇴역 대상 아님
  - 기록 전체 조건(함수 밖 호출자 검사): 항목 정렬·중복 없음, 기록 버전 세 자리·0.9.0 이상·실행 버전 이하(`parse_three_part_version`)
- 변경 1 보존 재설치 교체: `execute_preserving_reinstall`(UI:2714)에서 `execute_uninstall` 호출 삭제. 방식: `build_plan`→`validate_prior_ownership`(UI:3369–3380)에 인자 `adopt_unauthenticated_prior: bool` 추가. 참이고 `authenticated_user_inventory`가 `None`이면, 설치 기록 항목 중 `hive_exclusive_entry` 참인 것만으로 `AuthenticatedUserInventory` 생성(버전·범위·안내 경로는 기록 값), 이어서 기존 `validate_authenticated_installed_bytes`로 실제 바이트 확인(변조 항목은 채택 제외가 아니라 전체 실패). 판정 거짓 항목은 교체·퇴역 대상에서 제외하고 그대로 둠. 이후 일반 `execute_apply`가 파일·활성화를 한 거래로 처리
- 종료 조건(파괴적 대안 없음): 새 계획의 관리 경로가 점유돼 있고 채택 불가(UI:3409–3413 "occupied … lacks exact prior Hive ownership")이면 삭제 없이 실패, 코드 `hive.user-install-prior-not-adoptable`, 결과에 점유 경로와 백업 위치, 복구 안내 `hive backup restore`
- 변경 2 `execute_uninstall`(UI:734): 유일한 비시험 호출자가 보존 재설치(UI:2720)이므로 삭제, 시험(UI:8576·8899·9258)은 새 흐름으로 이전
- 변경 3 사용자 제거 명령(`user_install/uninstall.rs` `run` → `uninstall/removal.rs`): 시작 전 같은 백업 형식으로 사본 작성
- 변경 4 삭제 허용 목록: Hive가 지울 수 있는 것은 (a) 설치 기록 항목 중 실제 지문 일치 파일 (b) 재생성 가능 트리 `.hive/install`·`.hive/install-transactions`·`.hive/marketplaces`·`.hive/plugins`·`.hive/index`·`.hive/guides`만. `UNINSTALL_TREES`(UI:715–725)에서 `.hive/runtime`·`.hive/backups`·`.hive/claims` 제외. 보존 재설치와 갱신은 `.hive/runtime`·`.hive/claims` 아래 변경 금지. 사용자 제거 명령은 `uninstall/removal.rs`의 현재 `.hive/runtime` 처리(사용량 감시 중단 기록 보존) 유지, `.hive/claims`는 사용자 제거에서만 기존대로 삭제. `UNINSTALL_FILES`에서 `.hive/config/projects.yml` 제거, 이에 따라 `uninstall.rs:309`·`removal.rs:75`의 예외 필터 삭제. `storage.rs:19`의 `backups` 정리 분류는 사용자 명시 정리 명령에서만
- 변경 5 백업 대상 명시(거래 사본에 포함, 트리 전체 복사 금지): 작업이 쓰거나 지울 경로 + 고정 설정 묶음 `projects.yml`·`collections.yml`·`user-setup.yml`·`user-feature-answers.yml`·`user-preferences.json`·`rag-trust.json`·`.hive/roster.toml`·`.hive/runtime/usage-monitor/{config.json,disable.json}` + Hive가 고치는 호스트 파일 `~/.claude/settings.json`(statusLine 변경 시)·`.claude/CLAUDE.md`·`.codex/AGENTS.md`·`.gemini/GEMINI.md`. `.hive/mediation/keys`는 백업 제외(비밀 성격). 백업 폴더 권한은 소유자 전용
- 변경 6 보관: 호스트별 최근 10개 + 30일 이내 유지, 정리는 다음 거래 성공 뒤에만. 백업 크기 상한 512MiB 초과 예상 시 변경 전 실패(`hive.backup-too-large`)
- 변경 7 Windows 잠금: 이름 바꾸기 실패(예: 실행 중 `bin/hive-claude-usage-capture`) 시 작업 기록으로 되돌리기, 시험으로 실패 주입
- 변경 8 복원 명령 `hive backup list [--host <h>]|restore --host <h> --transaction <id> [--confirm <digest>] --user-root <home>`: 기존 `UserBackupManifest` 형식 읽기, 미리보기 → 지문 확인 뒤 복원. 새 파일 `crates/hive-cli/src/user_install/backup.rs`, `cli_help.rs`·`human_output/codes.rs` 등록
- 변경 9 등록 목록 재구성 `hive project registry rebuild --scan <dir>... [--confirm <digest>]`: 병합만(기존 항목 삭제 없음), 실행 전 백업, 루트는 `hive_wiki::shared::canonical_root`, ID·`enabled`·`visibility`는 `main.rs:925–961` `reconcile_project_registry`와 같은 규칙(`ProjectPrivate`, `harness.toml`의 `wiki_enabled`·`wiki_language`)
- 변경 10 갱신 경로: `hive update`는 핵심 단계 전 `update_activation.rs:188` → `authenticated_saved_projection_hosts`(UI:1792)에서 중단(`steps.core=not-started`). 결정: 이 인증 단계에서 인증 실패 호스트를 오류 대신 "채택 필요" 목록으로 반환, 모든 변경 전에 변경 1의 채택 판정을 미리 실행해 불가 호스트가 하나라도 있으면 아무것도 바꾸지 않고 `hive.user-install-prior-not-adoptable`로 실패. 전부 채택 가능하면 핵심 단계 뒤 투영 갱신에서 `adopt_unauthenticated_prior = true`로 적용, 결과에 백업 위치 표시. 설치 경로의 `requires_preserving_reinstall`(UI:2596)도 같은 흐름 사용
- 시험: `preserving_reinstall_keeps_projects_registry_and_backs_up`, `failed_reinstall_restores_files_and_activation`(중간 실패 주입), `locked_file_rename_rolls_back`, `backup_restore_requires_matching_digest`, `registry_rebuild_merges_and_matches_setup_ids`, **보초 파일 시험** `unknown_file_under_every_hive_directory_survives`(모든 `.hive/*` 하위에 모르는 파일 하나씩 → 설치·갱신·재설치·제거 뒤 바이트 동일)
- 증명 못 하는 것: 실제 Windows 호스트 활성화 되돌리기(020 local)

### OV160-014b Hive 전용 위치 규칙(S-A 완화)

- 의존: 016. 0.16.0 포함 조건: 아래 경계 전부 구현과 시험 통과, 미충족이면 0.17.0
- 범위: 016의 `hive_exclusive_entry`를 `authenticated_user_inventory`의 마지막 분기로 추가(재설치·채택 없이 갱신 인증 통과). 판정 규칙 자체는 016 소유
- 결정: 출시별 지문 표 대신 **위치 기반 소유**. `.hive/marketplaces/<host>/**`·`.hive/plugins/**`·`ANTIGRAVITY_SOURCE_RELATIVE` 아래는 Hive 전용, 백업 뒤 교체 가능. 그 밖 경로는 정확한 지문 일치 없이 변경 금지
- 경계: 016 변경 0의 판정 함수 그대로. 추가로 퇴역 판정은 `retirable_hive_tree_entry`(UI:4110) 재사용, 퇴역 파일 전부 백업
- 위협 판단: 실질 위험은 공격자보다 이전 빌드의 잘못된 설치 기록, 대응은 위치 경계 + 백업. UI:4788–4789 주석 갱신: 자체 일치 기록의 허가 범위를 위치 경계 안 바이트로 한정
- 시험: `self_consistent_manifest_naming_user_file_refused`, `self_consistent_manifest_naming_foreign_gemini_skill_refused`, `future_version_manifest_refused`, 기존 `authenticated_saved_projection_scope_rejects_forged_prior_release_manifest`(UI:9367, 0.5.9)는 0.9.0 미만 거부로 기대 유지 확인
- 증명 못 하는 것: 위치 안 사용자 직접 수정 파일(백업으로 복원 가능)

### OV160-017 호스트별 사용량 방향과 Antigravity 센서(S-F)

- 의존: 없음(016과 다른 파일, 병행 가능). 소유: cloud 작업자 + local 조사 1건
- 변경
  - `crates/hive-core/src/preferences.rs`: 호스트 속성 표 `HOST_USAGE_SENSORS`(호스트·`direction: Used|Remaining`·창 목록·지원 버전 범위). 표시·입력이 같은 표 사용. 저장 구조 변경 없음
  - `hive usage threshold --host <h> --window five-hour|weekly|all (--stop-used-percent <1..100> | --stop-remaining-percent <0..99>)`: `remaining = 100 - used`, 해당 호스트 `host_windows`의 창별 값 갱신. 호스트 `host_windows` 항목이 없으면(전역 기준만, `usage_control.rs:3311–3317`) 거부 + 설정 명령 안내. `--stop-used-percent` 50 미만은 `--confirm` 필요(방향 착오 방지). 기존 `--remaining-percent`(전역)는 유지
  - 표시 갱신 위치: enforce·status 메시지, `human_output/families.rs:124`, `human_output.rs`, Discord 알림, `cli_help.rs:1133`, 설정 질문, `usage-guard`·`aither` Skill 문구. Claude는 "사용한 양 X%·멈춤 Y% 사용", Codex·Antigravity는 "남은 양 X%·멈춤 Y% 남음". JSON에 `native_direction`·`native_value` 추가(기존 키 유지)
  - Antigravity: 조사 먼저(local, 제한: 이 PC Antigravity 1.2.17에서 Hive 콜백 입력 1개를 개인 정보 제거 후 `tests/fixtures/` 고정물로 저장). 결정 규칙: 키 구조가 1.1.18과 같으면 `VERSION` 두 곳(antigravity.rs:59·:233)을 범위 `>=1.1.18 <2.0.0` 검사로 교체(014a에서 `pub(crate)`로 공개한 `parse_three_part_version` 재사용), 다르면 새 구조 해석 추가 후 같은 범위 검사
- 시험: `claude_threshold_used_input_stores_remaining`, `threshold_without_host_windows_refused`, `low_used_threshold_requires_confirm`, `display_direction_per_host`, `antigravity_accepts_real_1_2_17_callback`(실제 고정물)
- 증명 못 하는 것: Antigravity 주간 묶음 표시 정확성(공식 명세 없음)

### OV160-018 사용 편의와 중재 기본값(R3·R4·R5·R6)

- 의존: 없음. 소유: cloud 작업자
- R3 Windows 경로: 대상 해석 위치 `crates/hive-cli/src/project_upgrade.rs:2567`, `--user-root` 해석 UI:1984·`update_discovery.rs:131`·`update_activation.rs:643`. `.`·상대 경로를 `current_dir()` 결합 후 정규화. 같은 기회에 사용량 대상 지문의 글자 크기 문제: 지문 계산 전 디스크 실제 글자 크기로 해석, 단 기존 기록이 입력 그대로의 지문 위치에 있으면 그 위치 사용(c32fe646 보존). 시험: `.` 입력, 글자 크기만 다른 대상의 같은 기록 조회, 기존 위치 기록 유지
- R5 교차 검토: 새 선택지 `--reviewer-provider <codex|claude|gemini>`(이름 문법 변경 없음, `checked_id` 그대로). 선택 호스트 → 명부 공급자 대응: codex→`codex`, claude→`claude`, antigravity→`gemini`. `Roster::resolve_named_for(name, provider)` 추가(`crates/hive-core/src/roster.rs`). `StepProfile.provider`는 이미 기록됨(flow.rs:227), 실제 수정은 `profile_current`(flow.rs:234–243, :312 사용)와 이후 소비·작업자 해석이 `resolve_named_for(role, profile.provider)` 호출. 기본값: 작성자와 다른 선택 호스트 공급자가 정확히 하나일 때만 자동, 그 밖은 기존 오류 + `next_action`에 정확한 명령
- R4 옛 Skill 이름: Claude Code의 `/plugin:skill` 호출은 호스트 처리라 Hive 개입 불가(이번 확인). 0.16.0은 출시 노트 COMPAT·README 안내만. 새 Skill 설명의 "옛 이름 …" 문구 유지
- R6 안내: `hive msg hooks apply` 결과 `next_action`에 Codex 훅 신뢰 승인 방법, Claude 자동 모드 허용 규칙 예시(프로젝트 `.claude/settings.json`, 사용자 선택). `harness/plugins/aigent-hive/README.md`·`docs/releases/0.16.0.md` COMPAT 갱신
- 시험: 각 항목 시험 + 출시 노트 검사 스크립트

### OV160-015 계획 검사 완화(S-C·S-D, 소스 전용)

- 의존: 없음. 소유: local 또는 cloud
- 변경 `scripts/plan_state.py`
  - `STATES` 끝에 `"reopened"` 추가. 위치 의존 교체: `state in STATES[2:]` → `state in WAITING`(`{"awaiting-user-authority","awaiting-external-evidence","blocked"}`), `summaries()`의 `STATES[1:]` → 완료 제외 목록
  - `reopened`: 체크 `[ ]`, 이전 `evidence` 유지 허용, `reason` 필수이고 저장소 근거 파일 링크 포함
  - `visit()`: 의존 대상이 `reopened`면 오류 대신 `warnings`(JSON 새 필드)
  - 새 허용 메타 키 `reopened-evidence`(`metadata()` 허용 목록에 추가): 다시 열 때 기존 `evidence` 값을 이 키로 옮김. `complete` 전환 시 `evidence != reopened-evidence` 필수
  - `MAX_FRAGMENT_BYTES = 32768`(조각), 루트 `PLAN.md`·`CURRENT.md`는 `MAX_PLAN_BYTES = 8192` 유지(`read`·`replace_section`, plan_state.py:76·246)
  - 새 선택지 `--release`: `reopened` 존재 또는 완료 항목이 `reopened`에 의존하면 실패. 연결: `.github/workflows/release.yml`의 정식 경로(`channel=stable`)에서 `check-public-stable-docs.py` 단계(52행 부근) 옆에 `python3 scripts/check-plan-state.py --release` 단계 추가, 인계 문서 5단계에도 명시
- `.agents/directives/04-documentation-state.md:5` 문구("Each stays below 8KiB")를 조각 32KiB·루트 8KiB로 갱신
- `.agents/directives/read-budgets.json`: 각 `max_bytes` = 현재 실제값 × 1.10, `baseline_revision` 갱신
- 계획 적용: OV160-003·004·007을 `reopened`(이유: OV160-012 링크)
- 시험: `tests/conformance/documentation/test_plan_state.py` 확장 — reopened 의존 경고, reason 없는 reopened 거부, `--release` 실패, 같은 근거로 complete 전환 거부, complete가 agent-owned에 의존하면 계속 오류
- 증명 못 하는 것: 제품 동작 무관

### OV160-019 local 확인 자동화와 판정 규칙(S-E, 소스 전용)

- 의존: 없음
- 새 `scripts/qualify-local-hosts.py`(읽기 전용 + `tests/work/` 일회용 폴더만): 설치 버전, 사용자 설치 검증, 사용량 enforce, Skill 투영 이름, 훅 미리보기, 흐름 시작 미리보기, 등록 목록 대 디스크 프로젝트 비교를 JSON 보고. 판정은 하지 않고 판정 입력만 출력
- 판정 규칙(인계 문서 머리):
  - 실패는 기본 `product-defect`
  - `host-limit`은 (a) 실행 전에 인계 문서에 그 한계가 적혀 있었거나 (b) Hive 없이도 재현되는 근거(호스트 문서·Hive 없는 재현)가 있을 때만
  - 이전 정식에서 통과한 확인의 실패는 항상 `product-defect`(퇴행은 호스트 한계가 아님)
  - `host-limit`마다 COMPAT 항목 필수, 이와 모순되는 출시 노트 주장 삭제
  - 새 종류의 host-limit은 `docs/decisions/`에 기록, 출시 안내에 명시
- 시험: `tests/conformance/contracts/test_qualify_local_hosts.py`(가짜 `hive` 실행 파일)

### OV160-020 0.16.0-test.4와 local 재확인

- 의존: 014a·014b·015·016·017·018·019. 014b를 0.17.0으로 옮기면 계획 조각의 OV160-020 의존에서 OV160-021 삭제
- 절차: 기존 번호 시험 절차(OV160-013-test2와 동일) → 세 OS 수용 + 재가동한 공개 갱신 단계 → 이 PC: **백업 먼저**(`~/.hive/config`·`~/.hive/install` 사본) → `npm install -g aigent-hive@0.16.0-test.4` → `hive update`(재설치 없이 성공 기대) → 019 스크립트 → 사람 확인(터미널 사용량, **연결 뒤 새로 연 Code 탭 세션** 사용량, 훅 왕복, 교차 검토 `--reviewer athena --reviewer-provider claude`) → `hive project registry rebuild` 미리보기로 이 PC 등록 목록(2026-11-21 수동 재구성 16개)과 비교
- Codex 확인 비차단(유지관리자 2026-11-21): Codex 사용 한도 소진 동안 Codex 호환성 확인(Codex 훅 실행, `codex exec` 작업자, Codex 작성자 교차 검토)은 정식 차단 아님. 결과는 "미검증(Codex 한도)"으로 기록하고 한도 회복 뒤 확인. 원문 "Being unable to run codex must not be a blocker."
- 통과 뒤: OV160-003·004·007 `complete`(새 근거), OV160-012 `complete`, `check-plan-state.py --release` 통과, OV160-011 정식
- 실패 시: 해당 항목 `reopened`, 근거 기록, 다음 번호 시험

### OV160-022 위임 실행 중 사용량 급감 보호

- 발견(2026-11-21 local): Codex 작업자 4개 병렬 실행 약 40분 만에 Codex 30일 한도 남은 양 94% → 0%(재설정 2026-12-15). Hive 읽기는 정확(`hive usage show --host codex`: 실행 전 94%, 소진 뒤 0%, 같은 창 `window_minutes` 43200). 원인: 07 사전 확인이 작업 시작 때 1회뿐이고, 조율 세션도 위임 작업 중 Codex 사용량 재확인 없음. 저장된 중간 보호(`usage_guard.midway`)는 기본 비활성
- 조사(제한): `crates/hive-core/src/preferences.rs`의 `midway` 설정과 `usage_control/monitor.rs` 동작 확인, 위임 감시 도구(`codex-run.py`)가 중간 재확인을 부를 수 있는지 판단. 결정 규칙: 기존 중간 보호로 위임 작업 중단이 가능하면 위임 안내·감시 도구에서 켜는 방식, 불가하면 `hive usage enforce`를 일정 간격 재호출하는 감시 단계 추가
- 변경 후보: 병렬 위임 작업 수만큼 기준을 나눈 작업별 예산, 남은 양 급감(예: 10분에 20%p 이상) 시 새 위임 중지 안내
- 확인: 중간 재확인 시험, 이 사건 기록을 근거 문서에 보존
- 0.16.0 차단 아님(정식 조건 OV160-020과 무관)

## 5. 최종 완료 기준

- 0.15.3 사용자 설치에서 `hive update`가 재설치 없이 성공(세 OS CI + 이 PC)
- 설치·갱신·재설치·제거가 허용 목록 밖 경로를 지우지 않음(보초 파일 시험), 변경 전 거래 백업, `hive backup restore`로 복원
- Claude 기준을 사용한 % 방향으로 입력·표시, Antigravity 1.2.17 실제 콜백 수용
- 교차 검토가 `--reviewer-provider` 지정 또는 단일 대안 공급자 자동 선택으로 시작, 흐름 끝까지 공급자 유지
- `reopened` 표현과 정식 단계 차단

## 6. 0.17.0으로 옮길 것

- 설치·갱신·제거·statusLine 외 모든 설정 쓰기 명령의 거래 백업
- `hive skills resolve <old-name>`
- 014b 경계 미충족 시 014b 전체
- 제안(Fable): 사용자 루트 삭제 단일 통로 `guarded_remove(path, proof)` + 직접 `remove_file`·`remove_dir_all` 금지 정적 검사
- 제안(Fable): 이전 정식·그 이전 정식 → 후보 갱신 행렬을 상시 CI 관문으로, 보초 파일 바이트 비교 포함
- 제안(Fable): `hive doctor --user-root` 읽기 전용 진단(호스트별 설치 기록 인증, 등록 목록 대 디스크, 마지막 백업, 사용량 센서 상태, 각 해결 명령)

## 7. Fable 5.2 자문 검토

- 1차(2026-11-21, 일반 하위 에이전트 Fable 5.2, 읽기 전용): **REQUEST CHANGES**. 차단 7건·권고 14건·사소 3건
- 반영 요약
  - 016: 새 백업 체계 대신 기존 거래(`apply_plan`·작업 기록·`recover`) 확장, 지식 트리 복사 금지·보관 규칙·크기 상한, `execute_uninstall`의 실제 호출자 정정(사용자 제거 명령은 `uninstall/removal.rs`), 비활성화 후 설치의 복구 불가 구간 제거, 삭제 허용 목록 방식과 보초 파일 시험, 호스트 파일 백업 명시, 등록 목록 재구성은 병합만
  - 014: 014a(상수·범위 수정, 공개 갱신 CI 고정 조건 제거)와 014b(위치 경계 포함 완화)로 분리, 실제 PC 고정물 삭제, 0.15.3만 검증된 마켓플레이스 틀 적용, 0.15.2→0.15.3 통과 경위 조사, `hive update`의 재설치 흐름 연결
  - 017: 창 선택·전역 기준만 있는 경우·50 미만 확인·표시 위치 목록, Antigravity 실제 콜백 조사 선행과 두 검사 위치
  - 018: 검토자 공급자 명시·기록(`profile_current` 실패 방지), R3 위치 명시와 글자 크기 지문 처리, R4는 문서만
  - 015·019: `--release` 차단, 루트 8KiB 유지, 시험 파일 경로 정정, 판정 규칙 강화(사전 기록·Hive 없는 재현·퇴행은 결함)
  - 순서: 014a 먼저, 015·019 병행
- 2차: [아래](#2차)

### 2차

- 2026-11-21, 같은 검토자: **APPROVE WITH FIXES**. 차단 3건(016·014b 순환 의존, 채택 목록 전달 방식, CI 스크립트 재가동 범위)·권고 6건·사소 4건
- 반영: 판정 함수 `hive_exclusive_entry`를 016 소유로 이동, `adopt_unauthenticated_prior` 인자와 채택 불가 종료 조건, `public-update-ci.py` 수정 목록과 Claude·Codex는 단위 시험 담당 명시, `startsWith` 금지와 허용 표시 주석, `hive update` 중단 위치(UI:1792·`update_activation.rs:188`)와 변경 전 일괄 판정, `.hive/runtime`·`.hive/claims` 처리, 검증 버전 0.15.0–0.15.3, `reopened-evidence` 키와 `release.yml` 연결, `--reviewer-provider`와 antigravity→gemini, 비관리 종류 경계, 사소 4건
- 검토자 미확인: 정식 경로 소유 파일(이 세션에서 `release.yml` 확인), `.hive/plugins/**` 내용, v0.15.0 이전 마켓플레이스 틀

## 8. 구현 근거

- [OV160-014](OV160-014.md)·[OV160-015](OV160-015.md)·[OV160-017](OV160-017.md)·[OV160-018](OV160-018.md)·[OV160-019](OV160-019.md)
- 이전 단계 근거: [OV160-010 develop 병합](OV160-010.md)
