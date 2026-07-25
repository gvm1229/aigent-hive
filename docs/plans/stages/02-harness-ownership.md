# Stage 2. Harness 생성과 ownership 적용

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

Hive는 consumer project에 다음만 생성:

- `.hive/config/`
- `.hive/knowledge/`
- `.hive/team/roles/`
- `.hive/runs/`
- `.hive/index/` runtime 위치
- root shared file의 exact Hive marker
- 사용자가 승인한 namespaced Skill/projection
- external capability absent와 별도 consent가 모두 성립할 때만 project-local Hive fallback hook projection

기존 root 문서가 있으면 전체 overwrite하지 않고 Hive marker만 merge. 손상·중첩 marker는 자동 추정하지 않고 conflict로 중지.

Setup-time role lifecycle:

1. `.hive/config/role-seeds.yml`은 승인한 초기 definition과 reconfigure preference
2. setup staging에서 각 seed를 `.hive/team/roles/<role-id>.md`로 materialize
3. materialize 후 role Markdown이 identity·assignment·handoff의 runtime 정본이며 runtime은 seed를 team member로 직접 사용 금지
4. 같은 seed 재적용은 no-op
5. 기존 role definition drift는 자동 overwrite하지 않고 conflict
6. 사용자가 reconfigure preview에서 명시 승인한 경우 definition field만 변경하고 assignment·handoff·body 보존
7. seed 제거만으로 role file을 삭제하지 않으며 명시 retire operation 필요

정확한 frontmatter/body, migration과 fixture 계약: `docs/architecture/role-lifecycle.md`.

#### 구현

- `harness/manifest.toml`을 compiled ownership manifest로 변환
- path traversal, absolute path, symlink escape 거부
- previous generated digest와 live bytes 비교
- shared marker 외 byte-preserving test
- generated file과 user-owned file 분류
- role ID 중복·path collision 거부와 role-profile schema 검증
- role file은 `canonical-data-protected`; update가 generated config처럼 overwrite/delete 금지
- host config는 foreign-owned가 기본이며 exact Hive hook entry만 `consented-shared-structure`로 merge
- JSON/TOML structured merge는 unknown key와 foreign array entry의 semantic identity·ordering·value를 보존하고 Hive-owned entry 외 삭제를 금지
- approved hook path는 project-local manifest allowlist 안에 있어야 하며 host-global path와 `.omx/.omc`는 항상 금지
- consumer `.hive/.gitignore`에는 SQLite, short-lived backup과 ephemeral
  `/runtime/` capability evidence만 제외

#### 완료 조건

- [x] 기존 user text와 external marker byte 동일
- [x] `.omx/.omc`, foreign runtime state와 host-global config read/write 0회
- [x] consent가 없는 `.codex/.claude/.agents` project hook projection read/write 0회
- [x] consented project-local hook merge 전후 foreign semantic tree와 entry digest 동일
- [x] generated path가 manifest 밖이면 setup 실패
- [x] canonical non-confidential files가 Git-visible
- [x] SQLite/WAL/SHM/journal, index stale/lock/temp, backup과 ephemeral `/runtime/`
  capability evidence만 consumer Git에서 제외
- [x] role reconfigure가 current assignment·handoff·user body를 보존
- [x] cross-major role candidate의 parse/schema 검증 실패 시 active role tree bytes 불변
