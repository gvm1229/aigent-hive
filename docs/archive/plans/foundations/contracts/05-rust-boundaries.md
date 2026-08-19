# 5. Rust 구현 경계

### 5.1 Crate 책임

| Crate | 책임 | 금지 |
| --- | --- | --- |
| `hive-core` | usage policy, ownership, schema-neutral invariant, orchestration reducer·scheduler | host SDK, model process launch |
| `hive-cli` | command parsing, target classification, user-facing result, port wiring | model API |
| `hive-render` | answer validation, template render, staging manifest | live uncontrolled write |
| `hive-wiki` | Markdown parse/lint, SQLite index/rebuild | canonical fact를 DB에만 저장 |
| `hive-projection` | host capability matrix, declarative envelope와 thin file projection | model/session runtime·host execution |
| `hive-update` | signature, compatibility, backup, migration, atomic activation | knowledge GC, external plugin update |

Dependency 방향:

```text
hive-core
  ↑
hive-render / hive-wiki / hive-projection
  ↑
hive-update
  ↑
hive-cli
```

Provider SDK, external orchestration package와 OMX/OMC source를 dependency graph에 추가 금지.

### 5.2 Host projection

Projection은 다음만 담당:

- common `AGENTS.md` 진입점 발견
- Hive namespaced Skill/action 노출
- approved Skill metadata와 compact routing precedence 노출
- active host의 envelope consume·receipt·cancel·lookup capability matrix 조회
- exact host event와 scoped consent가 모두 성립할 때만 Hive lifecycle adapter 노출
- signed CLI 호출
- result 표시

Projection의 소유 범위에서 model call, persistent model process, host-global config 제외. Team·goal canonical state는 Hive core 소유이며 projection은 exact declarative envelope·receipt transport만 담당. Consented hook은 project-local Hive entry와 exact run·head·epoch·one-time authority에 한정. Legacy OMX/OMC coexistence는 synthetic fixture의 foreign-byte checksum 비교만 허용.

### 5.3 Skill catalog

Implemented built-in:

- `setup-harness`
- `hive-simple-question`
- `hive-prompt-refine`
- `hive-knowledge-capture`
- `hive-knowledge-query`
- `hive-knowledge-maintenance`
- `hive-role-handoff`
- `hive-run-checkpoint`
- `hive-run-resume`
- `hive-judge-package`
- `hive-update`
- `hive-usage-guard`
- `hive-migrate`
- `hive-iterative-execution` 예정
- `hive-team-execution` 예정
- `hive-multi-goal` 예정

포함 금지:

- OMX/OMC alias
- provider API client
- model·subagent process launcher

Optional third-party Skill 순서: quarantine→provenance 검증→사용자 개별 승인→namespaced projection. Catalog 등록이나 추천은 activation이 아니며, 승인되지 않은 Skill은 host discovery surface에 추가 금지. 승인된 model-invocable Skill은 description과 task가 명확히 일치할 때 자동 선택 가능; side effect는 capability policy와 별도 runtime approval 적용.
