---
schema_version: 1
pair_id: v0-9-full-release
topic_slug: v0-9-full-release
language: ko
counterpart: ../en/v0-9-full-release.md
title: "Aigent Hive 0.9.x 시험·정식 릴리스"
summary: "Stable v0.9.2: 완료된 usage guard와 공개 문서 최신화 게시, v0.9.3: 후속 명시적 승인 필요"
tags: [distribution, release, signing, v0-9]
aliases: ["0.9.2 scope", "0.9.3 scope", "0.9.x release plan", "full release"]
sources:
  - "repo:.github/workflows/release-publish.yml#sha256:4e93f1bb01339ed05f69cdb773c27ba83b704de8b24465f761e08e201955eb39"
  - "repo:README.md#sha256:3c390ad3b1a884c49a15304b0a0799299384e2e319e626ff7a752ecf4d700d94"
  - "repo:docs/decisions/ADR-0017-0.9-full-release.md#sha256:53314be9705bd61590992cae77cfcf96a9d823e7142821399e6411492de76e00"
  - "repo:docs/guides/release-update.md#sha256:f046e838fa7f44c6fa336fd089d4740c6f3f2a8ab8fb8a010e748f7b1d4bcd10"
  - "repo:docs/guides/release-verification-builds.md#sha256:e9490fbcdd337f9935957e641d73f834bdf602030d28c8c0808699a1606eb9d9"
  - "repo:docs/plans/active/release-0.9.2-test-qualification.md#sha256:4efec44f39d2eaf46b1e734323557e6d300899329d99016bba32b7ca05b6d003"
links: [host-external-integrations, release-verification, test-distribution, version-policy]
reviewed_revision: "git:a502867e6b20e8f22bc014af05ca678f211f40ed"
status: active
---

# Aigent Hive 0.9.x 시험·정식 릴리스

Stable `v0.9.2`: exact source `a502867`, candidate run `31609928346`, publication run
`31611457288`에서 게시 완료했고 npm `latest=0.9.2`. `2cec037`까지 완료된 설치 usage guard
정본 전환과 release-only metadata·qualification 범위. 모든 공개 README·설치 안내·HTML·npm
README·plugin metadata·문서 색인·명령·version 예시 최신화 완료. 공개 README는 stable
설치만 노출하고 중립적인 유지보수자 링크 1개로 별도 시험판 안내에 연결. Native orchestration·custom
subagent 구현 제외. `0.9.3`: QA contributor 추가 지시와 유지보수자의 후속 명시적 승인 전 동결.
