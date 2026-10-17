# 세션 중단·알림 전용 0.12.1 출시

> Plan version: 0.12.1
> Scope: release

- [ ] [REL121-001] 폐기한 재개 차단의 역사 코드·재현·복구 문서
- [ ] [REL121-002] 세션 중단·알림만의 제품 범위와 출시 자료 검증
- [ ] [REL121-003] 공개 0.12.1-test.1 게시·정확 산출물 수용
- [ ] [REL121-004] 구독 안내 문구 승인·안정판 문서·보호 main 통합
- [ ] [REL121-005] 0.12.1 안정판 게시·독립 공개 조회·출시 기록

## 목적·승인·제외

- 2026-10-18 사용자 요청: 과거 재개 차단의 재현 문서, 세션 중단·알림 창만의 0.12.1 안정판 출시. main 통합·안정판 태그·게시 승인
- 기존 [RST121](reset-stop-0.12.1.md) 완료 3개 유지. 다른 예정 기능·재개 잠금·앱 종료 제외. 추가 실제 세션 중단 시험 금지
- Discord 정확 문구는 `.agents/skills/update-summary/SKILL.md`에 따라 별도 승인 필요. 승인 전 초안 저장·독립 출시 준비 지속, 전송·승인 지문 대리 생성 금지
- develop 사용, 다른 작업의 미커밋 파일 보존. 대상 Windows 11 x64·Codex 26.1013.4894.0. 다른 OS의 설치 검증과 실제 세션 중단 지원 구분

## 실행 순서

### REL121-001 — 역사 문서

- 읽기: cd0f8ac6의 usage_monitor.rs·state.rs·desktop.ps1, ae5455ff·b0c1d256의 제거 변경, 기존 실제 중단 근거
- 변경: `docs/research/codex-resume-lock-reference.md`. 사건 유지·1초 재중단·시작 전 차단의 결합, 정확한 명령·상태·해제 절차·격리 조건·검증 한계 기록
- 재현은 미래의 별도 승인 실험 절차만 문서화. 현재 감시기·사용자 루트·실제 채팅 변경 없음. 과거 잠금 코드로 되돌리기 금지
- 완료: 근거 커밋·명령과 해제 전제 일치, Markdown 링크·문체 검사

### REL121-002 — 제품 범위와 출시 입력

- 입력: v0.12.0..HEAD 제품 경로 비교. `main.rs` 전역 도움말의 폐기한 acknowledge 표시만 정리
- 필수 조사: `check-release-version.sh`·`check-project-base-coverage.py`·기존 valid-0.12.0 fixture·harness/release/0.12.0. v0.12.0의 이전 버전 보존 자료와 0.12.1 migration·fixture 생성 방식 확인 후 동일 방식 적용. 새 사용자 기능 추가 없음
- 변경: 예약한 0.12.0 프로젝트·사용자 기준본, registry, 0.12.1 release/fixture, 공개 노트·시험 의도. 기준본은 v0.12.0 원문 바이트 보존
- 검증: 버전·기준본 범위·안정판 Skill·출시 노트·제품 범위 검사, 필요한 Rust/설치 회귀. 실패 시 제품 수정은 첫 공개 시험 전에 완료
- 완료: 현재 제품 커밋 지문과 완료 RST121 3개를 test-release-intent에 고정, 로컬 검사·정확 소스 CI 통과

### REL121-003 — 공개 시험과 수용

- 의존: REL121-002 통과. develop의 Release candidate에 product0.12.1/package0.12.1-test.1/channeltest/date2026-10-18
- 후보 성공 뒤 별도 publication. 여섯 npm 정확 버전·test 태그·GitHub prerelease·latest0.12.0 보존 독립 조회
- Public test acceptance: 정확 공개 산출물의 Windows x64·macOS arm64·Linux musl x64 설치·갱신·복구·새 세션·보존·성능. 현재 Windows는 읽기 전용 감지·알림 표시와 기존 실제 중단 근거 결합, 새 전역 중단 없음
- 기록: 후보·게시·수용 run ID, source SHA, 제품/산출물 지문, OS·실제 실행·미증명 범위를 research/results·accepted registry에 저장
- 실패: 후보를 게시 성공으로 표시하지 않음. 제품·설치·메타데이터·수용 수정이면 다음 허용 시험 번호로 계획 갱신 후 재검증

### REL121-004 — 안정판 입력과 보호 통합

- 의존: 시험 수용 완료. 정확 구독 안내 승인 뒤 sha256 sidecar·register-stable-summary-approval.py 실행, 승인 전 발송 없음
- README5개·public-stable-release·CURRENT·PLAN을 수용한 범위에 맞춤. 기존 docs/01-index.md 사용자 변경은 해당 변경과 분리한 부분만 반영
- develop push·CI, develop→main PR·필수 Protected merge gate의 일반 병합. 보호 우회 없음
- 완료: 정확 main SHA·CI 성공, 시험 대비 제품 지문 동일, 미완료 구현 0건. 출시 단계 자체의 미래 게시 항목은 구현 결손으로 해석하지 않음

### REL121-005 — 안정판 게시와 종료

- main의 Release candidate: product/package0.12.1,stable,date2026-10-18,promoted_test_version·acceptance_run_id 고정
- 성공 후보로 publication, 승인한 구독 공지 전송 true. 여섯 npm latest0.12.1·GitHub 정식 v0.12.1·소스·지문·시험 태그 유지 독립 확인
- 기록·Source Wiki·문서 검사·목적별 커밋·원격 반영. 생성 시험 자료는 소유권·보존 정책에 따라 정리, 다른 작업 변경 보존
- 부분 게시: 기존 후보 지문 대조 복구만, 공개 버전 덮어쓰기 금지. 필수 외부 승인/권한만 해당 단계 대기, 독립 작업은 계속

## 완료 기준

- 역사 잠금 문서·최소 범위 공개 시험 수용·보호 main 통합·0.12.1 안정판 독립 공개 확인·승인 공지 전달 기록
- 현재 앱 유지, 재개 잠금 없음, 자동 재개 없음. 실제 제공자 초기화·서버 과금 종료·다른 OS의 세션 중단 보장 없음
