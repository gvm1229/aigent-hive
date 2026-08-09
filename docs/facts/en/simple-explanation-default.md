---
schema_version: 1
pair_id: simple-explanation-default
topic_slug: simple-explanation-default
language: en
counterpart: ../ko/simple-explanation-default.md
title: "Simple Explanation Default"
summary: "Source agents start with the user-visible files, data effect, and safe next action before defining internal implementation terms."
tags: [communication, guidance, projection]
aliases: ["concrete examples", "plain-language explanation"]
sources:
  - "repo:.agents/directives/01-behavior.md#sha256:2418d9cad5ad54ff9fdad0f117c66336826bbd34c19fc0c131340fe64cb31f01"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:cf2e1af6e7476bd05ae20bec0abd3ffcfe1730de632b0967b275331b3c00b1e3"
  - "repo:docs/guidance-schema.md#sha256:f5fc6aa2c36274d78d9703693a362c2f8d8eb81204d37f8a224434c14d1b196b"
  - "repo:harness/template/AGENTS.md.jinja#sha256:64f33fed294900badc58d8ff6b4f7144d0c43bf003884abdcae5c703a60cdd7a"
links: [language-consistency, verification-result-clarity]
reviewed_revision: "git:536f5076534cedcdb9ea3d118830792fe61cd75e"
status: active
---

# Simple Explanation Default

Start with user-visible files, settings or knowledge, and the safe next action.
Use `projection`, `manifest`, or `digest` only when diagnosis requires them,
with a plain definition in the same sentence. Lists use one Markdown entry per
line; independently selectable options never share a comma-separated paragraph.
