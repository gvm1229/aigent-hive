---
schema_version: 1
pair_id: version-policy
topic_slug: version-policy
language: ko
counterpart: ../en/version-policy.md
title: "버전 정책"
summary: "최초 공개 안정판 0.9.1부터 영구 적용하는 전체 버전 호환성 규칙."
tags: [release, semver, version]
aliases: ["Version lifecycle"]
sources:
  - "repo:docs/decisions/ADR-0006-version-lifecycle.md#sha256:c49c0f5cba2370bf031d454ed1e5fbc0fd70fa07793f55e14eeafe0d45e8f066"
links: [release-verification, test-distribution]
reviewed_revision: "git:c6d9362af3b489596fafe933e816fb2126bff83d"
status: active
---

# 버전 정책

호환 기능 추가는 바로 다음 부 버전, 호환 수정은 바로 다음 수정 버전 사용.
주 버전 준비는 사용자의 정확한 버전 지정과 별도 확인 필수.

최초 공개 안정판은 0.9.1. 이전 버전은 하위 호환성·과거 이력 참조 대상에서 제외하되 기존 자료 삭제 권한은 아님.
0.9.1부터 최신까지 모든 버전의 하위 호환성 보장 필수. 바로 이전 버전만이 아닌 전체 버전 대상이며 앞으로도 계속 적용.
버전 번호 증가로 의무 해제 금지. 사용자 요구사항이며 전체 호환성 검증 완료 주장은 아님.
