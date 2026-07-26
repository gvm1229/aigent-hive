# ADR-0009: user plugin과 project knowledge 경계

- 상태: accepted
- 날짜: 2026-07-25

## 결정

Hive artifact class를 네 종류로 분리:

1. Hive source workspace
2. Immutable release bundle
3. User-scope Hive installation
4. Independent project harness

User-scope installation의 mutable 정본 위치: `~/.hive/`. Host plugin cache,
marketplace checkout과 package-manager binary는 배포 surface일 뿐 user data 정본에서
제외.

Host adapter:

- Codex: native Codex plugin package
- Claude Code: native Claude Code plugin package
- Gemini Antigravity: current official user-scope Skill package
- 공통 guidance: active host global instruction file의 exact
  `AIGENT-HIVE:USER` marker

Project harness:

- Shared project `AGENTS.md`의 existing `AIGENT-HIVE` marker contract 유지
- `.hive/`: setup, consent, role, run, knowledge의 project canonical state
- `.agents/directives`와 `.agents/skills`: release-generated provider-neutral projection
- Host-specific discovery path: exact generated adapter

User knowledge:

- `~/.hive/knowledge/`: cross-project canonical Markdown
- `~/.hive/index/hive.sqlite3`: canonical Markdown에서 재구축 가능한 disposable index
- Promotion 순서: eligibility·secret 검증 → root Markdown staging → atomic activation
  → root SQLite rebuild
- Project-neutral fact, reusable preference와 portable workflow만 promotion 허용
- Confidential, credential-adjacent, project-exclusive와 ambiguous content의 자동
  promotion 금지

Upgrade merge:

- Base: signed historical release의 exact directive·Skill bytes
- `local == base`: incoming exact replace
- `local != base`: non-overlapping incoming change 추가, overlapping change의 local 우선
- Missing·unauthenticated base 또는 typed schema incompatibility: active generation
  불변 conflict
- Active file의 textual conflict marker 금지, preview report에 omitted incoming hunk 기록

## 결과

- Existing OMX·OMC·user guidance byte 보존
- Source `hive-prompt-refine`: `harness/skills` canonical contract의 source-only
  `.agents/skills` projection, consumer shipping source로 사용 금지
- Prompt quality gate: 명시적 작성·개선 intent는 automatic `refine-only`, 모호성
  또는 핵심 세부 부족은 한 줄 optional refine 제안만 허용
- Refine 제안에서 prompt rewrite·Skill load·execution 0회, 충분히 명확한
  ordinary work·simple question은 제안 제외
- Plugin uninstall과 user knowledge 삭제 lifecycle 분리
- Project SQLite와 root SQLite의 독립 rebuild
- Current `.hive/` compatibility 유지와 `.agents/` additive projection
- Shipped behavior 구현 전 product version `0.7.0` 유지
