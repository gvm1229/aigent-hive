# Claude Windows 전역 설치 수정

> Plan version: 0.11.1
> Scope: product
> 다음 시험 대상: `0.11.1-test.2`

## 목적과 경계

- Claude 등록 명령에 일반 Windows 경로 전달, 오류 원인 표시와 반복 실패 안내 개선
- 실제 Hive 0.11.0·Claude 2.1.163 격리 재현: 파일 생성 후 등록 호출, 확장 경로 거절, 일반 경로 설치·검증·재설치·갱신 성공
- `develop`·제품 0.11.1 유지, 기존 Codex 수용 기준 보존
- 구현·검증 승인; 안정판 게시·보호 브랜치 통합·실제 사용자 설치는 별도 버전별 승인
- 사용자 로그의 Claude 버전·전체 도구 호출 부재; 원래 기기와 완전한 동일성 미확인

## 수용 기준

- [x] [CUI-001] Claude 명령 경로 변환과 파일 기록 순서 회귀 검증
  - state: complete; evidence: repo:tests/results/runs/20261001T165920-6c6e38bdc900.md#sha256:547d9ead3a4b44be28ffc43a44c69d396003d5efc878c7cca159d230aabac32d
- [x] [CUI-002] 제한된 오류 출력 수집과 비밀 비노출 진단
  - state: complete; evidence: repo:tests/results/runs/20261001T162740-a955b691eb61.md#sha256:42cc22a9c97ebf5544a7bbe0723787a9982a51de935e84df9f6a8e352fa48258
- [x] [CUI-003] 반복 실패·복구·질문 도구 안내와 배포 복사본 정합화
  - state: complete; evidence: repo:tests/results/runs/20261001T163513-356483af2a78.md#sha256:f1eb86406c07d6c8f4a28a9cb4b5d875e7b767d72ec1c5736cd9448f05951b7c
- [x] [CUI-004] 수정 실행 파일의 실제 Windows Claude CLI·전체 회귀·근거 기록
  - state: complete; evidence: repo:docs/research/claude-user-install-0.11.1.md#sha256:b36c334d80e7c0db2140b4fd7effa801e5752796d84876e3a2734f138a3664bd

## 구현 인계

1. CUI-001: `user_install::activate_host`에서 Claude 경로만 기존 `normalize_host_path`로 변환. 내부 정규 경로·접근 권한·파일 기록→검사→등록 유지. 복구 경로 확인. `StatefulHostRunner`·Python 설치 모의 명령에서 확장 경로 거절과 등록 직전 명세·플러그인 내용 검사. 일반·공백·한글 경로, 신규·재설치·갱신·검증 확인.
2. CUI-002: `usage::CommandOutput`에 `stderr: Vec<u8>`, `exit_code: Option<i32>` 추가. 기존 제한·시간 초과·실행 파일 검사 유지. `host_state::sanitized_command_diagnostic`에 종료 코드와 양쪽 출력 길이·해시 추가. 확인된 경로 거절만 고정 분류, 출력 원문 비노출. 공개 CLI·JSON 형식 유지, 복구 실패에 최초 원인 보존. stderr 전용 실패·비정상 종료·출력 제한·비밀 문자열 시험.
3. CUI-003: `user-setup` 복구·진행 참조에 동일 조건 재시도 금지, 실패 후 파일 부재의 원인 단정 금지, 근거 없는 관리자 실행 권유 금지. 인증된 복구 거절 시 기록 보존, 자동 제거 우회 금지. 질문 도구 규격 확인 후 1회 재구성, 계속 실패 시 일반 질문·기존 답변 보존. 현재 세 배포 복사본 동기화. `Invalid tool parameters` 원인 미확인 유지.
4. CUI-004: `scripts/qualify-claude-user-install.py`로 실제 Claude CLI·수정 Hive 격리 시험. HOME·USERPROFILE·CLAUDE_CONFIG_DIR·작업 폴더를 시험 폴더로 고정, 제공자 자격 증명 제거, 모델 호출 제외. 신규 설치→검증→재설치→갱신. 외부 지침·설정·지식 보존·실패 복구의 기존 시험 유지. 관련 Rust·Python→실제 CLI→전체 Rust·Python 순서, `scripts/test-artifacts.py run` 결과 기록. 근거·영한 사실·상태 갱신.

## 완료와 한계

- 수정 실행 파일의 실제 Windows CLI 성공과 관련·전체 회귀 통과가 완료 근거
- 실제 Claude 대화의 스킬 발견·질문·자연어 호출은 구독 사용자 근거 필요; CLI 시험의 증명 범위 밖
- UNC·긴 경로 전체 지원 확대 제외; 모호한 중단·외부 변경의 기존 복구 안전 조건 유지
- 제품 구현 검증 완료와 다음 시험판 게시·실제 사용자 설치 승인 분리
