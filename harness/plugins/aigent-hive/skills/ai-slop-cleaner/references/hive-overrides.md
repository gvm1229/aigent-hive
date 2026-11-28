# Hive overrides

Read after recorded user preferences and before the target checklist. Each line states where Hive
differs from a bundled source under `upstream/`. When a bundled source and this file disagree, this
file wins; when a recorded user preference disagrees with this file, the preference wins.

## omc-ai-slop-cleaner

- Ignore OMC slash commands, `oh-my-claudecode:` invocations, Ralph integration and the `level`
  field. Hive has no Ralph loop.
- The cleanup plan is shown to the user and confirmed before any edit, not only written down.
- Scope also covers documents and designs; use the matching Hive checklist instead of the code
  passes alone.
- Its UI/design reviewer checklist applies only through [design.md](design.md).
- Review mode is a plain request ("review only"), not a `--review` flag.

## hallmark

- Use Hallmark only as a checklist for an existing design. Do not deal a new direction, swap the
  theme or rebuild the page; new direction belongs to `design-curate`.
- Skip its CSS stamp and critique-line requirement (Floor item 15) unless the project already uses
  Hallmark stamps.
- Only Hallmark's `SKILL.md` and license are bundled here. Use "The Floor" as a checklist; skip
  steps that need its other files or scripts, such as `sloplint.mjs`.
- Do not install its hooks or call its image generation.

## Writing

- Korean sentence rewriting goes to `humanize-kor`. This Skill fixes structure only.
