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
  - "repo:docs/guides/public-html-design-principles.md#sha256:6aae9ab9808ca927fe3736f3c4bde3e91a0e25a9700ae75f98592e1577ab01ec"
  - "repo:docs/hive-core-features.ko.html#sha256:8f77210359186752205a0b4dcffcefd5d1a0bb8530d3620463be073c81b33abf"
  - "repo:docs/hive-install-guide.ko.html#sha256:9338f3f1f23e99bfef5f0788ab14051789414cc7cffb6c10eb1b2e9bd8c982c2"
  - "repo:docs/plans/active/public-html-guides.md#sha256:efc1acbefb797798ba2deeec0653da3409599b91904b5fc8d7a1d12e92ebc9e8"
links: [global-onboarding, product-purpose]
reviewed_revision: "git:6f861b14f3a4931e89cb290504f94da311ec0339"
status: active
---

# Public Korean HTML Guides

The repository includes standalone Korean pages for Hive's core features and streamlined stable
installation. Both reuse the canonical logo, a 960 px information structure, and beehive gold
`#F5A623`. Each HTML embeds the exact canonical PNG bytes once, uses system fonts, and contains no
network or file-relative resource reference. The design guide records brand tokens, hierarchy, components, responsive breakpoints,
accessibility, and command-accuracy checks. Install step 3 covers single `--host`, comma-separated
`--hosts`, and repeated `--host`, including quoted CSV whitespace, whole-request preflight, and
partial-failure JSON.
README branding was already completed by commit `245ae80`.
