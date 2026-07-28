# Aigent Hive marketing deck

## Outcome

- Marketing deck 생성 완료
- Slide 수: 8
- 제작 도구: LumaDeck
- LumaDeck repository: `~/Documents/WebProjects/luma-deck`
- Deck project: `projects/aigent-hive-overview/`
- Editable source: `projects/aigent-hive-overview/slides.md`
- Style source: `projects/aigent-hive-overview/styles/index.css`
- Deck metadata: `projects/aigent-hive-overview/deck.json`
- Exported PDF:
  `artifacts/aigent-hive-overview/pdf/current/aigent-hive-overview.pdf`
- Last validated PDF SHA-256:
  `d2208a2dcc9bf4233a2548c181ab47a403c6ecfd76ac3fd0bdcf5ef81e1f3b0b`

## Creation criteria

Concise explanation targets:

1. What our Hive harness is about
2. Who Hive is for
3. What features Hive provides
4. How Hive is structured and functions
5. What optimization strategy Hive uses

Supporting constraints:

- Local, provider-neutral, user-owned positioning
- Codex·Claude Code·Antigravity host coverage
- Global Hive → project harness → host discovery structure
- Markdown canonical knowledge and user-root disposable SQLite index distinction
- Automatic onboarding, safe upgrade, usage guard, prompt refinement, narrow loading
- Product explanation rather than implementation-history chronology

## Initial request

Original request with the user-specific absolute path normalized to
`~/Documents/WebProjects/luma-deck`:

```text
Take a look at my ppt solution at ~/Documents/WebProjects/luma-deck
Try to automatically install a consumer product harness from our hive (automatically infer the onboarding questions) first. Come to think of it, an automated onboarding is nice. Create a skill for that too. The skill will involve inheriting the global preferences, reading the project source and ask only the truly necessary questions that cannot be automatically resolved. If the global hive preferences is inherited, then the only question left standing would be the project's purpose or goal, but if the README or the AGENTS.md of that project is detailed enough, that question's answer can also be inferred. So the auto onboarding should work.
Then create a deck project that concisely explains:
1. What our hive harness is about
2. Who hive is for (user target)
3. What feature it provides
4. How it's structured and how it functions
5. What kind of optimization strategy it uses
```

## Continuation

1. `~/Documents/WebProjects/luma-deck` 이동
2. `projects/aigent-hive-overview/slides.md`와 `styles/index.css` 수정
3. LumaDeck repository의 `AGENTS.md`와 project harness directive 우선 확인
4. Slidev build·PDF export·전체 slide overflow 검증
5. 이 source record와 `llm-wiki/{en,ko}/marketing-deck.md` current truth 동시 갱신
