# Fallback hook consent contract

## 적용 조건

Hive fallback hook은 active host의 compatible OMX/OMC가 `absent`로 확정된 경우에만 제안·승인·projection할 수 있다.

| Detection | Hook 동작 |
| --- | --- |
| `absent` | capability별 preview와 명시적 승인을 거쳐 선택적으로 projection |
| `available` | 질문·승인·artifact 0개, 기존 Hive hook은 inert |
| `incompatible` | 질문·승인·artifact 0개, 기존 Hive hook은 inert |
| `unknown` | 질문·승인·artifact 0개, 기존 Hive hook은 inert |

모든 hook을 거절해도 setup은 성공한다. 거절 상태는 host-native capability만 사용하는 완전한 지원 상태다.

## Descriptor와 content digest

승인 화면은 exact capability, event, project-local path, executable command와 content digest를 표시한다. 설치 descriptor는 다음 field만 가진 RFC 8785 JCS object의 UTF-8 bytes 뒤에 LF 한 byte를 붙인 값이다.

```json
{
  "schema_version": 1,
  "capability": "protect-hive-owned-state",
  "event": "PreToolUse",
  "path": ".hive/hooks/protect-hive-owned-state",
  "command": "hive hook --capability protect-hive-owned-state --event PreToolUse --output json"
}
```

```text
descriptor_bytes = UTF-8(RFC 8785 JCS(descriptor)) || LF
content_digest = "sha256:" + lowercase_hex(SHA-256(descriptor_bytes))
```

`content_digest`는 승인한 preview와 실제 설치 bytes를 결합한다.

## Consent digest

Hook approval payload는 `consent_version`, `capability`, `event`, `path`, `command`, `content_digest`, UTC-seconds `approved_at`을 포함한다. `consent_digest` 자체를 제외한 전체 payload를 RFC 8785 JCS로 canonicalize한다.

```text
consent_digest = "sha256:" + lowercase_hex(
  SHA-256(UTF-8(RFC 8785 JCS(approval_without_consent_digest)))
)
```

Ledger는 `.hive/config/approved-hooks.yml`에 저장하며 `detection: absent`와 현재 capability resolution의 `resolution_evidence_digest`를 함께 고정한다. Field 하나라도 바뀌면 기존 승인은 무효다.

## Activation gate

`hive hook`은 실행 전마다 다음을 다시 검증한다.

1. installed consumer target과 ownership 경계; target-relative file은 symlink ancestor를 먼저 검사한 뒤 읽음
2. 현재 capability resolution object와 full-object evidence digest
3. ledger의 `absent` detection과 resolution evidence 결합
4. capability/event/path/command approval과 consent digest
5. 설치 descriptor exact bytes와 content digest

미승인, 변조 또는 non-absent 상태는 hook input을 읽기 전에 `decision:allow`, `active:false`로 끝난다. Detection이 더 이상 `absent`가 아니면 기존 Hive hook은 neutral/inert이며 external runtime과 경쟁하지 않는다.

## Typed non-Stop 동작

승인과 activation gate를 모두 통과한 non-Stop hook만 typed JSON input을 읽는다.

| Capability / event | 검증된 동작 |
| --- | --- |
| `protect-hive-owned-state` / `PreToolUse` | Hive protected path의 destructive operation은 `active:true` block, 그 외는 allow |
| `update-integrity-guard` / `PreToolUse` | `update|migrate`의 dry-run, backup, staged validation을 확인하고 누락 시 block |
| `derived-state-invalidation` / `PostToolUse` | canonical knowledge/team/run 변경 뒤 `.hive/index/.stale`을 멱등 생성 |
| `checkpoint-reminder` / `PreCompact` | `.hive/runs/**/STATUS.md` checkpoint 유무를 non-blocking result로 보고 |

승인된 non-Stop hook의 malformed 또는 unsafe input과 실행 오류는 diagnostic을 남기고 exit `0`, `decision:allow`, `active:false`의 neutral allow로 끝난다. 유효하게 검증된 typed input에서 보호 대상 mutation 또는 update safety gate 누락을 확인한 경우에만 `active:true`, `decision:block`을 반환한다.

`Stop`은 별도 fast path다. Approval, installed state, input, tamper 또는 detection과 관계없이 authorization이나 input을 읽지 않고 exit `0`, `decision:allow`, `active:false`를 반환한다.

## 철회와 installed validation

새 setup answer가 기존 hook 승인을 제거하면:

1. `--dry-run`은 제거할 Hive-owned ledger와 descriptor를 `changed_paths`에 표시하고 target을 바꾸지 않음
2. `--apply`는 installed setup answer, absent resolution evidence, consent digest, exact ledger bytes와 descriptor bytes가 모두 이전 approval에 결합됨을 다시 검증한 뒤 그 artifact만 제거
3. `.hive/hooks/`의 인접 user/foreign file은 byte-for-byte 보존
4. 철회된 command의 이후 호출은 input을 읽지 않는 inactive allow

`--validate`는 version parity, setup answer와 capability schema, cross-file config, Skill/hook ledger와 digest, descriptor bytes, role definition, protected canonical seed와 shared marker를 함께 확인한다. Required artifact 누락, 변조 또는 supplied capability mismatch는 target mutation 없이 exit `5`의 verification failure다.

## Capability 경계

허용 capability는 `protect-hive-owned-state`, `update-integrity-guard`, `derived-state-invalidation`, `checkpoint-reminder`뿐이다. Hook은 다음 기능을 수행하지 않는다.

- `UserPromptSubmit` classification
- prompt rewrite
- Skill activation
- orchestration 또는 subagent spawn
- automatic memory ingestion
- continuation decision

`Stop` handler는 승인, 변조, malformed input, 재실행 또는 non-absent detection과 관계없이 항상 exit `0`과 neutral allow를 반환한다. `decision:block`, `continue`, continuation prompt 또는 재귀 실행 loop를 생성하지 않는다.
