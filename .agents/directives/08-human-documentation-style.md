# 08. Human documentation

Human project documents use concise Korean unless explicitly requested otherwise. AI directives
remain English. English uses ASD-STE100 Simplified Technical English: direct, short sentences,
clear pronouns, one idea each; no idioms or filler. Preserve requested/exact quotations.

## Korean style

- Prefer short headings, lists, tables and semantic noun phrases: 추가, 검증, 확인, 적용.
  Remove filler. No authored declarative/conversational endings, regardless of stem or tense.
- Do not mechanically replace endings with attached ㅁ/음 forms. Rewrite the clause to a natural
  concise noun phrase; ordinary lexical nouns are not prohibited.
- One base language. Preserve proper names, commands, identifiers, paths, schema keys, exact UI
  text and terms without a clear Korean equivalent. Avoid mixed Korean-English compounds and
  English used merely to look technical. Translate meaning rather than English word order.
  Add English parentheticals only when the exact literal is needed for an action or distinction.

## Exact bad and good examples

Read [exact examples](references/korean-style-examples.md) only for uncertain wording, style-rule
changes or a checker finding. They cover stems, imperatives, mixed wording and literal exceptions;
examples are not a finite allowlist. All authored paragraphs, headings, tables, captions, callouts
and blockquotes follow the rule. Quote syntax alone is not an exemption: exact external quotations,
UI prompts, protocol samples, fixtures and other byte-sensitive literals require the checker's
path, line, reason and exact-digest exception. Surrounding prose still follows this directive.

## Reader-first explanation

Apply [the explanation policy](01-behavior.md#explanation-policy). Prefer current truth over
chronology; teaching/handoffs favor clarity over extreme brevity. Preserve commands, identifiers,
versions, digests, ownership, security invariants and literal bytes when rewriting. Review the full
prose span; a remaining unapproved ending or stale exception is incomplete work.

## Release notes and update announcements

For release notes, Discord copy and conversational release previews, apply
[release-note audience rules](references/release-notes.md).

## Scope

Applies to README, docs, changelogs, findings, handoffs and other human-facing text. Do not rewrite
code, schemas or exact protocol text; respect an explicit external-audience language requirement.
