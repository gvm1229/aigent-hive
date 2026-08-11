# 한국어 global setup 용어 복구 계획

> Checklist owner: `KST-*`
> Target: `0.9.0-test.11`
> Scope: global user-scope setup의 한국어 질문·설명·projection 용어

## 조사 결과

- [x] [KST-001] 현재 경로·위험 범위 확인
  - Canonical: `harness/skills/setup-hive/SKILL.md`
  - Current plugin projection: `harness/plugins/aigent-hive/skills/setup-hive/SKILL.md`, canonical byte 동일
  - Signed catalog: `harness/user-setup/catalog.yml`
  - Generated user directive: `crates/hive-cli/src/user_setup.rs`
  - Korean update CLI: `crates/hive-cli/src/update_activation.rs`
  - Historical `harness/user-bases/0.9.0-test.3/`: frozen authenticated base, 수정 금지
- 현재 `setup-hive`는 한국어 대화의 고정 용어·예시가 없어 host의 즉석 직역 발생
- 확인된 부정확·불일치 후보: `Skill → 기술`, `recommended suite → 권장 모음`,
  `profile|persona|host`의 한국어/영어 혼용, generic 영어 자동 번역 위험
- 확인 범위의 Korean update CLI에는 `Skill → 기술` 같은 product-term 오역 미발견
- 현재 profile contract: `CatalogSelection.id` 단일 선택. 복합 사용자 맥락·직접 설명 동시 보존 불가

## 용어 계약

한국어 setup 대화의 product·식별자 표기는 아래 exact term 유지:

- `Aigent Hive`, `Skill`, `Wiki`, `Codex`, `Claude`, `Antigravity`, `CodexBar`, `Notion`
- command, path, schema key, Skill ID, release version, option value

명확한 일반어만 한국어 표기:

| 의미 | 승인 문구 |
| --- | --- |
| active hosts | 사용할 호스트 |
| Skill selection mode | Skill 선택 방식 |
| recommended suite | 권장 Skill 세트 |
| individual built-in Skills | 개별 내장 Skill |
| user profile | 사용자 프로필 |
| agent persona | 에이전트 페르소나 |
| usage guard | 사용량 보호 |
| daily update check | 일일 업데이트 확인 |

금지: `Skill`의 일반명사 번역 `기술`, 식별자·product name의 강제 한글화, 용어가 섞인
즉석 의역. Exact Korean question sample은 canonical `setup-hive` Skill의 byte로 유지.

## Global profile 경계

- 목적: Hive가 사용자의 배경·관심 분야·선호를 이해하기 위한 전역 기본 맥락
- 금지: 전역 profile 기반의 작업 우선순위, workflow 선택, project별 구현 방식 결정
- 입력: 복수 catalog context와 optional 사용자 설명의 동시 보존; `웹 개발자`·`게임 개발자`·
  `비개발자`는 배타적 역할이 아닌 선택 가능한 맥락
- Project scope: 현재 project의 workflow·기술 선택·delivery constraint·작업 우선순위만 결정

## Checklist

- [x] [KST-002] `setup-hive`에 한국어 interaction terminology contract와 host-independent exact
  sample 추가
- [x] [KST-003] 단일 `CatalogSelection` profile을 복수 context + optional 사용자 설명으로 교체;
  existing single profile의 무손실 migration·global user data 보존
- [x] [KST-004] catalog·`setup-hive`·generated user directive·README의 profile 질문을 사용자 맥락으로
  교체; workflow·우선순위·project별 작업 방식 문구 제거와 project scope 분리
- [x] [KST-005] canonical Skill → plugin·host projection parity와 Korean sample static regression;
  `기술`·전역 workflow 우선순위 회귀 차단과 세 host human smoke prompt 기록
- [x] [KST-006] `0.9.0-test.11` Windows actual user root의 Korean·bilingual global setup 수용;
  22개 product-only Skill, usage guard `20%`, Discord 설정, `latest=0.8.0` 유지

## 수용 기준

- `Skill 선택 방식`
  - `모든 내장 Skill 사용`
  - `개별 내장 Skill 선택`
- `Skill`을 `기술`로 번역한 setup 질문·설명 0건
- 복합 사용자: 복수 기본 맥락과 사용자 설명의 동시 입력·저장·재구성 가능
- 전역 profile: project workflow·작업 우선순위 변경 0건
- 현재 global-only scope, one-question-at-a-time, preview approval, host/projection ownership 불변
- Canonical source와 current projection exact parity
- Historical authenticated base 변경 0건
- test release만 독립 게시, `latest`는 기존 stable 유지

## 구현 증거

- `UserProfile`: 복수 `contexts`와 선택 `description`의 typed schema·semantic validation
- legacy single profile: canonical YAML 직렬화 전 무손실 user-context migration
- global guidance: 사용자 기본 맥락의 project workflow·구현 방식·작업 우선순위·Skill 선택 영향 없음
- Korean interaction: `Skill` product term 유지, `기술` 번역 금지, 항목당 한 줄 선택지
- projection: canonical `setup-hive`와 plugin projection byte 동일
- verification: `hive-cli user_setup` 32개, setup·project·static Python 70개 통과
- host smoke prompt: Codex·Claude·Antigravity 공통 `setup-hive` routing prompt와 Korean exact samples 고정; 실제 설치 수용은 `KST-006`의 numbered test release 범위
