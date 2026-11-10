# Nested project knowledge scan `0.10.0`

> Checklist owner: `SCP10-003`
> Release 결정: `0.9.5`로 `0.9.x` 종료, `0.9.6` 미출시

## Checklist

- [x] [SCP10-003] 상위 Git repository와 다른 registered project root의 knowledge scan 허용, 등록 root 밖 sibling read·write 차단, 전역 `safe.directory` mutation `0건`, symlink·junction·reparse point 탈출 거부, nested Vault 회귀 검증 — `7aab389`; nested target inventory와 foreign sibling sentinel 불변 회귀 시험, `cargo test -p hive-cli knowledge_scan::tests` 20건, `cargo test -p hive-cli` 393 unit·3 upgrade·1 version, `cargo clippy -p hive-cli -- -D warnings` 통과

## 수락 fixture

```text
parent-vault/
├── registered-project/
│   ├── Sources/
│   ├── Knowledge/
│   └── Maps/
└── foreign-sibling/
    └── sentinel
```

- Registered project allowlist만 inventory·scan
- Parent repository discovery와 project authority 분리
- Foreign sibling sentinel byte·metadata 불변
- Project-local·global Git 설정 mutation `0건`
