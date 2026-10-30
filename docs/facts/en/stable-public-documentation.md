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
  - "repo:README.md#sha256:e60a845b60ef813d986cf2a115573c78d20e0b0ec0b6061bdb98a1ed7f76b9c2"
  - "repo:docs/public-stable-release.json#sha256:6f8b90c07df2fa8def62d357ff9bedd945aa1a24c3f52ea0a3590a5322e1553f"
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
