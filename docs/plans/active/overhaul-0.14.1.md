# 단계 1 — 0.14.1 수정판

> Plan version: 0.14.1
> Scope: product

브랜치: `fix/overhaul-1-0.14.1`([브랜치 규칙](../overhaul/README.md)). 진입: 단계 0의 develop 병합과 버전 승인. 새 제품 바이트는 0.14.1과 test.1 필요([03](../../../.agents/directives/03-workflow.md)). 시험은 `tests/work/`의 임시 사용자·설치만 사용

## 항목

- [x] [OV141-001] 0.14.1 시작 승인과 계획 전환
  - state: complete; evidence: repo:docs/research/overhaul/OV141-001.md#sha256:b49e4be075ec55689a2252841998cfd0f74da19b9749200ca306f4a53a46d85d
  - 변경: 0.14.0 계획 묶음을 `docs/archive/`로 보관, 이 파일을 `docs/plans/active/overhaul-0.14.1.md`로 이동·등록, 제품 버전 0.14.1 동기화
  - 확인: `python scripts/check-plan-state.py` valid, `bash scripts/check-release-version.sh`
- [x] [OV141-002] 사용량 임계값 확인 기록 오류 수정
  - state: complete; depends: OV141-001; evidence: repo:docs/research/overhaul/OV141-002.md#sha256:326c576e5e49b35ca728949f095d6a419849f898fe99acc26f4171a8e40a5d0b
  - 변경: [임계값 백로그](../backlog/usage-threshold-provenance.md)의 "수정 후보" 그대로. `crates/hive-cli/src/user_setup.rs`의 `set_operational_usage_threshold`에서 기록 정규화, 시험 `global_threshold_with_valid_provenance_updates_atomically` 추가
  - 확인: `python scripts/dev-check.py rust -- test -p hive-cli usage --locked`, 백로그 "완료 조건" 4개 충족
- [x] [OV141-003] 공통 지식 연결 상태의 동작 결정
  - state: complete; depends: OV141-001; evidence: repo:docs/research/overhaul/OV141-003.md#sha256:6db8940f4cdc9861b6cff9505b5f56e5b42c281e09b03f16bf7d18c68254a805
  - 변경: [연결 상태 백로그](../backlog/knowledge-attachment-consistency.md)의 "별도 판단이 필요한 동작" 4건의 선택지와 영향 제시, 결정 기록
  - 확인: 4건 모두 결정 기록
- [x] [OV141-004] 공통 지식 연결 상태 수정
  - state: complete; depends: OV141-003; evidence: repo:docs/research/overhaul/OV141-004.md#sha256:4e68ab3812f0a7ec04039d0bf4f97dda451250903af5aae7f62b0e08d1620560
  - 변경: 결정 범위만 `crates/hive-wiki/src/store.rs`의 `sync_project_registry`·`synchronize_project_collection_metadata`와 CLI `ensure_rag_registry`에 반영
  - 확인: 백로그 "변경 후보와 검증" 4번의 회귀, `python scripts/dev-check.py rust -- test -p hive-wiki --locked`
- [x] [OV141-005] 감시 실행 파일 보관 2개 제한
  - state: complete; depends: OV141-001; evidence: repo:docs/research/overhaul/OV141-005.md#sha256:75312ecec68a450e2d9c86b8359e7db87419516f03fbf3b9a900616462506e64
  - 변경: `crates/hive-cli/src/usage_monitor.rs`(설치 경로 `:507-514`)에서 설치 성공 뒤 `runtime/usage-monitor/bin/hive-*.exe` 중 현재·직전 외 정리. 실행 중·잠긴 파일은 건너뛰고 기록
  - 확인: 임시 사용자 폴더의 5개 → 2개, 실행 중 파일 보존, 실패 시 기존 파일 불변 시험
- [ ] [OV141-006] 미완료 감시 갱신 복구
  - state: awaiting-external-evidence; depends: OV141-005; owner: user; reason: 격리 Windows 재현 필요, 클라우드 Linux 세션에서 PowerShell·작업 스케줄러 실행 불가. 단계 표시 진단과 조사는 [중간 기록](../../research/overhaul/OV141-006-partial.md)
  - 변경: `runtime/usage-monitor/upgrade.json`의 `phase: incomplete`("Windows monitor adapter failed")를 임시 사용자로 재현, 원인 기록, 재시도·복구 경로 구현
  - 확인: 격리 Windows 재현과 복구 통과. 실제 PC 적용은 OV141-011
- [ ] [OV141-007] 저장 용량 보고와 정리 미리보기
  - state: agent-owned; depends: OV141-005
  - 변경: `hive storage`(지식·색인·벡터·감시 실행 파일·백업별 용량)와 `hive prune --dry-run`(삭제 후보만 표시, 실제 삭제는 `--confirm`). 출력 접두 `[Hades]`
  - 확인: 임시 폴더의 분류 합계 = `du` 합계, dry-run 전후 파일 해시 불변
- [ ] [OV141-008] README·출시일·Claude Code 표시
  - state: agent-owned; depends: OV141-001
  - 변경: `README.md:15`와 `Cargo.toml:23` 출시일 일치, `README.md:326-327` 문구, `README.md:134`에 Claude Code "검증 전" 표시
  - 확인: `python scripts/check-public-stable-docs.py`, `python scripts/check-human-documentation-style.py`
- [ ] [OV141-009] 0.14.1-test.1 공개 시험
  - state: agent-owned; depends: OV141-002,OV141-004,OV141-005,OV141-006,OV141-007,OV141-008
  - 변경: 03의 번호 시험판 절차
  - 확인: 세 OS 공개 시험 통과와 `tests/results/` 근거
- [ ] [OV141-010] develop 병합과 단계 브랜치 종료
  - state: agent-owned; depends: OV141-009
  - 변경: 단계 브랜치 `fix/overhaul-1-0.14.1`를 최신 `develop`에 병합(fast-forward 우선, 아니면 merge 커밋), `develop` push. 정식 출시 항목은 이후 `develop`에서 진행(`main` PR의 head는 `develop`), 단계 브랜치 삭제는 정식 뒤
  - 확인: `git branch --merged develop`에 단계 브랜치 포함, `develop`과 `origin/develop` 일치
- [ ] [OV141-011] 정식 0.14.1 출시와 이 PC 갱신
  - state: awaiting-user-authority; depends: OV141-010; owner: user; reason: 정식 태그와 게시와 설치 승인
  - 변경: 승인 범위의 main 통합·태그·게시, 이 PC 갱신 뒤 감시 실행 파일 2개·갱신 완료 상태 확인
  - 확인: 공개 패키지 버전, `~/.hive/runtime/usage-monitor/bin` 파일 수 2 이하
