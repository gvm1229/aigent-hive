# Stage 5. 지속형 역할과 Hive-native orchestration

> 상태: role/run baseline implemented; orchestration 확장은 NAT-* 소유

#### 완성 후 동작

지속형 팀원은 setup-time materialization을 거친 `.hive/team/roles/<role-id>.md`로 표현:

- 담당 범위와 제외 범위
- context selector
- 허용 tool과 write scope
- 현재 assignment
- handoff와 verification 책임

Role identity는 session이 종료되어도 유지. 새 host session 또는 subagent가 같은 role document와 current handoff를 받아 역할을 계속 수행.

실행 소유권:

| 조건 | Control owner | Executor | 경계 |
| --- | --- | --- | --- |
| 현재 v0.9 baseline | Host-native prepare-only | Active host | Hive model process 없음 |
| NAT activation 뒤 | Hive event·scheduler·team·goal state | Active host | Provider API·direct process spawn 없음 |
| Legacy external run | Historical owner provenance | 원본 owner | 신규 dispatch 없음, explicit migration만 허용 |

Host capability 부족: `unsupported|dispatch-uncertain`, hidden fallback 없음.

#### 구현

- `.hive/team/roles/<role-id>.md`의 YAML frontmatter는 `schemas/role-profile.schema.json`으로 검증하고 Markdown body는 bounded handoff context로 사용
- task brief에는 최소 required context와 exact acceptance만 포함
- result는 `schemas/action-result.schema.json`으로 changed artifact, evidence, blocker와 next action을 정규화
- session ID는 optional runtime binding이며 role identity로 사용 금지
- host/version/surface qualification은 `schemas/capability-matrix.schema.json`으로 기록
- Host qualification은 envelope consume·typed receipt·cancel·lookup evidence로 결정
- Hive의 `omx|omc` command·foreign state mutation 없음
- Team lifecycle은 canonical event·mailbox·barrier·lease와 host envelope로 구성
- Lifecycle hook은 exact target·head·epoch·one-time authority에 한정
- non-clobber conformance fixture가 외부 namespace의 before/after checksum을 계산하며 Hive 제품 자체는 그 namespace를 읽기 금지

#### 완료 조건

- [x] session 교체 후 role 책임·assignment·handoff 복구
- [x] role document 없이 permanent team member claim 불가
- [x] V9 baseline의 external automatic priority·duplicate orchestration Skill 0개
- 후속 `NAT-016`: 세 host envelope·receipt·cancel adapter qualification
- 후속 `NAT-017–019`: iterative·team·multi-goal Skill qualification
- [x] fixture-side external namespace checksum 불변이고 Hive process의 foreign namespace read/write 0회
