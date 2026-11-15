# OV152 이어 하기 — 정식 0.15.2 남은 단계

- 기준일: 2026-11-15, 진행률 약 85%
- 남은 항목: [OV152-008 정식 0.15.2](../../plans/active/0.15.2-host-install.md)
- 완료: OV152-001–007·009·010. 공개 0.15.2-test.4 세 OS 수용([OV152-006](OV152-006.md)), Windows PC 다섯 앱 확인([OV152-007](OV152-007.md))
- develop 머리: 정식 공개 문서 준비 커밋 `5b5a6c0c`와 이 문서. develop CI 통과 확인 뒤 진행

## 이미 준비된 것

- 공개 문서 5곳·`docs/public-stable-release.json`: 정식 0.15.2, 날짜 2026-11-15. `check-public-stable-docs.py --channel stable` 통과(registry-latest 항목은 게시 뒤 검사)
- 출시 노트 [0.15.2](../../releases/0.15.2.md), 구독자 원문 `docs/releases/0.15.2.subscriber.ko.md`와 `.sha256`
- 수용 등록 `docs/public-test-product.json`: 0.15.2-test.4, 소스 77dd6c104fbfafd8b5f229c6553e597712d50664, 지문 sha256:aa598c56fd3408fbbb671b39da21d2f8db5e50d1776652ffaa64d65ec9b9f3af

## 남은 순서

1. develop CI 통과 확인. Windows 설정 터미널 시험의 os error 32는 알려진 일시 실패, `gh run rerun <id> --failed` 1회
2. develop→main PR. 제목 예: `build: 0.15.2 사용자 설치 복구와 여러 앱 동시 설치 정식 승격`. 본문은 PR #98 형식: 변경·검증(test.4 소스·지문·수용 43723697615)·게시와 승인(구독자 원문·지문, `AIGENT_HIVE_SUBSCRIBER_APPROVAL=main-pr`, `send_subscriber_update=true`). AI 표기·세션 링크 금지
3. PR 검사 통과 뒤 일반 병합(`gh pr merge <n> --merge`)
4. 정식 후보: `gh workflow run release.yml --ref main -f product_version=0.15.2 -f package_version=0.15.2 -f channel=stable -f release_date=2026-11-15 -f promoted_test_version=0.15.2-test.4 -f acceptance_run_id=43723697615`
5. 게시: `gh workflow run release-publish.yml --ref main -f product_version=0.15.2 -f package_version=0.15.2 -f candidate_run_id=<후보> -f channel=stable -f send_subscriber_update=true`. 종료 6(npm 전파 지연)이면 모든 패키지의 `npm view <pkg>@0.15.2 version` 표시를 기다린 뒤 같은 명령에 `-f recover_published_packages=true`. staged 버전 E409도 표시 대기 뒤 복구
6. 확인: 6개 패키지 `latest=0.15.2`(aigent-hive, @aigent-hive/darwin-arm64·darwin-x64·linux-x64·linux-arm64·win32-x64), GitHub Release v0.15.2 정식·파일 29개, 게시 실행의 Discord 승인·전송 단계 success
7. 종료 기록: OV152-008 근거 문서 작성·완료 표시, `check-plan-state.py --write`, CURRENT 갱신, 사실 출처 지문 갱신, documentation 시험, develop push
8. PC 설치(Windows 현장 전용, 클라우드 세션 제외): `npm install -g aigent-hive@0.15.2` 뒤 `hive update --user-root <home>`, `hive install --scope user --hosts codex,claude,antigravity --validate`

## 주의

- 사실 문서 `repo:<경로>#sha256:` 지문은 출처 파일이 바뀔 때마다 갱신. 시험 의도·수용 등록 수정 뒤 특히 `automatic-test-release-gate` 확인
- 문서 시험은 표준 입력을 닫고 실행(`</dev/null`), 열린 입력에서 멈춘 기록 있음
- 재정비 단계 5 이후 보류 유지. 이 요청은 0.15.2 정식까지
- 남은 후속: Windows CI os error 32 일시 실패, `test_setup_console_interrupt` 불안정, 0.15.1 고착 PC의 수동 npm 설치 필요(출시 노트 COMPAT-03)
