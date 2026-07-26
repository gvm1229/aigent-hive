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
| Project setup 질문·render | promotion·confidential scope·project identity·user store binding 포함 | 실제 세 host E2E 잔여 |
| Shared guidance merge | user·project marker append·own-block replace·foreign byte 보존 | 실제 host update E2E 잔여 |
| Host plugin·Skill projection | 세 host native plugin package, portable `.agents/skills`, Claude adapter | 실제 host install/update E2E 잔여 |
| Project·root knowledge | canonical Markdown, project/root SQLite, explicit promotion·rebuild | protected host qualification 잔여 |
| Project·user update | durable journal·backup·recover와 historical-base local-priority merge | 실제 세 host E2E 잔여 |

## 1. Boundary와 host adapter

- [x] [RPH-001] User installation·project harness·root knowledge·merge authority를
  ADR-0009로 고정
- [x] [RPH-002] Codex·Claude Code·Antigravity의 current official install, global
  guidance, global Skill, project Skill surface를 dated capability matrix로 고정
- [x] [RPH-003] Release bundle에 user-scope ownership manifest, host adapter metadata,
  source release digest와 supported host version 범위 추가
- [x] [RPH-004] `hive install --scope user --host <host>`의 dry-run·apply·validate
  machine contract와 stable exit class 구현
- [x] [RPH-005] Codex용 `.codex-plugin/plugin.json`, bundled Skills와 local marketplace
  qualification
- [x] [RPH-006] Claude Code용 `.claude-plugin/plugin.json`, namespaced Skills와
  marketplace qualification
- [x] [RPH-007] Antigravity의 native `plugin.json`·bundled `skills/` install adapter와
  compatibility global Skill projection
- [x] [RPH-008] Active user guidance file에 별도
  `AIGENT-HIVE:USER` marker append·own-block replace·foreign byte 보존
- [x] [RPH-009] Codex `AGENTS.override.md` precedence, existing OMX block, Claude OMC
  block과 arbitrary foreign marker 공존 conformance
- [x] [RPH-010] `hive update --scope user`의 release verify·dry-run·backup·atomic
  activation·validate·recover 구현
- [x] [RPH-011] Plugin cache·marketplace checkout과 mutable `~/.hive/` user data의
  lifecycle·ownership·삭제 경계 분리

## 1.1 Source prompt refinement

- [x] [RPH-037] `harness/skills/hive-prompt-refine`를 canonical source로 사용하는
  source-only `.agents/skills/hive-prompt-refine/SKILL.md` projection과 update parity
- [x] [RPH-038] 명시적 prompt 작성·개선 intent의 automatic `refine-only` routing,
  모호하거나 핵심 세부가 부족한 prompt의 한 줄 optional refine 제안, 자동
  rewrite·Skill load·execution 0회, 충분히 명확한 ordinary work·simple question
  negative route와 source·harness routing conformance

## 2. Project bootstrap

- [x] [RPH-012] User plugin의 `setup-harness` 호출에서 repository read-only survey와
  one-question-at-a-time setup sequence 연결
- [x] [RPH-013] Setup answers에 root knowledge promotion scope, confidential category,
  project identity와 user-store binding 추가
- [x] [RPH-014] 모든 consumer project에 `.agents/directives/` generated projection,
  digest ledger와 ownership class 추가
- [x] [RPH-015] 모든 consumer project에 `.agents/skills/` portable projection 유지,
  Claude의 `.claude/skills/`를 exact host-discovery adapter로 추가
- [x] [RPH-016] 기존 project `AGENTS.md` marker merge와 `.hive/` canonical config를
  유지하고 `.agents/`를 machine authority가 아닌 release projection으로 제한
- [x] [RPH-017] Setup apply에서 project별 Raw/Wiki/Schema, suppression ledger,
  ignored SQLite·lock·stale path 초기화와 rebuild 검증
- [x] [RPH-018] `hive-source.json` source root 거부, symlink escape, foreign namespace,
  optional Skill consent와 fallback hook consent 회귀 방지

## 3. User-scope knowledge promotion

- [x] [RPH-019] `~/.hive/knowledge/{Raw,Wiki,Schema}` canonical store와
  `~/.hive/index/hive.sqlite3` disposable root projection contract 구현
- [x] [RPH-020] `hive knowledge promote --target <project> --dry-run|--apply`와
  thin `hive-knowledge-promote` Skill 구현
- [x] [RPH-021] Project-neutral fact·reusable preference·portable workflow만 허용하는
  typed promotion policy와 ambiguous candidate explicit review
- [x] [RPH-022] Setup exclude, secret scanner, credential/private path, confidential Raw,
  unrelated repository와 unapproved category의 root promotion 차단
- [x] [RPH-023] Project pseudonymous provenance, source digest, deduplication,
  contradiction, replacement와 suppression 연결
- [x] [RPH-024] Multi-project concurrent promotion의 root lock, optimistic digest,
  staging validation과 atomic canonical activation
- [x] [RPH-025] Root canonical Markdown 우선 commit, root SQLite rebuild, project-local
  result 우선의 combined query와 provenance 표시
- [x] [RPH-026] Root SQLite 삭제·schema bump·corruption 뒤 model call·network 없는
  logical rebuild equivalence

## 4. Root·project upgrade merge

- [x] [RPH-027] Root installation과 project harness의 installed version, historical
  base digest, live local digest, incoming release digest scan·report
- [x] [RPH-028] Signed historical built-in registry를 exact three-way base로 사용하고
  missing·unauthenticated base에서 active bytes 불변 conflict
- [x] [RPH-029] `local == base`인 unmodified directive·Skill의 incoming exact replace
- [x] [RPH-030] `local != base`인 text의 disjoint incoming hunk 추가, overlapping
  hunk local 우선, active file conflict marker 0개
- [x] [RPH-031] YAML·TOML·JSON의 typed three-way merge, unknown user field·ordering
  보존과 incompatible schema fail-closed
- [x] [RPH-032] `hive-project-upgrade` Skill의 scan·preview·apply·recover 연결,
  omitted incoming hunk와 local-priority 결정 report
- [x] [RPH-033] Update 전 recoverable backup, exact plan digest, staged validation,
  atomic activation과 failed merge의 active generation 불변

## 5. Qualification

- [x] [RPH-034] Empty·existing·malformed·nested global marker, OMX/OMC coexistence,
  override precedence와 source-root refusal hostile conformance
- [x] [RPH-035] Cross-project leakage, secret candidate, duplicate preference,
  contradiction, concurrent promotion와 root index rebuild hostile conformance
- [ ] [RPH-036] 실제 Codex·Claude Code·Antigravity user install·update E2E

잔여 evidence:

- `RPH-036`: fixture 기반 install/update·merge·interruption 검증 완료, 실제 세 host
  install/update E2E 잔여

## Current host evidence — 2026-07-26

- Codex: `.codex-plugin/plugin.json`, bundled `skills/`, marketplace install과
  nonempty `~/.codex/AGENTS.override.md` 우선, 그 외 `~/.codex/AGENTS.md`
- Claude Code: `.claude-plugin/plugin.json`, bundled `skills/`, marketplace install,
  `~/.claude/CLAUDE.md`
- Antigravity: `~/.gemini/config/plugins/aigent-hive/plugin.json`, bundled `skills/`,
  compatibility global `~/.gemini/config/skills/`
- 공통 결론: 동일 product contract와 host별 native packaging adapter
