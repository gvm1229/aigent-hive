# `0.10.1` 일반 harness 갱신·사용량 보호

> Checklist owner: `HUP101-*`, `UGR101-*`
> 재현 환경: Windows Codex, 공개 `0.10.0`
> 소비자 적용: 합성 fixture만 허용, 실제 DuckSoul 변경 제외

## 확인된 원인

- `project upgrade`: 새 projection 생성이 historical base 인증보다 먼저 실행
- Skill 선택: `iterative-execution`과 `ralph-loop`의 선언된 `verified-workflow` merge를 중복으로 오판
- Historical base: tracked `0.9.5` full base 존재, runtime full-base registry 누락
- 수용 fixture: current binary 설정 생성 뒤 version 문자열만 교체하여 실제 predecessor support state 누락
- Usage guard: threshold 갱신 뒤 기존 `halt.json`의 old policy decision을 current session에서 영구 우선

## 일반 harness 갱신

- [x] [HUP101-001] DuckSoul형 비밀 없는 `0.9.5` support state 재현 fixture와 실패 회귀 고정
- [x] [HUP101-002] Stable source·compatibility epoch·project base·user projection·state schema의 공통 등록표
- [x] [HUP101-003] 등록표 기반 Rust historical registry 자동 생성과 hand-maintained version dispatch 제거
- [x] [HUP101-004] Project base 인증을 current candidate render보다 앞에 배치
- [x] [HUP101-005] Source-version-bound `ProjectState` parse·cross-file validation·current-state migration
- [x] [HUP101-006] Raw duplicate 거부와 declared rename·many-to-one merge 수렴 분리
- [x] [HUP101-007] Support files·projection·retired cleanup의 기존 journal·rollback 단일 경계 유지
- [x] [HUP101-008] Scan·dry-run의 source digest·migration ID·normalized field·Skill merge 보고
- [x] [HUP101-009] 모든 declared predecessor·세 host·설정 variant의 compiled CLI lifecycle matrix
- [x] [HUP101-010] Migration table·registry·full base·executable coverage 누락의 build/publication 차단

## 사용량 보호 재평가

- [x] [UGR101-001] Effective usage policy의 deterministic digest와 halt marker binding
- [x] [UGR101-002] Legacy marker의 policy-stale 분류와 `hive.usage-recheck-required` status
- [x] [UGR101-003] Current-policy halt short-circuit와 old-policy fresh sensor recheck 분리
- [x] [UGR101-004] Allow 때 exact-byte halt 제거, limited·unknown 때 current-policy marker 교체
- [x] [UGR101-005] 측정 중 policy 변경의 bounded retry와 fail-closed 결과
- [x] [UGR101-006] Threshold 결과의 `session_recheck_required`와 stored/project/effective 값 표시
- [x] [UGR101-007] `usage-guard` Skill의 threshold 직후 same-binding enforce, disable mutation `0건`
- [x] [UGR101-008] Source·project·global max·legacy·race·run/loop authorization 회귀

## 수락 기준

- `0.9.5` 25개 선택 → 24개 canonical 선택, `verified-workflow` 1개
- Unknown·raw duplicate·undeclared merge·tampered base: changed paths `0건`
- Threshold `60 → 10`, fresh remaining `>10`: guard enabled, halt 제거, control override 부재
- Fresh remaining `≤10` 또는 sensor unknown: current threshold로 중단 유지
- 다른 session: 다음 enforce 전 자동 해제 없음
