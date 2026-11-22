# 0.16.0 재개 계획 — 두 번째 자료 손실과 사용량 표시 결함

- 기준일: 2026-11-23, 실행 장소: local 세션(Windows 11)
- 권한: 유지관리자 2026-11-23 원문 "Disable the usage guard for this session only while proceeding. You are authorized to do so if needed. Continue autonomously without any approvals until the stable version of 0.16.0 is released."
- 권한 범위: 현재 세션의 사용량 보호 끄기, 0.16.0의 구현·번호 시험·이 PC 설치·main 통합·정식 태그·게시. 전역 기준·다른 세션·다음 버전은 범위 밖
- 유지 조건: 정식 전 인계 1–4단계 통과, Codex 확인은 한도 회복 전 비차단, Codex 위임은 호환성 확인만
- `UI` = `crates/hive-cli/src/user_install.rs`

## 사건 기록

### 2026-11-21 20:02 KST 두 번째 자료 손실

- 관찰: 이 PC의 npm 패키지와 세 호스트 사용자 설치가 0.16.0-test.3 → 0.15.3으로 교체. 실행 주체 미확인(해당 시각 npm 기록 부재)
- 손실: 프로젝트 등록 16개, `~/.hive/backups/` 아래 수동 백업과 이전 설치 백업, 교체 전 실행 상태
- 원인: 0.15.3의 보존 재설치가 `.hive/backups`·`.hive/runtime`·`projects.yml` 삭제. 조율 세션이 수동 백업을 같은 삭제 범위 안에 저장하고 그 위치를 지침 05에 기록한 잘못
- 조치: `~/hive-backups/`에 파일 1,051개 사본과 바이트 대조, 등록 목록 16개 재구성, 지침·지식의 백업 위치 정정
- 증명 범위 밖: 재구성 목록과 원본의 동일성, 다른 PC의 영향

### 2026-11-23 02:43 KST 중단 알림

- 관찰: 재개 전 확인에서 `hive.usage-unknown`, 남은 양 `null`, 측정 자료 0개, Discord 중단 알림 1건
- 원인: 0.15.3에 `hive usage statusline` 명령 부재, 0.15.3의 측정 없음 차단 규칙
- 3% 사용을 3% 남음으로 해석했는지: 해당 없음. 두 버전의 변환식 `100 - used_percentage`, 측정값 자체가 없는 상태

### 잘못된 완료 보고의 정정

- OV160-017: Discord·사람용 표시 완료 보고는 사실과 불일치. `discord.rs`는 test.3 이후 변경 0건
- OV160-023: test.4 비차단을 유지관리자 결정으로 적은 기록은 잘못. 조율 세션이 사용량을 아끼려고 내린 범위 축소

## OV160-024 사용량 표시와 안내(test.4 차단)

- 의존: 없음. 소유: Claude 조율 세션
- 불변: 저장·비교는 남은 % 유지. Codex·Antigravity 표시는 남은 %
- D1 `discord.rs` `notification_field_line`: Claude 측정값은 "사용한 양: N%", 그 밖 호스트는 "남은 사용량: N%" 표시. `usage_monitor/notice.ps1`은 Codex 전용 안내라 변경 없음
- D2 `discord.rs` `resume`: `remaining_percent`가 없으면 사용량 회복 안내 대신 측정 확인 안내와 `hive usage status`. 측정값이 있으면 기존 회복 안내
- D3 `usage_control.rs`: 멈춤 기준 자료의 키를 `native_threshold`로 변경. `notices/quota.rs`의 현재 측정값은 `native_value` 유지. 한 키의 두 의미 제거
- D4 `human_output.rs` `threshold_message`: "사용량 보호를 멈추도록" 문구를 "사용한 양이 N%에 이르면 작업 중단"으로 교체. 영어도 작업 중단 의미
- D5 `human_output/families.rs`: 설정 요약은 `primary_host`와 저장된 남은 기준으로 방향·값 계산. 기록된 적 없는 `native_value` 읽기 제거
- D7 중단 화면의 다음 단계: 한도·소진·초기화·측정 없음에 "입력을 고친 뒤 재실행" 안내 제거. 실제 실행 출력에서 추가 발견
- D6 출시 노트 COMPAT: 0.15.x로 내리기 전 `--claude-statusline revert`와 `~/.hive` 밖 백업 필요
- 시험: 상태 표시줄 입력 3% 사용 → 남은 값 97 저장·작업 허용·표시 3% 사용, 97% 사용 → 남은 값 3 저장·중단·Discord 전체 문구 97% 사용, 남은 기준 4 → "96% 사용에서 작업 중단", 측정 없음 → 회복 안내 부재, 설정 요약의 세 호스트 방향
- 완료 판정: 값 계산 시험만으로 완료 금지. 실제 출력 문자열 시험과 이 PC 명령 출력 확인 필수

## OV160-023 Hive 밖 백업과 복원(test.4 차단)

- 의존: OV160-016. 소유: Claude 조율 세션
- 결정: 사용자 루트의 `hive-backups/<작업>-<시각>-<난수>/`에 백업. 이유: 0.15.3 이하로 내려도 이전 제품의 삭제 범위 밖. `.hive/backups`의 기존 설치 거래 사본은 그대로 유지
- B1 새 모듈 `crates/hive-cli/src/user_install/backup.rs`: 정확한 바이트 사본, 쓰기 뒤 재읽기 대조, 마지막에 `manifest.json`. 소유자 전용 권한, 링크·경로 탈출 거부, 자동 삭제 없음
- B2 시점: 설치·갱신 적용 전, 보존 재설치 제거 전, 사용자 제거의 첫 변경 전. 백업 실패 시 변경 0건
- B3 대상: 바뀌거나 지워질 파일 + `.hive/config`의 설정 파일 + 설치 기록 + `roster.toml` + 상태 표시줄 되돌리기 기록 + 네 호스트 안내·설정 파일. 재생성 가능한 색인·플러그인 사본 제외
- B4 `user_install/uninstall.rs`·`removal.rs`: `.hive/backups` 삭제 금지, 보존 목록에 표시
- B5 `hive storage backup list|restore`(기존 Hades 소유 명령 아래, 새 최상위 명령 없음): `--user-root`, 복원은 `--id`, 변경 목록과 지문 미리보기, 같은 지문 확인 뒤 적용. 덮어쓰기 전 현재 파일 재백업. 백업에 없는 현재 파일은 삭제 금지
- 시험: 백업 실패 시 변경 없음, 제거·재설치 뒤 바깥 사본과 `.hive/backups` 생존, 내용 변조 거부, 경로 탈출 거부, 확인 뒤 바이트 복원, 확인 뒤 바뀐 파일 거부
- 남은 범위 OV160-025: 채택 흐름, 등록 목록 재구성 명령. OV160-021과 함께 0.17.0 후보

## 검증과 출시 순서

1. 검토 수정 브랜치 시험·병합, 남은 검토 지적 2·3 처리
2. OV160-024·023 구현과 음성 시험
3. Rust 전체, Python 전체 적합성 시험, 문서·계획·출시 노트 검사. 실패는 develop 기준과 비교하고 범위·이유 기록
4. Fable 단계 검토. 구현자의 완료 보고가 아닌 실제 출력 문자열과 자료 보존을 검사
5. 0.16.0-test.4 게시와 세 OS 수용
6. 이 PC: `~/.hive` 밖 백업 → 설치·갱신 → 인계 1–4단계 → 설치 전후 자료 대조
7. 통과 시 `check-plan-state.py --release`, main 통합, 정식 게시, 이 PC 정식 설치와 재대조

## Fable 5.2 단계 검토(2026-11-23)

- 검토자: Fable 5.2 일반 하위 에이전트, 읽기 전용. 판정 **APPROVE WITH FIXES**
- 차단 1건: 재가동한 공개 갱신 수용 스크립트가 0.15.3에서 세 OS 실패 예정(0.13.1 질문·초기값 기대, macOS 신원 조회). 설정 형식별 기대로 정정, 실제 실행은 test.4 수용
- 권고 9건: 기본 복원 허용 목록, Skill의 Claude 사용 비율 규칙, 관측 실패 다음 단계, Discord 회복 문구, README 권한 문구, 도움말, 출시 노트의 백업 범위, 설정 사본 고지, 실패 시 사본 위치. 모두 반영
- 사소 2묶음: 목록 밖 안내·설치 기록 사본, 백업 위치 표시, 기준 문구, 검토자 자동 선택 조건 반영. 백업 폴더 권한은 한계로 기록
- 확인(Windows): `cargo clippy`·`cargo test --workspace --all-features` 통과, Rust 주 시험 1,060개
- Python 전체 1,235개(수정 전 `0ce94aa7`): 실패 1(동작 목록, 수정), 오류 9(`copier` 실행 경로 환경, 경로 지정 재실행 23개 통과), 건너뜀 45
- 미검증: macOS·Linux 실제 설치·갱신, 실제 Discord 전송. 전자는 test.4 수용 대상

## test.4 local 실패와 test.5

- 공개·수용: test.4 세 OS 수용 45065124070 통과. 시험 범위는 가짜 Antigravity 호스트만, 실제 Claude 설치 기록 밖
- 실패(Windows 11, 2026-11-23): 이 PC의 정상 0.15.3 Claude 설치에 `hive install --dry-run`이 보존 재설치 계획. Codex·Antigravity는 정상 인증
- 상태: 전역 CLI만 test.4, 사용자 설치·설정·자료는 0.15.3 그대로. 바깥 백업 파일 2,184개 대조 뒤 변경 0건
- 원인: `UI` `historical_user_artifact_required`가 Claude에도 `skills/*/agents/openai.yaml` 요구. 투영 생성기는 Codex·Antigravity에만 이 파일 생성
- 시험 공백: 고정물이 검사 대상 함수로 파일 목록을 생성해 같은 잘못을 재현
- 수정: Claude의 완전성 검사에서만 이 파일 제외. 존재하는 파일의 바이트 인증과 다른 필수 파일은 유지
- 시험: 고정물 목록을 투영 생성기로 대조, 선택 Skill의 파일 누락 거부 유지, 이 PC 0.15.3 사본의 적용·검증 재현
- 순서: 시험 → test.5 게시·세 OS 수용 → 이 PC 인계 1–4단계 → 정식판. test.4는 정식 승격 금지
