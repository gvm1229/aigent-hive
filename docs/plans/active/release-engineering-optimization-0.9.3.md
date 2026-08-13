# `0.9.3` 출시 운영 최적화

> Checklist owner: `OPT93-*`
> 대상: `0.9.3`
> 범위: 안전성 유지, 중복 CI·후보·계획 gate 제거

## 원칙

- 변화 위험도에 맞는 최소 검사 후 release candidate에서 다중 플랫폼·설치 수용
- 최신 commit만 PR CI authority, publication workflow 취소 금지
- 같은 product tree·package input의 candidate 재생성 금지
- release checklist는 고유한 transition만 소유, 선행 구현 evidence는 ID 참조만 사용

## Checklist

- [x] [OPT93-001] PR·branch CI 최신 commit concurrency와 obsolete run 취소
- [x] [OPT93-002] Markdown-only change의 documentation lane 단일 실행과 non-Markdown risk lane 분리
- [x] [OPT93-003] Rust·pip dependency cache와 Linux full conformance·macOS/Windows smoke 분리
- [x] [OPT93-004] push-triggered release-runtime 제거, schedule·manual qualification과 exact candidate reuse 경계
- [x] [OPT93-005] release workflow·plan directive의 single-owner evidence·no duplicate candidate enforcement

## 수락 기준

- 문서-only integration에서 Rust·cross-platform·release runtime automatic run `0건`
- product change는 named risk lane과 host smoke 실행, numbered test candidate는 exact product byte 변경 때만 생성
- stable candidate는 accepted public test artifact를 rebuild 없이 publication workflow로 승격
