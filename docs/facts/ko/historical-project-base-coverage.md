---
schema_version: 1
pair_id: historical-project-base-coverage
topic_slug: historical-project-base-coverage
language: ko
counterpart: ../en/historical-project-base-coverage.md
title: "과거 프로젝트 기준본 수용 범위"
summary: "선언된 프로젝트 갱신 source range와 exact full 기준본·matrix 수용의 대응"
tags: [migration, project-upgrade, regression, release]
aliases: ["과거 기준본 정합성"]
sources:
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:8943d5559309ea5b084f211a4bda523bc88e1e5f6afdd23b6b1226e85a652bf5"
  - "repo:crates/hive-render/src/lib.rs#sha256:019c4b9187834d210c659a1ade13f9a30d5b04c45088e5184e04d0340797712e"
  - "repo:harness/release/0.9.4/migration-table.json#sha256:96fad7ef16ba8404447130124cdb21ac3bd4350492b438dbac88891f1ca1c3b3"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# 과거 프로젝트 기준본 수용 범위

프로젝트 갱신 source로 선언한 모든 version의 release bundle 내 exact full historical project base
필수. mutation 전 인증 필수. migration-table range: packaged binary·release test matrix의 coverage
증명 전 무효. 기준본 부재·digest 불일치: apply 전 중단·프로젝트 byte 보존.
