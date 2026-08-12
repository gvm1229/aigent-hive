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
  - "repo:docs/hive-install-guide.ko.html#sha256:08b5ac46102f4415ed5ca2899c01c3c7979240e1f32da978afd8c976ea31ff6d"
  - "repo:docs/plans/active/public-html-guides.md#sha256:ef7818fd4550419c585f2fae43c569fa2d1541fe4508e622f9ad738317e371bf"
links: [global-onboarding, product-purpose]
reviewed_revision: "git:0b3bbbbfcb5904262c5281a0415851b96779ab9e"
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
