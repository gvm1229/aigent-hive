# 현재 상태

- 작업 branch: `develop`; stable source branch: `main`
- 제품 버전: `0.10.2`
- Stable baseline: `0.10.1`
- 첫 공개 시험: `0.10.2-test.1`
- 활성 계획: [`PLAN.md`](../plans/PLAN.md)
- 구현 계획: [`global-user-update-0.10.2.md`](../plans/active/global-user-update-0.10.2.md)
- 출시 계획: [`release-0.10.2.md`](../plans/active/release-0.10.2.md)
- 결정: [`ADR-0022`](../decisions/ADR-0022-global-user-update.md)

## 현재 결함

### 전역 사용자 설치 자동 갱신

- 최신 `hive update`는 실행 파일만 최신이면 전역 사용자 상태를 재검증·복구하지 않고 종료
- 구형 전역 설정은 새 projection refresh 전에 parse되어, 이관이 필요한 상태에서 진입이 막힐 수 있음
- 신규 질문은 전역 installation transaction과 결합되지 않아 사용자에게 별도 명령을 요구할 수 있음
- 등록 프로젝트·실제 사용자 루트 변경 `0건`

### 사용량 보호

- 2026-09-07 유지보수자 명시 승인으로 global threshold `10% → 2%` 변경
- Windows Codex 같은 session fresh enforce: `hive.usage-allowed`, stale halt 제거

## 현재 실행 순서

1. `0.10.2` version·user compatibility·question state
2. update reconciliation·setup-required·answer resume
3. 전역 update와 README 회귀
4. 전체 회귀·release gate
5. `0.10.2-test.1` 세 운영체제 공개 수용과 stable 공개

## 권한·안전 경계

- Agent 소유: `INS102-*` 지침 개선·전역 갱신 완성·공개 시험·`main` 통합·stable `0.10.2` 공개
- 사용자 권한 대기: 실제 DuckSoul apply
- 사용자·외부 bytes: 보존
- Historical project/user base bytes: 변경 금지
- Provider API·credential·OMX/OMC: 사용 금지

## 이전 `0.10.1` 출시 근거 — `0.10.2` 완료 증명에서 제외

- 구현 commit: `b592e305`, `856e945f`, `31e437e6`, `8f399700`, `ecd92340`, `fede7a2a`, `df2a88f8`
- Rust 전체: 446 통과·수동 qualification 1 제외, core 109·projection 39·render 63·update 54·wiki 177 통과
- Historical project lifecycle: `0.9.1–0.10.0` scan·dry-run·rollback·apply·validate 통과
- 공개 `0.10.0` exact support-state fixture: Codex·Claude·Antigravity의 `test.2`·`test.4` 갱신 통과
- Source Wiki: 174개 page, 오류 `0건`, 경고 `0건`
- DuckSoul Git 상태: 기존 사용자 변경 존재, 이번 진단 변경 `0건`
- 사용량 보호: global threshold `10%`; 제품 수정은 같은 session 재평가를 사용하며 disable 불필요
- Python lane: documentation 87·security 103·contract 466·integration 94·release 115 통과; 플랫폼 조건부 건너뜀은 별도 유지
- 공개 시험 gate: `0.10.1-test.1`, product digest `sha256:033ae9d5bd8bfbcfe5ab6eeb8243546048bcff5f8954122b14162a5f34c793ff` 승인
- Candidate `34000885782`, source `da6636a679ea451d500f26550f7738e3063697f1`; 다섯 native artifact 통과
- npm 여섯 package `test=0.10.1-test.1`, `latest=0.10.0`; GitHub prerelease 25개 artifact 확인
- Public acceptance `34001760231`: Windows x64·macOS arm64·Linux musl x64 설치·한국어·rollback·vector 통과
- Stable `0.10.1`: 2026-09-06 유지보수자 명시 승인과 공개 완료
- Stable promotion mode: `35f89f9f`; accepted source·product digest·세 host acceptance run 결합, qualification 재실행 없음
- 구독자 요약 승인 digest: `sha256:36be7d519874b54e7817f26f8819be57ad7d011be3a430aff46ebb89c15768f7`, release 환경 등록 완료
- `main` 통합: PR #49 `5e64ce8a`, recovery PR #50 `bb56037f`
- Stable candidate `34008946911`: accepted `0.10.1-test.1`과 acceptance `34001760231` 결합, 다섯 native artifact·integrity bundle 통과
- Stable publication: 최초 run `34009500410`은 npm 전파 지연 뒤 tag 전 중단; recovery run `34010951366` 성공
- 독립 확인: npm 여섯 package `version/latest=0.10.1`, `test=0.10.1-test.1`; GitHub 정식 Release 29개 asset
- Windows 공개 stable 설치: `AIgent Hive v0.10.1 (released 2026-09-06)`
- 위 완료 근거의 적용 범위: 이전 `0.10.1` 출시만. 현재 `0.10.2` 미완료 항목은 활성 계획 기준
