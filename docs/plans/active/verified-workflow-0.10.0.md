# Verified workflow `0.10.0`

> Checklist owner: `VWF10-*`
> 이전 이름: `ralph-loop`
> 역할: 증거·의존성·bounded retry·독립 검증이 필요한 실행 graph

## Checklist

- [x] [VWF10-001] Canonical Skill ID `verified-workflow`와 한국어 표시 `검증형 작업 흐름` 유지보수자 승인
- [ ] [VWF10-002] `harness/skills/ralph-loop` canonical source·plugin·template·catalog를 `verified-workflow`로 rename, `ralph-loop` one-release migration alias와 retired-name ledger 추가
- [ ] [VWF10-003] 자연어 continuation의 자동 routing: dependency·중간 evidence gate·bounded retry·독립 verifier·steering·recovery 중 2개 이상, reason code와 `simple|verified-workflow|required-but-unsupported` 결과
- [ ] [VWF10-004] 작업 길이·bare `continue`만으로 자동 선택 금지, `간단한 continuation|검증형 workflow|retry 없음` 사용자 override
- [ ] [VWF10-005] Host Goal·task를 outer owner로 유지하고 verified workflow graph를 nested execution contract로 결합, Hive의 model·subagent process spawn `0건`
- [ ] [VWF10-006] 기존 `ralph-loop` run·setup·update·projection 무손실 migration과 Codex·Claude·Antigravity routing·복구 회귀 검증

## 자동 선택 기준

```text
strong signals >= 2
AND host capability verified
AND user did not opt out
```

Strong signal:

- 세 단계 이상의 실제 dependency
- 중간 evidence가 다음 edge의 조건
- node별 retry·backoff·반복 failure fingerprint 제한
- 실행자와 별도 verifier
- topology steering audit
- session·compaction 뒤 exact node recovery

## 역할 경계

- Natural continuation: 전체 run 지속·closure·cancel
- `verified-workflow`: 복잡한 내부 graph·retry·evidence·verification
- Host: Goal·task 실행과 native agent launch
- Stop hook: bounded accidental-stop nudge

