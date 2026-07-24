# 5. Rust 구현 경계

### 5.1 Crate 책임

| Crate | 책임 | 금지 |
| --- | --- | --- |
| `hive-core` | source guard, ownership, schema-neutral invariant | host SDK, filesystem mutation orchestration |
| `hive-cli` | command parsing, user-facing result, port wiring | model API |
| `hive-render` | answer validation, template render, staging manifest | live uncontrolled write |
| `hive-wiki` | Markdown parse/lint, SQLite index/rebuild | canonical fact를 DB에만 저장 |
| `hive-projection` | host capability matrix와 thin file projection | model/session runtime |
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

Provider SDK, orchestration package와 OMX/OMC source를 dependency graph에 추가 금지.

### 5.2 Host projection

Projection은 다음만 담당:

- common `AGENTS.md` 진입점 발견
- Hive namespaced Skill/action 노출
- approved Skill metadata와 compact routing precedence 노출
- active host의 OMX/OMC capability resolution과 version-pinned capability matrix 조회
- external capability absent와 hook consent가 모두 성립할 때만 Hive data-integrity hook adapter 노출
- signed CLI 호출
- result 표시

Projection의 소유 범위에서 model call, persistent process, team state, external orchestration hook, global config 제외. Consented fallback hook은 project-local Hive entry만 소유하고 semantic routing·continuation은 수행 범위에서 제외. OMX/OMC coexistence는 synthetic fixture의 외부 tree 준비와 checksum 비교로 검증하며, 출하 Hive process의 foreign runtime state 관찰 방식은 제외.

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

포함 금지:

- plan clone
- Ralph clone
- team/swarm clone
- OMX/OMC alias

Optional third-party Skill은 quarantine→provenance 검증→사용자 개별 승인→namespaced projection 순서. Catalog 등록이나 추천은 activation이 아니며, 승인되지 않은 Skill은 host discovery surface에 추가 금지. 승인된 model-invocable Skill은 description이 task와 명확히 일치할 때 자동 선택할 수 있지만 side effect는 capability policy와 별도 runtime approval을 따름.
