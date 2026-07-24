# ADR-0006: product version lifecycle

- 상태: accepted
- 날짜: 2026-07-23

## 결정

Aigent Hive source, release bundle과 installed harness는 `X.Y.Z` product version을
공유한다. 마지막 완료 milestone은 Phase 5 usage guard와 authenticated judge quorum
`0.6.0`이며 root `Cargo.toml`의 `workspace.package.version`이 source 정본이다.
Independent judge identity와 critical human approval은 protected external public-key
trust root와 detached Ed25519 attestation으로 검증한다.

Plan revision은 product version과 독립이다. Plan-only 또는 current-state documentation change는 shipped behavior가 바뀌지 않으면 product version을 증가시키지 않는다.

## 증가 규칙

- `Y`: backward-compatible user-facing feature, Skill, schema capability 또는 host projection
- `Z`: 같은 feature contract 안의 compatible quick bugfix, security fix, packaging·documentation correction
- `X`: breaking contract 또는 compatibility baseline 변경

Automation은 `X`를 추론하거나 자동 증가할 수 없다. 사용자가 exact next-major target을 명시하고 별도 human confirmation을 제공한 경우에만 release tooling이 major prepare를 허용한다.

## 호환성

Hive는 pre-1.0에도 같은-major non-breaking 정책을 적용한다. 따라서 `0.1.0 → 0.n.z` upgrade도 기존 supported contract를 깨뜨릴 수 없다. Breaking change를 `0.y.0` minor로 숨기지 않는다.

Cross-major migration은 project source, docs, canonical knowledge, role/run state와 가능한 harness preference를 보존해야 한다. SQLite와 backup은 compatibility 정본이 아니다.

## Version parity

Release gate는 다음 version이 모두 같지 않으면 실패한다.

- root Cargo workspace package
- Cargo lock의 Hive workspace package
- compiled `hive --version`
- release bundle manifest와 provenance
- migration table target
- generated consumer `.hive/config/harness.toml`
- README와 `docs/state/CURRENT.md`

## 결과

- 현재 project version은 `0.6.0`
- `0.1.0 → 0.2.0`은 backward-compatible Phase 1 feature milestone에 따른 minor 증가
- `0.2.0 → 0.3.0`은 backward-compatible Phase 2 knowledge/index feature milestone에 따른 minor 증가
- `0.3.0 → 0.4.0`은 backward-compatible Phase 3 Skill/projection feature milestone에 따른 minor 증가
- `0.4.0 → 0.5.0`은 backward-compatible Phase 4 role/run interoperability feature milestone에 따른 minor 증가
- `0.5.0 → 0.6.0`은 Phase 5 usage guard와 authenticated judge quorum completion gate를 충족한 backward-compatible minor 증가
- 실제 compatible feature delivery마다 minor를, 빠른 compatible fix마다 patch를 증가
- explicit user instruction 없는 major bump 0회
- same-major breaking fixture는 release/update 거부
