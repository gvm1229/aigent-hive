---
schema_version: 1
pair_id: native-host-qualified-scope
topic_slug: native-host-qualified-scope
language: ko
counterpart: ../en/native-host-qualified-scope.md
title: "네이티브 파일 훅의 수용 범위"
summary: "실측한 파일 경로와 검사기 실패 처리에 한정한 수용"
tags: [acceptance, hooks, scope]
aliases: []
sources:
  - "repo:docs/research/native-host-qualification-0.11.0.md#sha256:16e1a415e894e0502f46715f63d14d9b662f4c7c1fc445aecd0677e3e50f4aa2"
links: [antigravity-file-hook-acceptance, codex-native-error-observation]
reviewed_revision: "git:6c7a6485479b009b74d7ddac93c05291a061eef4"
status: active
---

# 네이티브 파일 훅의 수용 범위

0.11.0은 시험한 Windows Codex·Antigravity 버전의 신뢰·로드된 형식 3 파일 검사를 수용. 일반 표본 허용, 보호 표본 거부, 등록 검사기 부재 시 편집 거부 확인. 여섯 사례 비교에서 보호 편집은 훅 적용 시 0개·지침 단독 시 3개 성공. 호스트 시간 초과·손상 응답/설정·선택식 밖 도구는 필수 보호 경로에서 제외. Codex 자식은 같은 대상의 한 사례만 확인, Antigravity 상속·다른 운영체제 실제 앱은 미검증.
