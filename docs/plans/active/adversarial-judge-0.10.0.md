# Adversarial judge `0.10.0`

> Checklist owner: `JDG10-*`
> 공개 Skill ID: `adversarial-judge`
> 호출 방식: 사용자 또는 verified workflow의 명시적 Judge node

## 기존 기능 판정

- `package-review`: Judge package·assignment·attestation·quorum 검증 준비, Judge 실행 금지
- `iterative-execution`: terminal acceptance의 reserved independent Judge 요구, 독립 호출 단계 없음
- `hive judge`: package 생성·quorum 검증, model·agent launch 없음
- 결론: 명시적 adversarial Judge 실행 Skill 부재

## Checklist

- [x] [JDG10-001] 기존 Judge·`package-review`·`iterative-execution` 중복·결손 판정
- [ ] [JDG10-002] `adversarial-judge` canonical Skill·plugin·template·catalog와 narrow automatic description 추가
- [ ] [JDG10-003] Exact subject·risk tier·acceptance criteria·artifact/evidence digest·requester·task-agent exclusion을 결합한 adversarial Judge request·dispatch envelope schema
- [ ] [JDG10-004] `hive judge package` 결과와 verdict 이전 assignment·eligible slot 예약, clean-context evidence만 host에 전달
- [ ] [JDG10-005] Active host가 별도 adversarial Judge를 native launch하고 typed launch·result receipt 반환, Hive의 provider API·credential·direct process spawn `0건`
- [ ] [JDG10-006] Diagnostic adversarial finding과 completion-authorizing authenticated quorum 분리, elevated 2/3·critical 3/3+human 기존 신뢰 계약 재사용
- [ ] [JDG10-007] 사용자 cancel·host unsupported·Judge unavailable·assignment drift·self-judge·cross-result contamination의 fail-closed·복구 계약
- [ ] [JDG10-008] Codex·Claude·Antigravity clean-context launch fixture, 독립 identity·model·effort receipt와 package/quorum·verified-workflow node 결합 회귀 검증

## 호출 예시

```text
$aigent-hive:adversarial-judge
```

자연어 예시:

```text
이 구현의 전제·반례를 검토할 독립 Judge 실행 요청
```

## Adversarial 범위

- 누락 requirement·반례·failure path·권한 확대·data loss·rollback 결손 탐색
- 선호 verdict·task-agent reasoning·self-score·기존 Judge 결과 전달 금지
- Findings: evidence locator와 severity 포함
- PASS authority: 기존 authenticated quorum 통과 뒤에만 부여

## 자동 호출 경계

- 사용자 명시 호출: 항상 Skill route
- Verified workflow: graph의 명시적 Judge node에서만 호출
- 일반 continuation·단순 질문·형식 변경·scheduler tick·retry: 자동 호출 금지
