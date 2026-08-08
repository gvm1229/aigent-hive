# Optional host-native hook consent contract

## 적용 조건

Hive hook의 목적: supported host-native event를 통한 선택적 data-integrity guard.
Orchestration, subagent spawn, memory capture 또는 continuation 소유권 없음.

| Current owner·event evidence | Hook 동작 |
| --- | --- |
| `host-native` + exact event `supported` | capability별 preview와 명시적 승인 뒤 선택적 projection |
| `host-native` + event `best-effort|unsupported|unverified` | 질문·승인·artifact 0건; 기존 artifact inert |
| `host-native` + event claim 없음 | 질문·승인·artifact 0건; 기존 artifact inert |
| 명시적 OMX·OMC owner | 질문·승인·artifact 0건; 기존 artifact inert |
| 기존 external pinned owner | 질문·승인·artifact 0건; 기존 artifact inert |

모든 hook 거절 시에도 setup 성공. Hook 없이 verified host-native capability만 사용하는
지원 상태 유지. `approved_fallback_hooks`, `approved_fallback_hooks_file` 같은 schema
field는 0.8.x compatibility 이름; v0.9 activation authority는 exact supported
host-native hook event.

## Descriptor와 content digest

승인 화면의 exact 항목: capability, event, project-local path, executable command,
content digest. 설치 descriptor: 다음 field만 포함한 RFC 8785 JCS object의 UTF-8 bytes와
후행 LF 1 byte.

```json
{
  "schema_version": 1,
  "capability": "protect-hive-owned-state",
  "event": "PreToolUse",
  "path": ".hive/hooks/protect-hive-owned-state",
  "command": "hive hook --capability protect-hive-owned-state --event PreToolUse --capabilities .hive/runtime/current-capability-resolution.json --output json"
}
```

```text
descriptor_bytes = UTF-8(RFC 8785 JCS(descriptor)) || LF
content_digest = "sha256:" + lowercase_hex(SHA-256(descriptor_bytes))
```

`content_digest`: 승인 preview와 실제 설치 bytes의 결합.

모든 승인 command의 live capability evidence 경로:
`.hive/runtime/current-capability-resolution.json`. Setup의 이 파일 또는
`.hive/runtime/` directory 생성·추적 0건. Host adapter의 non-Stop 호출 직전 active
host catalog와 public executable probe 재정규화, 호출 뒤 제거. `.hive/.gitignore`의
`/runtime/` 규칙에 따른 commit·release bundle·backup·setup artifact 제외.

## Consent digest와 ledger

Hook approval payload: `consent_version`, `capability`, `event`, `path`, `command`,
`content_digest`, UTC-seconds `approved_at`. `consent_digest` 제외 후 RFC 8785 JCS
canonicalization.

```text
consent_digest = "sha256:" + lowercase_hex(
  SHA-256(UTF-8(RFC 8785 JCS(approval_without_consent_digest)))
)
```

Ledger 위치: `.hive/config/approved-hooks.yml`. Current supported host-native resolution의
`detection: available`과 full-object `resolution_evidence_digest` 고정. Field 하나의
변경도 기존 승인 무효.

## Activation gate

`hive hook`의 non-Stop 실행 직전 재검증:

1. exact `.hive/runtime/current-capability-resolution.json`, non-symlink regular file,
   60초 이하 수정 시각
2. fresh resolution의 installed host 일치와 full-object evidence digest
3. `resolved_owner: host-native`
4. requested event의 exact `support: supported`
5. installed consumer target과 ownership 경계, target-relative symlink ancestor 부재
6. ledger의 `detection: available`, resolution digest와 approval binding
7. capability·event·path·command·consent digest
8. 설치 descriptor exact bytes와 content digest

다음 상태는 hook event input 조회 전 `decision: allow`, `active: false`의 inert 종료:

- external owner
- exact event의 `best-effort|unsupported|unverified`
- missing event claim
- runtime capability surface의 `absent`
- 승인 없는 compatibility entrypoint

Missing·stale·future·malformed·unsafe live evidence: protected target mutation 없이 neutral
allow 또는 verification diagnostic. Installed
`.hive/config/capability-resolution.yml`: setup·validation 기록 전용; live activation
evidence 대체 불가.

## Typed non-Stop 동작

승인과 activation gate를 모두 통과한 non-Stop hook만 typed JSON input 조회.

| Capability / event | 검증된 동작 |
| --- | --- |
| `protect-hive-owned-state` / `PreToolUse` | Hive protected path의 destructive operation만 `active:true` block |
| `update-integrity-guard` / `PreToolUse` | `update|migrate`의 dry-run·backup·staged validation 누락만 block |
| `derived-state-invalidation` / `PostToolUse` | canonical knowledge·team·run 변경 뒤 derived index stale marker의 멱등 생성 |
| `checkpoint-reminder` / `PreCompact` | `.hive/runs/**/STATUS.md` checkpoint 유무의 non-blocking result |

승인된 non-Stop hook의 malformed·unsafe input 또는 실행 오류: diagnostic, exit `0`,
`decision: allow`, `active: false`. 유효한 typed input에서 protected mutation 또는 update
safety gate 누락 확인 시에만 `active: true`, `decision: block`.

## Stop fast path

`Stop` handler: runtime evidence, approval, installed state, input, tamper 또는 owner
detection 조회 0건. 항상 exit `0`, `decision: allow`, `active: false`.

금지 결과:

- `decision: block`
- `continue`
- continuation prompt
- host·model·subagent 재호출
- recursive execution loop

`checkpoint-reminder` / `Stop` descriptor의 역사적 schema compatibility와 무관한
neutral handler 동작 불변.

## 철회와 installed validation

새 setup answer의 기존 hook 승인 제거:

1. `--dry-run`: 제거 대상 Hive-owned ledger와 descriptor를 `changed_paths`에 표시,
   target 변경 0건
2. `--apply`: installed answer, supported host-native resolution evidence, consent digest,
   exact ledger·descriptor bytes의 prior approval binding 재검증 뒤 해당 artifact만 제거
3. `.hive/hooks/` 인접 user·foreign file의 byte-for-byte 보존
4. 철회 command의 이후 호출: input read 없는 inactive allow

`--validate`: version parity, setup answer·capability schema, cross-file config,
Skill/hook ledger·digest, descriptor bytes, role definition, protected canonical seed와 shared
marker의 결합 검증. Required artifact 누락·변조 또는 supplied capability mismatch:
target mutation 0건, exit `5` verification failure.

## Capability 경계

허용 capability:

- `protect-hive-owned-state`
- `update-integrity-guard`
- `derived-state-invalidation`
- `checkpoint-reminder`

Hook 금지 기능:

- `UserPromptSubmit` classification
- prompt rewrite
- Skill activation
- orchestration 또는 subagent spawn
- automatic memory ingestion
- dispatch·retry·continuation decision

Wiki-enabled turn의 agent-reviewed memory gate: hook capability가 아닌 final response 전
Skill·CLI 계약. Current authorized task와 reviewed artifact의 bounded semantic claim만
대상. Raw transcript, hook payload, tool output, hidden prompt, secret와 runtime state의
automatic ingestion 0건.
