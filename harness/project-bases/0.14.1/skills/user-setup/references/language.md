## Response language contract

The selected interface language applies to every setup question, warning, summary, and recovery
result. A user message in another language does not by itself change the saved interface
language.

When the selected interface language is English, use ASD-STE100 Simplified Technical English.
Use short direct sentences, concrete verbs, and one main instruction, condition, result, or
warning per sentence. Use an approved dictionary word when known. Do not use idiom, figurative
language, casual filler, vague pronouns, stacked clauses, or unnecessary synonyms.

When the selected interface language is Korean, use Korean vocabulary and Korean sentence
structure. Keep English only for proper nouns, product or package names, commands, code
identifiers, paths, schema keys, exact UI labels, and terms without a clear Korean equivalent. Do
not insert replaceable English general nouns, mixed Korean-English compounds, or an English
parenthetical after an unambiguous Korean term. Translate meaning rather than English word order.
Keep an English literal only when the user must enter, select, search, or distinguish that exact
literal.

Do not write `benign한 source claim ID의 credential 오인`,
`safe한 default 적용`, `global setting을 update`, `fallback으로 처리`, or
`사용자 설정(user configuration) 확인`. Write `일반 원본 지식 항목 식별자의 비밀 값 오인 방지`,
`안전한 기본값 적용`, `전역 설정 갱신`, `대체 경로 처리`, or
`사용자 설정 확인` instead. Do not use English to look technical, shorten an ordinary Korean
word, or add emphasis. These examples are mandatory patterns, not a closed list.

## Korean interaction contract

When the selected interface language is Korean, retain product terms and identifiers exactly as
`Aigent Hive`, `Skill`, `Wiki`, `Codex`, `Claude`, `Antigravity`, `CodexBar`, commands,
paths, schema keys, Skill IDs, and versions. Do not translate `Skill` as `기술`.

The CLI owns translated questions and their order. Do not compose or translate a second
questionnaire. Explain its reported state in the selected interface language, preserving uncertainty,
source and approval boundaries. A user message in another language is not a language-change request.
