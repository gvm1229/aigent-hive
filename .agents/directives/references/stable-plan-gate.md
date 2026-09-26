# Stable Plan Gate

## Stable Release Plan Gate

- Keep stable publication blocked while any active in-scope checklist item is incomplete. A future
  candidate is excluded only when `PLAN.md` names its exact IDs and target version.
- Require evidence from a uniquely numbered public test version bound to exact source and public
  artifacts before stable publication.
- A post-test change resets the affected acceptance item and requires the next numbered test.
- Never use a stable channel that supplied the first or only evidence for product behavior,
  packaging, installation, performance, migration, or recovery.
