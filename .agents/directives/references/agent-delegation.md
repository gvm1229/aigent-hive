# Agent delegation

Repository agent rules (maintainer, 2026-11-12). Items marked **maintainer PC** apply only there; other environments skip them.

## Roles and launch

- An orchestrator/executor split is allowed: one host plans and reviews; Codex executes heavy work.
- **Maintainer PC:** launch Codex through `python C:/Users/hojin/.claude/scripts/codex-run.py --brief <file> --workdir <dir> --log <log> --report <report>`; the watchdog runs `codex exec -m gpt-6.1-sol --dangerously-bypass-approvals-and-sandbox -c service_tier="priority"`. The default sandbox cannot read its own PID for 07 or write `.agents/work/active-sessions/`.
- Use `gpt-6.1-sol`. Fast mode (`service_tier = "priority"`) is allowed. Codex ULTRAFAST is prohibited: high token use and no support in the maintainer's GPT plan. Never use Claude fast mode.
- Official `codex exec` is allowed; subscription-account `claude -p` headless print mode is prohibited. Codex must use the maintainer's subscribed ChatGPT sign-in; never inspect credential files to check it.
- For 07 preflight, `--user-root` is the HOME directory, e.g. `C:/Users/hojin` on the **maintainer PC**, never `~/.hive`; use the exact executor PID/session and target worktree.
- Write briefs with quoted heredocs (PowerShell: literal here-strings); Windows backslash paths break string literals. Executors fix failures caused by their edits, including stale hashes, links and indexes; stop only the work dependent on out-of-scope failures, subject to 01/07 stops.
- Commit messages follow [Git commits](git-commits.md), including its AI trailer prohibition.

## Stalls

- Never terminate a long-running test, build or CI watch mid-way; a non-long command may be killed only after 45 minutes without output.
- If Codex itself has no running command and is quiet for 15 minutes, terminate it and use `codex exec resume <session-id>`; at most two resumes.
- Exception: if a CI step exceeds three times its normal duration, cancel that run, use `gh run rerun <id> --failed`, and record the duration, cancellation and retry in the owning evidence.

## Stage review and release

- Before a product stage's develop merge, use a **Fable 5.2 advisor** subagent. If no advisor subagent exists in that session, use Fable 5.2 as a plain subagent at its default effort, without an effort override; record the fallback in review evidence. No extra review of the develop-to-main PR before stable.
- Every advisor or plain-Fable stage-review brief must quote the maintainer (2026-11-12):
  > "This code base is yours. What it's for and how it's built are your call, and you'll keep building on it. If you think that editing something big would contribute to the goal, then you are free to point that out and delegate tasks."
- Review as codebase owner beyond diffs: product purpose and architecture are the advisor's call. It may propose large edits/restructuring toward the goal and concrete tasks for the orchestrator to delegate (e.g. to Codex). The orchestrator records proposals in review evidence and routes accepted ones to the owning plan or a new stage item; never silently drop them. Advisor stays read-only; executors edit. AGENTS always-on boundaries, 05 safety, no provider API calls, plan ownership and version-specific stable/publication approvals remain unchanged.
- Product stage order: fix -> tests -> notes -> stage review -> develop merge -> test.1 from develop (public-test workflows require develop) -> stable via develop-to-main PR -> **maintainer PC** update. Existing [release qualification](release-qualification.md) and version-specific approval boundaries still apply.
- For npm dist-tag lag, re-dispatch `release-publish.yml` with the same inputs plus `recover_published_packages=true`; recovery grants no new publication authority.
- Overhaul-scoped approval and stage-review rules remain in [the owning plan](../../../docs/plans/overhaul/README.md#브랜치-규칙); do not copy those approvals here.
