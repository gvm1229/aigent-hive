---
schema_version: 1
pair_id: stable-public-documentation
topic_slug: stable-public-documentation
language: en
counterpart: ../ko/stable-public-documentation.md
title: "Stable Public Documentation"
summary: "A manifest and release gates keep ordinary user documentation on the current stable release while maintenance prereleases remain unadvertised."
tags: [documentation, release, stable]
aliases: ["public stable docs"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:6d9b351dfbe99fef461d642285a5bc37730ef6ba29d3c62d38c800bdd8e6220f"
  - "repo:.github/workflows/release.yml#sha256:2f3760d989da12d1b07bfe706b9e7f1cd1e3121d3a53b18843e7825b56d86cac"
  - "repo:README.md#sha256:ac7b9cc92c876e73c7731f685482a15ef8ba9bc4a1ec9c1ff081e8dc2d14e089"
  - "repo:docs/public-stable-release.json#sha256:5a004d43808985a6652eaf8a7967352992a7313089cc2597d3fd69fc0ec65a1f"
  - "repo:scripts/check-public-stable-docs.py#sha256:39d8b26827f208a20c7785d8727c920fdef1e04fc23804c1ed21dd33aaee616a"
links: [product-purpose, release-verification]
reviewed_revision: "git:8a45250106590f065df639132298b840940a3a35"
status: active
---

# Stable Public Documentation

The public stable manifest owns the version, date, surfaces, and release-note coverage for ordinary users.
README files, the install HTML, product overview, and document index show only that stable release.
Numbered prereleases remain maintenance evidence in npm, GitHub, and maintainer documentation, not install guidance.
Test candidates preserve the manifest stable; stable candidates require the requested version and date before build and publication.
