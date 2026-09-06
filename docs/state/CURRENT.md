# 현재 상태

- 작업 branch: `develop`
- 제품 버전: `0.10.1`
- Stable baseline: `0.10.0`
- 수용 공개 시험: `0.10.1-test.1`
- 활성 계획: [`PLAN.md`](../plans/PLAN.md)
- 구현 계획: [`harness-upgrade-0.10.1.md`](../plans/active/harness-upgrade-0.10.1.md)
- 출시 계획: [`release-0.10.1.md`](../plans/active/release-0.10.1.md)
- 결정: [`ADR-0021`](../decisions/ADR-0021-0.10.1-upgrade-usage-fix.md)

## 현재 결함

### Project harness upgrade

- Windows Codex 공개 `0.10.0` 실제 실행: DuckSoul `0.9.5` scan 실패
- 오류: `hive.skill-selection-invalid: selected user Skills must be unique`
- 실제 raw Skill 이름: 25개·중복 `0건`
- Canonical 변환 뒤: 24개
- 충돌: `iterative-execution`, `ralph-loop` → `verified-workflow`
- 추가 발견: tracked `0.9.5` full project base 57개 파일, runtime full-base registry coverage 누락
- DuckSoul `.hive` 변경·apply·recover journal 생성 `0건`

### Usage guard threshold

- 공개 `0.10.0` 실제 실행: global threshold `60% → 10%` 저장 성공
- 기존 `halt.json`: `60%` policy decision 유지
- 같은 session `status`: `hive.usage-session-halted`, explicit disable 요구
- 현재 원인: halt binding에 effective policy identity 부재, enforce의 current-session marker 무조건 우선
- 목표: fresh recheck 뒤 allow 또는 current-policy halt, session disable 불필요

## 현재 실행 순서

1. 공통 compatibility registry와 historical-state fixture
2. 인증 우선 ProjectState migration과 Skill merge
3. Policy-bound halt marker와 same-session recheck
4. 전체 회귀·release gate
5. `0.10.1-test.1` 세 운영체제 공개 수용

## 권한·안전 경계

- Agent 소유: accepted-test promotion 구현·검증·`main` 통합·stable `0.10.1` 공개
- 사용자 권한 대기: 구독자 요약 문구 승인, 실제 DuckSoul apply
- 사용자·외부 bytes: 보존
- Historical project/user base bytes: 변경 금지
- Provider API·credential·OMX/OMC: 사용 금지

## 현재 근거

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
- Stable `0.10.1`: 2026-09-06 유지보수자 명시 승인; accepted-test promotion mode 구현 뒤 진행
- Stable promotion mode: `35f89f9f`; accepted source·product digest·세 host acceptance run 결합, qualification 재실행 없음
- 남은 검증: stable 문서·구독자 문구 승인, `main` 통합, stable 공개·독립 확인
