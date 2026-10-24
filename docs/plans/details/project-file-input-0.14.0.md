# M03 프로젝트 답안 파일의 공통 적용·복구

- 진입: M00 격리 도구·기존 프로젝트 `activation.rs` 처리 기록 확인. 기존 전역 설정 필수 조건 유지, 생성형 모델 호출 없음
- 소유: `main.rs::run_setup/parse_setup`, `user_setup.rs` 연결, `setup_engine/{mod.rs,project.rs,project/file_input.rs}`, 인접 시험
- 인터페이스: 기존 `--answers --capabilities --dry-run|--apply|--validate --target --user-root --output json`, 반복 `--reconfigure-role` 유지. 새 `--input-channel answers-file|agent`, 파일 입력의 질문 대기 없음

## 순서와 결정

1. 기존 파일 구문 분석과 성공/오류 코드 유지. 파일 처리만 새 프로젝트 엔진으로 위임
2. renderer의 정규화·스키마 검사 재사용. v1/v2 입력, 무시하던 `orchestration_layer`의 기존 호환 유지. 파일의 출처 주장으로 터미널 답변 승격 금지
3. 새 적용은 v2와 실제 입력 경로 기록. 제공 값은 명시 입력, 자동 보충은 기본값, 빠른 설정의 전역 참조는 기존 상속 선언. 변경 없는 신뢰 가능한 기존 출처·시각 보존. 생략한 프로젝트 식별자는 기존 프로젝트에서 유지
4. 프로젝트의 공통 준비·변경 미리보기·`project_context`·`projection_journal`·등록/검증·완료 기록 사용. renderer 오류 분류를 보존하는 준비 함수와 적용 확정 함수를 터미널/파일에서 공유
5. 파일 dry-run/validate는 진행·선호 쓰기 없음. 적용은 전역 잠금 아래 지문 재검사 후 기록 선행. 입력·기능 자료·역할 승인 범위가 다른 미완료 요청은 덮어쓰기 대신 충돌
6. 같은 입력의 미완료 재실행은 저장한 답안·시각·승인과 공통 복구 사용. `--resume` 단독 경로도 유지, 원래 입력 파일 부재가 이미 확인한 복구를 막는 조건으로 확대 금지

## 검증과 종료

- 완료 커밋: `892035fa`. Windows/Linux 프로젝트 각23·보존 병합3·Windows 전체 대상 clippy 통과. [Linux 일반 적용](../../../tests/results/runs/20261024T231334-b93c02708465.md)과 [입력 파일 삭제 뒤 복구](../../../tests/results/runs/20261024T231557-ad7f6787abad.md) 통과
- 격리 범위: Windows 임시 파일·콜백 단위 검사, Windows에서 실행한 Linux 컨테이너의 실제 PTY·CLI. 가짜 설치/앱, 임시 사용자/프로젝트, 실제 설치 폴더 미연결. 공개 0.14.0·실제 앱·macOS 전체 수용은 미실행

- 프로젝트 기존 회귀·구문 분석·파일 입력 출처/상속·비밀/범위·사용자 원문·역할 변경·동시 수정·같은 입력 재실행
- 격리 CLI에서 schema1 파일→v2 적용·validate·재적용, 종료 후 같은 프로젝트 재개. actual 0.13.1 설치·앱·사용자 설정 변경 없음
- 두 언어의 터미널 설정과 파일의 최종 의미 일치. 실행 OS·소스/파일 지문·증명하지 못한 범위 기록, 근거 뒤 ship. 전체 S140-003과 공개 후보 수용은 별도
