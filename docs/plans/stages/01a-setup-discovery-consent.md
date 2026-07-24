# Stage 1. Read-only 조사와 setup 질문

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

`setup-harness`는 먼저 repository를 조사:

- project root와 Git 상태
- 기존 `AGENTS.md`, `CLAUDE.md`, `GEMINI.md`
- project manifest와 확인 가능한 domain
- 기존 Hive marker
- active host가 노출한 Skill/plugin capability metadata
- side-effect-free public `omx --version` 또는 `omc --version` 결과

그 뒤 preference만 한 번에 하나씩 질문:

1. project identity 확인
2. domain profile
3. primary host
4. persistent roles
5. knowledge ingest 범위
6. usage stop threshold
7. judge policy
8. optional Skills
9. OMX/OMC가 conclusively absent일 때만 optional fallback hooks
10. 최종 write preview

Orchestration owner: preference가 아닌 capability resolution 결과. Codex는 compatible OMX capability, Claude Code는 compatible OMC capability를 먼저 사용하고, 둘이 active host에서 확인되지 않을 때만 host-native로 resolve. Positive evidence는 host가 현재 session에 노출한 Skill/plugin metadata 또는 public executable의 side-effect-free `--version` 중 하나면 충분. Hive는 `.omx/`, `.omc/`, host-global config, session state와 plugin cache를 읽어 추론 금지.

Detection 결과:

| 결과 | 의미 | setup 동작 |
| --- | --- | --- |
| `available` | compatible OMX/OMC capability 확인 | external owner 우선, Hive hook 질문·artifact 0개 |
| `absent` | host catalog와 public probe 모두 명확히 없음 | host-native, optional fallback hook consent 질문 가능 |
| `incompatible` | 설치 evidence와 지원 version/capability 불일치 | 진단 후 해당 고급 기능 `unsupported`, hook fallback 금지 |
| `unknown` | probe surface 부재로 외부 runtime 부재 증명 불가 | host-native best-effort, hook fallback 금지 |

한 run이 시작되면 resolved owner와 evidence digest를 `STATUS.md`에 고정. Run 도중 environment가 바뀌어도 조용히 owner를 교체 금지. 다음 새 run 또는 명시 reconfigure에서 다시 resolve.

Optional Skill은 자동 추천 가능. 각 항목은 개별 승인 대상. 승인 화면은 name, source, immutable revision, content digest와 `requested_capabilities`를 모두 표시. 사용자는 capability별로 승인하며 `approved_capabilities ⊆ requested_capabilities` 필수. 승인 시각과 전체 consent payload digest를 함께 기록. 승인하지 않은 Skill 또는 capability는 download, render, discovery root 배치, hook 등록과 실행을 금지.

Consent v1: `consent_version`, name, source, revision, `content_digest`, 정렬된 requested/approved capability와 UTC-seconds `approved_at`을 RFC 8785 JCS로 canonicalize한 UTF-8 bytes의 SHA-256. 정확한 계약은 `docs/architecture/skill-consent.md` 참조. Hive는 staging, projection, activation과 migration activation 전에 digest를 재계산. Field 변경 또는 digest 불일치 시 자동 재서명 금지, Skill inert 유지와 재승인 요구.

Fallback hook: Skill consent와 분리된 approval object. `absent`일 때만 다음 capability preview 표시.

> 이 프로젝트에서 compatible OMX/OMC installation을 감지하지 못했습니다. Aigent Hive는 선택적으로 project-local fallback hooks를 설치할 수 있습니다. 이 hooks는 `.hive/` ownership 위반 사전 경고, durable run checkpoint 누락 감지, update/migration 무결성 검사와 bounded diagnostic만 수행합니다. Skill routing, prompt 재작성, subagent orchestration, 자동 memory ingest 또는 Stop continuation은 수행하지 않습니다. 설치할 event, project-local path, executable command, requested capability와 content digest는 아래 preview와 같습니다. 설치하시겠습니까?

사용자 선택: 표시된 hook capability별 승인 또는 전체 거절. 거절해도 setup은 성공하며 host-native 기능만 사용. 승인 record는 `.hive/config/approved-hooks.yml`에 tracked canonical data로 저장. Approval이 없거나 digest/path/event가 변하면 hook을 render·등록·실행 금지.

허용하는 fallback hook capability:

| Capability | 가능한 host event | 동작 |
| --- | --- | --- |
| `protect-hive-owned-state` | `PreToolUse` | manifest 밖에서 `.hive` protected state를 파괴·우회하는 명백한 mutation을 경고 또는 차단 |
| `update-integrity-guard` | `PreToolUse` | update/migration command가 dry-run·backup·staging gate를 우회하는지 확인 |
| `derived-state-invalidation` | `PostToolUse` | canonical file 변경 뒤 SQLite/evidence projection을 stale로 표시; model-visible 장문 출력 없음 |
| `checkpoint-reminder` | `PreCompact`가 있는 host, 그 외 non-blocking `Stop` | active durable run의 `STATUS.md` checkpoint 누락을 bounded diagnostic으로 알림 |

`UserPromptSubmit`, prompt rewrite, automatic Skill activation, automatic Wiki ingest, subagent spawn와 continuation decision은 fallback hook capability에 포함 금지. `Stop` handler는 항상 neutral/empty allow 결과로 끝나며 `decision:block`, `continue`, 재호출 prompt를 반환 금지. 동일 input digest의 재실행은 idempotent하고 timeout/error는 orchestration을 반복시키지 않은 채 진단만 남김.

OMX/OMC가 이후 감지되면 기존 Hive fallback hook은 즉시 neutral/inert 동작만 하고, 다음 `hive setup --reconfigure` 또는 `hive update` preview가 Hive-owned entry 제거를 제안. 제거 전후 foreign hook entry와 user byte는 보존.

Descriptor bytes, content/consent digest와 activation-time 재검증의 normative contract: [`../../architecture/hook-consent.md`](../../architecture/hook-consent.md).
