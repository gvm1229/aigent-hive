# 현재 상태

- 제품 버전: `0.11.1`
- 공개 안정판: `0.11.0`
- 작업 브랜치: `develop`
- 공개 시험 수용: `0.11.1-test.6`
- 다음 시험 대상: `0.11.1-test.7`
- 정본: [활성 계획](../plans/PLAN.md), [스킬 제공 계획](../plans/active/skill-delivery-0.11.1.md), [Claude 설치 계획](../plans/active/claude-user-install-0.11.1.md)
- 이전 완료와 검증: [0.11.0 상태](../archive/state/0.11.0-before-skill-delivery.md)

## 생성된 현재 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.11.1`
- 현재 등록 항목: 18/23 완료

- `agent-owned`: `SGM-004`, `TCL-001`, `TCL-002`, `TCL-003`
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: `DPS-003`
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

## 현재 근거

- 사용자 의도 확인: 기본 스킬 제외 대신 새 Hive 개선과 사용자 수정의 결합; [결합 계획](../plans/active/skill-merge-0.11.1.md) 구현, 새 제품 변경의 test.7 검증 필요

- 사용자 재시작·GUI 목록 확인과 두 새 실제 Codex 대화의 명시/자연어 `user-setup` 호출·`hive.user-setup-valid` 확인 완료; 전역 호출 수용 SDP-005 완료, [검토 근거](../research/wproject-custom-ship-0.11.1.md#전역-스킬-검증)
- WProject의 사용자 관리 `ship`에 프로젝트 전용 검사 존재·Hive 기준 목록 부재, 공식 scan 충돌 재현과 파일 보존; 이동·삭제 제외
- 사용자 `ship` 유지·Hive 기본 `ship` 제외의 22개 선택 질문 제출, 공식 구버전 재설정 미리보기도 기존 목록 소유권 제약으로 중단; 프로젝트 적용과 제품 수정 경로는 선택 확인 뒤 검토

- test.6의 필수 CI 36963278427·후보 36963962342·복구 게시 36965966512·세 운영체제 수용 36966439849 완료; [공개 수용 정본](../research/skill-delivery-public-test-0.11.1.md)의 정확한 지문·실행 범위 확인
- 공개 test.6의 실제 Windows Claude 2.1.163 세 경로와 Codex 0.159.0 격리 CLI 검증 통과; GUI·명시/자연어 모델 호출 미증명, 모델 호출 0건
- 현재 사용자 test.6 설치 승인·적용 완료: 공개 실행 파일 지문 일치, 지식 491개·설정 2개·외부 지침 보존, 소비자 프로젝트 변경 0건; [설치 근거](../../tests/results/legacy/7d159c3524fa28e6fde6.md)
- 현재 사용자 새 실제 Codex 관리 서버: Hive 스킬 28개·중복/오류 0개·모델 호출 0건; [발견 근거](../../tests/results/legacy/f41c995a82c3db97f0a4.md), 이후 사용자 새 대화의 전역 명시/자연어 호출 확인 완료

- 사용자 승인 추가 범위: [지침 이식·프로젝트 스킬](../plans/active/directive-localization-and-project-skills-0.11.1.md); 지침 이식·기본 23개·자유 선택·기존 기본 전환 구현과 관련 검증 완료, [검증 근거](../research/directive-localization-and-project-skills-0.11.1.md); 자연어 호출 정책 구현 완료·실제 대화 호출 대기
- 추가 범위 최종 Windows 검사: Rust 967개 통과·4개 조건 제외, Python 917개 통과·44개 조건 제외, Rust 1.99 Clippy 통과; 실제 Claude CLI 2.1.163 세 경로 설치·검증·재설치·갱신과 사용자 자료 보존 통과, 모델 호출 0건

- CI 36904470136의 macOS 백업 이름 충돌·Linux Rust 1.99 검사 호환 수정 완료, [수정 계획](../plans/active/release-qualification-repairs-0.11.1.md) 적용
- test.2–4 후보 미게시; 전체 필수 CI 36923839882 통과 뒤 test.5 후보 36940738218·복구 게시 36942587351·세 OS 수용 36943131885 성공

- 0.11.1 정식 출시 승인, [실행 계획](../plans/0.11.1-stable-release.md)의 공개 시험·현재 사용자 설치 완료; Codex 재시작 뒤 실제 대화 호출 증거 대기
- 반복 지연 방지: 전체 실패군·도구 버전 확인, 필수 CI 통과 뒤 후보 생성, 입력 변화 없는 재시도 제외, 15분 정체 시 작업 상태 진단
- [0.11.1 구독자 공지 초안](../releases/0.11.1.subscriber.ko.md) 문구·기존 Discord 채널 전송 승인, 정식 게시 뒤 발송 예정

- [Claude Windows 설치 검증](../research/claude-user-install-0.11.1.md): 실제 격리 CLI 세 경로 통과, 전체 Rust 964개 통과·4개 제외, 전체 Python 917개 통과·44개 조건 제외

- 0.11.0 Git 태그의 정확한 프로젝트·사용자 원본 보존
- [로컬 전체 검증](../research/skill-delivery-0.11.1.md): Windows Rust 959개 통과·4개 제외, Python 927개 통과·33개 운영체제 조건 제외
- [공개 수용](../research/skill-delivery-public-test-0.11.1.md): 실제 test.5의 Windows x64·Linux musl x64·macOS arm64 네 조건·지침·한국어·검색 수용 통과, Codex 등록 응답은 모의
- 승인된 현재 사용자 설치·설정 검증 통과: 공개 test.5 실행 파일 지문 일치, 지식 일반 파일 489개·기존 설정 2개·Hive 표시 블록 밖 지침 보존, 소비자 프로젝트 변경 0건
- 현재 사용자 새 실제 Codex 0.159.0 서버: Hive 스킬 28개·중복 0개·오류 0개, 모델 호출 0건; GUI 선택·명시 모델 호출·자연어 모델 호출 미증명
- Claude CUI-001–004·실제 Windows 공개 CLI 수용 완료; 안정판 latest 0.11.0 유지, 현재 사용자 CLI test.5 적용

## 승인 인계

- 현재 사용자 test.5 설치 승인·적용 완료, test.6 설치 승인·적용 완료; 안정판 게시·main 통합·881자 Discord 문구·발송 승인 유지
- `SDP-005` 사용자 실행·검토 완료; `DPS-003`의 프로젝트 선택 스킬 활용과 사용자 관리 스킬 충돌 처리 구분
- 기존 대화의 0.11.0 캐시 경로를 담은 스킬 목록으로 새 0.11.1 호출 성공 판정 제외; host 프로세스 신호·자격 증명 전달·모델 제공자 API를 통한 우회 금지
- 두 호출은 설정 변경 없는 설치 검증으로 제한, 소비자 프로젝트 변경 제외; 실제 호출 증거 확보 뒤 SDP-005 완료와 보호 main 통합·안정판 후보·게시·Discord 전송 진행
