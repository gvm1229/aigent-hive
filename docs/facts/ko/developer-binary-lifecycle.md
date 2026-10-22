---
schema_version: 1
pair_id: developer-binary-lifecycle
topic_slug: developer-binary-lifecycle
language: ko
counterpart: ../en/developer-binary-lifecycle.md
title: "개발 실행 파일과 공개판 전환"
summary: "전역 개발 덮어쓰기 차단과 실제 Windows 공개판 전환의 데이터·설정 보존 검증."
tags: [development, installation, version]
aliases: ["Dev install", "Local developer build"]
sources:
  - "repo:crates/hive-cli/build.rs#sha256:248e82fd5ea76c34fca96b454e8192c30e0f24cd470b0f609fa2df6f47bf2eed"
  - "repo:crates/hive-cli/src/main.rs#sha256:c7dfa1349f5239cd9df726f0fbf7d9a73a3da5a90218551cac420c0d7331a9bd"
  - "repo:crates/hive-cli/src/user_install.rs#sha256:ee4aa374f3452d250a376e66c1d1d49ce28c076b95aab8a8bdb9b155e05bca7e"
  - "repo:docs/research/developer-install-recovery.md#sha256:552390190deb914cf2dfd84106461b994b7ee3587a5ce954102cfa62f74d9925"
  - "repo:scripts/dev-install.sh#sha256:49d7e923900b37bf30b8678e150bfb7f21ad4427876da5031b46aac04df8560e"
  - "repo:scripts/recover-dev-install.py#sha256:128ee7d1aa5dbd883f963df2ec9eb1449771d4343bda1deef1e94161d7bf964b"
links: [interactive-binary-update, source-development, version-policy]
reviewed_revision: "git:9fc4580a60a40af3161efb32b68726d66b41f294"
status: active
---

# 개발 실행 파일과 공개판 전환

- 사용자 요청: 데이터·설정 보존 갱신과 개발 절차 수정, 실제 전환 검증
- 개발 빌드: 별도 실행 파일·시험 사용자 루트. 전역 교체·과거 감시 배포는 변경 전 거부
- 과거 실행 파일 원복만으로 사용자 투영 복구 불가. Windows 복구 도구의 원본 CLI 인증·생성 파일 한정 백업 뒤 공식 CLI 재생성·갱신
- 실제0.13.0 설치 검증·대화형 업데이트 통과, 지식·선호·기록 보존. 다른 OS 복구 미지원
- 공개판 소유 검사 유지. 같은 버전 개발판의 기존 투영 인증도 실제 관리 바이트 일치 조건 유지
