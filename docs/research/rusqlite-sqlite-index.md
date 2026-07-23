# rusqlite SQLite index dependency 조사

- 조사일: 2026-07-24
- 검증 release: `rusqlite 0.40.1`
- 적용 feature: `bundled`
- upstream license: MIT
- 판정: Phase 2 disposable local index에 exact pin으로 채택

## 확인한 결손

Rust 표준 library는 SQLite connection, transaction, FTS5 virtual table 또는
prepared statement binding을 제공하지 않는다. Hive가 C API binding이나 SQL value
conversion을 직접 구현하면 `unsafe_code = "forbid"` 경계와 cross-platform build
재현성을 해친다. 기존 dependency 없이 Phase 2 FTS5·tag·link projection을 구현할
수 있는 유지보수된 표준 해법이 없다.

## 채택 범위

`rusqlite = 0.40.1`의 `bundled` feature만 사용한다.

- SQLite를 derived local index에만 사용
- FTS5, transaction, prepared statement와 read-only query 사용
- system SQLite version에 의존하지 않는 macOS·Linux·Windows build
- network, model-provider SDK, extension loading과 provider credential 0개
- canonical fact를 SQLite에만 저장하지 않음
- index file은 Git 제외이며 Markdown/YAML에서 무네트워크 재구축

`bundled-full`, runtime extension loading, SQLCipher, backup API와 async wrapper는
필요하지 않아 활성화하지 않는다.

## License 검토

`rusqlite`는 MIT license다. Aigent Hive의 Apache-2.0 배포와 양립 가능하며,
Cargo.lock과 release dependency notice에서 upstream attribution을 유지한다.
Bundled SQLite 자체는 public domain이다.

## 근거

- [rusqlite repository와 bundled 권장](https://github.com/rusqlite/rusqlite)
- [rusqlite 0.40.1 manifest와 feature](https://docs.rs/crate/rusqlite/0.40.1/source/Cargo.toml.orig)
- [rusqlite 0.40.1 license metadata](https://docs.rs/crate/rusqlite/0.40.1)
