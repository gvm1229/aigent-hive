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
  - "repo:.github/workflows/release-publish.yml#sha256:e664105a2734fc5ec7c35f93ddc5ce0362ad5e391ae881c63e326a8c25866bca"
  - "repo:.github/workflows/release.yml#sha256:0b800d9f74b331f34aad1507b57129fb319fdf49934815026c6352c6aa91a5d7"
  - "repo:README.md#sha256:10e54403d7f420452254c6e9389c5bd0b2707d39b6c9a7c02877b70833a01d5f"
  - "repo:docs/public-stable-release.json#sha256:d846190c356b383da6183becc4d37eff0e1338b31a0dbefe4d6f270ad2377d42"
  - "repo:scripts/check-public-stable-docs.py#sha256:69b25685285621ee94a515748de03c56b9100ca0e2f9e283bdc35a2278cb9f04"
links: [product-purpose, release-verification]
reviewed_revision: "git:8a45250106590f065df639132298b840940a3a35"
status: active
---

# Stable Public Documentation

The public stable manifest owns the version, date, surfaces, and release-note coverage for ordinary users.
README files, the install HTML, product overview, and document index show only that stable release.
Numbered prereleases remain maintenance evidence in npm, GitHub, and maintainer documentation, not install guidance.
Test candidates preserve the manifest stable; stable candidates require the requested version and date before build and publication.
