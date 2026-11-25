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
  - "repo:.github/workflows/release-publish.yml#sha256:3f4998b2889cfd7cb3ada0b94a7a1648ef6bbd9c8cd7e848676742d13bc17a42"
  - "repo:.github/workflows/release.yml#sha256:6fa640b5ce08594bdcbccd28530846ff7a34f7dfd6f918f4b62a860555ed7567"
  - "repo:README.md#sha256:cfdd9e02efef5feba18f2ccd3594a2aaa60fdfc73bbc687b949ddde9eed09f0a"
  - "repo:docs/public-stable-release.json#sha256:42b25a5294c4d945f8bad57fec402f6afec0e50ef39d4820a5e1f1fe63601346"
  - "repo:scripts/check-public-stable-docs.py#sha256:69b25685285621ee94a515748de03c56b9100ca0e2f9e283bdc35a2278cb9f04"
  - "repo:tests/conformance/release/test_release_notes.py#sha256:0e6e4201c7c3369ea787c956b7022add3aae86f1331894787331e955a4c9a63c"
links: [product-purpose, release-verification]
reviewed_revision: "git:c5b13bc26c4b2485dcad7509bc8590f97d0831fe"
status: active
---

# Stable Public Documentation

The public stable manifest owns the version, date, surfaces, and release-note coverage for ordinary users.
README files, the install HTML, product overview, and document index show only that stable release.
Numbered prereleases remain maintenance evidence in npm, GitHub, and maintainer documentation, not install guidance.
Test candidates preserve the manifest stable; stable candidates require the requested version and date before build and publication.

CI runs the current release note and source-version metadata validator before candidates. Synthetic fixtures alone do not validate the actual release documents.
