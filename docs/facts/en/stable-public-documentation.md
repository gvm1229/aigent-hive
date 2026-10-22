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
  - "repo:.github/workflows/release-publish.yml#sha256:240f5376bc7ea26109a1c34be965c6e74f9dc840e400af78bb1e55a2bac01263"
  - "repo:.github/workflows/release.yml#sha256:fe8bb871aaa0710a655f41521b7fe7960c63ff5c660795a2ee09aed29cb92631"
  - "repo:README.md#sha256:165c9867d908d6b3774380a621952156e343989ad88f24e942663f552ba85077"
  - "repo:docs/public-stable-release.json#sha256:97f9eab19f9bb17554db37c4cc811e10a59beea61c680aa0ae1a108c04862d2e"
  - "repo:scripts/check-public-stable-docs.py#sha256:69b25685285621ee94a515748de03c56b9100ca0e2f9e283bdc35a2278cb9f04"
  - "repo:tests/conformance/release/test_release_notes.py#sha256:f32a5b2ed683b7369676d905c4dfeef0546d742a2a2317f6679992b034ed5299"
links: [product-purpose, release-verification]
reviewed_revision: "git:708386fbb9ff493ef9597f566cfc594121fc1cd8"
status: active
---

# Stable Public Documentation

The public stable manifest owns the version, date, surfaces, and release-note coverage for ordinary users.
README files, the install HTML, product overview, and document index show only that stable release.
Numbered prereleases remain maintenance evidence in npm, GitHub, and maintainer documentation, not install guidance.
Test candidates preserve the manifest stable; stable candidates require the requested version and date before build and publication.

CI runs the current release note and source-version metadata validator before candidates. Synthetic fixtures alone do not validate the actual release documents.
