# Phase 1 conformance fixtures

이 디렉터리는 실제 host나 사용자 저장소를 읽지 않는 synthetic setup 입력만 포함한다.

## CLI contract

검증기는 다음 표면을 사용한다.

```text
hive setup \
  --target <consumer-project> \
  --answers <setup-answers.yml> \
  --capabilities <capability-evidence.json> \
  --dry-run|--apply|--validate \
  [--reconfigure-role <role-id>]... \
  --output json
```

JSON mode의 stdout은 `schemas/action-result.schema.json`을 만족하는 JSON object 하나다.
진단은 stderr로 분리하며 process exit와 `exit_code`가 같아야 한다.

`--capabilities` 입력은 다음 normalized object다.

```json
{
  "schema_version": 1,
  "host": "codex",
  "host_version": "fixture",
  "surface": "cli",
  "detection": "available",
  "external_runtime": "omx",
  "resolved_owner": "omx",
  "capabilities": {},
  "evidence_digest": "sha256:<64 lowercase hex>",
  "evidence": [
    {
      "source": "host-catalog",
      "locator": "fixture:codex-catalog",
      "outcome": "compatible",
      "digest": "sha256:<64 lowercase hex>"
    }
  ]
}
```

`available`은 compatible external owner를 사용한다. `absent`는 host catalog와 public
executable 양쪽의 명시적 absent evidence가 필요하다. `incompatible`과 `unknown`은
host-native best effort를 사용하지만 fallback hook을 설치하지 않는다.

성공한 `ActionResult.evidence`에는 선택 결과를 나타내는 `report` evidence locator
`orchestration-owner:<omx|omc|host-native>`가 포함된다. 이 evidence는 setup preference로
저장하지 않고 현재 resolution의 관찰 결과만 증명한다.

Fallback hook의 exact command는 다음 runtime 표면을 사용한다.

```text
hive hook --capability <capability> --event <event> --input <json> --output json
```

`Stop` event는 항상 exit `0`과 neutral allow 결과를 반환하며 block, continue,
continuation prompt를 반환하지 않는다.

Approved non-Stop hook invocation reports `active: true`. Missing, malformed,
tampered, or non-`absent` approval state reports `active: false` and performs no
mutation. Hook authorization is resolved from the installed consumer cwd,
`.hive/config/approved-hooks.yml`, current capability resolution, and exact
projected descriptor bytes.

Stage 0 semantic Skill routing is deferred to Phase 3. Phase 1 tests only require
unknown CLI/setup actions to fail before project mutation and verify the stable
`ActionResult` exit mapping; they do not claim semantic Skill routing support.
