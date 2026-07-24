# Stage 11. Update와 migration

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

사용자는 CLI 또는 host의 thin action에서 `hive update` 한 번 실행:

1. current install·origin·source/release/harness version 확인
2. GitHub Release metadata와 signature/provenance 검증
3. compatibility 계산과 경고
4. protected canonical tree snapshot과 backup
5. shadow directory에 새 template/projection render
6. same-major 또는 cross-major migration
7. ownership·schema·link·query·host smoke test
8. atomic activation
9. SQLite rebuild
10. 최대 7일 후 backup 삭제

#### Same-major

`X.a.b → X.c.d`:

- canonical schema와 setup answer backward compatible
- project/user content rewrite 없음
- additive projection과 index rebuild 허용
- breaking change 발견 시 release 자체를 거부

이 규칙은 major `0`에도 동일하게 적용. `0.1.0 → 0.2.0`은 feature addition일 수 있지만 existing supported contract를 깨뜨릴 수 없음.

#### Cross-major

`X.* → Y.*`:

- breaking change 경고
- source version별 signed migration route
- canonical Markdown과 preferences snapshot
- shadow successor에서 자동 변환
- project file, docs와 user-authored body 보존
- deprecated system representation만 새 format으로 재구성
- SQLite는 migrate하지 않고 새 schema로 rebuild
- conflict는 active install을 바꾸지 않고 중지

#### Installer와 release

- GitHub Releases가 artifact 정본
- macOS: direct bootstrap + Homebrew 편의 경로
- Windows: signed PowerShell bootstrap + WinGet 편의 경로
- package manager install은 self-updater가 managed binary를 덮어쓰기 금지
- host plugin은 update action을 노출하는 thin surface이며 제품 정본에서 제외
- feature implementation은 release-facing contract가 완성되고 acceptance evidence가 생긴 commit에서 `Y`를 증가
- 같은 feature contract의 작은 compatible fix는 `Z`를 증가
- plan/docs-only change는 shipped behavior가 바뀌지 않으면 product version 유지
- release tooling은 root Cargo version, CLI `--version`, bundle manifest, migration table과 generated harness version 불일치를 거부
- next-major prepare는 user-supplied exact target과 별도 human confirmation이 모두 없으면 거부하며 자동 계산·자동 증가 금지

#### Signing

- GitHub artifact attestation과 provenance
- macOS Developer ID signing/notarization
- Windows Artifact Signing 또는 hardware-backed Authenticode key
- offline threshold root와 online release role 분리
- private key repository 저장 금지
- protected environment와 human approval gate

#### 완료 조건

- [x] tampered/expired/rollback release 거부
- [x] root Cargo, CLI, bundle와 installed harness version parity
- [x] feature fixture가 patch-only bump를 거부하고 compatible minor bump를 요구
- [x] bugfix fixture가 같은 minor의 patch bump로 upgrade
- [x] explicit exact target과 human confirmation 없는 major bump 0회
- [x] same-major 모든 supported fixture non-breaking
- [x] cross-major project/docs/preferences 무손실
- [x] migration failure 시 active generation 불변
- [x] SQLite file을 backup/migration input으로 요구 금지
- [x] user/external bytes와 namespace checksum 불변
- [x] backup 7일 초과 자동 정리
- [x] update와 knowledge deletion/GC가 같은 transaction에 없음
