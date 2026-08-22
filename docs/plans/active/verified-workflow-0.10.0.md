# Verified workflow `0.10.0`

> Checklist owner: `VWF10-*`
> 이전 이름: `ralph-loop`
> 역할: 증거·의존성·bounded retry·독립 검증이 필요한 실행 graph
> 병합 source: `ralph-loop`, `iterative-execution`

## Checklist

- [x] [VWF10-001] Canonical Skill ID `verified-workflow`·한국어 표시 `검증형 작업 흐름`과 `iterative-execution` protocol 병합 유지보수자 승인
- [ ] [VWF10-002] `ralph-loop` graph 계약과 `iterative-execution` receipt·dispatch-uncertain·budget·cancel·recovery 계약을 하나의 `verified-workflow` canonical Skill로 병합
- [ ] [VWF10-003] 자연어 continuation의 자동 routing: dependency·중간 evidence gate·bounded retry·독립 verifier·steering·recovery 중 2개 이상, reason code와 `simple|verified-workflow|required-but-unsupported` 결과
- [ ] [VWF10-004] 작업 길이·bare `continue`만으로 자동 선택 금지, `간단한 continuation|검증형 workflow|retry 없음` 사용자 override
- [ ] [VWF10-005] Host Goal·task를 outer owner로 유지하고 verified workflow graph를 nested execution contract로 결합, Hive의 model·subagent process spawn `0건`
- [ ] [VWF10-006] `SKM10-*` cleanup 계약을 통한 기존 `ralph-loop|iterative-execution` run·setup·update·projection 무손실 migration과 three-host routing·복구 검증

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
