# 단계 1b — 0.14.2 수정판

> Plan version: 0.14.2
> Scope: product

- 승인: 2026-11-10 사전 승인·2026-11-12 추가 단계 결정. 브랜치 `fix/overhaul-1-0.14.2`
- 목적: 실제 크기의 감시 실행 파일 정리. OV141-011의 미충족 조건은 정식 갱신에서 재검증
- 경계: 이번 실행은 OV142-004까지. Fable 검토는 orchestrator 소유. test.1은 develop 병합 뒤, 실제 사용자 설치 영역·기존 approval 작업 트리·외부 조사 파일 변경 금지
- 설계: `run.rs::remove_parent_file`의 격리 뒤 비교만 고정 크기 버퍼로 스트림 비교. 전체 예상 바이트와 길이 일치 필요. no-follow·동일 부모 격리·불일치 복구 유지. 일반 문서 읽기 한도와 게시 경로 변경 제외
- 안전 조건: 현재·직전·기록된 실행 파일의 보존 규칙 유지. Windows 사용 중 파일은 기존 운영체제 잠금에 따른 건너뜀 유지

## 항목

- [x] [OV142-001] 시작·계획 전환과 버전 동기화
  - state: complete; evidence: repo:docs/research/overhaul/OV142-001.md#sha256:a294cd84ecb59ec04915a1c55f5fa0ed1e0a8fc8ffce153693b782a4828305ba
  - 변경: 0.14.1 계획·상태 보관(OV141-011 미완료 유지), PLAN/CURRENT/재정비 표 전환. Cargo·템플릿·플러그인 0.14.2, 출시 자료 순번 27. 공개 v0.14.1의 이전 패치 원본 등록과 렌더러·소유권 목록 동기화. 개발용 상호작용 시험 6개의 버전과 공개판 목록 시험의 0.14.1 항목 동기화
  - 확인: `python scripts/check-plan-state.py`, `bash scripts/check-release-version.sh 0.14.2`, project-base coverage와 historical 회귀. 실패 시 해당 누락만 보완
- [x] [OV142-002] 큰 실행 파일의 정확 삭제 수정
  - state: complete; depends: OV142-001; evidence: repo:docs/research/overhaul/OV142-002.md#sha256:c447ef664d558ad3a4e8bb0adeb827ad1035e4d9874f8ac5c5849fa082282401
  - 변경: `crates/hive-cli/src/run.rs::remove_parent_file`에서 격리 파일을 no-follow로 열고 64 KiB씩 비교. 예상 길이 뒤 EOF 확인. 읽기 실패·다른 바이트는 기존 restore_claim_error로 복구
  - 확인: hive-cli run 시험과 strict Clippy. 변경·추가·짧은 파일·비정상 경로 보존 확인. 예상 스냅샷의 128 MiB 한도 유지
- [x] [OV142-003] 자동 정리·수동 정리 큰 파일 회귀
  - state: complete; depends: OV142-002; evidence: repo:docs/research/overhaul/OV142-003.md#sha256:d47f74746c332a0ef804eb943106ecd472ed490995c924e7303587dcfd545ae9
  - 변경: `usage_monitor/upgrade.rs`의 임시 사용자 bin에 1 MiB 초과 파일 추가, upgrade_with의 binary_cleanup 및 prune_unused의 preview/confirm 경로 확인. run.rs의 길이·꼬리 변조 복구 반례
  - 확인: `python scripts/dev-check.py rust -- test --workspace --locked -- --test-threads=4`, Python documentation·security·contract·integration 검사 묶음, fmt·Clippy. 공개 0.14.1의 누락된 이전판 시험 자료는 출처 증명·SHA-256 확인 뒤 임시 setup으로 생성. release 묶음은 안내문 파일이 필요한 OV142-004에서 실행. Windows Codex의 실제 실행만 완료 근거, macOS/Linux 공개 수용은 OV142-007
- [x] [OV142-004] 출시 설명·구독자 안내와 로컬 검사
  - state: complete; depends: OV142-003; evidence: repo:docs/research/overhaul/OV142-004.md#sha256:f4581fc734d54ad88154fa280782aebbedf1c6f5a142de6d14b43e4806d7979c
  - 변경: `docs/releases/0.14.2.md`, `0.14.2.subscriber.ko.md`와 SHA-256 파일. 큰 감시 프로그램 정리와 현재·직전·사용 중 보존을 쉬운 한국어로 설명. 근거·사실 쌍·현재 상태 갱신
  - 확인: documentation·release 검사 묶음·`python scripts/check-human-documentation-style.py --all`·출시 설명 검사·Source Wiki index/lint. 항목별 한국어 커밋과 정확한 단계 브랜치 push, 근거 지문 기록
- [ ] [OV142-005] Fable 5.2 advisor 병합 전 검토
  - state: agent-owned; depends: OV142-004; owner: orchestrator; reason: 이번 실행의 명시 중단 경계
  - 변경: orchestrator가 Fable 5.2 advisor 하위 에이전트로 `origin/develop..fix/overhaul-1-0.14.2` 검토. 발견 사항은 같은 브랜치에서 수정·재검사
  - 확인: 검토 결과·범위·미검증 한계를 근거 문서에 기록. 이번 실행에서 호출 금지
- [ ] [OV142-006] develop 병합
  - state: agent-owned; depends: OV142-005
  - 변경: 검토된 단계 브랜치를 최신 develop에 병합·push
  - 확인: 단계 브랜치의 develop 포함과 원격 일치, 이번 실행 제외
- [ ] [OV142-007] develop에서 0.14.2-test.1 공개 시험
  - state: agent-owned; depends: OV142-006
  - 변경: 03의 공개 시험 절차와 OV141-009 근거를 따라 정확한 develop SHA의 후보·게시·공개 수용
  - 확인: Windows x64·macOS arm64·Linux x64 공개 설치 수용 실행 ID와 제품 지문. 이번 실행 미실행
- [ ] [OV142-008] 정식 0.14.2 출시와 이 PC 갱신
  - state: agent-owned; depends: OV142-007
  - 변경: 버전별 사전 승인 범위의 develop→main·정식 게시·구독자 안내·이 PC 갱신
  - 확인: 공개 버전·감시 bin 파일 수 2 이하·`hive prune --dry-run` 후보 0. OV141-011 실패 조건의 해결 증거 연결. 실제 관측 전 완료 금지, 이번 실행 제외

## 순서·종료

OV142-001→002→003→004 뒤 이번 실행 종료. 이후 005→006→007→008. 자가 편집으로 생긴 지문·링크·문체 실패는 복구·재검사. 범위 내 해결 불가 시 README의 날짜·ID·완료/잔여/변경 파일 중단 메모와 커밋·push. 모든 완료 항목은 근거 문서 SHA-256 연결 필요.
