# `0.9.2` 완료 기능 안정판과 `0.9.3` 분리 계획

> Checklist owner: `REL92-*`
> Target: `2cec037` 기능 기준의 `0.9.2-test.N` 수용 뒤 `0.9.2` 정식판
> 후속: `NAT-002–024`·`MRA-001–032`의 `0.9.3` 전용 branch

## 범위

- `0.9.2` 기능 기준: `2cec0377748874748d126b6b55e59975a3f20a02`
- `0.9.2` 포함: 공개 `v0.9.1` 뒤 완료된 설치 product usage guard 단일 정본 전환과 CI 보정
- `0.9.2` 추가 허용: version·release note·historical base·package metadata·시험·게시 계약
- `0.9.3` 이관: `c777da1` 이후 Native orchestration·workflow·custom subagent 구현
- `0.10.0` 유지: `N10-002–011`

## Branch topology

- `codex/0.9.3-native-agents`: 현재 `603668f` 보존 뒤 모든 Native·custom subagent 후속 작업
- `codex/release-0.9.2`: `2cec037`에서 생성한 완료 기능 release lane
- `develop`: 기존 공개 이력 유지, reset·rebase·force-push·대량 revert 없음
- `main`: accepted `0.9.2-test.N` exact source의 protected stable 통합 대상

## 불변 조건

- branch 생성 순서: `0.9.3` 보존 branch 우선, `0.9.2` release branch 후속
- `0.9.2` release tree의 Native·custom subagent 제품 파일 유입 금지
- 정식판의 탐색·회귀·성능·설치·최종 수용 시험 사용 금지
- 실제 공개 numbered test artifact와 exact source commit·digest 결합
- 시험판 뒤 제품·package·installer·metadata 수정 시 다음 번호 시험판 재수용
- Source Wiki lint·index 오류와 stale source digest `0건` 후 후보 생성

## Checklist

- [x] [REL92-001] 공개 `v0.9.1`·`2cec037`·`c777da1` 경계와 tree 차이 확정
  - Evidence: `v0.9.1` exact `1e5e7b3`, release tree와 merge parent `0a61c74` 동일,
    `2cec037`까지 17 commits·79 files, `c777da1`까지 18 commits·84 files
- [x] [REL92-002] `0.9.2` 완료 기능 release와 `0.9.3` Native·custom subagent 분리 결정 반영
  - Evidence: 사용자 결정, 이 계획·`PLAN.md`·`CURRENT.md`·ADR-0017 current truth
- [ ] [REL92-003] `603668f`의 `codex/0.9.3-native-agents` branch 생성·원격 보존 뒤
  `2cec037`의 `codex/release-0.9.2` branch 생성·원격 exact ref 확인
- [ ] [REL92-004] Release branch에 `0.9.2` 한정 계획과 stable-as-test 금지 규칙을 clean 적용하고
  `NAT-002–024`·`MRA-001–032`·`N10-002–011` 제외·Source Wiki 정합성 확인
- [ ] [REL92-005] `0.9.2` version·build date·release note·npm README·Codex plugin metadata·
  `0.9.1` historical base와 update 경로 동기화
- [ ] [REL92-006] Release branch full Rust·Python·문서·보안 검사와 Native·custom subagent
  유입 `0건`, 5개 native target clean candidate·exact SHA 검증
- [ ] [REL92-007] Release branch exact source에서 다음 빈 `0.9.2-test.N` GitHub prerelease·
  npm `test` 게시, npm `latest` mutation `0건` 확인
- [ ] [REL92-008] 이 Windows의 공개 시험판 clean install·upgrade·rollback·recovery·
  fresh-session·성능·지식·preference 보존 수용
- [ ] [REL92-009] 시험판 결함 수정마다 다음 numbered test 게시와 영향 수용 반복,
  최신 시험판 결함 `0건` 확인
- [ ] [REL92-010] Accepted test exact source의 protected `main` PR·stable candidate 검증 뒤
  `v0.9.2` GitHub Release·npm `latest` 게시
- [ ] [REL92-011] 공개 `0.9.2` clean install·upgrade·version·build date·plugin 표시·npm README와
  rollback·recovery 최종 재검증
- [ ] [REL92-012] `0.9.2` exact `main`을 `codex/0.9.3-native-agents`에 통합하고
  `NAT-002–024`·`MRA-001–032` target을 `0.9.3`으로 고정한 별도 release plan 활성화

## 중단 경계

- 공개 시험판·정식판의 GitHub `release-publication` environment 승인
- protected `main` 검토
- 실제 fresh-session 수용에 필요한 설치 host와 로그인 상태

위 경계 전까지 Agent 소유 branch 준비·검증·candidate 실행 지속.
