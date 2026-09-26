# Release notes and update announcements

- Assume the reader uses Hive but knows none of its internals. Being a developer does not imply
  knowledge of Hive's implementation. Apply this to patch notes, GitHub Release descriptions,
  subscriber summaries, and conversational previews of an upcoming release.
- Lead each feature heading and explanation with the user's improved experience: what becomes
  possible, easier, safer, or less disruptive. Use familiar actions and concrete situations.
- Translate implementation evidence into its supported user benefit. For example, replace
  `모든 공개 안정판의 갱신 검증 강화` with `오래 사용한 버전에서도 설정을 지키며 업데이트`.
  State the supported range nearby: `0.9.1 이후 모든 공개 안정판 지원`. Do not turn this into
  an unconditional success guarantee or silently include prereleases or every configuration.
- Keep internal terms such as projection, authenticated base, migration matrix, reservation,
  and line-ending formats out of feature headlines. Name a Skill or technology only when it
  helps the reader find, use, or choose the feature, and explain its purpose in ordinary words.
- Internal restructuring or more tests are not user benefits by themselves. Omit those entries
  when no verified change in the user's experience follows; retain the evidence in source records.
- Failed qualification runs, retries, raw errors, and investigation belong in maintainer evidence.
  Do not include them anywhere in public release notes or Discord copy, including verification
  sections and footnotes. If positive evidence is missing, omit the unsupported product claim.
- Keep material compatibility, cost, consent, and support limits in plain language near
  the relevant benefit. Separate release status and verification evidence from feature highlights.
- Before delivery, ask whether a person unfamiliar with Hive's design understands why to update
  from the heading and first explanation alone. Rewrite any entry that still needs internal context.
