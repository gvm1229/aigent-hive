# Durable run lifecycle

## 소유 경계

Hive run은 `.hive/runs/<run-id>/`의 tracked Markdown과 evidence file로 복구 가능한
상태를 제공. Hive는 model call, subagent process, scheduler, retry 또는 지속
loop 실행 금지. 실제 dispatch와 continuation은 run을 시작할 때 자동
resolve한 host-native, OMX 또는 OMC owner가 소유.

새 run의 owner는 fresh capability resolution에서 결정.

| Detection | 새 run owner | Fallback hook |
| --- | --- | --- |
| compatible `available` on Codex | OMX | 금지 |
| compatible `available` on Claude | OMC | 금지 |
| `absent` | host-native, 보고된 support 수준만 사용 | exact preview와 별도 승인 후에만 가능 |
| `incompatible` | host-native, 부족한 기능은 `unsupported` | 금지 |
| `unknown` | host-native best-effort 또는 `unverified` | 금지 |

사용자 대상 orchestration owner 질문 없음. Antigravity는 host-native.

## Canonical artifact

```text
.hive/runs/<run-id>/
├── PLAN.md
├── STATUS.md
├── HANDOFF.md
└── evidence/
```

- `PLAN.md`의 checkbox criterion ID가 required acceptance의 정본. Checkpoint
  request의 별도 required criterion set 주입 금지.
- `STATUS.md`는 `schemas/run-status.schema.json`을 따르는 canonical JSON-compatible
  frontmatter와 exact Markdown body.
- `HANDOFF.md`는 여러 active role이 공유하는 역할별 handoff envelope. 자세한
  형식: [`role-lifecycle.md`](role-lifecycle.md). Envelope와 모든 handoff
  entry의 `updated_at`은 Draft 2020-12 `date-time` format validation 통과 필수.
- `evidence/`는 checkpoint가 참조하는 bounded local file. SQLite, transcript와
  host runtime state는 resume 정본에서 제외.

Evidence locator는 다음 exact 형식만 허용.

```text
.hive/runs/<run-id>/evidence/<safe-relative-file>#sha256:<64-lowercase-hex>
```

각 passed criterion의 `criterion_evidence` 필수. Checkpoint와 resume는
referenced file을 no-follow로 읽고 exact digest를 다시 계산. 하나라도 missing,
unsafe 또는 tampered 상태에서는 status 쓰기와 dispatch brief 생성 차단.

## Fresh capability input

Checkpoint와 resume는 모두 명시적인 fresh capability JSON을 요구. Hive는 먼저
capability path를 no-follow regular-file preflight하고 nonblocking·no-follow로 열기.
Read 전 opened handle의 regular file 여부와 preflight device/inode 일치 확인 필수.
같은 handle에서 bytes를 읽고 다시 stat해 type, device/inode, size, `mtime`과 읽은
byte length 안정성 확인. `mtime`이 미래이거나 현재보다 60초 이상 오래되면 capability
JSON parse, owner resolution·continuity, `STATUS.md` write와 dispatch brief 생성 전에
거부.

## Checkpoint

`hive run checkpoint`는 request, fresh capability resolution, `PLAN.md`, active role과
shared handoff를 검증한 뒤 `STATUS.md` 하나만 optimistic transaction으로 기록.

- 새 status는 `expected_revision: 0`, 이후 update는 현재 revision을 exact하게
  요구.
- 동일 request의 retry는 byte-identical no-op.
- 같은 revision의 다른 bytes 또는 revision 손실 시 conflict와 write 0건.
- `succeeded`는 모든 required criterion이 passed이고 failed criterion이 없으며 각
  pass에 verified evidence가 있을 때만 가능.
- Owner field는 request 입력 대상에서 제외. 첫 checkpoint가 fresh capability
  evidence에서 자동 파생해 pin.

Pin은 `host`, `host_version`, `surface`, `external_runtime`, `resolved_owner`,
`resolution_evidence_digest`, `subagent_support` 전체. 이후 checkpoint와 resume의
pin 불변.

신규 `STATUS.md`와 shared `HANDOFF.md`의 첫 publish는 같은 exclusive hard-link
primitive를 사용. Temp handle metadata에서 exact file identity를 잡고, link 뒤
cleanup/fault가 나면 canonical destination을 exclusive quarantine으로 claim해
device/inode를 비교. Published temp와 같은 inode이면 canonical path에서
rollback하고 temp recovery link를 보존. 다른 inode이면 racing destination을
덮어쓰기와 삭제 없이 원위치 또는 recovery에 보존한 채 operation 실패 처리.

## Owner continuity

Run 도중 fresh evidence와 pin 불일치 시 owner 교체 금지.

- external runtime missing 또는 `unknown`: blocked, exit `3`
- pinned external runtime incompatible: unsupported, exit `4`
- host, version, surface 또는 full resolution digest drift: blocked, exit `3`
- malformed capability나 digest mismatch: verification failure, exit `5`

이 실패는 `changed_paths: []`이며 `STATUS.md`, role, handoff, evidence와
`.omx/.omc` namespace 변경 없음. 새 owner resolution은 새 run에서만 수행.

## Resume와 usage-guarded dispatch preparation

`hive run resume`는 canonical run artifact에 대해 read-only. `PLAN.md`,
`STATUS.md`, active role body, shared handoff entry와 evidence를 검증하고
provider-neutral recovery data를 반환. Automatic intent에만 아래 두 Git-ignored
Hive runtime record 갱신 허용.

| Durable state 또는 support | 결과 |
| --- | --- |
| `executing`, `verifying` + `supported|best-effort` | role별 `DispatchBrief`, `prepared_only: true`, `spawned: false` |
| `executing`, `verifying` + `unsupported|unverified` | exit `4`, brief와 spawn data 없음 |
| `blocked`, `usage-limited` | exit `3`, recovery data만 반환 |
| `resume-ready` | recovery data만 반환; hidden transition 없음 |
| `succeeded`, `cancelled` | terminal recovery data만 반환; continuation 없음 |

기본값인 manual intent는 CodexBar와 usage runtime record를 읽거나 쓰지 않고 기존
prepare-only brief를 반환하며 usage enforcement 주장은 제외. Explicit
automatic intent는 SHA-256 account digest와 active role ID 하나를 요구.
`.hive/config/harness.toml`의 root `usage_stop_remaining_percent`가 권위값이며
missing, malformed, duplicate, symlink config는 fail closed. Optional
`--threshold 1..99`는 설치값과 exact하게 같을 때만 허용하며 policy override 금지.
Durable run, owner continuity, selected active role과 evidence 검증 뒤
exact-qualified CodexBar snapshot을 읽고 provider-neutral permit을 평가.

Session window가 있으면 weekly 값이 low, malformed 또는 duplicate여도 session만
선택. Session이 없을 때만 단일 weekly window를 fallback으로 사용. 선택된
snapshot은 account digest별로 Git에서 제외된
`.hive/runtime/usage-history/<digest>.json`에 bounded·integrity-bound record로만
저장. 첫 sample은 prior comparison 없이 truthful하게 처리하고, 이후 measurement
또는 reset timestamp 역행과 같은 reset의 remaining 증가는 `usage_unknown`.
Malformed, tampered, symlink history도 fail closed.

Permit은 dispatch brief 준비 closure 직전에 현재 시각으로 한 번 소비. Authorized
결과는 exact run revision·selected role·brief digest로 결정되는 authorization ID와
brief 하나만 반환하고
`.hive/runtime/dispatch-authorizations/<authorization-id>.json`에 issued claim을
기록. 같은 binding은 sensor 재호출 없이 `already_issued`, exit `3`, brief 0개로
종료. Limited, unknown 또는 expired permit도 exit `3`, brief 0개와 recovery
data만 반환.

Issued claim은 Hive가 같은 command 결과를 다시 발급하지 않게 할 뿐, caller가 이미
capture한 JSON의 Hive 외부 replay를 막는 cryptographic capability는 제공 범위 밖.
실제 host/orchestration owner의 필수 동작: authorization ID를 dispatch boundary에서
한 번만 소비하고 중복 사용 거부.

Dispatch brief는 role 책임·비책임·verification duty, exact context/write scope,
acceptance criterion, evidence, handoff, next action과 immutable owner pin을 포함.
Hive 책임은 brief 준비까지이며 owner 실행과 model/subagent spawn은 범위 밖.

## Compatible external orchestration과 공존

Compatible OMX/OMC owner가 있으면 Hive는 plan, Ralph, team, retry·persistent loop
Skill 또는 lifecycle hook projection 금지. `hive-role-handoff`,
`hive-run-checkpoint`, `hive-run-resume`: canonical Hive data 검증·기록·복구용 data
Skill. 외부 orchestration과 함께 projection 허용. 두 Skill의 OMX/OMC command 호출과
foreign state 접근 금지.

## 검증 계약

- 모든 action과 실패는 `schemas/action-result.schema.json` 준수.
- Role, handoff request, checkpoint request, status, capability와 dispatch brief는
  Draft 2020-12 schema와 format validation을 통과.
- Shared handoff envelope와 모든 entry의 `updated_at`은 strict RFC 3339
  `date-time`이며 불가능한 월·일·시각·offset은 handoff/checkpoint write 전에
  거부.
- Source root, traversal, symlink, directory, FIFO, unreadable 또는 oversized input은
  bounded하게 거부.
- Project와 fake HOME의 `.omx/.omc` foreign sentinel은 성공·실패 모두 불변.
- Fresh-session resume는 transcript나 SQLite 없이 canonical tracked artifact만으로
  next action과 role context를 복구.
- Automatic resume의 유일한 project write는 Git에서 제외된 Hive-owned
  `.hive/runtime/usage-history/`와 `.hive/runtime/dispatch-authorizations/`의 bounded
  record. Manual resume의 project write는 0건.
