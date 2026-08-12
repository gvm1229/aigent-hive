# `0.9.2` 완료 기능 안정판 출시 계획

> Checklist owner: `REL92-*`
> Target: `2cec037` 기능 기준의 `0.9.2-test.N` 수용 뒤 `0.9.2` 정식판
> 제외: `NAT-002–024`·`MRA-001–032`의 `0.9.3`, `N10-002–011`의 `0.10.0-test`

## 범위

- 기능 기준: `2cec0377748874748d126b6b55e59975a3f20a02`
- 포함: 공개 `v0.9.1` 뒤 완료된 설치 product usage guard 단일 정본 전환과 CI 보정
- 추가 허용: version·release note·historical base·package metadata·시험·게시 계약
- 필수 문서: root·번역 README, 설치 안내, 공개 HTML, npm README, plugin metadata,
  문서 색인·명령·version 예시
- 제외: Native orchestration·workflow·custom subagent 제품 구현

## 불변 조건

- 정식판의 탐색·회귀·성능·설치·최종 수용 시험 사용 금지
- 실제 공개 numbered test artifact와 exact release branch commit·digest 결합
- 시험판 뒤 제품·package·installer·metadata·공개 문서 수정 시 다음 번호 시험판 재수용
- Source Wiki lint·index 오류와 stale source digest `0건` 후 후보 생성
- 공개 문서의 stale version·명령·링크·표시 metadata `0건` 후 후보 생성
- `0.9.3` 구현·검증·출시: QA contributor 추가 지시 뒤 유지보수자의 별도 명시적 승인 전 금지

## Checklist

- [x] [REL92-001] 공개 `v0.9.1`·`2cec037`·`c777da1` 경계와 tree 차이 확정
  - Evidence: `v0.9.1` exact `1e5e7b3`, release tree와 merge parent `0a61c74` 동일,
    `2cec037`까지 17 commits·79 files, `c777da1`까지 18 commits·84 files
- [x] [REL92-002] stable-as-test 금지·numbered-test-first와 version 제외 범위의 source
  directive·ADR·plan·정적 회귀 고정
- [x] [REL92-003] `codex/release-0.9.2` exact ref의 원격 보존과 candidate workflow 허용 branch 검증
- [x] [REL92-004] Release tree의 `NAT-002–024`·`MRA-001–032` 제품 구현 유입 `0건`, Source Wiki 정합성 확인
- [x] [REL92-005] `0.9.2` version·build date·release note·npm README·Codex plugin metadata·
  `0.9.1` historical base와 update 경로 동기화. Root·번역 README, 설치 안내, 공개 HTML,
  문서 색인·명령·version 예시 전수 최신화와 stale public reference `0건` 확인
- [x] [REL92-006] Full Rust·Python·문서·보안 검사와 5개 native target clean candidate·exact SHA 검증
- [x] [REL92-007] Release branch exact source에서 다음 빈 `0.9.2-test.N` GitHub prerelease·
  npm `test` 게시, npm `latest` mutation `0건` 확인
- [x] [REL92-008] 이 Windows의 공개 시험판 clean install·upgrade·rollback·recovery·
  fresh-session·성능·지식·preference 보존 수용
- [x] [REL92-009] 시험판 결함 수정마다 다음 numbered test 게시와 영향 수용 반복,
  최신 시험판 결함 `0건` 확인
  - `0.9.2-test.1`: candidate run `31596919466`·publication run `31597939956`는 통과했으나,
    게시 workflow가 생성한 `install.sh`·`install.ps1`·`install.cmd`를 GitHub Release 자산에
    첨부하지 않아 공개 설치 URL `404`. 수용 거부·`test.2` 재검증 대상으로 전환
  - `0.9.2-test.2`: candidate run `31599834995`·publication run `31600929652`, 25개 GitHub
    자산·npm `test`·Windows clean install·`0.9.1` upgrade·pending receipt recovery·성능·지식·
    preference 보존 PASS. README 두 언어의 선택형 시험판 안내가 `test.1`에 고정된 stale
    reference 발견으로 수용 보류·번호 독립 npm `test` 안내 보정·`test.3` 재검증 대상으로 전환
  - `0.9.2-test.3`: candidate run `31602608609`·publication run `31603511607`, 25개 GitHub
    자산·npm `test`·npm tarball README·Windows 공개 설치·plugin 표시·user-scope validate·5% usage
    guard·지식과 preference 보존 PASS. 결함 `0건`; 완료 checklist exact tree를 `test.4`로 최종 고정
- [ ] [REL92-010] Accepted test exact source의 protected `main` PR·stable candidate 검증 뒤
  `v0.9.2` GitHub Release·npm `latest` 게시
- [ ] [REL92-011] 공개 `0.9.2` clean install·upgrade·version·build date·plugin 표시·npm README와
  모든 README·설치 안내·공개 HTML·링크·명령·version 예시, rollback·recovery 최종 재검증
- [ ] [REL92-012] `0.9.2` 완료 상태·정확한 공개 증거 기록. QA contributor 추가와
  유지보수자의 후속 명시적 승인 전 `0.9.3` 상태 `awaiting-user-authority`·구현 mutation `0건`

## 중단 경계

- 공개 시험판·정식판의 GitHub `release-publication` environment 승인
- protected `main` 검토
- 실제 fresh-session 수용에 필요한 설치 host와 로그인 상태
- QA contributor 추가 뒤 `0.9.3` 재개에 필요한 유지보수자의 별도 명시적 승인

위 경계 전까지 Agent 소유 release 준비·검증·candidate 실행 지속.
