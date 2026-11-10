# 단계 0 — 저장소 정리

> Plan version: 0.14.0
> Scope: source

브랜치: `chore/overhaul-0-repo-cleanup`([브랜치 규칙](README.md)). 제품 바이트 변경 없음. 근거 문서 위치: `docs/research/overhaul/<ID>.md`

## 항목

- [x] [OVD-001] 미커밋·미추적 파일의 소유 대조표
  - state: complete; evidence: repo:docs/research/overhaul/OVD-002.md#sha256:b5d3e03cecee3a8626850cdf6d183941c391bfefcb9be80953bb9204e2003a36
  - 변경: `git status --short` 전체를 `.agents/work/active-sessions/` 기록과 대조. 파일별 소유 세션·변경 요약·제안(커밋/보류/보관)을 근거 문서에 표로 기록. 대상 파일 수정 금지
  - 확인: 표의 행 수 = `git status --short | wc -l`, 소유 불명 행은 "불명"으로 명시
- [x] [OVD-002] 대조표의 커밋·보류 결정 반영
  - state: complete; depends: OVD-001; evidence: repo:docs/research/overhaul/OVD-002.md#sha256:b5d3e03cecee3a8626850cdf6d183941c391bfefcb9be80953bb9204e2003a36
  - 변경: 결정대로 관심사별 커밋([커밋 규칙](../../guides/commit-rules.md)), 보류 파일은 그대로 유지
  - 확인: `git status --short`에 보류 결정 파일만 잔존
- [ ] [OVD-003] develop push와 병합 완료 브랜치 정리
  - state: awaiting-user-authority; depends: OVD-002; owner: user; reason: 사용자 PC 자격 증명의 push와 브랜치 삭제 승인
  - 변경: 로컬 `develop`의 `feature/interactive-cli-0.14.0` fast-forward는 완료. 남은 일은 `git push origin feature/interactive-cli-0.14.0 develop`, 병합 완료 `docs/update-notes-sections`와 `feature/interactive-cli-0.14.0`의 삭제 여부 결정
  - 확인: `git status -sb`, `git branch --merged origin/main`
- [x] [OVD-004] 미등록 활성 조각의 보관 이동
  - state: complete; depends: OVD-002; evidence: repo:docs/research/overhaul/OVD-004.md#sha256:204d2a464ecd224fd3116b9241d2e3e4b82c228548723bc45bcbef6c8806c2d5
  - 변경: `docs/plans/active/` 중 `PLAN.md` 미등록 파일을 `docs/archive/plans/`로 `git mv`, 깨진 링크 수정
  - 확인: `ls docs/plans/active | wc -l` = 등록 수, `python scripts/check-plan-state.py`, `python scripts/check-markdown-links.py`
- [ ] [OVD-005] CURRENT.md 축소와 표기 수정
  - state: agent-owned; depends: OVD-002
  - 변경: 출시 완료된 0.14.0 근거 단락을 `docs/archive/state/`로 이동, "PC 본체14.0" 오타와 붙여 쓴 버전 표기(`정식0.14.0` 등) 수정, `PLAN.md` 같은 표기 수정
  - 확인: `wc -c docs/state/CURRENT.md` 6144 이하, `python scripts/check-human-documentation-style.py`, `python scripts/check-plan-state.py`
- [ ] [OVD-006] 사용량 지침 보완
  - state: awaiting-user-authority; depends: OVD-002; owner: user; reason: 에이전트 지침 변경 승인
  - 변경: `.agents/directives/07-installed-usage-guard.md`에 "`hive` 실행 불가 세션은 읽기 전용 작업만 하고 사전 확인 미실행을 보고" 추가, 날짜 붙은 승인 문단(`:36`)을 `docs/decisions/`로 이동
  - 확인: `python scripts/check-agent-directives.py`
- [ ] [OVD-007] AI 공동 작성자 표기 거부 검사
  - state: agent-owned; depends: OVD-002
  - 변경: `.github/workflows/branch-policy.yml`에 커밋 메시지의 AI `Co-Authored-By` 거부 단계 추가([git 규칙](../../../.agents/directives/references/git-commits.md))
  - 확인: 표기 포함 시험 커밋의 실패, 표기 없는 커밋의 통과
- [ ] [OVD-008] 백로그·개요 문서 갱신
  - state: agent-owned; depends: OVD-002
  - 변경: `docs/plans/backlog/claude-host-acceptance.md`의 상태·항목 수 갱신, 모든 백로그 첫머리에 검토일, `docs/overview/product.md:31` 갱신
  - 확인: `python scripts/check-markdown-links.py`, 문체 검사기
- [ ] [OVD-009] 시험 기록 보존과 검색 제외
  - state: agent-owned; depends: OVD-002
  - 변경: `tests/results/runs/`의 정식 이전 기록은 버전별 요약과 압축 보관, `tests/results/INDEX.md` 유지. 과거 기준본·시험 기록을 `.ignore`에 등록, 파일은 보존
  - 확인: `INDEX.md`의 모든 링크 유효, 저장소 전체 `rg` 검색 시간 전후 비교

- [ ] [OVD-010] develop 병합과 단계 브랜치 종료
  - state: agent-owned; depends: OVD-004,OVD-005,OVD-007,OVD-008,OVD-009
  - 변경: 단계 브랜치 `chore/overhaul-0-repo-cleanup`를 최신 `develop`에 병합(fast-forward 우선, 아니면 merge 커밋), `develop` push. 병합 뒤 단계 브랜치 삭제
  - 확인: `git branch --merged develop`에 단계 브랜치 포함, `develop`과 `origin/develop` 일치

## 메모

- OVD-006은 승인 대기라 병합 조건에서 제외, 승인 시 같은 브랜치 또는 다음 단계에서 처리
- 0.14.0 정식의 남은 사용자 승인(이 PC 갱신·감시 복구)은 OV141-006·OV141-010과 함께 처리
