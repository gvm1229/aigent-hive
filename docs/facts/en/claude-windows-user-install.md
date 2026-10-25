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
  - "repo:crates/hive-cli/src/usage.rs#sha256:ce45a82f02376c71b9bd25a7daf377c11fa1c9b55a4f08098098f97cd7ae920f"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:2fade132e2fe4e60b3f7b70e089d5deaa145a25e2976bdfdb62a58a82fa86192"
  - "repo:crates/hive-cli/src/user_install/host_state.rs#sha256:8c2ada111bbfd27361a56a75a38c2c379c3517110233590717cd97caa6f5c8be"
  - "repo:scripts/qualify-claude-user-install.py#sha256:29efd013c8f258c979e14095297db8e277737bfdbb63620ec776dd315fe2de52"
  - "repo:tests/results/legacy/af6a478d5074c2d3a277.md#sha256:ff1b2012dc26150493fa439cf4eab9f546d64549640d94622d9e1ee9704ff9bd"
  - "repo:tests/results/runs/20261001T234717-11673280f355.md#sha256:72fa345ddb6e4feff83592041055cf32edb4a3bd3347cee6a5af5423328ea692"
links: [multi-host-user-install, supported-hosts]
reviewed_revision: "git:2c804e7809832adcdd9cba8ec1cfd8f4e0bffb6e"
status: active
---

# Claude Windows User Installation

Claude marketplace registration rejects Windows verbatim paths even when validation accepts them. Hive keeps canonical filesystem paths internally and converts only Claude command arguments. Files are written before registration; rollback can remove them after failure.

Safe diagnostics retain bounded stderr, exit status, fixed classification and output sizes/digests without raw output. Ambiguous recovery preserves external state and its journal.

The isolated native CLI test covers ordinary, space-containing and Korean paths, install, validation, reinstall, update and foreign data preservation. It does not prove interactive Skill discovery or model-driven questions.

Public `0.11.1-test.6` passed Windows Claude `2.1.163` CLI qualification.
