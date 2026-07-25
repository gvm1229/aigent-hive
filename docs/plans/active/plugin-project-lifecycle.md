# 사용자 plugin과 project harness lifecycle 계획

> Checklist owner: `RPH-*`
> Load condition: user-scope 설치·update, project bootstrap, root knowledge promotion,
> project upgrade 구현·검증
> Decision: [`ADR-0009`](../../decisions/ADR-0009-user-plugin-project-knowledge-boundary.md)

## 목표

- Codex·Claude Code·Gemini Antigravity의 native install surface에 맞춘 Aigent Hive 배포
- User-scope guidance의 exact Hive marker append와 foreign bytes 보존
- Project별 `AGENTS.md`, `.agents/`, `.hive/knowledge`, disposable SQLite 격리
- Project knowledge의 안전한 user-scope promotion과 root SQLite 재구축
- Signed base 기반 local-priority three-way update와 one-action root·project upgrade

## 확인된 기준선

| 범위 | 현재 상태 | 구현 gap |
| --- | --- | --- |
| Project setup 질문·render | 구현 완료 | user-scope plugin bootstrap과 promotion 질문 없음 |
| Shared `AGENTS.md` merge | marker append·own-block replace 구현 완료 | user-scope global guidance target 없음 |
| Host Skill projection | Codex·Antigravity `.agents/skills`, Claude `.claude/skills` | plugin package·항상 존재하는 project `.agents/directives` 없음 |
| Project knowledge | `.hive/knowledge` 정본과 `.hive/index/hive.sqlite3` | user-scope canonical knowledge·root index·promotion 없음 |
| Signed update | consumer target의 dry-run·backup·atomic activation | root install update·upgrade scan·local-priority three-way merge 없음 |

## 1. Boundary와 host adapter

- [x] [RPH-001] User installation·project harness·root knowledge·merge authority를
  ADR-0009로 고정
- [ ] [RPH-002] Codex·Claude Code·Antigravity의 current official install, global
  guidance, global Skill, project Skill surface를 dated capability matrix로 고정
- [ ] [RPH-003] Release bundle에 user-scope ownership manifest, host adapter metadata,
  source release digest와 supported host version 범위 추가
- [ ] [RPH-004] `hive install --scope user --host <host>`의 dry-run·apply·validate
  machine contract와 stable exit class 구현
- [ ] [RPH-005] Codex용 `.codex-plugin/plugin.json`, bundled Skills와 local marketplace
  qualification
- [ ] [RPH-006] Claude Code용 `.claude-plugin/plugin.json`, namespaced Skills와
  marketplace qualification
- [ ] [RPH-007] Antigravity의 official user-scope Skill surface를 사용하는 install
  adapter와 future native plugin manifest capability gate
- [ ] [RPH-008] Active user guidance file에 별도
  `AIGENT-HIVE:USER` marker append·own-block replace·foreign byte 보존
- [ ] [RPH-009] Codex `AGENTS.override.md` precedence, existing OMX block, Claude OMC
  block과 arbitrary foreign marker 공존 conformance
- [ ] [RPH-010] `hive update --scope user`의 release verify·dry-run·backup·atomic
  activation·validate·recover 구현
- [ ] [RPH-011] Plugin cache·marketplace checkout과 mutable `~/.hive/` user data의
  lifecycle·ownership·삭제 경계 분리

## 1.1 Source prompt refinement

- [ ] [RPH-037] `harness/skills/hive-prompt-refine`를 canonical source로 사용하는
  source-only `.agents/skills/hive-prompt-refine/SKILL.md` projection과 update parity
- [ ] [RPH-038] 명시적 prompt 작성·개선 intent의 source automatic routing,
  `refine-only` default, ordinary work negative route와 source·harness digest conformance

## 2. Project bootstrap

- [ ] [RPH-012] User plugin의 `setup-harness` 호출에서 repository read-only survey와
  one-question-at-a-time setup sequence 연결
- [ ] [RPH-013] Setup answers에 root knowledge promotion scope, confidential category,
  project identity와 user-store binding 추가
- [ ] [RPH-014] 모든 consumer project에 `.agents/directives/` generated projection,
  digest ledger와 ownership class 추가
- [ ] [RPH-015] 모든 consumer project에 `.agents/skills/` portable projection 유지,
  Claude의 `.claude/skills/`를 exact host-discovery adapter로 추가
- [ ] [RPH-016] 기존 project `AGENTS.md` marker merge와 `.hive/` canonical config를
  유지하고 `.agents/`를 machine authority가 아닌 release projection으로 제한
- [ ] [RPH-017] Setup apply에서 project별 Raw/Wiki/Schema, suppression ledger,
  ignored SQLite·lock·stale path 초기화와 rebuild 검증
- [ ] [RPH-018] `hive-source.json` source root 거부, symlink escape, foreign namespace,
  optional Skill consent와 fallback hook consent 회귀 방지

## 3. User-scope knowledge promotion

- [ ] [RPH-019] `~/.hive/knowledge/{Raw,Wiki,Schema}` canonical store와
  `~/.hive/index/hive.sqlite3` disposable root projection contract 구현
- [ ] [RPH-020] `hive knowledge promote --target <project> --dry-run|--apply`와
  thin `hive-knowledge-promote` Skill 구현
- [ ] [RPH-021] Project-neutral fact·reusable preference·portable workflow만 허용하는
  typed promotion policy와 ambiguous candidate explicit review
- [ ] [RPH-022] Setup exclude, secret scanner, credential/private path, confidential Raw,
  unrelated repository와 unapproved category의 root promotion 차단
- [ ] [RPH-023] Project pseudonymous provenance, source digest, deduplication,
  contradiction, replacement와 suppression 연결
- [ ] [RPH-024] Multi-project concurrent promotion의 root lock, optimistic digest,
  staging validation과 atomic canonical activation
- [ ] [RPH-025] Root canonical Markdown 우선 commit, root SQLite rebuild, project-local
  result 우선의 combined query와 provenance 표시
- [ ] [RPH-026] Root SQLite 삭제·schema bump·corruption 뒤 model call·network 없는
  logical rebuild equivalence

## 4. Root·project upgrade merge

- [ ] [RPH-027] Root installation과 project harness의 installed version, historical
  base digest, live local digest, incoming release digest scan·report
- [ ] [RPH-028] Signed historical built-in registry를 exact three-way base로 사용하고
  missing·unauthenticated base에서 active bytes 불변 conflict
- [ ] [RPH-029] `local == base`인 unmodified directive·Skill의 incoming exact replace
- [ ] [RPH-030] `local != base`인 text의 disjoint incoming hunk 추가, overlapping
  hunk local 우선, active file conflict marker 0개
- [ ] [RPH-031] YAML·TOML·JSON의 typed three-way merge, unknown user field·ordering
  보존과 incompatible schema fail-closed
- [ ] [RPH-032] `hive-project-upgrade` Skill의 scan·preview·apply·recover 연결,
  omitted incoming hunk와 local-priority 결정 report
- [ ] [RPH-033] Update 전 recoverable backup, exact plan digest, staged validation,
  atomic activation과 failed merge의 active generation 불변

## 5. Qualification

- [ ] [RPH-034] Empty·existing·malformed·nested global marker, OMX/OMC coexistence,
  override precedence와 source-root refusal hostile conformance
- [ ] [RPH-035] Cross-project leakage, secret candidate, duplicate preference,
  contradiction, concurrent promotion와 root index rebuild hostile conformance
- [ ] [RPH-036] Unmodified replace, user-modified local priority, disjoint upstream add,
  missing base, interrupted root·project update와 세 host install/update E2E

## Current host evidence — 2026-07-25

- Codex: `.codex-plugin/plugin.json`, bundled `skills/`, marketplace install과
  global `~/.codex/AGENTS.md`
- Claude Code: `.claude-plugin/plugin.json`, bundled `skills/`, marketplace install
- Antigravity: global `~/.gemini/config/skills/`, project `.agents/skills/`
- 공통 결론: 동일 product contract와 host별 native packaging adapter
