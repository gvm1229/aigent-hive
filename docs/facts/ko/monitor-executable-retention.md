---
schema_version: 1
pair_id: monitor-executable-retention
topic_slug: monitor-executable-retention
language: ko
counterpart: ../en/monitor-executable-retention.md
title: "감시 실행 파일 보관"
summary: "격리한 감시 파일의 전체 바이트와 끝을 대조하는 정확 삭제."
tags: [monitor, safety, storage]
aliases: [monitor-executable-retention]
sources:
  - "repo:crates/hive-cli/src/run.rs#sha256:4fdb81d3893e68681945090bedccfaa3b4a74f88593ec169a4ec0ef10c19fbc9"
  - "repo:crates/hive-cli/src/usage_monitor/upgrade.rs#sha256:00b66cc8825ac157550b7dd280cb7599f94bb9172069866df550bb4d8ae2525b"
  - "repo:docs/research/overhaul/OV142-003.md#sha256:d47f74746c332a0ef804eb943106ecd472ed490995c924e7303587dcfd545ae9"
  - "repo:docs/research/overhaul/OV142-008.md#sha256:c95277586522b5b471f3e2e4968999bda076573c5dd68dbaf16919f30dded9d0"
links: [developer-binary-lifecycle]
reviewed_revision: "git:8edffe0b8ddc102983ba6ad40439d349ea6b7b1c"
status: active
---

# 감시 실행 파일 보관

- 현재·직전 실행 파일 보관
- 동일 부모의 격리 폴더로 확보한 파일을 링크 추적 없이 열어 64 KiB씩 전체 예상 바이트·파일 끝 대조
- 읽기 실패·바이트 변경·Windows 삭제 공유 잠금 시 파일 보존, 호출자의 128 MiB 스냅샷 한도 유지
- Windows의 51 MiB 파일·갱신 정리 경로·실제 prune 명령 시험 통과. 실제 감시 교체·다른 운영체제의 수용 증명 제외

- 실행 중 Windows 사본의 삭제 오류 5 뒤 원위치 복구·skipped·격리 폴더 제거 확인

- 정식 0.14.2 Windows 갱신: 실제 큰 사본 5개 정리 뒤 현재·직전 2개 보관, prune 후보 0
