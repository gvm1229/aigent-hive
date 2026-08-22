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
- [ ] [SKM10-002] Historical built-in registry에 게시된 stable `0.9.0|0.9.1|0.9.2|0.9.3|0.9.4|0.9.5`의 exact Skill name·digest·side-effect·capability를 모두 영구 보존, stable release ledger·surface inventory와 집합 parity
- [ ] [SKM10-003] Retired-name ledger의 모든 과거 alias를 `verified-workflow|judge-evidence` 최종 canonical ID로 직접 수렴, chain·cycle·current-name collision `0건`
- [ ] [SKM10-004] Codex·Claude·Antigravity의 user plugin·user projection·project projection·active-Skill ledger·ownership manifest 전체 retired path inventory
- [ ] [SKM10-005] `hive update` direct jump dry-run에서 source version과 무관하게 authenticated retired file·directory·manifest entry의 exact 제거 계획 표시
- [ ] [SKM10-006] Exact historical bytes는 자동 제거, authenticated base+local edit는 safe merge 또는 conflict, foreign·unknown bytes는 삭제 없이 conflict; conflict 상태의 `0.10.0` activation 금지
- [ ] [SKM10-007] 새 canonical Skill activation과 retired artifact 제거의 단일 journal·rollback boundary, nested empty directory prune와 final ownership closure 검증
- [ ] [SKM10-008] Upgrade matrix: `0.7.0`, `0.8.0`, `0.9.0`, `0.9.1`, `0.9.2`, `0.9.3`, `0.9.4`, `0.9.5`, 공개 test predecessor의 direct `0.10.0` upgrade
- [ ] [SKM10-009] 세 host의 clean·modified·missing·foreign·interrupted upgrade와 rollback·reinstall·uninstall 뒤 retired discovery `0건`, canonical·user bytes 보존
- [ ] [SKM10-010] 향후 stable publication 전 current Skill snapshot의 historical registry append·immutable prior entry·GitHub non-prerelease stable ledger parity를 검증하고 누락 시 publication 차단

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

- 현재 공개 stable 집합: `0.9.0–0.9.5`
- Historical registry stable 집합: 공개 stable 집합과 exact equality
- 과거 stable entry 수정·삭제 금지
- 새 stable publication: 게시 전 current Skill catalog snapshot append 필수
- Test release: accepted predecessor upgrade fixture로 별도 관리, stable registry 대체 금지
