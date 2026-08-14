# `0.9.4` GitHub Release 이중 언어 설명

> Checklist owner: `RNL94-*`
> 대상: `0.9.4` patch
> 표면: `docs/releases/<product-version>.md`와 GitHub Release description

## 문제

현재 GitHub Release: 한국어 release note만 게시. 영어 사용자의 변경 내용·호환성·검증 범위 확인 불가.

## 원칙

- GitHub Release description 순서: 영어 먼저, 한국어 다음
- 영어 section: ASD-STE100 Simplified Technical English. 짧고 직접적인 문장, 관용 표현·불필요한
  동의어·모호한 대명사·중첩 문장 금지
- 한국어 section: 한국어 어휘·문장 구조. 대체 가능한 일반 영어 혼용·영어 어순 직역·기술적 인상용
  영어 사용 금지
- 두 section: 같은 제품 사실·호환성·검증 경계. 번역 누락·과장·서로 다른 약속 금지
- GitHub Release는 `docs/releases/<product-version>.md`를 그대로 사용. 별도 수동 설명 입력 경로 금지

## Checklist

- [ ] [RNL94-001] release note 정본 형식의 English-first section과 Korean section 정의. 현재·신규
  release note의 version·scope·compatibility·verification·publication field 동등성
- [ ] [RNL94-002] release workflow의 release note structure·순서·두 언어 필수 field·ASD-STE100
  English·Korean language contract 검사와 명확한 failure receipt
- [ ] [RNL94-003] `0.9.4-test` GitHub prerelease description의 English-first bilingual rendering
  확인. stable publication 전 같은 source note·순서·내용 재검증

## 수락 기준

- GitHub Release description 첫 section: English
- English section: ASD-STE100 rules 준수
- Korean section: 의미 중심 한국어, 불필요한 영어 혼용 `0건`
- 두 section: 같은 기능·제한·검증 결과

## 범위 제외

- GitHub Release title의 이중 언어화
- 이미 게시한 immutable release description 수정
- npm README 언어 정책 변경
