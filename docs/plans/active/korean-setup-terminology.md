# 한국어 global setup 용어 복구 계획

> Checklist owner: `KST-*`
> Target: 다음 독립 test release
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

## Checklist

- [ ] [KST-002] `setup-hive`에 한국어 interaction terminology contract와 host-independent exact
  sample 추가
- [ ] [KST-003] catalog·generated user directive·README의 Korean setup 표기를 용어 계약에 맞춰
  정리; historical frozen base byte 보존
- [ ] [KST-004] canonical Skill → plugin·host projection parity와 Korean sample static regression;
  `기술` 회귀 차단과 세 host human smoke prompt 기록
- [ ] [KST-005] 새 numbered test candidate·publication으로 설치된 global setup Korean flow 수용;
  stable `latest` mutation 없음

## 수용 기준

- “Skill 선택 방식을 골라 주세요” 이후 `권장 Skill 세트`·`개별 내장 Skill` 표기
- `Skill`을 `기술`로 번역한 setup 질문·설명 0건
- 현재 global-only scope, one-question-at-a-time, preview approval, host/projection ownership 불변
- Canonical source와 current projection exact parity
- Historical authenticated base 변경 0건
- test release만 독립 게시, `latest`는 기존 stable 유지
