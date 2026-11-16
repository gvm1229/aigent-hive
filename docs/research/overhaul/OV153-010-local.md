# OV153-010 local — 이 PC 실제 호스트 확인과 정식 0.15.3

- 작성: 2026-11-16 cloud 세션. 상태: `awaiting-external-evidence; owner: local-pc`
- 대상: 공개 시험 `aigent-hive@0.15.3-test.1`(후보 43816758281, develop `60c4c4f0`, 세 OS 수용 43823805950). [공개 시험 근거](OV153-008.md)
- 이 파일의 1–3단계는 이 Windows PC 전용. 4단계 이후 정식 게시는 cloud·local 어느 쪽도 가능(워크플로 실행 cloud 가능, [실행 장소](../../plans/overhaul/venue.md))

## 0. 준비

```powershell
git switch develop
git pull --ff-only
hive --version
```

- 기대: 저장소 최신, 현재 PC `AIgent Hive v0.15.2 (released 2026-11-16)`
- 07 사용량 사전 확인 실행. 차단이면 중지

## 1. 0.15.2에서 0.15.3-test.1로 갱신

```powershell
npm install -g aigent-hive@0.15.3-test.1
hive --version
hive update --user-root $HOME --non-interactive --confirm --output json
hive install --scope user --hosts codex,claude,antigravity --validate --output json
```

- 기대: `AIgent Hive v0.15.3`, update `status=success`, validate `hive.user-install-multi-host-valid`
- 실패 시 볼 곳: `~/.hive/state/` 갱신 기록, `hive doctor --output json`. 기존 지식 저장소(`~/.hive/knowledge/Claims/user-root`) 파일 수 갱신 전후 동일 확인

## 2. 호스트별 표시와 Skill 문구

```powershell
codex plugin list --json
claude plugin list
agy plugin list
```

- 기대: 세 호스트 모두 aigent-hive 0.15.3 enabled·import. Codex App·Claude App·Antigravity IDE 화면에서 Hive Skill 표시(유지관리자 확인)
- Skill 문구 확인: 설치된 knowledge-capture에 `--batch`·`subjects list`, knowledge-recall에 `--expansions "<term,term>"`(쉼표·따옴표)

## 3. 실제 호스트의 기억 기록·회수 왕복(이 PC 구독 로그인, 모델 실행은 호스트 소유)

- Claude Code 대화형 세션 하나에서 아래 1의 요청 뒤 응답 확인, 같은 세션 끝에 2 실행, 새 세션에서 3 질문. `claude -p` 금지

```text
1. 앞으로 HTML 화면 디자인: 미니멀, 색은 남색과 흰색, 보라색 그라데이션 금지
2. hive knowledge subjects list --user-root $HOME --output json
3. HTML 디자인 취향 기억나?
```

- 기대:
  - 1의 턴에서 `remember --batch` 1회로 사실 3개, 같은 주제(예 `design/html-style`) 기록
  - 2에 해당 주제와 `active_claims: 3`
  - 3의 회수가 세 사실을 함께 반환(`subject` 묶음), 확장어는 `--expansions "...,..."` 형식
- Codex에서도 1·3을 반복해 같은 결과 확인. Antigravity는 설치·Skill 표시까지(Gemini CLI 지원 종료로 실제 수용은 대체 확인 표시)
- 실패 시: 호스트 응답·실행된 명령 기록을 `docs/research/overhaul/OV153-010.md`에 남기고 OV153-010을 `agent-owned`로 되돌림. 수정은 cloud 가능, 새 수정은 0.15.3-test.2부터

## 4. 통과 시 정식 0.15.3(1–3 통과 뒤에만)

1. 정식 공개 문서 준비(이전 `5b5a6c0c` 형식): README 공개 안정판 절·`docs/01-index.md`·관련 사실 페이지를 0.15.3과 실제 출시일로, `docs/releases/0.15.3.md` 유지, 미발행 구독자 안내 [0.15.3.subscriber.ko.md](../../releases/0.15.3.subscriber.ko.md) 최종 확인 뒤 sha256 갱신
2. develop→main PR(일반 병합). 이 PR이 구독자 안내 승인
3. `release.yml`을 `main`에서 실행: `product_version=0.15.3`, `package_version=0.15.3`, `channel=stable`, `release_date=<출시일>`, `promoted_test_version=0.15.3-test.1`, `acceptance_run_id=43823805950`
4. `release-publish.yml`(`channel=stable`, 같은 후보 run ID, `send_subscriber_update=true`). npm 전파 지연(종료 6)이면 같은 입력에 `recover_published_packages=true`
5. 확인: npm 6개 `latest=0.15.3`, GitHub Release `v0.15.3` 정식, 구독자 전송 단계 success
6. 이 PC: `npm install -g aigent-hive@0.15.3`, `hive --version`, `hive update`, 2단계 명령 재확인
7. 근거 `docs/research/overhaul/OV153-010.md`, OV153-010 `complete`, 단계 브랜치 `feature/overhaul-5-0.15.3-memory` 원격·로컬 삭제, 단계 6 시작은 [재정비 순서](../../plans/overhaul/README.md)
