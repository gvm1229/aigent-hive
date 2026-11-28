# Conflict rulings

Maintained by the Hive maintainer and reviewed at each weekly source sync. A recorded user
preference or the project `DESIGN.md` overrides every ruling here; accessibility overrides the
rulings for contrast, motion and text size.

| Topic | Conflicting rules | Sources | Ruling | Reason |
| --- | --- | --- | --- | --- |
| Typeface choice | Avoid Inter and stock families vs. Hallmark's dealt font pairs vs. impeccable's font index | web-artifacts-builder, hallmark, impeccable, frontend-design | Use the project or user font first; otherwise choose deliberately per brief and never default to Inter | Brief-specific type is the shared intent; a user font preference (for example Pretendard) wins |
| Accent palette | Avoid purple-to-blue gradients vs. bold color permitted when the brand owns it | hallmark, web-artifacts-builder, frontend-design | Ban the purple-to-blue gradient and stock blue only without brand rationale | All sources reject unreasoned defaults; brand truth decides |
| Labels and eyebrows | Never an eyebrow or tracked capitals vs. labels allowed when they encode information | hallmark, frontend-design | No decorative eyebrows; keep a label only when it carries information the heading lacks | frontend-design treats structure as information; hallmark's floor targets decoration |
| Motion | One orchestrated moment vs. animate and delight commands | frontend-design, impeccable, hallmark | One deliberate motion moment per screen, always with a reduced-motion fallback | Accessibility rung plus the restraint shared by all sources |
| Density and word count | About 80 words per screen vs. dense Operate and Read surfaces | hallmark, impeccable | Apply the 80-word cap to Persuade surfaces only; Operate and Read follow task density | impeccable's modes describe when density serves the visitor |
| Cards and layering | Never a card inside a card vs. shadcn/ui component kit | hallmark, web-artifacts-builder | Use components, but never nest cards or put the same shadow on every surface | Nested surfaces are the shared tell, not the component library |
| Scripts and setup | Run launchers, install hooks, download binaries | impeccable, hallmark | Run a source script only with consent and never install global hooks | Hive does not change user-global settings without approval |
