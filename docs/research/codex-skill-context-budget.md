# Codex Skill context budget 진단

- 기준일: 2026-07-28
- Codex: `0.145.0`
- 경고: `Skill descriptions were shortened to fit the 2% skills context budget`
- 근거: 현재 session Skill catalog와 local Codex binary telemetry 문자열

## 계측 결과

| 범위 | Skill entry | 설명 문자 | 설명 단어 |
| --- | ---: | ---: | ---: |
| 전체 | 110 | 26,669 | 3,823 |
| Hive 전체 | 37 | 12,067 | 1,702 |
| OMX | 29 | 2,806 | 377 |
| 기타 plugin·OpenAI bundle | 44 | 11,796 | 1,744 |

Hive 세부:

- Prefix 정규화 뒤 고유 Skill 이름 약 19개
- Global·plugin·source projection 사이 중복 entry 18개
- `hive-prompt-refine`와 `hive-usage-guard` 각 3개 노출
- 전체 설명 문자의 약 45%를 차지하는 Hive entry
- 250K context의 2%에 해당하는 약 5K token metadata 한도 추정

## 원인 판정

- 단일 Hive Skill의 과도한 본문이 아닌 전체 enabled Skill metadata 합산 문제
- 설치된 모든 Skill의 이름·설명·경로를 implicit discovery context에 넣는 host 동작
- 동일 Hive Skill의 global·plugin·source 동시 discovery
- 명시적 호출 전용 long-tail Skill의 implicit metadata 노출
- 여러 third-party plugin 동시 활성화에 따른 비-Hive 기여

결론:

- Hive 단독 원인 아님
- Hive가 가장 큰 제거 가능 기여자
- 타 plugin 자동 비활성화가 아닌 Hive projection 자체 budget 통제 필요

## 수정 계획

1. Host별 projected Skill metadata audit 추가
   - canonical 이름, discovery path, implicit 여부, 설명 길이, 중복 identity
   - Hive 소유 implicit metadata budget 초과 시 conformance failure
2. Codex의 canonical Hive Skill당 implicit projection 1개
   - Plugin-native projection 우선
   - Global compatibility mirror와 source 중복의 explicit-only 전환
3. Long-tail Skill의 `allow_implicit_invocation: false`
   - update, migrate, judge package, checkpoint·resume, role handoff,
     project upgrade, knowledge maintenance·promotion
4. 자동 routing 최소 집합
   - setup, usage guard, prompt refine intent, simple question 경계
   - 설치 상태와 implicit context injection의 분리
5. 설명 metadata 압축
   - Trigger와 negative boundary 중심
   - implicit Skill 약 240자 이하, explicit-only Skill 약 160자 이하 목표
6. Projection conformance
   - 정규화 canonical identity의 implicit 중복 0건
   - `All` 설치에서도 explicit-only Skill의 `$name` 사용 가능
7. Fresh Codex session qualification
   - Hive 중복·초과 기여 0건
   - 전체 경고 잔존 시 third-party 기여를 별도 표시

## 수용 기준

- Hive canonical Skill당 implicit entry 최대 1개
- Hive implicit metadata의 정해진 budget 이하
- Explicit-only Skill의 직접 `$name` 호출 유지
- Setup의 `All` 의미를 전체 설치로 유지하되 전체 implicit 주입 의미에서 분리
- 사용자 소유 third-party plugin 자동 비활성화 0건
- Fresh session warning과 telemetry 결과의 release evidence 기록
