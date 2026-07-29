# Phase 7 public qualification

> Checklist owner: `P7-*`
> Load condition: Phase 7 qualification 항목 실행·검증·reconciliation

### Phase 7. Public qualification — target `0.8.0`; `1.0.0`은 explicit major approval 전 금지

- [x] [P7-001] Shipping `hive-usage-guard` Skill과 typed `hive usage enforce|status|threshold|session` command 구현
- [x] [P7-002] Installed threshold 변경의 ownership·atomic update·same-major migration conformance
- [x] [P7-003] Explicit current-session disable/enable/toggle, 새 session default-enable와 stale override hostile test
- [x] [P7-004] Installed·pinned host binding, current halt 우선, exit `3` automatic dispatch 차단, exit `0` preflight-only와 별도 one-brief authorization conformance
- [x] [P7-005] Skill 이름 없는 명백한 threshold·disable·enable semantic intent routing contract
- [x] [P7-006] Source directive와 consumer guidance marker에 사람용 문서 명사형·간결한 한국어 style projection
- [x] [P7-007] Ignored session-bound marker와 one-shot pre-dispatch `enforce`의 세 host projection parity
- [x] [P7-008] Session window 우선·weekly-only fallback을 threshold/session-control 경로에서도 유지
- [x] [P7-009] Compatible OMX/OMC cancellation은 보조 evidence로만 사용하고 durable goal/task halt로 오인 금지
- [x] [P7-010] Harness guard가 fallback hook, prompt rewrite, Skill activation, watcher, orchestration 또는 Stop continuation을 설치 금지
- [x] [P7-011] macOS arm64/x86_64 release build·install·runtime qualification
- [x] [P7-012] Windows x86_64 release build·install·runtime qualification
- [x] [P7-013] Codex·Antigravity 실제 E2E와 Claude fixture·unverified disclosure
- [x] [P7-014] host-native/OMX/OMC support matrix schema·fixture conformance
- [x] [P7-015] upgrade/migration fault injection
- [x] [P7-016] in-toto/SLSA provenance verifier와 candidate-workflow attestation contract
- [x] [P7-017] public license 확정 — 전체 source·harness `Apache-2.0`, GitHub 감지와 REUSE 검증 완료
- [ ] [P7-018] `0.8.x` release candidate qualification

P7-011·012 historical evidence:
[`ec27458` run](https://github.com/gvm1229/aigent-hive/actions/runs/30201803879).
Current attestation·publication은 P7-020·037, Windows 실제 기기는 P7-041 소유.

## 7. 핵심 conformance와 fault injection

| Scenario | 기대 결과 |
| --- | --- |
| source root setup | write 0회, 명확한 거부 |
| user `AGENTS.md` + Hive marker | marker만 변경 |
| OMC/OMX marker 공존 | external bytes 불변 |
| Codex/Claude + compatible OMX/OMC | external owner 자동 resolve, Hive duplicate Skill·hook 0개 |
| external runtime absent | host-native resolve, hook consent 질문 가능 |
| external runtime incompatible/unknown | 진단 또는 best-effort native, hook install 0개 |
| fallback hook 거절 | setup 성공, hook artifact·command 0개 |
| fallback hook 일부 승인 | 승인 event/capability/path만 projection |
| fallback hook 승인 후 external runtime 등장 | hook neutral/inert, reconfigure가 Hive entry만 제거 |
| optional Skill 미승인 | artifact·hook 0개 |
| 승인 capability가 요청 범위를 초과 | staging 전 거부 |
| Skill consent payload field 변조 | projection/activation 0회, 재승인 요구 |
| investigate 요청 + OMX/OMC analyze available | external `analyze` 자동 선택 |
| prompt 작성·개선 요청 | `hive-prompt-refine` 자동 선택, refine-only default |
| 일반 prompt | hidden prompt rewrite 0회 |
| prompt refine에서 필수 정보 누락 | 한 번에 한 질문 또는 explicit placeholder |
| role seed 재적용 | role file byte 변경 0건 |
| role definition drift | 명시 승인 전 conflict, assignment·handoff·body 보존 |
| CLI 실패 | exit class와 schema-valid `ActionResult` 일치 |
| SQLite 삭제 | Markdown에서 동일 logical index 재구축 |
| deprecated Wiki 삭제 | active query 0건, suppression metadata만 유지 |
| stale usage sensor | automatic continuation 0회 |
| sensor 없음 | 20% enforcement claim 없음 |
| installed threshold override mismatch | automatic brief 0개, 입력 거부 |
| same-reset remaining 증가 또는 timestamp 역행 | `usage_unknown`, automatic brief 0개 |
| source watcher가 10% line 감지 | current session halt marker 생성, 다음 source action 0개 |
| source session guard disable | explicit confirmation이 있을 때 current session만 bypass, 새 session은 enabled |
| shipping guard threshold 변경 | owned config만 atomic 변경, user/foreign bytes와 run authority 불변 |
| shipping session override replay | 다른 session/PID에서 거부, automatic continuation 0회 |
| exact authorization 재요청 | sensor 재호출 없이 `already_issued`, brief 0개 |
| capture된 authorization JSON 외부 replay | host/orchestration owner가 authorization ID 중복 소비 거부 |
| task-agent self review | final approval 거부 |
| requester/task agent가 judge roster 또는 approver | assignment/approval 거부 |
| owner provenance missing/invalid | `INDETERMINATE` |
| unsigned legacy judge request | `authenticated:false`, `INDETERMINATE` |
| target-contained 또는 caller-writable trust root | command blocked, PASS 0개 |
| judge signature/key purpose/principal/artifact mismatch | 해당 verdict 제외, PASS 승격 금지 |
| revoked/out-of-window/duplicate judge key | trust root 또는 attestation 거부 |
| judge disagreement | tier quorum에 따라 FAIL/INDETERMINATE |
| session 종료 | PLAN/STATUS/evidence로 재개 |
| resolved OMX/OMC가 run 중 실패 | hidden fallback·owner switch 0회 |
| feature release | `Y` 증가와 same-major compatibility 검증 |
| compatible quick bugfix | `Z` 증가 |
| major bump 요청 없음 | `X` 증가 0회 |
| same-major breaking template | release/update 거부 |
| cross-major migration crash | active generation 불변 또는 forward recovery |
| update 중 user edit | conflict, user bytes 보존 |
| backup age > 7 days | expired backup만 정리 |
| tampered release | install/update 거부 |
| provider API dependency 추가 | architecture/CI gate 실패 |

## 8. 완료 gate

`0.8.0` preview release의 필수 조건:

- [x] [P7-019] source, release, consumer tree 분리
- [ ] [P7-020] macOS·Windows archive SHA-256, GitHub attestation·source provenance
- [x] [P7-021] Codex·Antigravity 실제 matrix와 Claude fixture·unverified 표시
- [x] [P7-022] model-provider API dependency와 credential path 0개
- [x] [P7-023] setup dry-run, ownership, conflict와 source guard
- [x] [P7-024] action/role/run/judge/capability machine contract conformance
- [x] [P7-025] simple-question negative capability test
- [x] [P7-026] `hive-prompt-refine` automatic intent match, meaning preservation과 refine-only isolation
- [x] [P7-027] approved Skill의 automatic minimal routing과 OMX/OMC precedence
- [x] [P7-028] external absent + explicit consent에서만 fallback hook projection
- [x] [P7-029] fallback hook이 routing, prompt rewrite, orchestration과 Stop continuation을 수행 금지
- [x] [P7-030] persistent role/run fresh-session recovery
- [x] [P7-031] host-native 또는 external orchestration truthful support 표시
- [x] [P7-032] usage guard의 freshness와 fail-closed 증거
- [x] [P7-033] hostile judge context isolation과 authenticated provenance-bound quorum
- [x] [P7-034] Karpathy Raw/Wiki/Schema와 SQLite rebuild
- [x] [P7-035] same-major compatibility
- [x] [P7-036] cross-major no-data-loss migration
- [ ] [P7-037] `Claude-unverified preview` label·provenance·known limitation publication
- [x] [P7-038] product version parity, compatible minor/patch bump와 explicit-only major gate
- [x] [P7-039] public license — 전체 source·harness `Apache-2.0`, 전문, package metadata와 render fixture
- [ ] [P7-040] current candidate clean clone에서 전체 CI PASS
- [ ] [P7-041] `WSI-*` 통과 Windows 실제 기기 install·setup·auto onboarding·shared index·update
- [ ] [P7-042] Hive Skill implicit 중복 0건과 metadata budget·fresh-session qualification

---
