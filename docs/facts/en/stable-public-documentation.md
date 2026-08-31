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
  - "repo:.github/workflows/release-publish.yml#sha256:35420bffac94da9392c605c6512edffa879458e177e892e407d9a979feffc693"
  - "repo:.github/workflows/release.yml#sha256:a15f748db5a727188a90c8836fe1a80235a5221f3896dff6f088b3dfaa3b28a4"
  - "repo:README.md#sha256:27679c3c338ef2f82b352800ccb882c2536bcc2c7dbfd18b93df52e3349554b0"
  - "repo:docs/public-stable-release.json#sha256:d06e22bccdbd8dc6b359be7e827e6bb2a2d981777f42d4fe8f600d92244c203c"
  - "repo:scripts/check-public-stable-docs.py#sha256:fecedad7d9cde787974550b0d754ceedfd0f432f9f035bddb044fbb986b6d6b6"
links: [product-purpose, release-verification]
reviewed_revision: "git:8a45250106590f065df639132298b840940a3a35"
status: active
---

# Stable Public Documentation

The public stable manifest owns the version, date, surfaces, and release-note coverage for ordinary users.
README files, the install HTML, product overview, and document index show only that stable release.
Numbered prereleases remain maintenance evidence in npm, GitHub, and maintainer documentation, not install guidance.
Test candidates preserve the manifest stable; stable candidates require the requested version and date before build and publication.
