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
  - "repo:README.md#sha256:05c2f90b0096663c3c8ded051eec7d7632df4b99442d90a77faca4f26b5bf80b"
  - "repo:docs/public-stable-release.json#sha256:45913c2779432631e5873c9f9f221c76673fea90a878a05cb5fee827aaf46474"
  - "repo:scripts/check-public-stable-docs.py#sha256:1b0ff65b6c7b7d37fd45554d2465885515ab5f9f06c6f3f46a0c08b99d5a89f9"
  - "repo:tests/conformance/release/test_release_notes.py#sha256:0e6e4201c7c3369ea787c956b7022add3aae86f1331894787331e955a4c9a63c"
links: [product-purpose, release-verification]
reviewed_revision: "git:c5b13bc26c4b2485dcad7509bc8590f97d0831fe"
status: active
---

# Stable Public Documentation

The public stable manifest owns the version, date, surfaces, and release-note coverage for ordinary users.
README files, the install HTML, product overview, and document index show only that stable release.
README files carry no release history: they link to GitHub Releases, and release-note coverage points to the release note.
Numbered prereleases remain maintenance evidence in npm, GitHub, and maintainer documentation, not install guidance.
Test candidates preserve the manifest stable; stable candidates require the requested version and date before build and publication.

CI runs the current release note and source-version metadata validator before candidates. Synthetic fixtures alone do not validate the actual release documents.
