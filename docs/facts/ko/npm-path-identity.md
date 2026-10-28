---
schema_version: 1
pair_id: npm-path-identity
topic_slug: npm-path-identity
language: ko
counterpart: ../en/npm-path-identity.md
title: "npm 경로의 소유 범위와 명령 표기"
summary: "정규 경로 검증과 npm 원래 인수 표기의 분리"
tags: [npm, preservation, uninstall, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/update_activation/removal.rs#sha256:282796ffeb06e2aa788402c0a950b8a35159a271a4ccde6daaf313df052b8451"
  - "repo:tests/results/runs/20261028T032255-d34a85916924.md#sha256:9838c31c45fc49982c643fcf1aa444eca433bcd61bb436c7f8bbf7d51cbe4bf1"
links: [hive-preserving-uninstall]
reviewed_revision: "git:4f4d621911ee0c15efab21a4e7d27b7022ea1101"
status: active
---

# npm 경로의 소유 범위와 명령 표기

Windows npm 제거의 소유 범위는 정규 경로로 대조하고, 명령 인수는 npm이 반환한 원래 절대 경로 표기 유지. Hive의 내부 `\\?\` 표기를 npm에 새로 전달하지 않는 계약.

0.14.0 보존형 제거 요청의 실제 격리 Windows npm 수용: 사용자 자료와 별도 패키지 보존. 개인 설치·실제 앱/감시·공개0.14.0 배포 파일은 증명 범위 제외.
