# Aigent Hive project harness

- Treat this repository as an independent consumer project.
- Read `AGENTS.md` first, then load only the narrow `.agents/directives/` file needed by the task.
- Treat `.hive/config/` and canonical Markdown as authority.
- Treat `.agents/directives/` and `.agents/skills/` as release projections, not user data authority.
- Preserve every user-authored and third-party byte outside an exact Hive-owned marker or manifest path.
- Never request provider API credentials or call model-provider APIs on Hive's behalf.
- Use compatible OMX on Codex and OMC on Claude before duplicating their orchestration workflows.
