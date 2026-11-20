# 0.16.0 local 인계 — local 세션 시작점

- 작성: 2026-11-20 cloud 세션. cloud 범위 작업 전부 완료 뒤의 유일한 local 진입 문서
- 대상 시험판: `aigent-hive@0.16.0-test.1`(후보 [44745029436](https://github.com/gvm1229/aigent-hive/actions/runs/44745029436), develop `714ab371`, 세 OS 수용 [44755373032](https://github.com/gvm1229/aigent-hive/actions/runs/44755373032)). 근거 [시험판 기록](OV160-009.md)
- 포함: 파일 수신 상자 `hive msg`([OV160-002](OV160-002.md)), 훅 사건·비동기 반환([OV160-003](OV160-003.md)), 작성·검토·자문 흐름([OV160-004](OV160-004.md)), 피해 범위 통제·`hive stop`([OV160-005](OV160-005.md)), 결정 대기열·Discord 발신([OV160-006](OV160-006.md)), Skill 통합([OV160-007](OV160-007.md)), usage-guard 축소·`hive advanced`([OV160-008](OV160-008.md)), [단계 검토 수정](stage-6-review.md)
- 결과 기록: 근거는 `docs/research/overhaul/OV160-012.md`. 통과하면 계획의 OV160-012 `complete`, 실패하면 실패 근거 첨부와 해당 cloud 항목(003·004·007)을 `agent-owned`로 되돌림([넘기기 규칙](../../plans/overhaul/venue.md#넘기기)). 수정은 cloud에서 0.16.0-test.2로
- 금지: `claude -p`, 공급자 API 호출, 자격 증명 파일 작성·검사, 전역 `~/.claude`·`~/.codex`·`~/.gemini` 설정 변경. 훅은 프로젝트 파일에만
- 메시지 훅·흐름은 실제 확인 전 실험 단계(출시 노트 COMPAT-04). Gemini 훅은 미리보기만

## 0. 준비

```powershell
git switch develop
git pull --ff-only
hive --version
```

- 기대: 저장소 최신, 현재 PC `AIgent Hive v0.15.3`
- 07 사용량 사전 확인 실행. 차단이면 중지

## 1. 시험판 설치와 갱신

```powershell
npm install -g aigent-hive@0.16.0-test.1
hive --version
hive update --user-root $HOME --non-interactive --confirm --output json
hive install --scope user --hosts codex,claude,antigravity --validate --output json
```

- 기대: `AIgent Hive v0.16.0`, update `status=success`, validate `hive.user-install-multi-host-valid`
- 기존 지식 파일 수와 사용자 답변 갱신 전후 동일
- 실패 시 볼 곳: `~/.hive/state/` 갱신 기록, `hive doctor --output json`

## 2. Skill 통합 표시(OV160-007 local 부분)

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
3. `release.yml`을 `main`에서: `product_version=0.16.0`, `package_version=0.16.0`, `channel=stable`, `release_date=<출시일>`, `promoted_test_version=0.16.0-test.1`, `acceptance_run_id=44755373032`
4. `release-publish.yml`: `channel=stable`, 3의 후보 run ID, `send_subscriber_update=true`. npm 전파 지연(종료 6)이면 같은 입력에 `recover_published_packages=true`
5. 이 PC: `npm install -g aigent-hive@0.16.0`, `hive update --user-root $HOME --non-interactive --confirm --output json`, 1단계 validate 재확인
6. 근거 `OV160-011.md`, 계획 OV160-011·012 `complete`, 단계 브랜치 `feature/overhaul-6-0.16.0-mediation` 원격·로컬 삭제, 단계 7 시작은 [재정비 순서](../../plans/overhaul/README.md)
