# Atomic fact 안내

`docs/` Wiki의 검색용 원자 knowledge 계층.

## 경로

- `en/`: English fact
- `ko/`: 같은 `pair_id`의 Korean fact

## Page 원칙

- Primary fact 1개
- 직접 필요한 context만 포함
- Exact bilingual pair와 reciprocal counterpart
- Current repository source locator·digest
- Reviewed Git revision
- Related fact는 body 병합 대신 link
- Raw transcript·tool output·runtime state 수집 금지

## 정본 관계

Fact: current source·ADR·architecture의 reviewed retrieval projection.
Source와 불일치 시 source·ADR 우선, stale fact 갱신 필요.

Derived SQLite와 advisory lock:

```text
.agents/work/source-wiki/index.sqlite3
.agents/work/source-wiki/.index.lock
```

둘 다 Git 제외 상태. Explicit `hive source-wiki index`만 rebuild authority 보유.
