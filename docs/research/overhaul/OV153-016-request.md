# OV153-016 요청 — npm 설치 뒤 `hive update`의 이전 버전 투영 인증

- 기준일: 2026-11-19. local 세션 1단계에서 발견
- 환경: Windows 11, 0.15.2 정식 설치 상태에서 `npm install -g aigent-hive@0.15.3-test.2`

## 재현

- `hive update --user-root $HOME --non-interactive --confirm --output json` → exit 2, `hive.update-failed`, "installed ownership manifest is not bound to the running Hive product: codex", `steps.core=not-started`. `--channel test` 결과 동일
- 같은 상태의 `hive install --scope user --hosts codex,claude,antigravity --dry-run --output json` → `hive.user-install-multi-host-dry-run-complete`(이전 투영 인증 성공)

## 원인

- `crates/hive-cli/src/update_activation.rs`의 갱신 흐름이 먼저 `user_install::authenticated_saved_projection_hosts`를 부르고, `authenticate_saved_projection_host`가 `manifest.product_version != CARGO_PKG_VERSION`이면 바로 거부
- 설치 경로(`validate_prior_ownership`)는 `authenticated_user_inventory`로 이전 공개 버전의 투영을 인증하므로, 두 경로의 인증 기준 불일치
- 결과: 안내된 갱신 순서(npm으로 새 버전 설치 → `hive update`)가 이전 버전 투영이 있는 모든 PC에서 실패

## 범위(0.15.3 필수)

- C1 갱신 흐름의 호스트 인증이 이전 버전 투영을 설치 경로와 같은 기준(`authenticated_user_inventory`·바이트 검증)으로 인증. 위조·변조·해석 불가 기록은 계속 실패
- C2 인증된 이전 버전 호스트는 갱신의 투영 재적용 대상에 포함
- C3 시험: 이전 공개 버전 투영이 있는 사용자 루트에서 인증 성공, 변조 시 실패 유지
- C4 local 확인: 정식 0.15.3 설치 뒤 `hive update` 성공

## 추가 범위(같은 날 1단계 우회 중 발견)

- 재현: test.2에서 `hive install --scope user --hosts codex,claude,antigravity --apply` → exit 5, "claude still reports a Hive-owned activation after uninstall". 이후 Codex·Claude의 `aigent-hive` 플러그인과 `~/.codex/AGENTS.md`의 Hive 묶음, `.hive/install/codex.json`이 사라진 상태로 남음(Antigravity 유지). 지식 파일 578개 유지
- 원인 1: Claude CLI 2.1.325의 `claude plugin marketplace remove aigent-hive --scope user`가 사용자 설정 밖(알려진 마켓플레이스 목록)에만 있는 등록을 `not_configured`로 남김. "Omit --scope to remove it from all scopes" 안내
- 원인 2: 보존 재설치의 제거 단계(`execute_uninstall`)가 호스트마다 활성화 제거와 파일 삭제를 이어서 하므로, 뒤 호스트가 실패하면 앞 호스트만 완전히 제거된 상태로 멈춤. 오류에 복구 안내 없음
- 이 PC 복구: 범위 없는 `claude plugin marketplace remove aigent-hive` 뒤 호스트별 `hive install --scope user --host <h> --apply`, 마지막 `--hosts codex,claude,antigravity --validate` → `hive.user-install-multi-host-valid`
- C5 Claude 마켓플레이스 제거는 범위 없이 실행(설치·비활성화·조정 경로 모두)
- C6 제거 단계는 모든 호스트의 활성화 제거가 끝난 뒤에만 파일 삭제. 실패 시 투영·소유 기록 유지, 오류에 비활성화된 호스트의 복구 명령 표시
