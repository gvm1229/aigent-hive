# ADR-0006: product version lifecycle

- 상태: accepted
- 날짜: 2026-07-23

## 결정

Aigent Hive source, release bundle과 installed harness는 `X.Y.Z` product version을
공유. 현재 정식 릴리스 준비 target은 `0.9.0`이며 root `Cargo.toml`의
`workspace.package.version`이 source 정본.
Release date 정본: 같은 manifest의 `workspace.metadata.hive.release-date`.
Independent judge identity와 critical human approval은 protected external public-key
trust root와 detached Ed25519 attestation으로 검증.

Plan revision은 product version과 독립. Shipped behavior 변화가 없는 plan-only 또는 current-state documentation change는 product version 증가 대상에서 제외.

## 증가 규칙

- `Y`: backward-compatible user-facing feature, Skill, schema capability 또는 host projection
- `Z`: 같은 feature contract 안의 compatible quick bugfix, security fix, packaging·documentation correction
- `X`: breaking contract 또는 compatibility baseline 변경

Automation의 `X` 추론·자동 증가 금지. Exact next-major target 명시와 별도 human confirmation이 있을 때만 release tooling의 major prepare 허용.

## 호환성

Hive는 pre-1.0에도 같은-major non-breaking 정책을 적용. 따라서 `0.1.0 → 0.n.z` upgrade의 기존 supported contract 파괴 금지. Breaking change의 `0.y.0` minor 은폐 금지.

Cross-major migration의 project source, docs, canonical knowledge, role/run state, 가능한 harness preference 보존 필수. SQLite와 backup은 compatibility 정본에서 제외.

## Version parity

Release gate는 다음 version이 모두 같지 않으면 실패.

- root Cargo workspace package
- Cargo lock의 Hive workspace package
- compiled `hive --version|--version aliases`의 product version·release date
- release bundle manifest와 provenance
- migration table target
- generated consumer `.hive/config/harness.toml`
- README와 `docs/state/CURRENT.md`

## Project upgrade baseline coverage

Project migration table의 source range는 release bundle에 exact full historical project base가
포함되고 binary가 mutation 전 이를 인증할 수 있는 release만 선언 가능. Release gate는 declared
source release마다 packaged binary의 scan·dry-run·apply·validate matrix evidence를 요구.
기준본 부재·digest 불일치: apply 전 no-mutation failure. 이 coverage defect의 correction은
compatible quick bugfix이므로 `Z` patch 범위.

## 결과

- 현재 source version: `0.9.0`
- `0.9.0` release date 정본: `2026-08-01`
- `0.1.0 → 0.2.0`은 backward-compatible Phase 1 feature milestone에 따른 minor 증가
- `0.2.0 → 0.3.0`은 backward-compatible Phase 2 knowledge/index feature milestone에 따른 minor 증가
- `0.3.0 → 0.4.0`은 backward-compatible Phase 3 Skill/projection feature milestone에 따른 minor 증가
- `0.4.0 → 0.5.0`은 backward-compatible Phase 4 role/run interoperability feature milestone에 따른 minor 증가
- `0.5.0 → 0.6.0`은 Phase 5 usage guard와 authenticated judge quorum completion gate를 충족한 backward-compatible minor 증가
- `0.6.0 → 0.7.0`은 Phase 6 verifier-only signed release, update/migration과 crash-safe recovery completion gate를 충족한 backward-compatible minor 증가
- `0.7.0 → 0.8.0`은 전역 초기 설정·공유 색인, Linux와 npm·직접 설치 시험
  배포 기능을 검증하기 위한 backward-compatible minor 증가
- `0.8.0 → 0.9.0`은 host-native loop·Skill suite·전역 RAG·knowledge 이식·scan의
  backward-compatible minor 증가
- 실제 compatible feature delivery마다 minor를, 빠른 compatible fix마다 patch를 증가
- explicit user instruction 없는 major bump 0회
- same-major breaking fixture는 release/update 거부
