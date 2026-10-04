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
  - "repo:.github/workflows/release.yml#sha256:fe8bb871aaa0710a655f41521b7fe7960c63ff5c660795a2ee09aed29cb92631"
  - "repo:README.md#sha256:ef0e22aa026b267338f193ae7a9ebe5ff341609a9f10410f21d0dbb54fe7f853"
  - "repo:docs/public-stable-release.json#sha256:4afe5f0576ee7b6cf52338d94c3534336f5d79d6941abd9ad7acc4e578c03882"
  - "repo:scripts/check-public-stable-docs.py#sha256:69b25685285621ee94a515748de03c56b9100ca0e2f9e283bdc35a2278cb9f04"
links: [product-purpose, release-verification]
reviewed_revision: "git:62f70999f0bf83f7f9eacfb5add2cdb47a0b5716"
status: active
---

# Stable Public Documentation

The public stable manifest owns the version, date, surfaces, and release-note coverage for ordinary users.
README files, the install HTML, product overview, and document index show only that stable release.
Numbered prereleases remain maintenance evidence in npm, GitHub, and maintainer documentation, not install guidance.
Test candidates preserve the manifest stable; stable candidates require the requested version and date before build and publication.
