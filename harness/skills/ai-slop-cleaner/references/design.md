# Design checklist

Read only when the target is a built interface: HTML, CSS, components or a screenshot of them.
Apply recorded `design/*` preferences and the project's `DESIGN.md` first; they override every
item below. Bundled detail: `upstream/hallmark/SKILL.md` ("The Floor") and the UI checklist in
`upstream/omc-ai-slop-cleaner/SKILL.md`, with
[hive-overrides.md](hive-overrides.md) applied.

## Keep fixed

Brand colors and type, accessibility (contrast, focus, reduced motion, text size), responsive
states and interaction behavior.

## Smells and examples

| Class | Look for |
| --- | --- |
| Repetition | the same card three or four times in a perfect grid with no emphasis |
| Padding | eyebrow plus title plus description plus extra paragraph; decorative status dots |
| Needless layers | a card inside a card, shadows on every surface, wrappers that only add padding |
| Generic defaults | purple-to-blue gradients, stock `#3B82F6` blue, emoji as icons, `transition: all` |
| Unsupported claims | invented metrics, testimonials, logo walls or case-study counts |

## Check

Compare the page before and after at desktop and phone width. Confirm focus states, reduced-motion
fallbacks and contrast still pass.
