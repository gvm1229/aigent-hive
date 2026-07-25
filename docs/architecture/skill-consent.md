# Optional Skill consent contract

## Consent payload v1

각 optional Skill approval은 다음 field만 포함한 payload에 결합.

```json
{
  "consent_version": 1,
  "name": "skill-name",
  "source": "immutable source locator",
  "revision": "immutable revision",
  "content_digest": "sha256:<lowercase-hex>",
  "requested_capabilities": [],
  "approved_capabilities": [],
  "approved_at": "YYYY-MM-DDTHH:MM:SSZ"
}
```

- capability array는 중복 없이 lexicographic sort
- `approved_capabilities`는 `requested_capabilities`의 부분집합
- timestamp는 UTC seconds precision
- string을 임의로 trim, case-fold 또는 URL-normalize하지 않고 승인 화면에 표시한 exact value를 사용

## Digest

`consent_digest`는 payload 자체에서 제외.

```text
consent_digest = "sha256:" + lowercase_hex(
  SHA-256(RFC 8785 JCS(payload)의 UTF-8 bytes)
)
```

제품 구현은 검토된 RFC 8785 library를 사용. 자체 canonicalizer의 제품 코드 구현 금지. CI의 known-answer fixture는 v1 payload와 expected digest를 고정.

## 검증 시점

Hive는 다음 시점마다 payload에서 digest를 재계산.

1. setup/reconfigure staging 전
2. Skill을 discovery root에 projection하기 전
3. Skill을 활성화하거나 실행하기 전
4. update/migration 결과를 activation하기 전

Mismatch, malformed timestamp, unsorted capability 또는 capability subset 위반은 승인 무효. Hive는 digest를 자동 수정하지 않고 Skill을 inert 상태로 유지. Name, source, revision, content digest, requested/approved capability 또는 approval timestamp가 하나라도 바뀌면 새 사용자 승인이 필요.

Host가 capability 제한을 기술적으로 enforce하지 못하면 requested capability 전부가 승인된 Skill만 활성화.
