# 00-editing-discipline.md

Apply these editing rules within the user's authorized scope.

## 1. Think Before Coding

Before implementing:
- Resolve uncertainty with relevant read-only evidence first. State material assumptions.
- Ask only when different interpretations require a material user choice or new authority.
- If a simpler approach exists, say so. Push back when warranted.
- Pause only the dependent action when a user decision is required; continue independent authorized work.

## 2. Simplicity First

- No features beyond what was asked.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't requested.
- No error handling for impossible scenarios.

## 3. Surgical Changes

When editing existing code:
- Don't "improve" adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style, even if you'd do it differently.
- If you notice unrelated dead code, mention it - don't delete it.

For documentation simplification, apply the current-truth preservation procedure in
`04-documentation-state.md`. This editing discipline grants no knowledge deletion authority.

When your changes create orphans:
- Remove imports/variables/functions that YOUR changes made unused.
- Don't remove pre-existing dead code unless asked.

The test: Every changed line should trace directly to the user's request.

## 4. Goal-Driven Execution

**Define success criteria. Loop until verified.**

Use reproducible acceptance checks for the requested behavior. Documentation removal preserves
valid claims through the current-truth procedure above.

For material multi-step work, use the source plan contract. Match verification to the changed
behavior and preserve evidence limits; a passed unit test does not prove an unrun user workflow.
