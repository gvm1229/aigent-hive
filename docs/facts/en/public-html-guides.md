---
schema_version: 1
pair_id: public-html-guides
topic_slug: public-html-guides
language: en
counterpart: ../ko/public-html-guides.md
title: "Public Korean HTML Guides"
summary: "Two Korean HTML pages and their design guide cover Hive features, installation, and branding."
tags: [branding, documentation, onboarding]
aliases: ["Hive core features guide", "Hive quick install guide"]
sources:
  - "repo:docs/guides/public-html-design-principles.md#sha256:68274e825f67e04751c9039d855af711503d4fe37929467acdce763f0b14ca82"
  - "repo:docs/hive-core-features.ko.html#sha256:99c818208dcdda45552a0f4962f7853222a57706463c2dd13a70f2f49863177e"
  - "repo:docs/hive-install-guide.ko.html#sha256:e1fd1196f466dc197a12e7da2e5f31a59c7cd8583fea4796f899f31b07a6534d"
  - "repo:docs/plans/active/public-html-guides.md#sha256:8bddcbc123282fba063c067a0fe869775eb69417fca6d95f138da1d2421cb167"
links: [global-onboarding, product-purpose]
reviewed_revision: "git:7b62967890033df8e6974327606ebad05b1500d8"
status: active
---

# Public Korean HTML Guides

The repository includes standalone Korean pages for Hive's core features and streamlined stable
installation. Both reuse the canonical logo, a 960 px information structure, and beehive gold
`#F5A623`. The design guide records brand tokens, hierarchy, components, responsive breakpoints,
accessibility, and command-accuracy checks. Install step 3 states that `--host` accepts one host
per command; multi-host use repeats the command per host and selects the same hosts during global
setup. README branding was already completed by commit `245ae80`.
