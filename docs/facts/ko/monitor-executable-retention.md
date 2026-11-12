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
  - "repo:crates/hive-cli/src/run.rs#sha256:6123c4aa36e88044434ef8bdca5ecfd8b470edcd5cfda1ea7c63171450506dfe"
  - "repo:crates/hive-cli/src/usage_monitor/upgrade.rs#sha256:5d30e3818a7aa00b7b08510ef512e668a0e189055932b8ba8787724c2278a4dd"
  - "repo:docs/research/overhaul/OV142-003.md#sha256:d47f74746c332a0ef804eb943106ecd472ed490995c924e7303587dcfd545ae9"
links: [developer-binary-lifecycle]
reviewed_revision: "git:88ecb7a13736634afca601fe354e53c88b0dc58e"
status: active
---

# 감시 실행 파일 보관

- 현재·직전 실행 파일 보관
- 동일 부모의 격리 폴더로 확보한 파일을 링크 추적 없이 열어 64 KiB씩 전체 예상 바이트·파일 끝 대조
- 읽기 실패·바이트 변경·Windows 삭제 공유 잠금 시 파일 보존, 호출자의 128 MiB 스냅샷 한도 유지
- Windows의 51 MiB 파일·갱신 정리 경로·실제 prune 명령 시험 통과. 실제 감시 교체·다른 운영체제의 수용 증명 제외

- 실행 중 Windows 사본의 삭제 오류 5 뒤 원위치 복구·skipped·격리 폴더 제거 확인
