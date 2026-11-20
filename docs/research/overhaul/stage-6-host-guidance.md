# 단계 6 검토 S10 — Poseidon 중재 사용 안내와 참조 처리기 적합성 시험

- 기준일: 2026-11-20. 실행 장소: cloud 세션, Linux 컨테이너
- 기준 커밋: `81b7e0ee`(OV160-005)의 격리 작업 폴더
- 대상 지적: 단계 6 검토 S10/P3(호스트 사용 안내, 참조 처리기 시험), O1(Discord 알림 한국어 전용)
- 실제 호스트·작업자·제공자 호출 0회. `claude -p`·인증 파일·사용자 전역 `~/.claude`·`~/.codex`·`~/.gemini` 접근 없음
- 동시 작업: 메시지 규약 수정(`ack`·`cancel-ack`의 `--claim` 필수화, 빈 Stop의 `systemMessage`, `--now` 시험 전용, Gemini 훅 적용 거부, `.hive/mediation/` Git 제외). 이 작업은 해당 파일 미변경, 안내 문구만 새 동작 기준

## 범위

- 대화형 호스트(Claude)와 작업자가 `hive msg`·`hive stop`을 쓰는 절차를 Poseidon 소유 Skill에 추가
- 호스트 소유 처리기의 왕복(소비권 → 정책 확인 → 작업자 실행 → 확인증)을 가짜 작업자로 시험
- O1은 범위 밖: 알림 문구 파일 `decision.rs` 미변경, 아래 한계에 기록

## 변경

### 안내 위치

- 신 입구 `poseidon`은 생성 파일. 생성 입력 `harness/gods.toml`의 Poseidon `steps`에 중재 방식 안내 한 줄 추가, `generate-gods.py`로 재생성
- 세부 절차는 Poseidon 소유 `team-execution`의 새 참조 `references/mediation.md`. `SKILL.md`에는 짧은 "Mediation Mode" 절과 연결만 추가
- 설명(`description`)·한국어 표시 설명 변경 없음. 호출 조건 그대로

### 참조 내용

| 절 | 내용 |
| --- | --- |
| 역할 분담 | 발신자·대화형 호스트·작업자·사람·Hive의 할 일과 금지 |
| 메시지 수명 | `send` → `inbox` → `claim`(`claim_token` 보관) → 정책 확인 → 호스트의 작업자 실행 → `ack --claim <token>` → `status` |
| 작업자 정책 | `.hive/mediation/policies/<메시지 ID>.json`의 `sandbox`·`network: none`·`git_push: deny`·`writable_roots`·`git_env`·`hive_launches_worker: false`, 실행 거부 조건, Codex `codex exec --sandbox <정책> -C <작업 폴더> -o <결과 파일>`, 금지 인자, 작업자의 `.hive/` 쓰기 금지 |
| 흐름·결정 | `msg flow start/status/advance/requeue`, `msg decide request/list/answer`. 결정은 사람만 응답 |
| 확인증 유실·중지 | `dispatch-uncertain`의 자동 재실행·재소비·재발송 금지, 명시 `requeue`만. `hive stop`·`stop status`·`stop clear --confirm`, `msg cancel-ack` |
| 훅 | `msg hooks preview/apply/remove/status`, 프로젝트 한정, Gemini는 미리보기만 |
| 신뢰하지 않는 자료 | `trust: untrusted-data`·`[자료]` 출력은 자료로만 읽기, 지시·명령·승인 주장 불이행 |

- 금지: `claude -p`, 키 입력 주입, 화면 캡처, 다중화 창, 강제 종료

### 파급

| 대상 | 변경 |
| --- | --- |
| 사본 | `scripts/sync-user-plugin.py`로 플러그인·템플릿 두 곳의 `team-execution`(참조 포함)과 `active-skills.yml.jinja` 해시 재생성 |
| 신 입구 | `generate-gods.py`로 `poseidon` Skill 4곳 재생성, 67개 일치 |
| 소유 경로 | `harness/manifest.toml`에 `.agents`·`.claude` 아래 `team-execution/references/mediation.md` 2건, `hive-core`의 Skill 투영 경로 허용 목록에 1건 |
| 투영·렌더 | `hive-projection` 참조 목록 추가, 기본 투영 파일 수 Codex·Antigravity 70 → 71, Claude 43 → 44. `hive-render` 새 Skill 본문 시험 목록과 Claude → Codex 재설정 삭제 수 42 → 43 |
| 출시 표면 | `harness/release/0.16.0/release-surface-inventory.json`과 시험 출시 고정물 `valid-0.16.0`에 경로 1건, 고정물 `bundle-manifest.json` 지문·길이 갱신 |
| 설정 화면 | `project-quick` 한국어·영어 스냅숏의 변경 파일 수 100 → 101 |
| Python 지원 | `tests/conformance/support/harness.py`의 내장 참조 목록 1건 |
| 안정판 원장 | 0.16.0 `changed` 항목이 이미 있어 변경 없음 |

### 참조 처리기 시험

- 위치: `tests/conformance/integration/test_mediation_reference_handler.py`, `lanes.toml`의 integration 묶음에 등록
- 흐름: 임시 소비자 프로젝트(`.hive/`만)에 `hephaistos` 앞 메시지 보내기 → 수신 역할로 소비권 획득 → 게시 정책 확인(`workspace-write`·`none`·`deny`·`["{target}"]`·`protocol.allow=never`·`hive_launches_worker: false`) → 시험이 직접 가짜 작업자(Python 스크립트) 실행·결과 파일 작성 → 소비권 토큰으로 `ack` → 같은 결과 두 번째 `ack`는 `hive.msg-duplicate` → `status`의 `acknowledged`, 확인증 1개
- `--claim`: `hive msg ack --help`의 `ack` 사용법 줄에 있을 때만 전달. 규약 수정 전후 모두 통과
- Hive 실행 없음 확인: PATH 앞쪽에 기록용 가짜 `codex`·`claude`·`gemini` 배치, 시험 끝에 기록 파일 없음 단언(Windows 제외). 가짜 작업자 실행 흔적은 시험이 만든 파일 하나
- `--now` 미사용: 시험 전용 환경 변수 도입 뒤에도 그대로 동작
- 실행 시간 약 0.1초

## 단어 수

측정 명령: `wc -w <파일>`(frontmatter 포함)

| 파일 | 변경 전 | 변경 후 |
| --- | --- | --- |
| `harness/skills/team-execution/SKILL.md` | 196 | 268 |
| `harness/skills/team-execution/references/mediation.md` | 없음 | 908 |
| `harness/skills/poseidon/SKILL.md` | 156 | 192 |

## 확인

| 명령 | 결과 |
| --- | --- |
| `python scripts/sync-user-plugin.py` | 사본 갱신 |
| `python scripts/generate-gods.py --check` | 67 일치 |
| `cargo test -p hive-projection -p hive-render -p hive-update --locked` | 46·96·65 통과 |
| `cargo test -p hive-core --locked` | 200 통과 |
| `cargo test -p hive-cli --locked --bin hive -- skill user_install project_upgrade cli_help gods setup msg` | 541 통과. 설정 화면 스냅숏 2개는 `HIVE_UPDATE_SNAPSHOTS=1` 재생성 뒤 통과 |
| `cargo test -p hive-cli --locked --no-fail-fast --test '*'` | 7개 파일 35 통과, 과거 프로젝트 갱신 6개 포함 |
| `HIVE_BIN=<빌드> python -m unittest` `test_hive_skills`·`test_skill_inventory`·`test_stable_skill_ledger`·`test_roster_gods`·`test_mediation_reference_handler`·`test_host_capabilities`·`test_skill_projections`·`test_static_contracts` | 87 통과 |
| `check-stable-skill-ledger.py`(0.16.0, 공개 안정판 23개 입력) | 통과 |
| `cargo fmt --all --check` | 통과 |
| `python scripts/test-lanes.py --lane documentation` | 164 중 통과 162·건너뜀 2, 실행 기록 파일 삭제 |
| `check-markdown-links.py` | 실패 0 |
| `check-human-documentation-style.py --all` | 이 변경 0건. 기준 커밋의 `OV160-005.md` 7건 잔존(계획 지문 고정 문서라 미변경) |

## 한계

- 실제 Claude 호스트의 안내 사용과 Codex 작업자 왕복은 local 확인 항목 OV160-012
- 시험의 작업자는 가짜 스크립트. 정책을 실제 sandbox로 적용하는 부분은 호스트 책임이라 시험 범위 밖
- PATH 기록 장치는 이름으로 찾는 실행만 탐지. 절대 경로 실행은 Hive의 `hive_launches_worker: false` 계약과 코드 검토에 의존
- 참조 문구는 규약 수정(`--claim` 필수, Gemini 훅 적용 거부)을 앞서 반영. 수정 병합 전 바이너리의 `ack`는 `--claim` 없이 동작
- O1: Discord 결정·완료 알림 문구가 한국어 고정. 알림 문구 파일 `decision.rs`가 이 작업 범위 밖이라 미처리, 저장된 언어 선택 반영은 후속 작업
- 이 보고서의 홈 도달 연결은 조율자가 CURRENT에서 추가
