---
schema_version: 1
pair_id: npm-readme-packaging
topic_slug: npm-readme-packaging
language: ko
counterpart: ../en/npm-readme-packaging.md
title: "npm umbrella README 패키징"
summary: "공개 aigent-hive npm package의 README: root English README 기반, QA Contributors 제외, repository-local link 변환"
tags: [distribution, npm, readme]
aliases: ["npm README", "package README"]
sources:
  - "repo:scripts/package-npm.mjs#sha256:94aa95c81a3a694e44ede1f1189ffb9588c70b6246d06d07ad0c177dba3783b9"
links: [release-verification, test-distribution]
reviewed_revision: "git:dbba8080101fe7b01168c49bee35228d0278b239"
status: active
---

# npm umbrella README 패키징

`aigent-hive` umbrella package 패키징 시 root English `README.md` 사용. `QA Contributors`
구간만 제외하고 repository-relative 문서 link·banner asset은 public GitHub URL로 변환. 나머지
README 본문 유지. Platform package는 기존의 짧은 package 전용 README 유지. Packaging conformance
test에서 생성 package directory와 packed tarball 확인.
