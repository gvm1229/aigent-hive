# 세션 중단·알림 전용 0.13.0 출시

> Plan version: 0.13.0
> Scope: release

- [x] [REL121-001] 폐기한 재개 차단의 역사 코드·재현·복구 문서
  - state: complete; evidence: repo:docs/research/codex-resume-lock-reference.md#sha256:bb99293eb9bb3a588f91783c04eb2c8eb63479097802ac87a3d548fb9a79c87b
- [ ] [REL121-002] 세션 중단·알림만의 제품 범위와 출시 자료 검증
  - state: agent-owned; owner: Codex; reason: 필수 이전 버전 자료와 출시 입력 준비
- [ ] [REL121-003] 공개 0.13.0-test.1 게시·정확 산출물 수용
  - state: agent-owned; depends: REL121-002; owner: Codex; reason: 공개 후보·게시·독립 수용 대기
- [ ] [REL121-004] 구독 안내 문구 승인·안정판 문서·보호 main 통합
  - state: agent-owned; depends: REL121-003; owner: Codex; reason: 문구 승인 요청과 독립 통합 준비 진행
- [ ] [REL121-005] 0.13.0 안정판 게시·독립 공개 조회·출시 기록
  - state: agent-owned; depends: REL121-004; owner: Codex; reason: 수용·통합 뒤 승인된 안정판 게시

## 목적·승인·제외

- 2026-10-18 최종 사용자 선택: 기존 버전 규칙에 따라 출시 번호0.13.0,0.12.1 한정 예외 미적용. 초기 요청: 과거 재개 차단의 재현 문서, 세션 중단·알림 창만의 0.13.0 안정판 출시. main 통합·안정판 태그·게시 승인
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

- 추가 확정: Cargo workspace와 Hive 패키지6개·현재 plugin2개·설정 template2개·release contract/fixture/ledger를0.13.0으로 변경. 외부 Cargo 의존성의0.12.1·역사 기준본·실측 원문은 보존. 이전 공개0.12.0과의 additive feature 분류 유지, 버전 정책 코드 변경 없음

- 입력: v0.12.0..HEAD 제품 경로 비교. `main.rs` 전역 도움말의 폐기한 acknowledge 표시만 정리
- 필수 조사: `check-release-version.sh`·`check-project-base-coverage.py`·기존 valid-0.12.0 fixture·harness/release/0.12.0. v0.12.0의 이전 버전 보존 자료와 0.13.0 migration·fixture 생성 방식 확인 후 동일 방식 적용. 새 사용자 기능 추가 없음
- 변경: 예약한 0.12.0 프로젝트·사용자 기준본과 Skill·출시 표면 목록, registry, 0.13.0 release/fixture, 공개 노트·시험 의도. 기준본은 v0.12.0 원문 바이트 보존
- 조사 확정: 0.12.0과0.11.1 프로젝트 파일 집합 동일. hive-render/src/lib.rs의 frozen_project_base_0_11_1 매크로·부속 파일 처리를0.12.0으로 기계적 복제, registry의 새 renderer 연결. 안정판 Skill 검사 표본에도 이미 공개한0.12.0 추가
- 회귀에서 확인한 추가 필수 연결: hive-projection/src/lib.rs의 historical_builtin_skills 지원 목록·예상 개수에0.12.0/28 등록. 기존 정확 목록 검사 유지, 미등록 버전 허용 확대 없음
- 첫 원격 CI의 시작 전 거부: 새 검사4개가 lanes.toml에 미등록. developer_digest·monitor_qualification·session_events·usage_notice를 기존 contract 묶음에 등록, 전체 파일의 한 차례 배정 검사 유지. 제품 지문 변경 없음
- 이전 버전 실제 표본: 공개 npm0.12.0의 무결성 검증한 Windows 실행 파일을 격리 폴더에만 추출, 기존 합성 설정으로 setup 수행. tests/fixtures/project-predecessors/0.12.0/public-stable에 개인 경로 없는 고정 ZIP·원본 지문 보존. 현재 설치 변경 없이 기존 qualify-project-predecessors.py의 갱신·원복·복구 검사에 사용
- 검증: 버전·기준본 범위·안정판 Skill·출시 노트·제품 범위 검사, 필요한 Rust/설치 회귀. 실패 시 제품 수정은 첫 공개 시험 전에 완료
- 완료: 현재 제품 커밋 지문과 완료 RST121 3개를 test-release-intent에 고정, 로컬 검사·정확 소스 CI 통과

### REL121-003 — 공개 시험과 수용

- 의존: REL121-002 통과. develop의 Release candidate에 product0.13.0/package0.13.0-test.1/channeltest/date2026-10-18
- 후보 성공 뒤 별도 publication. 여섯 npm 정확 버전·test 태그·GitHub prerelease·latest0.12.0 보존 독립 조회
- Public test acceptance: 정확 공개 산출물의 Windows x64·macOS arm64·Linux musl x64 설치·갱신·복구·새 세션·보존·성능. 현재 Windows는 읽기 전용 감지·알림 표시와 기존 실제 중단 근거 결합, 새 전역 중단 없음
- 기록: 후보·게시·수용 run ID, source SHA, 제품/산출물 지문, OS·실제 실행·미증명 범위를 research/results·accepted registry에 저장
- 실패: 후보 실패와 게시 성공 구분. 제품·설치·메타데이터·수용 수정이면 다음 허용 시험 번호로 계획 갱신 후 재검증

### REL121-004 — 안정판 입력과 보호 통합

- 의존: 시험 수용 완료. 정확 구독 안내 승인 뒤 sha256 sidecar·register-stable-summary-approval.py 실행, 승인 전 발송 없음
- README5개·public-stable-release·CURRENT·PLAN을 수용한 범위에 맞춤. 기존 docs/01-index.md 사용자 변경은 해당 변경과 분리한 부분만 반영
- develop push·CI, develop→main PR·필수 Protected merge gate의 일반 병합. 보호 우회 없음
- 완료: 정확 main SHA·CI 성공, 시험 대비 제품 지문 동일, 미완료 구현 0건. 출시 단계 자체의 미래 게시 항목과 구현 결손 구분

### REL121-005 — 안정판 게시와 종료

- main의 Release candidate: product/package0.13.0,stable,date2026-10-18,promoted_test_version·acceptance_run_id 고정
- 성공 후보로 publication, 승인한 구독 공지 전송 true. 여섯 npm latest0.13.0·GitHub 정식 v0.13.0·소스·지문·시험 태그 유지 독립 확인
- 기록·Source Wiki·문서 검사·목적별 커밋·원격 반영. 생성 시험 자료는 소유권·보존 정책에 따라 정리, 다른 작업 변경 보존
- 부분 게시: 기존 후보 지문 대조 복구만, 공개 버전 덮어쓰기 금지. 필수 외부 승인/권한만 해당 단계 대기, 독립 작업 지속

## 완료 기준

- 역사 잠금 문서·최소 범위 공개 시험 수용·보호 main 통합·0.13.0 안정판 독립 공개 확인·승인 공지 전달 기록
- 현재 앱 유지, 재개 잠금 없음, 자동 재개 없음. 실제 제공자 초기화·서버 과금 종료·다른 OS의 세션 중단 보장 없음
