# Global Skill 선택 단순화 계획

> Checklist owner: `GSS-*`
> Target: 다음 독립 test release
> Scope: global user-scope built-in Skill 기본값·개별 토글·목록 표기

## 조사 결과

- [x] [GSS-001] 현재 `recommended_skill_suites` 3개가 profile과 연결된 구조 확인
- [x] [GSS-002] game-developer 묶음의 `hive-usage-guard` 누락 확인
- [x] [GSS-003] 개별 선택 prompt의 Skill 목록이 comma-separated paragraph로 출력되는 문제 확인

## 결정

- 모든 built-in Skill: 전역 setup 기본 활성화
- `recommended` mode·profile 기반 Skill 묶음: 제거
- 선택 변경: Skill별 on/off 토글; `setup-hive`는 필수 유지
- optional third-party Skill: 기존 explicit approval 경계 유지
- 모든 사용자 대면 목록: 항목당 Markdown 한 줄
- 기존 `recommended` 설정: recorded closure 보존, all-built-in 전환은 preview·명시 approval 이후

## Checklist

- [x] [GSS-004] `recommended_skill_suites`·`SkillSelectionMode::Recommended` 제거와 `all|individual`
  typed config·schema·catalog·saved answer migration 구현
- [x] [GSS-005] global setup 기본 all-built-in·per-Skill toggle 질문·한 줄 목록·dependency preview 구현
- [x] [GSS-006] canonical Skill·plugin·host projection·README·generated directive 동기화;
  profile과 Skill 선택 연결 0건
- [x] [GSS-007] existing user configuration migration·all/individual toggle·one-entry-per-line static·Rust·host
  smoke regression
- [ ] [GSS-008] 새 numbered test release의 global Skill selection 수용; `latest` mutation 없음

## 수용 기준

- 새 global setup: 모든 built-in Skill 활성화 기본값
- 선택 변경: Skill 이름별 독립 on/off, `setup-hive` 유지
- profile 선택과 활성 Skill set의 결합 0건
- user-facing Skill·dependency 목록: 항목당 한 줄, comma-separated 선택 목록 0건
- existing recommended user configuration: preview 없는 Skill 추가 0건

## 구현 증거

- `a30eb47`: global catalog·schema·`SkillSelectionMode`의 `all|individual` 전환
- legacy `recommended`: 저장된 동일 answer의 validate만 과거 closure 해석, 새 answer 거부,
  apply 전 dry-run preview 유지
- project 추천 세트: `harness/project-setup/skill-suites.yml`로 분리, global profile 결합 없음
- 검증: `hive-cli` user setup 30개, `hive-render` 66개, user/project·connected·static·v0.9
  Python 69개 통과
- release 대기: `0.9.0-test.4` candidate `31125304895`는 Linux x86_64 hosted runner를
  15분 28초 동안 배정하지 못해 publish 전 중단. 동일 source 재시도 `31125945638`도 모든
  hosted runner가 배정되지 않아 취소. 2026-08-07 GitHub Status API는 Actions를
  `major_outage`·critical investigating incident로 보고했으며, npm·tag·GitHub Release·`latest`
  mutation은 모두 0건. Actions 복구 뒤 exact `548983d`로 candidate를 재실행하고 성공한
  candidate ID를 사용해 test publication을 dispatch한다.
