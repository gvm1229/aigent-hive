# `0.8.0` 프리뷰 릴리스 실행 계획

> Target: `0.8.0`
> Checklist owner: [`P7-*`](../phases/07-public-qualification.md)
> Decision: [`ADR-0013`](../../decisions/ADR-0013-preview-release-scope.md)
> Skill budget research:
> [`codex-skill-context-budget.md`](../../research/codex-skill-context-budget.md)

## 릴리스 정의

- 공개 명칭: `0.8.0 Claude-unverified preview`
- 실제 host 검증: Codex `0.145.0`, Antigravity `1.1.7`
- Claude 상태: package·fixture·projection 이론 지원, subscription-backed 실제 검증 없음
- Windows 상태: CI artifact qualification 이력, 실제 Windows 기기 사용자 검증 필요
- Windows consumer shell: 기본 PowerShell 5.1·`cmd.exe`, PowerShell 7 dependency 없음
- Windows source shell: PowerShell 7 LTS 개발·release dependency
- 업데이트 방식: package manager 또는 digest 고정 수동 설치
- Network self-update: 비활성
- 신뢰 기준: exact tag, SHA-256, GitHub artifact attestation, source commit provenance
- 비필수 신뢰 기준: Developer ID·notarization, Authenticode, external TUF 2-of-3

## 현재 blocker evidence

| Gate | 현재 상태 | 정본 항목 |
| --- | --- | --- |
| Skill metadata | Project projection implicit owner 1개, 나머지 explicit-only, fresh-session 중복 warning 0건 | 완료 |
| Clean-clone CI | `9b1e951`의 Linux·Windows·contract failure | `P7-040` |
| Windows UAT | 실제 기기 설치·setup·project harness·upgrade 미검증 | `P7-041` |
| Windows shell | PowerShell 5.1·`cmd.exe`·source-only PowerShell 7 경계 완료 | 완료 |
| Candidate trust | Current tag artifact·digest·attestation 미생성 | `P7-020` |
| Release candidate | Clean candidate qualification 미실행 | `P7-018` |
| Publication | Protected GitHub Release 미실행 | `P7-037` |

원격 evidence:

- [GitHub Actions CI `30347960157`](https://github.com/gvm1229/aigent-hive/actions/runs/30347960157)
- [Native release runtime `30347960118`](https://github.com/gvm1229/aigent-hive/actions/runs/30347960118)

## 실행 순서

1. `P7-040`: Linux shared-index directory sync, Windows canonical user-root pinning,
   release-version output contract, schema validation failure 수정
2. `P7-041`: Windows 실제 기기 clean install, global setup, project auto onboarding,
   shared index, update·rollback smoke
3. `P7-020`: exact candidate tag 기반 multi-platform archive, SHA-256,
   GitHub artifact attestation 생성
4. `P7-018`: clean clone, 전체 Rust·Python, 세 OS CI, installer·upgrade·recovery,
   Wiki·Skill budget qualification
5. `P7-037`: `Claude-unverified preview` disclosure와 known limitation을 포함한
   protected GitHub Release publication

## Claude preview 경계

- 실제 Claude Code install·update E2E의 `0.8.0` 차단 조건 제외
- 실제 Claude Pro/Max usage sensor parity의 `0.8.0` 차단 조건 제외
- Claude 지원 주장: fixture·schema·projection conformance 한정
- Release note·README·support matrix의 미검증 상태 표시 필수
- Subscription 확보 이후 별도 qualification checklist 재활성화

## Windows 인계

- Clean Windows 11 x86_64 기기
- Windows PowerShell 5.1·`cmd.exe`·Git 상태 기록
- PowerShell 7: source test·release workflow 전용 별도 설치·version 기록
- `powershell.exe` direct install과 `cmd.exe` paste/run command 모두 검증
- Consumer install·setup·update 중 PowerShell 7 탐지·설치 제안 0건
- Release archive SHA-256 확인
- `hive --version`, user setup, Codex 또는 Antigravity install 검증
- Project auto onboarding과 user-root shared SQLite 검증
- Repeat update, local modification merge, failed update recovery 검증
- 개인정보·credential·raw quota payload 미수집

## 중지 경계

- Protected GitHub Release publication 직전 사용자 최종 확인
- Credential·private signing material 접근 금지
- 기존 published commit 이력 rewrite 금지
- Exact `1.0.0` 별도 명시 전 major release 준비 금지
