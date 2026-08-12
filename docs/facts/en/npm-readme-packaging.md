---
schema_version: 1
pair_id: npm-readme-packaging
topic_slug: npm-readme-packaging
language: en
counterpart: ../ko/npm-readme-packaging.md
title: "npm Umbrella README Packaging"
summary: "The public aigent-hive npm package derives its README from the root English README, excluding QA Contributors and rewriting repository-local links."
tags: [distribution, npm, readme]
aliases: ["npm README", "package README"]
sources:
  - "repo:scripts/package-npm.mjs#sha256:94aa95c81a3a694e44ede1f1189ffb9588c70b6246d06d07ad0c177dba3783b9"
links: [release-verification, test-distribution]
reviewed_revision: "git:dbba8080101fe7b01168c49bee35228d0278b239"
status: active
---

# npm Umbrella README Packaging

The `aigent-hive` umbrella package reads the root English `README.md` during packaging.
It removes only the `QA Contributors` section, changes repository-relative documentation links
and the banner asset to public GitHub URLs, and preserves the remaining README content. Platform
packages retain their compact package-specific README. The packaging conformance test verifies
both the generated package directory and the packed tarball.
