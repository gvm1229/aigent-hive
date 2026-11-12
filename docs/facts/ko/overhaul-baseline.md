---
schema_version: 1
pair_id: overhaul-baseline
topic_slug: overhaul-baseline
language: ko
counterpart: ../en/overhaul-baseline.md
title: "재정비 기준선"
summary: "Hive 0.14.2의 Windows 프로젝트·검색·크기 실측 기준선"
tags: [baseline, overhaul, retrieval]
aliases: [overhaul-baseline]
sources:
  - "repo:docs/research/overhaul/OVX-001.md#sha256:a915958a3ba29cede7f163a2bf3b7ddb1bb4a3225e856427d67c8fe89943e55b"
  - "repo:docs/research/overhaul/OVX-002.md#sha256:9d142f8151f828020262bfad89a34f41e989435ec7c3e60ef26e16be1c4681a7"
  - "repo:docs/research/overhaul/OVX-004.md#sha256:32b2ddcd2538fb0beae217e5ff27d94fe450abf26f27153d94447b796a19b821"
links: []
reviewed_revision: "git:52a1aec50a92ea4a4647871304487a5c9ba2f3b4"
status: active
---

# 재정비 기준선

2026-11-12 Windows의 Hive 0.14.2 측정: 최소 프로젝트 30종 모두 기본 general, 실제 선택값 없음. 고정 한·영 단어 검색의 상위 다섯 개 정답 적중 22/40, 보류 질의 3/10. 새 Codex 프로젝트 파일 81개, 정본 Skill 28개·공백 분리 14,527단어, 합성 사용자 저장량 198,235바이트. 자동 프로젝트 판별·의미 검색·다른 구성·다른 운영체제의 수용 근거와 구분.

단계 2 검토 보완: `tests/fixtures/project-detection/expected.json`에 30종·정확한 표시 파일 내용 보존. 같은 Windows 호스트 재측정에서 general 기본값 30개·실제 선택값 null 30개 확인. `docs/research/overhaul/OVX-002-inputs.json`에 정답 10개·방해 원문 5개·고정 질의 40개 보존. 요청 범위: 기준선 입력 재현성 보완, 다른 운영체제 판별·검색 품질 증명 제외.
