---
schema_version: 1
pair_id: automatic-test-release-gate
topic_slug: automatic-test-release-gate
language: ko
counterpart: ../en/automatic-test-release-gate.md
title: "번호 시험판 자동 게시 gate"
summary: "완료된 승인 제품 milestone의 시험판 자동 게시·수용, source-only 변경·동일 제품의 후보 생성 차단"
tags: [automation, product, release]
aliases: ["번호 공개 시험 gate"]
sources:
  - "repo:.agents/directives/references/ci-and-candidates.md#sha256:b1b2a41cfcad009d6561bcc57e94bdd4ebcaee771931ed3369d85a6935dcd52c"
  - "repo:.github/workflows/release.yml#sha256:fe8bb871aaa0710a655f41521b7fe7960c63ff5c660795a2ee09aed29cb92631"
  - "repo:docs/public-test-product.json#sha256:e74df2d51c22fd8c21ca79f3b8919aadf3d0f115466976d7508868f99a57a799"
  - "repo:scripts/check-test-release-gate.py#sha256:75a37fd28d2aaf302c7079088b54c4cedb4060bd4497f4aa9219198ff024ce95"
links: [source-development, v0-9-full-release]
reviewed_revision: "git:62f70999f0bf83f7f9eacfb5add2cdb47a0b5716"
status: active
---

# 번호 시험판 자동 게시 gate

번호 시험판의 별도 승인 질문 없음. milestone 완료 때 에이전트가 `docs/test-release-intent.json`에 다음 번호·완료 plan ID·제품 지문 기록.
`check-test-release-gate.py`에서 해당 의도·마지막 수용 제품·후보 비교. 새 제품 byte만 후보·게시·공개 수용 자동 진행.
동일 제품과 문서·계획·사실·source-only Skill·지침·시험·CI·안내 변경은 후보 생성 거부.
안정판 명시 승인 유지.

후보의 정확한 SHA에 대한 필수 CI 성공 이후 생성. 재시도 입력 변화 기록, 같은 실패군 두 번째 발생 시 전체 실패 작업·운영체제 조건 일괄 조사. 입력 변화 없는 재시도 제외.
