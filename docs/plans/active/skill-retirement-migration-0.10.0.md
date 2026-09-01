# Skill rename·폐기 cleanup `0.10.0`

> Checklist owner: `SKM10-*`
> 적용 범위: 게시된 모든 stable과 지원 predecessor에서 `0.10.0` direct upgrade
> 성공 invariant: retired Skill·projection artifact `0건`

## Rename·merge mapping

```text
ralph-loop          → verified-workflow
iterative-execution → verified-workflow
package-review      → judge-evidence
```

기존 alias가 위 이름을 가리키는 경우 최종 canonical 이름으로 직접 mapping. Retired-name
chain과 cycle 금지.

## Checklist

- [x] [SKM10-001] 현재 cleanup 경로 조사: authenticated retired Skill exact-byte 제거·empty directory prune 보유, user projection historical registry는 `0.9.0`까지만 포함
- [x] [SKM10-002] 게시 stable `0.8.0|0.9.0|0.9.1|0.9.2|0.9.3|0.9.4|0.9.5`가 exact Skill compatibility epoch로 모두 resolve되도록 name·digest·side-effect·capability와 stable-to-epoch mapping 영구 보존 — `354ea0a`; release tag의 `active-skills.yml` 기반 registry 7 stable 수용·`hive-projection` history regression 통과
- [x] [SKM10-003] Skill lifecycle ledger에 `introduced_in|renamed_in|deprecated_in|removed_in|replacement|transition_kind` 기록, 모든 과거 alias를 `verified-workflow|judge-evidence` 최종 canonical ID로 직접 수렴, chain·cycle·current-name collision `0건` — `1cc59d8`; 세 `0.10.0` transition의 version lifecycle·직접 canonical mapping·current-name collision fail-closed 검증
- [x] [SKM10-004] Codex·Claude·Antigravity의 user plugin·user projection·project projection·active-Skill ledger·ownership manifest 전체 retired path inventory — `2f82bc1`, `f494053`; host별 source·projection artifact table
- [x] [SKM10-005] `hive update` direct jump dry-run에서 source version과 무관하게 authenticated retired file·directory·manifest entry의 exact 제거 계획 표시 — `f494053`; 18개 `0.9.x` stable·host 조합 dry-run 통과
- [x] [SKM10-006] Exact historical bytes는 자동 제거, authenticated base+local edit는 safe merge 또는 conflict, foreign·unknown bytes는 삭제 없이 conflict; conflict 상태의 `0.10.0` activation 금지 — `hive-cli` user install 91개 회귀 통과
- [x] [SKM10-007] 새 canonical Skill activation과 retired artifact 제거의 단일 journal·rollback boundary, nested empty directory prune와 final ownership closure 검증 — deletion journal·실패 rollback·recovery·empty directory 회귀 통과
- [x] [SKM10-008] Upgrade matrix: `0.7.0`, `0.8.0`, `0.9.0`, `0.9.1`, `0.9.2`, `0.9.3`, `0.9.4`, `0.9.5`, 공개 test predecessor의 direct `0.10.0` upgrade — stable tag exact plugin base와 기존 test predecessor 인증 통과
- [x] [SKM10-009] 세 host의 clean·modified·missing·foreign·interrupted upgrade와 rollback·reinstall·uninstall 뒤 retired discovery `0건`, canonical·user bytes 보존 — Rust 91·38, Python 53 통과
- [x] [SKM10-010] 향후 stable publication 전 Skill transition event 또는 exact no-change epoch proof append·immutable prior entry·npm/GitHub stable ledger parity를 검증하고 누락 시 publication 차단 — `0b8328d`; stable Skill ledger와 publication gate
- [x] [SKM10-011] `0.10.0` stable Skill snapshot·changed compatibility epoch·projection historical coverage와 `test.12` 공개 수용

## 성공·충돌 계약

성공한 upgrade:

- Old Skill file·agent descriptor·host projection·active ledger entry `0건`
- Retired leaf directory와 Hive-created empty ancestor 제거
- 새 canonical Skill 1개만 discoverable
- Saved selection·routing preference의 canonical ID migration

충돌 upgrade:

- Foreign·modified bytes 삭제 `0건`
- 새 release activation `0건`
- Exact path·observed digest·expected historical digest·해결 방법 반환
- Prior installation byte 보존과 retry 가능한 journal

## Direct jump 원칙

- `0.9.2 → 0.10.0`: 중간 release 순차 설치 비의존
- Running `0.10.0` updater: compiled historical registry·release surface·retired-name ledger로 source artifact 직접 인증과 최종 closure 수렴

## Stable registry invariant

- 현재 공개 stable 집합: npm `0.8.0`, npm·GitHub `0.9.0–0.9.5`
- Historical registry stable 집합: 공개 stable 집합과 exact equality
- 과거 stable entry 수정·삭제 금지
- 새 stable publication: 게시 전 transition event 또는 exact no-change epoch proof append 필수
- Test release: accepted predecessor upgrade fixture로 별도 관리, stable registry 대체 금지
- Stable discovery: npm non-prerelease version과 GitHub non-prerelease Release의 append-only 합집합
- Registry의 이후 unpublish·삭제: historical coverage 제거 권한 없음
- Stable Skill이 name·digest·side-effect·capability·projection path에서 이전 stable과 exact 동일: 같은 epoch 공유 가능, stable coverage entry 생략 금지

## 확인된 stable transition

| Transition | Skill 변화 | Registry 결과 |
| --- | --- | --- |
| `0.8.0 → 0.9.0` | 16개 legacy 이름 제거, 21개 canonical 이름 도입 | Rename·removal event와 별도 epoch 필수 |
| `0.9.0 → 0.9.1` | `knowledge-capture|knowledge-maintain` content 변경 | 새 digest epoch |
| `0.9.1 → 0.9.2` | `usage-guard` content 변경 | 새 digest epoch |
| `0.9.2 → 0.9.3` | 네 Skill 추가와 기존 8개 content 변경 | Introduction event와 새 epoch |
| `0.9.3 → 0.9.4` | 26개 Skill content 변경 | 새 digest epoch |
| `0.9.4 → 0.9.5` | Exact Skill 변화 `0건` | `0.9.4` epoch 공유와 `0.9.5` coverage entry |
