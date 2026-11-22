# 0.16.0 local 인계 — local 세션 시작점

- **2026-11-23 재개**: [재개 계획](OV160-resume.md)의 순서와 권한 사용. 백업은 `~/hive-backups/`처럼 `~/.hive` 밖에만 저장
## 판정 규칙

- 실패는 기본 `product-defect` 판정 입력
- `host-limit`은 실행 전 이 문서에 한계가 기록됐거나, Hive 없이도 재현되는 근거(호스트 문서 또는 재현 기록)가 있을 때만 선택
- 이전 안정판에서 통과한 확인의 실패는 항상 `product-defect`
- `host-limit`마다 출시 COMPAT 항목 필수, 모순되는 출시 안내 주장은 제거
- 새 종류의 `host-limit`은 `docs/decisions/`에 기록하고 출시 안내에 명시
- `python3 scripts/check-plan-state.py --release` 통과 뒤 안정판 준비 계속

- **2026-11-21 test.3 local 결과: 실패**([OV160-012](OV160-012.md)). cloud가 R1–R6 수정과 test.4 게시 뒤 이 문서 대상 갱신. 그 전에는 실행 금지
- 작성: 2026-11-20 cloud 세션. cloud 범위 작업 전부 완료 뒤의 유일한 local 진입 문서
- 대상 시험판: `aigent-hive@0.16.0-test.3`(후보 [44838848659](https://github.com/gvm1229/aigent-hive/actions/runs/44838848659), develop `73d21b87`, 세 OS 수용 [44842442665](https://github.com/gvm1229/aigent-hive/actions/runs/44842442665)). 근거 [시험 기록](OV160-013-test2.md). test.2는 Windows 수용 실패로 설치 금지. test.1 기록은 [OV160-009](OV160-009.md)
- test.3 추가 내용(test.1 대비): [OV160-013](OV160-013.md) 측정 없음 경고 후 계속, 소스 폴더 Claude 기록, 기록 실패 사유, 동의 기반 `statusLine` 연결
- develop 이력 주의: cloud가 2026-11-20 19:57Z에 자기 커밋 3개(`3cae7fde`·`adb96315`·`02e4c39d`)를 작성자 정정으로 교체(현재 develop 이력). 그 사이 pull한 local 사본은 `git fetch origin develop` 뒤 `git reset --keep origin/develop`
- 포함: 파일 수신 상자 `hive msg`([OV160-002](OV160-002.md)), 훅 사건·비동기 반환([OV160-003](OV160-003.md)), 작성·검토·자문 흐름([OV160-004](OV160-004.md)), 피해 범위 통제·`hive stop`([OV160-005](OV160-005.md)), 결정 대기열·Discord 발신([OV160-006](OV160-006.md)), Skill 통합([OV160-007](OV160-007.md)), usage-guard 축소·`hive advanced`([OV160-008](OV160-008.md)), [단계 검토 수정](stage-6-review.md)
- 결과 기록: 근거는 `docs/research/overhaul/OV160-012.md`. 통과하면 계획의 OV160-012 `complete`, 실패하면 실패 근거 첨부와 해당 cloud 항목(003·004·007·013)을 `agent-owned`로 되돌림([넘기기 규칙](../../plans/overhaul/venue.md#넘기기)). 수정은 cloud에서 다음 번호 시험판으로
- 금지: `claude -p`, 공급자 API 호출, 자격 증명 파일 작성·검사, 에이전트의 전역 `~/.claude`·`~/.codex`·`~/.gemini` 직접 수정. 훅은 프로젝트 파일에만. 예외는 2단계의 사용자 승인 `statusLine` 연결(Hive 명령이 미리보기·확인 지문 뒤 변경)
- 메시지 훅·흐름은 실제 확인 전 실험 단계(출시 노트 COMPAT-04). Gemini 훅은 미리보기만

## 0. 준비

```powershell
git switch develop
git pull --ff-only
hive --version
```

- 기대: 저장소 최신, 현재 PC `AIgent Hive v0.15.3`
- 07 사용량 사전 확인 실행. 0.15.3에서 `hive.usage-unknown` 차단이면 기존처럼 이 세션만 `hive usage session --action disable --confirm-session-disable`로 진행(사용자 선택), 1단계 설치 뒤 재확인

## 1. 시험판 설치와 갱신

```powershell
npm install -g aigent-hive@0.16.0-test.3
hive --version
hive update --user-root $HOME --non-interactive --confirm --output json
hive install --scope user --hosts codex,claude,antigravity --validate --output json
```

- 기대: `AIgent Hive v0.16.0`, update `status=success`, validate `hive.user-install-multi-host-valid`
- 기존 지식 파일 수와 사용자 답변 갱신 전후 동일
- 실패 시 볼 곳: `~/.hive/state/` 갱신 기록, `hive doctor --output json`

## 2. Claude 사용량 기록 확인(OV160-013 local 부분)

```powershell
hive usage enforce --target <소스 폴더> --host claude --session-id <현재 세션> --process-id <현재 프로세스> --user-root $HOME --output json
hive setup --scope user --section usage --claude-statusline preview --output json
```

- 기대(연결 전): exit 0 `hive.usage-observation-warning`, `missing_windows=["session","weekly"]`, `observation_cause=no-measurement`, `fix_command`에 preview 명령. 차단·"작업 중단" Discord 없음
- 미리보기의 `current_status_line`·`proposed_status_line`을 사용자에게 보여 주고 승인 받은 뒤에만:

```powershell
hive setup --scope user --section usage --claude-statusline apply --confirm <approval_digest> --output json
```

- 기대: `~/.claude/settings.json`에서 `statusLine`만 변경(다른 키 바이트 동일), 기존 상태 표시줄 명령은 `hive usage statusline --host claude --pass-through | <기존 명령>`으로 이어 실행되어 화면 유지
- Claude Code 터미널 세션에서 응답 1회 뒤 같은 enforce → `hive.usage-allowed`(또는 실제 측정이 기준 이하면 `hive.usage-limited`). 기록 위치 `$HOME\.hive\runtime\usage-guard\targets\<지문>\claude\`
- Claude 데스크톱 앱 Code 탭에서도 응답 1회 뒤 같은 확인. 기록이 생기지 않으면 Code 탭 미실행으로 기록(출시 노트 COMPAT-05 유지), 생기면 COMPAT-05 수정 요청을 OV160-012.md에 기록
- 실패 시 `hive usage status --target <소스 폴더> --host claude --session-id <id> --process-id <pid> --user-root $HOME --output json`의 `claude_capture_failure.reason` 기록
- 확인 뒤 원하면 되돌리기: `hive setup --scope user --section usage --claude-statusline revert` → 표시된 `--confirm <지문>`으로 재실행, 원래 바이트 복원 확인

## 2-1. Skill 통합 표시(OV160-007 local 부분)

```powershell
codex plugin list --json
claude plugin list
agy plugin list
```

- 기대: 다섯 앱(Codex CLI·Codex 앱·Claude Code·Claude 데스크톱·Antigravity)에서 새 이름 `knowledge-manage`·`run`·`judge` 표시, 옛 이름 9개(`knowledge-maintain`·`knowledge-promote`·`knowledge-transfer`·`knowledge-scan`·`run-checkpoint`·`run-handoff`·`run-resume`·`adversarial-judge`·`judge-evidence`) 표시 없음
- Claude Code에서 `/aigent-hive:run-resume`처럼 옛 이름을 부르면 새 이름 `run`의 이어 하기 방식 안내 1회
- 기존 프로젝트 하나에서 `hive project upgrade --target . --user-root $HOME --scan --output json` → 옛 Skill 폴더 제거·새 폴더 생성 미리보기, 사용자가 바꾼 파일은 보존 표시

## 3. 실제 Claude+Codex 왕복과 5초 측정(OV160-003 local 부분)

새 빈 폴더에서 프로젝트 한정 훅 적용.

```powershell
New-Item -ItemType Directory $HOME\hive-160-hook
Set-Location $HOME\hive-160-hook
hive setup --target . --user-root $HOME
hive msg hooks preview --host claude --role athena --target . --user-root $HOME --output json
hive msg hooks apply --host claude --role athena --confirm <preview의 approval_digest> --target . --user-root $HOME --output json
hive msg hooks preview --host codex --role hephaistos --target . --user-root $HOME --output json
hive msg hooks apply --host codex --role hephaistos --confirm <approval_digest> --target . --user-root $HOME --output json
```

- 기대: `.claude/settings.local.json`·`.codex/hooks.json`만 바뀜, 전역 설정 바이트 동일
- Claude Code 대화형 세션에서 `hive msg send --to hephaistos --from athena --file task.md --target . --user-root $HOME` 실행 요청 → 같은 세션이 [중재 안내](../../../harness/skills/team-execution/references/mediation.md)에 따라 `msg claim`, 정책 파일 확인, `codex exec --sandbox <정책> -o result.txt`, `msg ack --id <id> --claim <token> --result result.txt`
- 기대: 다음 Stop에서 결과가 `additionalContext`로 1회 전달, "Stop hook error" 표시 없음. 전달할 결과가 없는 Stop은 대화 계속 없음(추가 모델 호출 0회). 같은 질문의 계속 Stop은 재전달 없음
- 5초 측정: `.hive/mediation/events/` 기록의 `received_at_ms`와 호스트 훅 시작 시각 차이 5초 이하 3회
- Claude가 `additionalContext` 반환을 받지 못하는 버전이면 `--return block`으로 다시 적용 후 같은 확인, 결과 기록
- Codex 설치 버전의 공식 훅 이벤트 이름 확인 기록(`codex --version`, 공식 문서)
- 실패 시: 훅 입력·출력(`.hive/mediation/hook-responses/`)과 호스트 표시를 `OV160-012.md`에 기록, OV160-003을 `agent-owned`로 되돌림

## 4. 교차 검토 흐름(OV160-004 local 부분)

```powershell
hive msg flow start --builder hephaistos --reviewer athena --advisor artemis --task t1 --file brief.md --target . --user-root $HOME --output json
hive msg flow status --flow <flow-id> --target . --user-root $HOME
```

- 기대: 작성 단계(Codex 작업자) → 검토 단계(Claude 호스트) 순서. 상태 줄 예 `Athena (Claude Fable 5.2 · high) ▸ 검토 중`이 Hive 기록과 일치
- 작업자를 확인증 전에 끝낸 뒤 기한 경과 → `확인 불가`, `hive msg flow requeue --flow <id> --confirm <지문>` 1회로 새 시도 1개
- Gemini 동료 작업자는 미검증 유지, Antigravity 대체 확인은 [실행 장소](../../plans/overhaul/venue.md) 결정 기준

## 5. 통과 뒤 정식 0.16.0(1–4 통과 뒤에만, OV160-011)

1. 정식 공개 문서 준비(README·설치 안내·출시 목록·`docs/public-stable-release.json`)와 구독자 안내 `docs/releases/0.16.0.subscriber.ko.md` 작성, `python scripts/check-public-stable-docs.py` 통과
2. develop→main PR(일반 병합). 이 PR이 구독자 안내 승인
3. `release.yml`을 `main`에서: `product_version=0.16.0`, `package_version=0.16.0`, `channel=stable`, `release_date=<출시일>`, `promoted_test_version=0.16.0-test.3`, `acceptance_run_id=44842442665`
4. `release-publish.yml`: `channel=stable`, 3의 후보 run ID, `send_subscriber_update=true`. npm 전파 지연(종료 6)이면 같은 입력에 `recover_published_packages=true`
5. 이 PC: `npm install -g aigent-hive@0.16.0`, `hive update --user-root $HOME --non-interactive --confirm --output json`, 1단계 validate 재확인, `python3 scripts/check-plan-state.py --release` 확인
6. 근거 `OV160-011.md`, 계획 OV160-011·012 `complete`, 단계 브랜치 `feature/overhaul-6-0.16.0-mediation` 원격·로컬 삭제, 단계 7 시작은 [재정비 순서](../../plans/overhaul/README.md)
