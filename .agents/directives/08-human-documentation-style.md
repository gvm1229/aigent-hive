# 08. Human Documentation Style Directive

This directive governs human-readable project documents created or updated while developing
Aigent Hive.

AI-readable directives under `.agents/directives/` stay in English. Human-readable project
documents use concise Korean unless the user explicitly requests another language for that
document.

## Korean style

- Prefer short headings, bullets, tables, and checklists over long prose.
- Prefer noun phrases or concise verb-noun endings such as `추가`, `정리`, `검증`, `확인`,
  `보강`, `제거`, and `적용`.
- Do not end authored explanatory Korean prose with a declarative or conversational sentence
  form. This prohibition applies regardless of the verb stem or tense. Forms such as `~다`,
  `~한다`, `~된다`, `~이다`, `~있다`, `~없다`, `~않는다`, `~했다`, `~됐다`, `~합니다`,
  `~됩니다`, and `~해요` are examples, not an exhaustive allowlist.
- Do not mechanically replace a prohibited ending with `~함`, `~됐음`, `~했음`, or `~않음`
  when a shorter semantic noun phrase is available. Rewrite the whole sentence or clause.
- Remove filler that does not help the reader act, remember, or verify.
- Keep code identifiers, schema keys, paths, commands, product names, and exact UI labels in
  their original form when clearer.
- Do not force awkward Korean transliterations for technical identifiers.

## Exact bad and good examples

Treat every `Avoid` entry below as prohibited authored prose. Use the paired `Use` form or an
equally concise noun phrase. These examples define the expected transformation but do not limit
the prohibition to these exact strings.

| Avoid | Use |
| --- | --- |
| `Aigent Hive는 provider-neutral 로컬 agent harness다.` | `Aigent Hive: provider-neutral 로컬 agent harness` |
| `Product version은 0.7.0이다.` | `Product version: 0.7.0` |
| `Release 계약이 구현됐다.` | `Release 계약 구현 완료` |
| `API key를 요청하거나 저장하지 않는다.` | `API key 요청·저장 없음` |
| `이 기능을 사용합니다.` | `기능 사용` |
| `다음 단계에서 검증해요.` | `다음 단계: 검증` |
| `검증이 필요합니다.` | `검증 필요` |
| `업데이트가 완료되었습니다.` | `업데이트 완료` |
| `Release 계약이 구현됐음.` | `Release 계약 구현 완료` |
| `API key를 요청하거나 저장하지 않음.` | `API key 요청·저장 없음` |

Authored Markdown callouts and blockquotes follow the same rule:

```text
Avoid: > 현재 상태는 0.7.0이다.
Use:   > 현재 상태: 0.7.0
```

Blockquote syntax alone never makes text an exact quotation. Preserve a narrative-form sentence
only when it is an exact external quotation, exact UI prompt, protocol sample, fixture payload,
or another byte-sensitive literal. Automated checks must bind each literal exception to its
path, line, reason, and exact line digest. Surrounding explanation still follows this directive.

## Reader-first explanation

- State the decision, result, or prerequisite concept first.
- Use a numbered or arrow flow when runtime order, ownership, or state transition matters.
- Separate verified facts from assumptions and distinguish the cause from plausible non-causes
  when confusion is likely.
- Include the minimum example needed to understand, reproduce, or verify the behavior.
- Prefer current truth over chronological investigation notes.
- For teaching notes and handoffs, prefer clarity over maximum brevity.

## Completion gate

- Review the full authored prose span, not only the final suffix.
- Confirm that headings, paragraphs, list items, table cells, captions, callouts, and authored
  blockquotes contain no unapproved narrative-form ending.
- Confirm that a rewrite preserves commands, identifiers, versions, digests, ownership,
  security invariants, and exact literals.
- Treat any remaining unapproved narrative ending or stale literal exception as incomplete work.

## Scope

Apply this style to `README.md`, `docs/`, changelogs, findings, handoffs, implementation notes,
and other files intended for human readers. Do not rewrite code, schemas, exact protocol text, or
external-audience documents that explicitly require another language.
