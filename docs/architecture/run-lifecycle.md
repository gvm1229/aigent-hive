# Durable run lifecycle

## 소유 경계

Hive run은 `.hive/runs/<run-id>/`의 tracked Markdown와 evidence file로 복구 가능한
상태를 제공한다. Hive는 model call, subagent process, scheduler, retry 또는 지속
loop를 실행하지 않는다. 실제 dispatch와 continuation은 run을 시작할 때 자동
resolve한 host-native, OMX 또는 OMC owner가 소유한다.

새 run은 fresh capability resolution에서 owner를 정한다.

| Detection | 새 run owner | Fallback hook |
| --- | --- | --- |
| compatible `available` on Codex | OMX | 금지 |
| compatible `available` on Claude | OMC | 금지 |
| `absent` | host-native, 보고된 support 수준만 사용 | exact preview와 별도 승인 후에만 가능 |
| `incompatible` | host-native, 부족한 기능은 `unsupported` | 금지 |
| `unknown` | host-native best-effort 또는 `unverified` | 금지 |

사용자에게 orchestration owner를 묻지 않는다. Antigravity는 host-native다.

## Canonical artifact

```text
.hive/runs/<run-id>/
├── PLAN.md
├── STATUS.md
├── HANDOFF.md
└── evidence/
```

- `PLAN.md`의 checkbox criterion ID가 required acceptance의 정본이다. Checkpoint
  request가 별도 required criterion set을 주입할 수 없다.
- `STATUS.md`는 `schemas/run-status.schema.json`을 따르는 canonical JSON-compatible
  frontmatter와 exact Markdown body다.
- `HANDOFF.md`는 여러 active role이 공유하는 역할별 handoff envelope다. 자세한
  형식은 [`role-lifecycle.md`](role-lifecycle.md)에 둔다. Envelope와 모든 handoff
  entry의 `updated_at`은 Draft 2020-12 `date-time` format validation을 통과해야 한다.
- `evidence/`는 checkpoint가 참조하는 bounded local file다. SQLite, transcript와
  host runtime state는 resume 정본이 아니다.

Evidence locator는 다음 exact 형식만 허용한다.

```text
.hive/runs/<run-id>/evidence/<safe-relative-file>#sha256:<64-lowercase-hex>
```

Passed criterion마다 `criterion_evidence`가 있어야 한다. Checkpoint와 resume는
referenced file을 no-follow로 읽고 exact digest를 다시 계산한다. 하나라도 missing,
unsafe 또는 tampered면 status를 쓰거나 dispatch brief를 만들지 않는다.

## Fresh capability input

Checkpoint와 resume는 모두 명시적인 fresh capability JSON을 요구한다. Hive는 먼저
capability path를 no-follow regular-file preflight하고 nonblocking·no-follow로 연다.
Read 전에 opened handle이 regular file이고 preflight의 device/inode와 일치해야 한다.
같은 handle에서 bytes를 읽고 다시 stat해 type, device/inode, size, `mtime`과 읽은
byte length가 모두 안정적인지 확인한다. `mtime`이 미래이거나 현재보다 60초를 초과해
오래됐으면 capability JSON parse, owner resolution·continuity, `STATUS.md` write와
dispatch brief 생성 전에 거부한다.

## Checkpoint

`hive run checkpoint`는 request, fresh capability resolution, `PLAN.md`, active role과
shared handoff를 검증한 뒤 `STATUS.md` 하나만 optimistic transaction으로 기록한다.

- 새 status는 `expected_revision: 0`, 이후 update는 현재 revision을 exact하게
  요구한다.
- 동일 request의 retry는 byte-identical no-op다.
- 같은 revision에 다른 bytes가 있거나 revision을 잃으면 conflict와 write 0건이다.
- `succeeded`는 모든 required criterion이 passed이고 failed criterion이 없으며 각
  pass에 verified evidence가 있을 때만 가능하다.
- Owner field는 request 입력이 아니다. 첫 checkpoint가 fresh capability evidence에서
  자동 파생해 pin한다.

Pin은 `host`, `host_version`, `surface`, `external_runtime`, `resolved_owner`,
`resolution_evidence_digest`, `subagent_support` 전체다. 이후 checkpoint와 resume는
pin을 바꾸지 않는다.

신규 `STATUS.md`와 shared `HANDOFF.md`의 첫 publish는 같은 exclusive hard-link
primitive를 사용한다. Temp handle metadata에서 exact file identity를 잡고, link 뒤
cleanup/fault가 나면 canonical destination을 exclusive quarantine으로 claim해
device/inode를 비교한다. Published temp와 같은 inode이면 canonical path에서
rollback하고 temp recovery link를 보존한다. 다른 inode이면 racing destination을
덮어쓰거나 삭제하지 않고 원위치 또는 recovery에 보존한 채 operation을 실패시킨다.

## Owner continuity

Run 도중 fresh evidence가 pin과 다르면 owner를 교체하지 않는다.

- external runtime missing 또는 `unknown`: blocked, exit `3`
- pinned external runtime incompatible: unsupported, exit `4`
- host, version, surface 또는 full resolution digest drift: blocked, exit `3`
- malformed capability나 digest mismatch: verification failure, exit `5`

이 실패는 `changed_paths: []`이며 `STATUS.md`, role, handoff, evidence와
`.omx/.omc` namespace를 바꾸지 않는다. 새 owner resolution은 새 run에서만 한다.

## Resume와 prepare-only dispatch

`hive run resume`는 read-only다. `PLAN.md`, `STATUS.md`, active role body, shared
handoff entry와 evidence를 검증하고 provider-neutral recovery data를 반환한다.

| Durable state 또는 support | 결과 |
| --- | --- |
| `executing`, `verifying` + `supported|best-effort` | role별 `DispatchBrief`, `prepared_only: true`, `spawned: false` |
| `executing`, `verifying` + `unsupported|unverified` | exit `4`, brief와 spawn data 없음 |
| `blocked`, `usage-limited` | exit `3`, recovery data만 반환 |
| `resume-ready` | recovery data만 반환; hidden transition 없음 |
| `succeeded`, `cancelled` | terminal recovery data만 반환; continuation 없음 |

Dispatch brief는 role 책임·비책임·verification duty, exact context/write scope,
acceptance criterion, evidence, handoff, next action과 immutable owner pin을 포함한다.
Hive는 brief를 준비할 뿐 owner를 실행하거나 subagent를 spawn하지 않는다.

## Compatible external orchestration과 공존

Compatible OMX/OMC owner가 있으면 Hive는 plan, Ralph, team, retry·persistent loop
Skill 또는 lifecycle hook을 projection하지 않는다. `hive-role-handoff`,
`hive-run-checkpoint`, `hive-run-resume`는 canonical Hive data를 검증·기록·복구하는
data Skills이므로 외부 orchestration과 함께 projection할 수 있다. 이 Skills는
OMX/OMC command나 foreign state를 호출하지 않는다.

## 검증 계약

- 모든 action과 실패는 `schemas/action-result.schema.json`을 따른다.
- Role, handoff request, checkpoint request, status, capability와 dispatch brief는
  Draft 2020-12 schema와 format validation을 통과한다.
- Shared handoff envelope와 모든 entry의 `updated_at`은 strict RFC 3339
  `date-time`이며 불가능한 월·일·시각·offset은 handoff/checkpoint write 전에
  거부한다.
- Source root, traversal, symlink, directory, FIFO, unreadable 또는 oversized input은
  bounded하게 거부한다.
- Project와 fake HOME의 `.omx/.omc` foreign sentinel은 성공·실패 모두 불변이다.
- Fresh-session resume는 transcript나 SQLite 없이 canonical tracked artifact만으로
  next action과 role context를 복구한다.
