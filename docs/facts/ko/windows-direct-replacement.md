---
schema_version: 1
pair_id: windows-direct-replacement
topic_slug: windows-direct-replacement
language: ko
counterpart: ../en/windows-direct-replacement.md
title: "Windows 직접 교체와 옛 사본 정리"
summary: "검증한 갱신 완료와 정확한 옛 사본의 제한된 정리"
tags: [installation, update, windows]
aliases: []
sources:
  - "repo:crates/hive-cli/src/update_activation/direct_replacement.rs#sha256:79e8da18d7b1bc52635ebc7ac6f8a49258efced50104baefb1002a592f35ee6c"
  - "repo:tests/results/runs/20261028T145420-9c52bbfb7ef7.md#sha256:7b3faecc2237ab66cb606251706dbb748dd8b76e4be09d677587827623463e1d"
  - "repo:tests/results/runs/20261028T150434-59ea1195546c.md#sha256:57e83b84a6f8abb170de3c9fdc4cb1bdc8946b1640cc65ab023aac810824dc0b"
links: [interactive-binary-update]
reviewed_revision: "git:3ee8193d37b842ec45c87c1dcde70c18843c0694"
status: active
---

# Windows 직접 교체와 옛 사본 정리

0.14.0 완료 조건: 본체·설정 검증 후 부모 종료, 기록된 옛 사본만 종료 뒤 정리. 대기와 삭제 확인 구분. 폴더 식별자·버전·전후 지문 결합, Windows가 핸들 종료 때 없애는 새 전용 파일로 복구·정리 잠금. 기존 파일 보존·거부, 같은 기록 재개 시 재설치 없음. 격리 Windows의 비공개 실행 파일로 정상/중단 재개 수용, 로컬 파일 시험8개로 잠금 충돌·복구 검증. 공개 서명·실제 앱 제어는 별도 근거 필요.
