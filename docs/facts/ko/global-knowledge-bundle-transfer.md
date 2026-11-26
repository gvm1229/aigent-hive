---
schema_version: 1
pair_id: global-knowledge-bundle-transfer
topic_slug: global-knowledge-bundle-transfer
language: ko
counterpart: ../en/global-knowledge-bundle-transfer.md
title: "전역 지식 번들 이전"
summary: "전역 .hivekb 이전은 현재 셸의 홈 경로, SHA-256 확인, dry-run, 명시 apply 순서 사용."
tags: [bundle, global, knowledge, portability]
aliases: [".hivekb 이전", "지식 내보내기 가져오기"]
sources:
  - "repo:README.md#sha256:05c2f90b0096663c3c8ded051eec7d7632df4b99442d90a77faca4f26b5bf80b"
  - "repo:docs/archive/plans/releases/0.9.5/knowledge-bundle-portability-0.9.5.md#sha256:78721fbbaf589353a17fdee534e5c86f1406283cf546eb32acd9996e84adb3c3"
  - "repo:docs/hive-install-guide.ko.html#sha256:23b37461ada532dd3d847b82ea1af10ddc9c7fa164d95b3e6868c897aa47640b"
links: [knowledge-portability-scan, knowledge-storage]
reviewed_revision: "git:c5b13bc26c4b2485dcad7509bc8590f97d0831fe"
status: active
---

# 전역 지식 번들 이전

`--user-root`는 `.hive`가 아닌 사용자 홈 디렉터리. 현재 호스트에 맞는 macOS/Linux 또는
Windows 셸 예시만 사용. SHA-256 확인과 충돌 없는 `--dry-run` 뒤에만 `--apply` 실행.
bundle에는 이식 가능한 Markdown만 포함하며 SQLite 색인·runtime 상태·project-private 지식·자격 증명·절대 경로는 없음.
