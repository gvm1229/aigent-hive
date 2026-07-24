# Stage 5. 지속형 역할과 host-owned orchestration

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

지속형 팀원은 setup-time materialization을 거친 `.hive/team/roles/<role-id>.md`로 표현:

- 담당 범위와 제외 범위
- context selector
- 허용 tool과 write scope
- 현재 assignment
- handoff와 verification 책임

Role identity는 session이 종료되어도 유지. 새 host session 또는 subagent가 같은 role document와 current handoff를 받아 역할을 계속 수행.

새 run의 실제 실행 owner는 사용자 선택 없이 다음 precedence로 resolve:

| 조건 | 실행 소유자 | Hive가 제공 | Hive가 제공 금지 |
| --- | --- | --- | --- |
| Codex + compatible OMX available | OMX | canonical project context와 coexistence contract | plan/Ralph/team 복제, OMX state 조작 |
| Claude + compatible OMC available | OMC | canonical project context와 coexistence contract | plan/Ralph/team 복제, OMC state 조작 |
| Codex/Claude `absent|incompatible|unknown`, 또는 Antigravity | host-native capability | role/run document, bounded brief, result schema | scheduler, model process |

Host-native capability가 부족하면 해당 기능은 `unsupported`. Hive가 hidden fallback을 생성 금지.

#### 구현

- `.hive/team/roles/<role-id>.md`의 YAML frontmatter는 `schemas/role-profile.schema.json`으로 검증하고 Markdown body는 bounded handoff context로 사용
- task brief에는 최소 required context와 exact acceptance만 포함
- result는 `schemas/action-result.schema.json`으로 changed artifact, evidence, blocker와 next action을 정규화
- session ID는 optional runtime binding이며 role identity로 사용 금지
- host/version/surface qualification은 `schemas/capability-matrix.schema.json`으로 기록
- owner resolution은 active host capability metadata와 public executable path/`--version` evidence로 결정하고 run status에 digest와 함께 pin
- Hive가 `omx setup/update`, `omc setup/update`, team lifecycle 또는 foreign state를 호출 금지
- resolved owner가 OMX/OMC이면 Hive hook과 duplicate orchestration Skill projection 0개; canonical role/run data Skill은 공존 가능
- resolved owner가 host-native여도 fallback hook은 Hive data integrity guard만 제공하고 orchestration 기능을 추가 금지
- non-clobber conformance fixture가 외부 namespace의 before/after checksum을 계산하며 Hive 제품 자체는 그 namespace를 읽기 금지

#### 완료 조건

- [x] session 교체 후 role 책임·assignment·handoff 복구
- [x] role document 없이 permanent team member claim 불가
- [x] compatible external layer detected 시 Hive orchestration command/hook와 duplicate orchestration Skill 0개
- [x] resolved external runtime이 run 도중 실패하면 host-native로 자동 전환 금지
- [x] 새 run은 `absent|incompatible|unknown`에서 truthful host-native로 resolve하고 fallback hook은 conclusive `absent`에서만 허용
- [x] fixture-side external namespace checksum 불변이고 Hive process의 foreign namespace read/write 0회
