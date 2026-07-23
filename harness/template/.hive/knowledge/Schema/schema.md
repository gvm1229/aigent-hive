# Knowledge schema

## Wiki frontmatter

```yaml
---
id: wiki-<stable-id>
kind: source-summary | entity | concept | comparison | synthesis | open-question
summary: 한두 문장
tags: [example]
sources: [raw:<content-hash>#locator]
status: active | contradicted | open-question
created_at: RFC-3339
updated_at: RFC-3339
---
```

`deprecated`, `superseded`, `archived` page는 active tree에 남기지 않는다.
필요한 이력은 Git에서 조회.
