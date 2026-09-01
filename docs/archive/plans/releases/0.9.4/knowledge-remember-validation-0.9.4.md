# `0.9.4` 전역 지식 기록 credential 오탐

> Checklist owner: `KRV94-*`
> 대상: `0.9.4` patch
> 관찰: 2026-08-14, Windows x64 installed `0.9.3`

## 문제

안전한 사용자 결정 한 건의 `hive knowledge remember --user-root` 기록 요청이
`hive.knowledge-verification-failed`로 거부. 입력: Skill 설명의 정본 ID 표시와 전역 설치
validation 정합성 결정. credential·secret·private source 입력 `0건`.

## 원칙

- 실제 credential·secret·private key의 user-root 기록·승격·전송 금지
- 안전한 사실·선호·결정·규약의 정상 기록 허용
- command flag 이름·claim key·일반 ID·일반 설명의 credential 오탐 금지
- reject 결과: 정확한 field·reason과 재현 가능한 안전한 복구 안내

## Checklist

- [x] [KRV94-001] 안전한 user-root `knowledge remember` 입력의 credential false-positive
  regression 고정. scanner·request construction·claim key 경계 원인 식별
- [x] [KRV94-002] safe statement·claim key·canonical metadata 기록 수용. 실제 credential fixture는
  user-root Markdown·SQLite mutation 전 fail-closed 유지
- [x] [KRV94-003] unit·CLI integration과 Windows x64 installed `0.9.4-test` automatic capture
  receipt 수용 확인

## 수락 기준

- 안전한 사용자 결정 기록 성공, canonical Markdown·derived SQLite receipt 생성
- 실제 비밀 값 기록·승격·외부 전송 `0건`
- `hive.knowledge-verification-failed` 오류: raw source 포괄 표현 대신 exact field·reason 표시

## 구현 증거

- `crates/hive-wiki/src/lib.rs`: `claim_key`와 자동 user-statement source 식별자에 들어간 긴
  출시 식별자 수용. 이미 있는 token prefix·assignment·opaque value 탐지는 유지
- `crates/hive-wiki/src/rag.rs`: 실제 비밀 값 거부에 `normalized_fact` 또는
  `provenance.summary`의 정확한 field와 이유 표시
- `crates/hive-cli/src/knowledge.rs`: user-root `remember`의 안전한 긴 출시 claim key,
  canonical Markdown·derived SQLite 수용과 실제 GitHub token의 변경 전 거부 회귀
- 이 Windows x64 source workspace의 `cargo test -p hive-wiki --locked` 113개,
  `cargo test -p hive-cli knowledge --locked` 49개 통과. `KRV94-003`의 public
  `0.9.4-test.1` automatic capture receipt 수용 완료: safe user-root statement는
  `hive.knowledge-remembered`, credential-shaped input은 canonical Markdown·SQLite 변경 없이
  exact field·reason 오류 확인

## 범위 제외

- 비밀 값 기록 허용
- provider API credential 처리
- project-private 지식의 자동 전역 공유
