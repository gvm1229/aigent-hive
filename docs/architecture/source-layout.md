# Source 구조

## 경계

```text
Hive source ──build/test──> release bundle ──setup/update──> consumer harness
```

- Hive source: 이 Git 저장소
- release bundle: 컴파일된 Rust binary, schema, template pack, projection metadata
- consumer harness: 사용자의 독립 프로젝트에 생성되는 로컬 파일

세 tree는 서로를 runtime path로 역참조하지 않는다.

라이선스 경계도 같은 분리를 따른다. Hive source의 기본 라이선스는 `GPL-3.0-only`지만 `harness/**`와 그로부터 생성된 Aigent Hive 소유 material은 `Apache-2.0`이다. 생성된 harness는 `.hive/` 안에 자체 라이선스 전문을 두며 소비자 프로젝트 root의 license를 변경하지 않는다.

## 현재 구조

```text
aigent-hive/
├── AGENTS.md
├── .agents/                    # Hive 개발 지침, 출하 금지
├── crates/
│   ├── hive-core/              # provider-neutral invariant
│   └── hive-cli/               # 현재 doctor/check-target
├── harness/
│   ├── template/               # Copier authoring·CI source
│   ├── skills/                 # portable shipping Skill source
│   ├── projections/            # host별 얇은 projection 확장점
│   ├── profiles/               # domain profile 확장점
│   └── manifest.toml           # ownership·금지 경로
├── schemas/
├── tests/                     # schema/render/materializer conformance
├── docs/
├── LICENSES/                  # GPL-3.0-only·Apache-2.0 전문
├── REUSE.toml                 # file-scope license mapping
├── copier.yml
└── hive-source.json            # consumer setup 거부 marker
```

## Source `.agents`와 출하물

루트 `.agents/`는 Hive 자체를 개발하는 에이전트 전용이다. 일부 external runtime이 `.agents/skills`를 자동 탐색할 수 있으므로 출하용 Skill을 루트 `.agents/skills`에 두지 않는다.

출하용 Skill과 directive는 `harness/`에서만 관리하고 release projection 단계에서 소비자 경로를 결정한다.
Role lifecycle과 Skill consent의 normative contract는 각각
[`role-lifecycle.md`](role-lifecycle.md)와 [`skill-consent.md`](skill-consent.md)에 둔다.

## Crate 추가 원칙

빈 crate를 미리 만들지 않는다. 다음 acceptance를 구현할 때 owning crate 추가:

- renderer 계약 확정 → `hive-render`
- Markdown/SQLite index 구현 → `hive-wiki`
- staged update와 migration 구현 → `hive-update`
- host projection compile 구현 → `hive-projection`

crate 이름만으로 미구현 capability를 지원하는 것처럼 보이게 하지 않는다.
