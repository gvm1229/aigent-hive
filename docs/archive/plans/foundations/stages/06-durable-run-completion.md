# Stage 6. Durable run과 100% completion

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

장기 작업은 `.hive/runs/<run-id>/`에 저장:

```text
PLAN.md
STATUS.md
HANDOFF.md
evidence/
```

`PLAN.md`:

- 목표와 범위
- 필수 acceptance checklist
- task dependency와 owner role
- verification method
- 위험 tier
- stop/block condition

`STATUS.md`:

- current revision
- 완료·진행·blocked task
- active role binding
- next action
- latest evidence locator
- resume note

`HANDOFF.md`는 한 run의 active role별 bounded Markdown entry를 canonical shared
envelope로 저장하며 다른 role entry를 덮어쓰기 금지.

사용자가 “100% 완료까지 계속”을 요청하면 resolved runtime이 plan의 미통과 criterion을 기준으로 execute→verify→repair를 지속. Harness는 durable run contract를 제공하고 runtime loop 자체는 구현 금지.

#### 구현

- checkbox와 criterion ID를 parser로 검증
- `STATUS.md` YAML frontmatter를 `schemas/run-status.schema.json`으로 검증
- 완료율은 필수 criterion PASS 수만 계산
- passed criterion마다
  `.hive/runs/<run-id>/evidence/<safe-file>#sha256:<digest>` exact locator 검증
- transcript 대신 bounded status/handoff 저장
- compaction, session 종료, handoff 전 `STATUS.md` 갱신
- 같은 artifact/evidence hash의 중복 result 멱등 처리
- runtime이 지속 loop를 지원하지 않으면 `resume-ready`까지만 제공하고 unattended completion을 지원했다고 표시 금지
- manual resume는 unenforced provider-neutral brief를 만들고, automatic resume는 fresh
  usage permit을 brief 준비 closure 직전에 한 번 소비한 경우에만 brief를 반환
- 어느 resume 경로도 process/subagent를 spawn 금지

#### 완료 조건

- [x] criterion 하나가 unchecked/failed/unverified면 성공 불가
- [x] fresh session이 PLAN+STATUS+evidence만으로 next action 복구
- [x] transcript 없이 재개 가능
- [x] 안전·승인·usage blocker에서 무한 반복 금지
- [x] resolved runtime capability와 실제 제품 표시 일치
