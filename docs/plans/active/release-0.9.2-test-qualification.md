# `0.9.2` 시험판 선행 출시 자격 계획

> Checklist owner: `REL92-*`
> Target: `0.9.2-test.N` 수용 뒤 `0.9.2` 정식판
> 제외: `N10-002–011`의 `0.10.0-test` 후보만 해당

## 불변 조건

- 정식판의 탐색·회귀·성능·설치·최종 수용 시험 사용 금지
- `N10-002–011` 외 활성 계획 미완료 `0건` 전 stable candidate 생성 금지
- 실제 공개 numbered test artifact와 exact `develop` commit·digest를 수용 증거에 결합
- 시험판 뒤 제품·package·installer·metadata 수정 시 다음 번호 시험판으로 영향 범위 재수용
- `main` stable candidate와 `latest` 게시를 모든 시험 증거가 끝난 마지막 단계로 제한

## Checklist

- [x] [REL92-001] 활성 계획 전수 대조와 비-`0.10.0` 미완료 55개 확정
  - Evidence: `develop` `2cec0377748874748d126b6b55e59975a3f20a02`, `NAT-002–024` 23개,
    `MRA-001–032` 32개, 제외 `N10-002–011` 10개
- [ ] [REL92-002] stable-as-test 금지와 numbered-test-first 순서를 source directive·ADR·정적 회귀에 고정
- [ ] [REL92-003] `NAT-002–005`·`MRA-001–006` host feasibility와 지원 경계 확정
- [ ] [REL92-004] `NAT-006–015` canonical protocol·scheduler·authority·CLI·migration 구현과 hostile 검증
- [ ] [REL92-005] `NAT-016–024`·`MRA-007–032` host adapter·Skill·role·Judge·생성·수용 구현과 검증
- [ ] [REL92-006] 모든 active checklist 재대조와 `N10-002–011` 외 미완료 `0건` 확인
- [ ] [REL92-007] `0.9.2` version·release note·npm README·Codex plugin metadata·build date source 동기화
- [ ] [REL92-008] full Rust·Python·문서·보안·5개 native target clean candidate와 exact SHA 검증
- [ ] [REL92-009] exact `develop`에서 `0.9.2-test.1` 또는 다음 빈 번호의 GitHub prerelease·npm `test` 게시
- [ ] [REL92-010] 이 Windows의 공개 시험판 clean install·upgrade·rollback·recovery·fresh-session·성능·지식·preference 보존 수용
- [ ] [REL92-011] 시험판 결함 수정마다 다음 numbered test 게시와 영향 수용 반복, 최신 시험판 결함 `0건` 확인
- [ ] [REL92-012] accepted test exact source를 protected `main` 후보로 통합한 뒤 별도 stable gate에서 `0.9.2`·npm `latest` 게시·공개 재검증

## 중단 경계

- 공개 시험판의 GitHub `release-publication` environment 승인
- 실제 Claude Code·Codex fresh-session host 수용에 필요한 설치 host와 로그인 상태
- protected `main` 검토와 stable publication environment 승인

위 경계 전까지 Agent 소유 구현·로컬 검증·`develop` push·candidate 실행을 계속 진행.
