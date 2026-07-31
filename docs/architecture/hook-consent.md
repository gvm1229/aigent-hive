# Fallback hook consent contract

## 적용 조건

Hive fallback hook의 제안·승인·projection 범위: active host의 compatible OMX/OMC가
`absent`로 확정된 경우만.

| Detection | Hook 동작 |
| --- | --- |
| `absent` | capability별 preview와 명시적 승인을 거쳐 선택적으로 projection |
| `available` | 질문·승인·artifact 0개, 기존 Hive hook은 inert |
| `incompatible` | 질문·승인·artifact 0개, 기존 Hive hook은 inert |
| `unknown` | 질문·승인·artifact 0개, 기존 Hive hook은 inert |

모든 hook을 거절해도 setup은 성공. 거절 상태는 host-native capability만 사용하는 완전한 지원 상태.

## Descriptor와 content digest

승인 화면은 exact capability, event, project-local path, executable command와 content digest를 표시. 설치 descriptor는 다음 field만 가진 RFC 8785 JCS object의 UTF-8 bytes 뒤에 LF 한 byte를 붙인 값.

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

`content_digest`는 승인한 preview와 실제 설치 bytes를 결합.

모든 승인 command는 live capability evidence의 유일한 project-local 경로
`.hive/runtime/current-capability-resolution.json`을 exact argument로 포함.
Setup의 이 파일 또는 `.hive/runtime/` directory 생성 금지. Host adapter가
각 non-Stop 호출 직전에 active host catalog와 public executable probe를 다시
정규화하여 이 경로에 기록하고, 호출 뒤 제거. 이 파일은 `.hive/.gitignore`의
`/runtime/` 규칙으로 항상 제외되며 commit, release bundle, backup 또는 setup
artifact에서 제외.

## Consent digest

Hook approval payload는 `consent_version`, `capability`, `event`, `path`, `command`, `content_digest`, UTC-seconds `approved_at`을 포함. `consent_digest` 자체를 제외한 전체 payload를 RFC 8785 JCS로 canonicalize.

```text
consent_digest = "sha256:" + lowercase_hex(
  SHA-256(UTF-8(RFC 8785 JCS(approval_without_consent_digest)))
)
```

Ledger는 `.hive/config/approved-hooks.yml`에 저장하며 `detection: absent`와 현재 capability resolution의 `resolution_evidence_digest`를 함께 고정. Field 하나라도 바뀌면 기존 승인은 무효.

## Activation gate

`hive hook` 실행 직전 재검증 항목:

1. exact `.hive/runtime/current-capability-resolution.json` 경로, non-symlink regular file, 60초 이하 수정 시각
2. installed consumer target과 ownership 경계; target-relative file은 symlink ancestor 검사 뒤 read-only 조회
3. 현재 capability resolution object와 full-object evidence digest
4. ledger의 `absent` detection과 resolution evidence 결합
5. capability/event/path/command approval과 consent digest
6. 설치 descriptor exact bytes와 content digest

Runtime evidence가 없거나, 60초보다 오래되었거나, 미래 timestamp이거나,
malformed이거나, exact 경로 밖이거나, detection이 `absent`가 아니면 installed
approval 또는 hook input 조회 전에 `decision:allow`, `active:false`로 neutral 종료.
Installed `.hive/config/capability-resolution.yml`은 setup/validation 기록이며 live
activation evidence를 대체 불가. Detection이 더 이상 `absent`가 아니면
기존 Hive hook은 neutral/inert 상태로 전환하여 external runtime과의 경쟁 차단.

## Typed non-Stop 동작

승인과 activation gate를 모두 통과한 non-Stop hook만 typed JSON input 조회.

| Capability / event | 검증된 동작 |
| --- | --- |
| `protect-hive-owned-state` / `PreToolUse` | Hive protected path의 destructive operation은 `active:true` block, 그 외는 allow |
| `update-integrity-guard` / `PreToolUse` | `update|migrate`의 dry-run, backup, staged validation을 확인하고 누락 시 block |
| `derived-state-invalidation` / `PostToolUse` | canonical knowledge/team/run 변경 뒤 `.hive/index/.stale`을 멱등 생성 |
| `checkpoint-reminder` / `PreCompact` | `.hive/runs/**/STATUS.md` checkpoint 유무를 non-blocking result로 보고 |

승인된 non-Stop hook의 malformed 또는 unsafe input과 실행 오류는 diagnostic을 남기고
exit `0`, `decision:allow`, `active:false`의 neutral allow로 종료. 유효하게 검증된
typed input에서 보호 대상 mutation 또는 update safety gate 누락을 확인한 경우에만
`active:true`, `decision:block` 반환.

`Stop`은 별도 fast path. Runtime evidence file이 없더라도 approval, installed
state, input, tamper 또는 detection을 읽지 않고 exit `0`, `decision:allow`,
`active:false`를 반환.

## 철회와 installed validation

새 setup answer가 기존 hook 승인을 제거하면:

1. `--dry-run`은 제거할 Hive-owned ledger와 descriptor를 `changed_paths`에 표시하고 target 변경 없음
2. `--apply`는 installed setup answer, absent resolution evidence, consent digest, exact ledger bytes와 descriptor bytes가 모두 이전 approval에 결합됨을 다시 검증한 뒤 그 artifact만 제거
3. `.hive/hooks/`의 인접 user/foreign file은 byte-for-byte 보존
4. 철회된 command의 이후 호출은 input을 읽지 않는 inactive allow

`--validate`는 version parity, setup answer와 capability schema, cross-file config, Skill/hook ledger와 digest, descriptor bytes, role definition, protected canonical seed와 shared marker를 함께 확인. Required artifact 누락, 변조 또는 supplied capability mismatch는 target mutation 없이 exit `5`의 verification failure.

## Capability 경계

허용 capability: `protect-hive-owned-state`, `update-integrity-guard`, `derived-state-invalidation`, `checkpoint-reminder`. Hook의 금지 기능:

- `UserPromptSubmit` classification
- prompt rewrite
- Skill activation
- orchestration 또는 subagent spawn
- automatic memory ingestion
- continuation decision

`Stop` handler는 승인, 변조, malformed input, 재실행 또는 non-absent detection과 관계없이 항상 exit `0`과 neutral allow를 반환. `decision:block`, `continue`, continuation prompt 또는 재귀 실행 loop 생성 금지.

Wiki enabled 상태의 agent-reviewed task-fact autocapture는 hook capability가 아닌 final
response 전 completion gate. Current authorized task와 reviewed Git-suitable artifact의 bounded
semantic fact만 사용. Raw transcript, hook payload, tool output, hidden prompt와 runtime state의
자동 ingestion 금지 유지.
