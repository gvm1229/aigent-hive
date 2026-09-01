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
9. host가 지원하는 exact integrity event가 있을 때만 optional hooks
10. 최종 write preview

Orchestration owner: 검증된 host-native capability가 기본. OMX·OMC는 사용자가 명시 선택했거나 기존 `0.8.x` run owner가 고정된 경우에만 side-effect-free evidence로 호환성을 검증. Hive는 `.omx/`, `.omc/`, host-global config, session state와 plugin cache를 읽어 추론 금지.

Detection 결과:

| 결과 | 의미 | setup 동작 |
| --- | --- | --- |
| `available` | 명시 선택·고정된 owner의 compatible OMX/OMC 확인 | 해당 compatibility owner 유지 |
| `absent` | 선택한 external runtime의 명확한 부재 | 진단 후 중지, 조용한 owner 전환 금지 |
| `incompatible` | 설치 evidence와 지원 version/capability 불일치 | 진단 후 해당 고급 기능 `unsupported` |
| `unknown` | 선택한 external runtime 검증 불가 | 진단 후 중지, host-native 신규 run은 별도 시작 |

한 run이 시작되면 resolved owner와 evidence digest를 `STATUS.md`에 고정. Run 도중 environment가 바뀌어도 조용히 owner를 교체 금지. 다음 새 run 또는 명시 reconfigure에서 다시 resolve.

Optional Skill은 자동 추천 가능. 각 항목은 개별 승인 대상. 승인 화면은 name, source, immutable revision, content digest와 `requested_capabilities`를 모두 표시. 사용자는 capability별로 승인하며 `approved_capabilities ⊆ requested_capabilities` 필수. 승인 시각과 전체 consent payload digest를 함께 기록. 승인하지 않은 Skill 또는 capability는 download, render, discovery root 배치, hook 등록과 실행을 금지.

Consent v1: `consent_version`, name, source, revision, `content_digest`, 정렬된 requested/approved capability와 UTC-seconds `approved_at`을 RFC 8785 JCS로 canonicalize한 UTF-8 bytes의 SHA-256. 정확한 계약은 `docs/architecture/skill-consent.md` 참조. Hive는 staging, projection, activation과 migration activation 전에 digest를 재계산. Field 변경 또는 digest 불일치 시 자동 재서명 금지, Skill inert 유지와 재승인 요구.

Optional integrity hook: Skill consent와 분리된 approval object. 현재 host가 exact event를 지원할 때만 다음 capability preview 표시.

> 현재 host가 아래 exact integrity event를 지원합니다. Aigent Hive는 선택적으로 project-local hooks를 설치할 수 있습니다. 이 hooks는 `.hive/` ownership 위반 사전 경고, durable run checkpoint 누락 감지, update/migration 무결성 검사와 bounded diagnostic만 수행합니다. Skill routing, prompt 재작성, subagent orchestration, 자동 memory ingest 또는 Stop continuation은 수행하지 않습니다. 설치할 event, project-local path, executable command, requested capability와 content digest는 아래 preview와 같습니다. 설치하시겠습니까?

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
