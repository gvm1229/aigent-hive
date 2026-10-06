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
  - "repo:crates/hive-cli/src/usage.rs#sha256:08df602b839ca6ced6cd1571c37111b300853def9bba4e25c9c24774453535f6"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:07c53684b15ca76c0912c67ab9d2dac2464a98580fa186956492a95fa238af63"
  - "repo:crates/hive-cli/src/user_install/host_state.rs#sha256:f7f2d78da2e843f3a75267a106251eb26af09f22773bf67ec8f7bc2cca7f8c60"
  - "repo:scripts/qualify-claude-user-install.py#sha256:29efd013c8f258c979e14095297db8e277737bfdbb63620ec776dd315fe2de52"
  - "repo:tests/results/legacy/af6a478d5074c2d3a277.md#sha256:ff1b2012dc26150493fa439cf4eab9f546d64549640d94622d9e1ee9704ff9bd"
  - "repo:tests/results/runs/20261001T234717-11673280f355.md#sha256:72fa345ddb6e4feff83592041055cf32edb4a3bd3347cee6a5af5423328ea692"
links: [multi-host-user-install, supported-hosts]
reviewed_revision: "git:ad412a7b2aa6bba927f5f841e0e5161f4fb60488"
status: active
---

# Claude Windows User Installation

Claude marketplace registration rejects Windows verbatim paths even when validation accepts them. Hive keeps canonical filesystem paths internally and converts only Claude command arguments. Files are written before registration; rollback can remove them after failure.

Safe diagnostics retain bounded stderr, exit status, fixed classification and output sizes/digests without raw output. Ambiguous recovery preserves external state and its journal.

The isolated native CLI test covers ordinary, space-containing and Korean paths, install, validation, reinstall, update and foreign data preservation. It does not prove interactive Skill discovery or model-driven questions.

Public `0.11.1-test.6` passed Windows Claude `2.1.163` CLI qualification.
