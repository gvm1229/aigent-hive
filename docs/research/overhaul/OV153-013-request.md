# OV153-013 요청 — 변경 0건인데 프로젝트 갱신 검증 실패

- 기준일: 2026-11-17
- 환경: Aigent Hive 0.15.2, Windows 11, 대화형 터미널 빠른 설정 직후의 새 프로젝트. 사용자 수정 없음

## 보고 요약

- `hive project upgrade --target <project> --user-root <user-root> --validate --output json` → `verification-failed`, exit 5, `hive.update-release-verification-failed`, "installed project harness has applicable upgrades"
- `hive project upgrade --target <project> --user-root <user-root> --scan --output json` → `success`, `hive.project-upgrade-available`, `source_version` 0.15.2 → `target_version` 0.15.2, 보고 117개 전부 `disposition: unchanged`
- 기대: 바꿀 파일 0개면 `--scan`은 `hive.project-upgrade-current`, `--validate`는 성공. 파일 외 차이(기록·메타데이터)가 있다면 그 차이를 보고에 표시

## 진단 단서(local 코드 읽기, 미실행)

- `crates/hive-cli/src/project_upgrade.rs`의 `--validate` 실패 조건과 `plan_result`의 코드 선택 모두 `plan.changed_paths` 비어 있음 여부로 판단
- 보고 117개 전부 `unchanged`, `changed_paths`는 비어 있지 않은 상태. 후보 원인: 보고 목록 밖 경로(관리 목록·설정 기록·플러그인 파일 등)의 혼입, 또는 내용이 같은 파일의 변경 집계. 어느 쪽인지 미확인

## 범위(0.15.3 필수)

- B1 원인 확인: 새로 설정한 프로젝트에서 `changed_paths`에 들어가는 경로와 이유 특정
- B2 내용이 같은 경로는 변경에서 제외. 실제 변경이 0건이면 `--scan`은 `hive.project-upgrade-current`, `--validate`는 성공
- B3 파일 외 차이가 실제로 있으면 `changed_paths`의 모든 경로를 사유와 함께 보고 목록에 표시. 보고 없는 변경 경로는 오류로 처리
- B4 시험: 터미널 빠른 설정 직후 `--scan`·`--validate`, 대화형과 에이전트 적용 모두. 보고·`changed_paths` 일치 시험
- B5 local 확인: test.2에서 새 프로젝트 설정 직후 두 명령 재현
