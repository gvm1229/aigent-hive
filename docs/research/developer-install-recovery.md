# 개발 설치의 데이터 보존 갱신

## 원인과 수정 범위

- 2026-10-18 Windows 11 x64·Codex CLI0.160.1에서 실제 실행. 공개 시험판과 다른0.12.1-dev 전역 교체 뒤 npm 기록0.12.0·실행 파일0.12.1·사용자 설치 기록0.12.1의 불일치
- 개발 배포 절차의 실행 파일 교체와 사용자 투영 적용 사이에 공개판 복귀 검증 부재. 기존 공개 시험판→정식판의 일반 지원과 구분
- `scripts/dev-install.sh --global`은 빌드 전 거부. 과거 `complete-usage-monitor-install.ps1`도 원복·설치·감시 변경 전 거부. 기본 개발 빌드는 별도 실행 파일과 시험 사용자 루트 사용
- 소스 전용 절차 수정. 이미 공개한0.13.0 패키지·버전·태그 변경과 새 출시 없음

## 무삭제 복구 절차

1. 설치된 개발 실행 파일의 관리 지문과 `install --scope user --host codex --validate` 성공 확인. 사용자 지식·설정·공유 지침 바깥 영역·실행 기록 목록·Codex 설정·프로세스의 사전 비교값 보존
2. `scripts/recover-dev-install.py plan`으로 생성된 플러그인 파일·목록 파일·호스트 소유 기록만 선택. 정확한 계획 지문을 `stage --confirm`으로 전달, 외부 백업 뒤 해당 생성 파일만 비우는 처리. 공유 지침·지식·설정·실행 기록은 대상 제외
3. 기존 관리된 `dev-install.sh --rollback`으로 이전 공식 실행 파일 복원. Codex가 비워진 목록을 읽지 않도록 공식 `codex plugin marketplace remove aigent-hive --json`으로 해당 등록만 잠시 해제. 다른 플러그인 등록 변경 없음
4. 공식 CLI의 `install --scope user --host codex --dry-run`이 일반 설치 계획인 경우에만 적용·검증. `preserving-reinstall` 결과는 적용 금지. 같은 저장소 위치의 Hive 등록 재생성
5. 일반 `hive update`로 공개 패키지 갱신. 이전 공식 실행 파일은 복구 완료까지 보관. 새 실행 파일이 이전 사용자 투영을 거부하면, 보관한 원본의 검증된 SHA-256을 `--validator-sha256`으로 고정하고 그 원본의 실제 설치 검증을 거쳐2·3·4단계 반복
6. 새 공식 CLI의 설치 검증과 후속 `hive update` 성공, 데이터·설정·등록 보존 대조. 백업은 검토·복구 기간 동안 보관

- `--validator-sha256`은 이전에 인증한 원본의 고정값만 사용. 임의 실행 파일에서 방금 계산한 지문을 공개 출처 인증으로 대체 금지
- 백업 기존 경로·변조·경로 탈출·연결 경로·허용 밖 소유 유형 거부. 중간 파일 처리 실패 시 복원, 새 파일이 차지한 복원 위치 덮어쓰기 금지
- `restore --backup`은 생성 파일 복원만 수행. 공식 CLI로 재생성한 파일이 이미 있으면 충돌로 중단. 호스트 등록·실행 파일의 상태까지 복원 완료로 표시 금지
- `hive uninstall`·실행 기록 정리·사용자 설정 초기화·Codex 종료·세션 중단 요청 미실행

## 실제 확인

| 항목 | 결과·증명 범위 |
| --- | --- |
| 최초 개발 상태 검증 | 실제 개발 CLI 설치 검증 성공, 생성 파일·소유 기록74개 백업 |
|0.12.0 공식 기준 복구 | 일반 설치·검증 성공. 첫 미리보기의 목록 파일 부재 거부 뒤 Hive 등록만 공식 해제하여 해결 |
| 일반 패키지 갱신 | npm 소유 updater의0.12.0→0.13.0 실행 파일 설치 성공. 사용자 투영 갱신은 이전 소유 인증에서 거부, 이 중간 실행 전체를 성공으로 표시하지 않는 경계 |
|0.13.0 투영 복구 | 보관한 공식0.12.0 원본으로 이전 투영 검증 뒤 생성 파일만 백업·재생성.0.13.0 설치 검증 성공 |
| 최종 사용자 명령 | 실제 Windows 대화형 터미널의 인자 없는 `hive update` 종료0, 최신 상태와 Codex 투영 갱신·검증 확인 |
| 데이터·설정 | 비교578개 중577개 바이트 동일. 자동 생성 `user-active-skills.yml`은28개 Skill의 `content_digest`만 변경, 이름·선택·권한 등 나머지 값 동일 |
| 기존 기록·호스트 | 기존 실행 기록·백업 파일 누락0개, 공유 지침의 Hive 바깥 바이트 동일, Codex 설정 전체 값 동일, 사전 관측 Codex 프로세스3개와 생성 시각 유지 |
| 관련 회귀 | Windows에서61개 실행,60개 통과·1개 건너뜀. 실제 연결 파일 생성 권한 부재로 해당1개 제외, Windows 연결 지점 속성 모의 거부는 통과 |

- 보관한 공식0.12.0 원본 SHA-256: `bb9eb889dca71daa241530a966623beda127b75d4233df955f9af6b85e6b6ef8`
- 회귀 명령: `python -m unittest tests.conformance.contracts.test_dev_install_recovery tests.conformance.documentation.test_dev_install_script tests.conformance.contracts.test_developer_digest tests.conformance.contracts.test_usage_monitor tests.conformance.contracts.test_monitor_qualification -q`
- 실제 사용자 갱신은 Windows 범위. Linux·macOS 복구와 다른 설치 소유자는 미실행·미지원, 새 공개판→미래판의 전환 실측 제외

## 별도 관측과 후속 후보

- 기존 감시 설정·고정0.12.1 실행 파일·로그인 예약 보존. 갱신 전부터 정상 관측 공백과 제어 연결 부재, 예약 작업 미실행 상태. 이번 갱신으로 감시 정상 작동까지 복구했다는 주장 제외
-0.13.0 `usage-guard`에는 감시 해제 명령과 지침 존재. 사용자의 자연어 요청을 받은 Codex가 대신 실행 가능, PowerShell 직접 입력 필수 아님
- 후속 검증 후보: 자연어의 전역 중간 중단 감시 해제와 현재 세션 하한 보호 해제 구분, 비프로젝트 폴더에서도 올바른 감시 제어 안내. 임의 새 버전 배정·기능 미구현 단정 제외
