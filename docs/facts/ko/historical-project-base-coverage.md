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
  - "repo:crates/hive-cli/src/project_upgrade.rs#sha256:162954ace665a9f30166cf241abe18b5e1168ebd8e862c106819a142d496bd46"
  - "repo:crates/hive-cli/tests/historical_project_upgrade.rs#sha256:0aa8c272002f64443b8204e80f5744c02474e4621ca807d28cfe36ff3bdb49f6"
  - "repo:crates/hive-render/src/lib.rs#sha256:9ac7b87b5dde4f582027a219d4695c9158115e99041f10e304089cce4f55a30e"
  - "repo:scripts/check-project-base-coverage.py#sha256:b7d0887ccd3e3a9019383c3cea283189361c17fa62d806f44096cd6825b67579"
links: [projection-upgrade-purge, update-transaction, version-policy]
reviewed_revision: "git:9170c884c9c96d99abcea1f5ab96a4a3a62541be"
status: active
---

# 과거 프로젝트 기준본 수용 범위

향후 `0.9.5` 후보의 전체 프로젝트 기준본 원본 범위는 `0.9.1`부터 `0.9.4`까지다. 수용 범위 검사기는
다이제스트 결합 보고서를 만들고, `0.9.1`보다 낮은 같은 주 버전 원본 범위는 거부한다. 컴파일한 명령줄
도구는 `0.9.1`부터 `0.9.3`까지 검사·미리 보기·되돌리기·적용·검증을 통과했고, `0.9.4` 동결 기준본은
해당 출시 태그와 바이트까지 일치한다. 기준본이 없거나 변조되면 적용 전에 중단하며 프로젝트와 외부 파일을 보존한다.
