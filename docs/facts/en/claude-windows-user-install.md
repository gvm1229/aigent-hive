---
schema_version: 1
pair_id: claude-windows-user-install
topic_slug: claude-windows-user-install
language: en
counterpart: ../ko/claude-windows-user-install.md
title: "Claude Windows User Installation"
summary: "Hive passes ordinary Windows paths to Claude plugin commands and retains bounded safe failure diagnostics."
tags: [claude, installation, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/usage.rs#sha256:d53779d091d848cea32a1f79a4a96ad11112f361b032758aed008fb9dbf7d9fa"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:6da6fa6180feb983e71a9be4915ff93a12cc1bfefae6016bad026f5c94b0238e"
  - "repo:crates/hive-cli/src/user_install/host_state.rs#sha256:f7f2d78da2e843f3a75267a106251eb26af09f22773bf67ec8f7bc2cca7f8c60"
  - "repo:scripts/qualify-claude-user-install.py#sha256:29efd013c8f258c979e14095297db8e277737bfdbb63620ec776dd315fe2de52"
links: [multi-host-user-install, supported-hosts]
reviewed_revision: "git:b3feedb374b8c3edab9ebf40f50931989b0e448e"
status: active
---

# Claude Windows User Installation

Claude marketplace registration rejects Windows verbatim paths even when validation accepts them. Hive keeps canonical filesystem paths internally and converts only Claude command arguments. Files are written before registration; rollback can remove them after failure.

Safe diagnostics retain bounded stderr, exit status, fixed classification and output sizes/digests without raw output. Ambiguous recovery preserves external state and its journal.

The isolated native CLI test covers ordinary, space-containing and Korean paths, install, validation, reinstall, update and foreign data preservation. It does not prove interactive Skill discovery or model-driven questions.
