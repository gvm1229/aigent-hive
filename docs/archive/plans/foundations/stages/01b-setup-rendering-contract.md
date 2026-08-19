# Stage 1B. Setup rendering contract

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### Copier 경계

Copier 9.17.0은 template authoring, 질문 UX 검토와 CI parity test에 사용.

- source root의 `copier.yml`이 question schema 정본
- `schemas/setup-answers.schema.json`이 answer의 machine contract
- `harness/template/`가 단일 template source
- CI가 Copier static render 결과와 Rust static renderer 결과 비교
- dynamic role materialization은 versioned role contract known-answer fixture와 Rust output 비교
- release에는 compiled template pack 포함
- consumer는 Python이나 Copier 불필요
- Copier에는 live project update authority 없음

#### 구현

- `schemas/setup-answers.schema.json`으로 answer 검증
- setup answer에서 `orchestration_layer` preference 제거
- host capability probe를 `available|absent|incompatible|unknown`으로 정규화
- source-root guard를 모든 write보다 먼저 실행
- staging directory에 render
- manifest 기반 path·marker ownership 검증
- dry-run diff와 conflict 출력
- setup answers를 `.hive/setup-answers.yml`에 저장
- `persistent_roles`를 `.hive/config/role-seeds.yml`에, ingest include/exclude 범위를 `.hive/config/knowledge-scope.yml`에 projection
- 승인한 Skill만 `.hive/config/approved-skills.yml`에 immutable provenance, capability grant와 consent digest로 저장
- 승인한 fallback hook만 `.hive/config/approved-hooks.yml`과 project-local namespaced projection으로 저장
- role seed를 staging에서 `.hive/team/roles/<role-id>.md` canonical role로 materialize

#### 완료 조건

- [x] 같은 answer로 두 번 render한 normalized digest 동일
- [x] Codex+compatible OMX는 자동 OMX, Claude+compatible OMC는 자동 OMC로 resolve
- [x] Antigravity 또는 `absent|incompatible|unknown` detection은 truthful host-native로 resolve
- [x] `incompatible|unknown`에서 Hive fallback hook install 0개
- [x] external capability detected 상태에서 hook 질문·artifact·command 0개
- [x] external capability absent + hook 거절 setup 성공, hook artifact 0개
- [x] external capability absent + 일부 hook 승인 시 승인 event/capability만 projection
- [x] hook event/path/capability/digest tamper 시 activation 0회와 재승인 요구
- [x] fallback `Stop` fixture가 모든 상태에서 block/continue prompt 0개와 neutral output
- [x] 동일 hook input replay에서 canonical file 중복 mutation 0개
- [x] optional Skill 0개 승인 setup 성공
- [x] 승인하지 않은 Skill output 0개
- [x] `approved_capabilities`가 `requested_capabilities`를 벗어나면 staging 전 거부
- [x] Skill provenance/capability/timestamp 어느 한 field tamper도 기존 consent digest로 activation 불가
- [x] role seed와 knowledge scope가 setup answer에서 손실 없이 render
- [x] 모든 role seed가 schema-valid role file 하나로 materialize되고 두 번째 setup은 byte-identical
- [x] `hive-source.json` target write 0개
- [x] Copier/Rust static tree parity와 role materialization known-answer parity
