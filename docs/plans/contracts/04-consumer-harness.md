# 4. Consumer harness 구조

```text
consumer-project/
├── AGENTS.md                         # shared file, Hive marker only
└── .hive/
    ├── setup-answers.yml             # tracked, non-secret
    ├── config/
    │   ├── harness.toml              # tracked
    │   ├── role-seeds.yml            # tracked setup projection
    │   ├── knowledge-scope.yml       # tracked setup projection
    │   ├── approved-skills.yml       # tracked Skill consent ledger
    │   └── approved-hooks.yml        # tracked fallback hook consent ledger
    ├── team/
    │   └── roles/*.md                # tracked
    ├── runs/
    │   └── <run-id>/                 # tracked unless user marks confidential
    ├── knowledge/
    │   ├── Raw/                      # tracked, non-confidential only
    │   ├── Wiki/                     # tracked
    │   ├── Schema/                   # tracked
    │   └── suppression.yml           # tracked, no deleted prose
    ├── index/
    │   └── hive.sqlite               # ignored, rebuildable
    └── backups/                      # ignored, maximum 7 days
```

Consumer project는 독립 `.gitignore`를 소유. Hive 기본 권장은 canonical
non-confidential Markdown/YAML/TOML과 Raw source object를 모두 추적하고
SQLite/WAL/SHM/journal, index stale/lock/temp, generated backup과
`.hive/runtime/current-capability-resolution.json` 같은 ephemeral runtime evidence만
제외하는 것.

Host가 발견하는 Skill과 consented fallback hook은 이 canonical config에서 thin projection. Host-specific project path는 ownership manifest에 별도로 열거하며, external runtime detected 상태에서는 fallback hook projection이 존재하지 않아야 함.
