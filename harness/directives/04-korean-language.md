# Korean output contract

Load this directive before a Korean response, Korean document change, release summary, CLI message,
or explicit Korean rewriting task.

## Automatic output path

1. Draft in natural Korean. Keep English only for names, commands, identifiers, paths, schema keys,
   exact UI text, and terms without a clear Korean equivalent.
2. Protect facts, modality, numbers, dates, units, versions, names, quotations, links, Markdown,
   code, commands, paths, list structure, and attribution.
3. Apply small generation-time constraints only to absolute errors. Do not treat
   frequency-conditional tendencies as banned expressions.
4. Inspect the finished text against the active Korean profile. Rewrite only reported local spans,
   then verify against the exact draft.
5. Use the exact draft when deterministic verification fails. Limit automatic rewrite to one
   retry. A host hook may request that retry but never claims direct final-response replacement.

## Profiles

- `response`: clear intent, plain words, proportionate examples, no over-editing of short answers.
- `release-note`: changes as a main list and user scenarios as sublists.
- `documentation`: preserve headings, links, contracts, warnings, and current facts.
- `technical`: preserve code, fields, paths, and exact diagnostics byte-for-byte.
- `verbatim`: inspect only. Do not rewrite.

Codex uses this instruction and final self-review because no verified final-response replacement
hook is available. Claude may use a consented bounded `Stop` validation retry. Antigravity may use
a consented bounded `AfterAgent` validation retry when fresh capability evidence supports it.
Unsupported host events remain instruction-only and must not be described as active hooks.

## Integrity boundary

Do not optimize detector scores, evade statistical watermarks, remove required attribution or
disclosures, conceal sources, or claim false human authorship. Zero-width and bidi-control removal
is permitted text hygiene, not watermark removal.
