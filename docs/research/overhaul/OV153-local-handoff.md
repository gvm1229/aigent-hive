# 0.15.3 local 인계 — local 세션 시작점

- 작성: 2026-11-17 cloud 세션. cloud 범위 작업 전부 완료 뒤의 유일한 local 진입 문서
- 대상 시험판: `aigent-hive@0.15.3-test.2`(후보 TEST2_CANDIDATE, develop `TEST2_COMMIT`, 세 OS 수용 TEST2_ACCEPTANCE). 근거 [시험판 기록](OV153-011-test2.md)
- 포함: 기억 회수(OV153-002–007), 에이전트 주도 설정([OV153-011](OV153-011.md), 추가 범위 A1–A4), 설정 화면 가독성([OV153-012](OV153-012.md)), 변경 0건 프로젝트 갱신 검증([OV153-013](OV153-013.md))
- 결과 기록: 근거는 `docs/research/overhaul/OV153-010.md`. 통과한 항목은 계획에서 `complete`, 실패한 항목은 `agent-owned`로 되돌리고 실패 근거 첨부([넘기기 규칙](../../plans/overhaul/venue.md#넘기기))
- 금지: `claude -p`, 공급자 API 호출, 자격 증명 파일 작성
- 1–3b단계는 이 Windows PC 전용. 4단계 정식 게시는 워크플로 실행이므로 cloud·local 모두 가능([실행 장소](../../plans/overhaul/venue.md))

## 0. 준비

```powershell
git switch develop
git pull --ff-only
hive --version
```

- 기대: 저장소 최신, 현재 PC `AIgent Hive v0.15.2` 또는 `v0.15.3`(test.1)
- 07 사용량 사전 확인 실행. 차단이면 중지

## 1. 시험판 설치와 갱신

```powershell
npm install -g aigent-hive@0.15.3-test.2
hive --version
hive update --user-root $HOME --non-interactive --confirm --output json
hive install --scope user --hosts codex,claude,antigravity --validate --output json
```

- 기대: `AIgent Hive v0.15.3`, update `status=success`, validate `hive.user-install-multi-host-valid`
- 기존 지식 저장소(`~/.hive/knowledge/Claims/user-root`) 파일 수 갱신 전후 동일
- 실패 시 볼 곳: `~/.hive/state/` 갱신 기록, `hive doctor --output json`

## 2. 호스트 표시와 기억 왕복

```powershell
codex plugin list --json
claude plugin list
agy plugin list
```

- 기대: 세 호스트 모두 aigent-hive 0.15.3 enabled·import, 각 앱 화면에서 Hive Skill 표시(유지관리자 확인)
- Claude Code 대화형 세션 하나에서 아래 1 요청, 같은 세션 끝에 2 실행, 새 세션에서 3 질문. Codex에서 1·3 반복

```text
1. 앞으로 HTML 화면 디자인: 미니멀, 색은 남색과 흰색, 보라색 그라데이션 금지
2. hive knowledge subjects list --user-root $HOME --output json
3. HTML 디자인 취향 기억나?
```

- 기대: 1의 턴에서 `remember --batch` 1회로 같은 주제의 사실 3개, 2에 그 주제와 `active_claims: 3`, 3의 회수가 세 사실을 함께 반환

## 3. 실제 세션의 project-setup Skill(OV153-011 local 부분)

빈 폴더와 기존 프로젝트를 각각 Claude Code와 Codex 대화형 세션에서 확인.

```powershell
New-Item -ItemType Directory $HOME\hive-setup-empty
Set-Location $HOME\hive-setup-empty
claude
```

- 요청: `/aigent-hive:project-setup` 호출(Codex는 같은 Skill 이름으로 호출)
- 빈 폴더 기대:
  - 에이전트가 터미널 명령을 넘기지 않고 대화에서 질문
  - 프로젝트 내용, 목표, 개발 방식을 한 번에 하나씩 질문. 종류 추측 뒤 즉시 적용 없음
  - 제안 한 번과 사용자 확인 뒤 `--input-channel agent --apply`, 이어 `--show`로 확인
  - 능력 파일을 만들지 않고 `--capabilities` 없이 적용

기존 프로젝트(예: 이 PC의 `korean-ai-job-search`)에서 같은 호출.

- 기존 프로젝트 기대:
  - 제한된 파일 탐색 뒤 근거(파일 종류, 의존성)를 들어 종류를 제안. 모호하면 근거와 함께 질문
  - 역할, hook, 선택 Skill, 지식 범위, 임계값의 질문·임의 생성 모두 없음
  - 적용 뒤 `hive setup --target . --show --summary --output json`의 `status=success`, 저장된 선택이 제안과 일치

```powershell
hive install --scope user --hosts codex,claude,antigravity --validate --output json
```

- 기대: `hive.user-install-multi-host-valid`
- 실패 시: 호스트 응답과 실행 명령을 `docs/research/overhaul/OV153-010.md`에 기록, 해당 항목을 `agent-owned`로 되돌림. 수정은 cloud에서 0.15.3-test.3로

## 3a. 보고된 두 명령 재현(OV153-011 A4)

새 빈 폴더에서 0.15.2 보고의 두 명령을 test.2로 재현. 사용자 설정 완료 상태의 `$HOME` 사용.

```powershell
New-Item -ItemType Directory $HOME\hive-a4-empty
$patch = Join-Path $env:TEMP 'hive-a4-patch.json'
'{"schema_version":1,"kind":"hive.setup-answer-patch","answers":{"setup_mode":"expedited","project_name":"hive-a4-empty","project_kind":"general","primary_host":"claude"}}' | Set-Content -Encoding utf8 $patch
hive setup --target $HOME\hive-a4-empty --user-root $HOME --answers $patch --input-channel agent --dry-run --output json
hive setup --target $HOME\hive-a4-empty --user-root $HOME --non-interactive --dry-run --output json
hive setup --target $HOME\hive-a4-empty --user-root $HOME --answers $patch --input-channel agent --apply --output json
hive setup --target $HOME\hive-a4-empty --user-root $HOME --answers $patch --input-channel agent --validate --output json
```

- 기대 1(현상 1): `status=success`, `hive.setup-dry-run-complete`, `data.capabilities.source=saved-or-detected-by-cli`, 폴더 비어 있음. `missing required option --capabilities` 없음
- 기대 2(현상 2): `hive.project-setup-input-pending`, `next_action`에 `--show --summary`와 `--input-channel agent --dry-run`, 문구에 능력 파일 선택 사항 안내. 터미널 재개는 `data.terminal_next_action`에만
- 기대 3: 적용 `hive.setup-complete`, 검증 `hive.setup-valid`
- 실패 시: 출력 JSON 전체를 `OV153-010.md`에 기록, OV153-011을 `agent-owned`로 되돌림

## 3b. 설정 화면 가독성(OV153-012)

실제 터미널(Windows Terminal의 PowerShell)에서 새 빈 폴더의 빠른 설정과 사용자 설정 조회를 확인. 마지막 확인 질문은 `아니요`.

```powershell
New-Item -ItemType Directory $HOME\hive-012-screen
hive setup --target $HOME\hive-012-screen --user-root $HOME
hive setup --scope user --user-root $HOME --show
$env:NO_COLOR = '1'; hive setup --scope user --user-root $HOME --show; Remove-Item Env:NO_COLOR
```

- 기대(질문): 질문마다 앞 빈 줄, 청록 굵은 제목 `[1/4] 설정 방식`–`[4/4] 사용할 앱`, 들여쓴 질문 문장·선택지, Enter로 유지되는 선택지에 `(현재값)`, 빈 줄 뒤 입력 줄 `> `
- 기대(미리보기): 제목 `프로젝트 설정 미리보기`, 굵은 구역 `일반`·`기능`·`지식 경로`·`변경 규모`, `이름 : 값` 정렬, 빈 값 `없음`, 기능 묶음은 `개별 선택 N개`와 여러 줄 이름, 확인 질문 앞 구분선
- 기대(사용자 설정 조회): 제목 `적용 전 확인`과 `일반`·`기능`·`지식`·`사용량`·`알림` 구역. `NO_COLOR=1`에서는 같은 배치에 색 없음
- 좁은 창(80열)에서 줄이 깨지지 않음을 유지관리자가 화면으로 확인. 고정 화면 비교는 `tests/fixtures/setup/screens/`
- 실패 시: 화면 캡처와 문제 줄을 `OV153-010.md`에 기록, OV153-012를 `agent-owned`로 되돌림

## 3c. 새 프로젝트 직후 갱신 검사(OV153-013 B5)

3b에서 멈춘 빈 폴더 설정을 `--resume`으로 이어 `예`로 적용한 뒤, 그리고 3a의 에이전트 적용 폴더에서 각각 실행.

```powershell
hive setup --target $HOME\hive-012-screen --user-root $HOME --resume
hive project upgrade --target $HOME\hive-012-screen --user-root $HOME --scan --output json
hive project upgrade --target $HOME\hive-012-screen --user-root $HOME --validate --output json
hive project upgrade --target $HOME\hive-a4-empty --user-root $HOME --scan --output json
hive project upgrade --target $HOME\hive-a4-empty --user-root $HOME --validate --output json
```

- 기대: 네 결과 모두 `status=success`, `code=hive.project-upgrade-current`, `changed_paths` 빈 목록, `data.change_reasons` 빈 목록. `.hive\config\project-overrides.json` 미생성
- 실패 시: `--scan` 결과의 `changed_paths`·`change_reasons`·`reports`를 `OV153-010.md`에 기록, OV153-013을 `agent-owned`로 되돌림

## 4. 통과 뒤 정식 0.15.3(1–3c 통과 뒤에만)

정식 공개 문서는 cloud에서 준비 완료(출시일 자리 `2026-11-16`). 출시일이 다르면 아래 파일의 날짜만 바꿈: `README.md`, `docs/readme/README.ko.md`, `docs/hive-install-guide.ko.html`, `docs/overview/product.md`, `docs/01-index.md`, `docs/releases/README.md`, `docs/public-stable-release.json`. 바꾼 뒤 `python scripts/check-public-stable-docs.py` 통과 확인.

1. develop→main PR(일반 병합). 이 PR이 구독자 안내 [0.15.3.subscriber.ko.md](../../releases/0.15.3.subscriber.ko.md) 승인
2. `release.yml`을 `main`에서 실행: `product_version=0.15.3`, `package_version=0.15.3`, `channel=stable`, `release_date=<출시일>`, `promoted_test_version=0.15.3-test.2`, `acceptance_run_id=TEST2_ACCEPTANCE`
3. `release-publish.yml`: `channel=stable`, 2의 후보 run ID, `send_subscriber_update=true`. npm 전파 지연(종료 6)이면 같은 입력에 `recover_published_packages=true`로 재실행
4. 확인: npm 6개 `latest=0.15.3`, GitHub Release `v0.15.3` 정식, 구독자 전송 단계 success
5. 이 PC: `npm install -g aigent-hive@0.15.3`, `hive --version`, `hive update --user-root $HOME --non-interactive --confirm --output json`, 1단계의 validate 재확인
6. 근거 `docs/research/overhaul/OV153-010.md`, 계획의 OV153-010·OV153-011·OV153-012·OV153-013 `complete`, 단계 브랜치 `feature/overhaul-5-0.15.3-memory` 원격·로컬 삭제, 단계 6 시작은 [재정비 순서](../../plans/overhaul/README.md)
