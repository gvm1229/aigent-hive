# 2. Artifact와 source 구조

### 2.1 세 artifact

| Artifact | 정본 | 포함 | 금지 |
| --- | --- | --- | --- |
| Hive source | 이 Git 저장소 | Rust, schema, Copier source, Skill source, projection, fixture, docs | 실제 consumer state, credential |
| Release bundle | GitHub Release | signed binary, compiled template pack, schema, migration, manifest, provenance | source-only directive, mutable user data |
| Consumer harness | 사용자의 독립 프로젝트 | `.hive`, shared marker, approved projection, Markdown data | Hive source tree, plugin cache-only 정본 |

`hive-source.json`가 있는 target에는 consumer setup을 실행 금지. Source root의 `.agents/`는 Hive 개발 전용이고 `harness/`만 출하 source.

### 2.2 Durable state 정본

| Data class | 정본 format·위치 | SQLite 역할 |
| --- | --- | --- |
| knowledge | `.hive/knowledge/Raw`, `Wiki`, `Schema`의 tracked Markdown 또는 작은 원본 object | FTS, tag, alias, link, content-hash index |
| role identity | `.hive/team/roles/*.md` | 검색용 projection만 허용 |
| run plan/status/evidence manifest | `.hive/runs/**/*.md` | 검색·집계 cache만 허용 |
| setup answers | `.hive/setup-answers.yml` | 사용 금지 |
| typed config·role seed·knowledge scope | `.hive/config/*.{toml,yml}` | 사용 금지 |
| optional Skill approval | `.hive/config/approved-skills.yml` | 사용 금지 |
| fallback hook approval | `.hive/config/approved-hooks.yml` | 사용 금지 |
| deleted-content suppression | `.hive/knowledge/suppression.yml` | re-ingest filter projection 가능 |

Markdown body가 유리한 narrative state와 typed YAML/TOML이 유리한 configuration·consent state 분리. 둘 다 tracked canonical source. SQLite 역복구 금지. 새 machine checkout에서는 tracked tree만으로 model call·network 없이 SQLite 재구축 가능.

### 2.3 Source workspace 목표 구조

```text
aigent-hive/
├── AGENTS.md
├── .agents/                       # Hive 개발 지침
├── crates/
│   ├── hive-core/                 # invariant와 ownership
│   ├── hive-cli/                  # user command
│   ├── hive-render/               # Phase 1
│   ├── hive-wiki/                 # Phase 2
│   ├── hive-projection/           # Phase 3
│   └── hive-update/               # Phase 6
├── harness/
│   ├── template/                  # Copier와 Rust가 공유하는 canonical template
│   ├── skills/                    # portable shipping Skill
│   ├── projections/               # host별 thin projection
│   ├── profiles/                  # general/custom과 검증된 domain 확장점
│   └── manifest.toml              # path ownership
├── schemas/
├── tests/
│   ├── fixtures/
│   ├── conformance/
│   └── work/                      # ignored disposable output
├── docs/
│   ├── plans/PLAN.md
│   ├── state/CURRENT.md
│   ├── decisions/
│   ├── architecture/
│   ├── research/
│   └── guides/
├── copier.yml
└── hive-source.json
```

빈 crate를 미리 생성 금지. 구현과 acceptance가 함께 시작될 때 owning crate를 추가.

### 2.4 Source tracking

Git 추적:

- Rust source와 Cargo manifest/lock
- template, projection, profile, schema
- synthetic fixture와 normalized expected output
- `.agents`, `AGENTS.md`, thin host redirect
- plan, ADR, current state와 research
- CI와 release recipe

Git 제외:

- `target/`, `dist/`, `artifacts/`
- `.omx/`, `.omc/`, `.codex/`, `.claude/`
- `.agents/work/`
- `tests/work/`
- SQLite, WAL, SHM
- local backup, cache, temp file
- credential와 signing private key
