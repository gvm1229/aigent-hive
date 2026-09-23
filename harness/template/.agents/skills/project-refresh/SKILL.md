---
name: project-refresh
description: (project-refresh) Update existing Hive-generated project guidance when the user asks to refresh or upgrade a project's Hive directives or Skills. Supports preview-only requests; not global Hive installation or ordinary application updates.
---

# Hive project upgrade

Use only for an installed consumer project.

## Request routing

- Select this Skill for a natural request such as "update this project's Hive guidance" or
  "이 프로젝트의 Hive 지침을 갱신해 줘". The user need not name the Skill or type CLI commands;
  the agent runs the commands below and explains the changes in the user's language.
- A preview-only request authorizes inspection and preview, not apply. An explicit request to
  update the identified project authorizes the reviewed in-scope update; do not ask again for
  the same authority. Ask only for a missing target, material conflict decision, or new scope.
- General questions and ordinary code or dependency updates do not select this workflow.
  Global Hive installation or updates belong to `user-setup` or `product-update`.
- Apply the project's `00-project-harness.md` availability gate before commands. Missing local
  Hive or setup blocks only the requested Hive upgrade, not ordinary collaboration. Do not
  install Hive, alter global settings, or substitute hand-written generated files implicitly.
- Resolve the project and verified executable, inspect Git changes and Hive ownership, and
  preserve unrelated edits. Never select a different release or install it without authority.

## Upgrade workflow

1. Run `hive project upgrade --target <project-root> --scan --output json`.
2. Report installed, base, local, and incoming digests plus every applicable change, including retired Hive Skill removal only when ownership is authenticated.
3. For preview, run `hive project upgrade --target <project-root> --dry-run --output json`.
4. Apply only after the user requests the update:
   `hive project upgrade --target <project-root> --apply --output json`.
5. Validate with
   `hive project upgrade --target <project-root> --validate --output json`.
6. If an interrupted activation leaves an upgrade journal requiring recovery, run
   `hive project upgrade --target <project-root> --recover --output json`.
   Inspect whether recovery rolled back or completed forward. This is not an undo command for
   a successful upgrade. Stop on an unauthenticated base or unresolved ownership conflict.

Report changed paths, preserved local changes, validation results, and remaining conflicts.
Validation proves the Hive projection state, not application tests or the host's compliance.

Merge contract:

- `local == base`: incoming exact replacement
- disjoint local and incoming changes: include both
- overlapping changes: preserve the local hunk and report the omitted incoming hunk, except a direct conflict with an incoming Hive safety or ownership rule
- an outdated Hive directive or the Hive marker in `AGENTS.md`: replace only a Hive-owned clause that directly contradicts an incoming safety or ownership rule; preserve user-authored text, foreign blocks, and non-conflicting local Hive clauses byte-for-byte
- retired Hive Skill: remove only a retired-name path proved by the authenticated project base; preserve modified or foreign paths and report any resulting conflict
- missing or unauthenticated base: active bytes unchanged and conflict
- active conflict markers: forbidden
- existing run owner pins are canonical run state: preserve every 0.8.x OMX/OMC owner and every v0.9 host-native or explicitly selected external owner; never migrate an owner as a projection side effect

Do not edit `.omx/`, `.omc/`, provider credentials, or foreign paths.
