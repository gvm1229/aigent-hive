---
schema_version: 1
pair_id: upgrade
topic_slug: upgrade
language: en
counterpart: ../ko/upgrade.md
title: "Update, Upgrade, and Migration"
summary: "Version policy, signed root updates, project-local three-way upgrades, and recovery boundaries."
tags: [migration, update, versioning]
aliases: ["update and migration"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:a2246dba819fa9cae5a793ba812b33822396ee5ad6a15db01f6b00118ec8b131"
  - "repo:crates/hive-update/src/lib.rs#sha256:46a5cc32939b251d8f866df24c701356bfa645a24ca63bdbf531deaa8ea221da"
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:7ff3746a77517b1efd80e7637513997e33915add6c27be713b92c2221f36089e"
links: [boundaries, plugin-lifecycle, security-release]
reviewed_revision: "git:7b6cef8887dbc0571e5a65e5bf32bc829ce3c5d5"
status: active
---

# Update, Upgrade, and Migration

Hive source, release bundles, and installed harnesses share one `X.Y.Z` product version. Compatible
features increment the minor version, compatible corrections increment the patch, and breaking
changes require an explicitly authorized major transition. Plan revisions are independent.

Root Hive updates verify signed release metadata, compatibility, provenance, backup, activation,
and recovery. Project upgrades compare a recorded base, current local bytes, and the incoming
release-generated directives or Skills. Unmodified files accept the incoming version. Locally
modified files keep local conflicting hunks and add non-conflicting incoming changes.

Both paths use pinned filesystem capabilities, preflight validation, journals, and explicit
recovery after interrupted mutation. Canonical project documents, preferences, knowledge, roles,
runs, user Markdown, and foreign bytes remain preservation inputs. SQLite and runtime caches remain
rebuildable outputs rather than migration authority.
