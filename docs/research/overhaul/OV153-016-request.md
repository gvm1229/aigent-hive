# OV153-016 요청 — npm 설치 뒤 `hive update`의 이전 버전 투영 인증

- 기준일: 2026-11-19. local 세션 1단계에서 발견
- 환경: Windows 11, 0.15.2 정식 설치 상태에서 `npm install -g aigent-hive@0.15.3-test.2`

## 재현

- `hive update --user-root $HOME --non-interactive --confirm --output json` → exit 2, `hive.update-failed`, "installed ownership manifest is not bound to the running Hive product: codex", `steps.core=not-started`. `--channel test`도 같음
- 같은 상태의 `hive install --scope user --hosts codex,claude,antigravity --dry-run --output json` → `hive.user-install-multi-host-dry-run-complete`(이전 투영 인증 성공)

## 원인

- `crates/hive-cli/src/update_activation.rs`의 갱신 흐름이 먼저 `user_install::authenticated_saved_projection_hosts`를 부르고, `authenticate_saved_projection_host`가 `manifest.product_version != CARGO_PKG_VERSION`이면 바로 거부
- 설치 경로(`validate_prior_ownership`)는 `authenticated_user_inventory`로 이전 공개 버전의 투영을 인증하므로, 두 경로의 인증 기준이 다름
- 결과: 안내된 갱신 순서(npm으로 새 버전 설치 → `hive update`)가 이전 버전 투영이 있는 모든 PC에서 실패

## 범위(0.15.3 필수)

- C1 갱신 흐름의 호스트 인증이 이전 버전 투영을 설치 경로와 같은 기준(`authenticated_user_inventory`·바이트 검증)으로 인증. 위조·변조·해석 불가 기록은 계속 실패
- C2 인증된 이전 버전 호스트는 갱신의 투영 재적용 대상에 포함
- C3 시험: 이전 공개 버전 투영이 있는 사용자 루트에서 인증 성공, 변조 시 실패 유지
- C4 local 확인: 정식 0.15.3 설치 뒤 `hive update` 성공
