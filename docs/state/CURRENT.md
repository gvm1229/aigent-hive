# 현재 상태

- 기준 branch: `main` stable `8b37323`, `develop` patch handoff `4dcc9a7`
- product version: `0.9.4`
- 다음 release target: `0.9.4` stable publication 완료; `0.10.0` 후보 작업 시작 제외
- plan revision: `219`
- 현재 milestone: `0.9.3` 정식 출시·Windows 수용 완료, `0.9.4` GitHub Release 이중 언어 설명·공개 HTML·PDF 지식 기능 안내·전체 Skill 식별자 표시·전역 projection validation·지식 기록 credential 오탐·완료한 source knowledge scan·응답 언어·프롬프트 영어 기본값·작업 폴더 수명주기
- 기능 기준: `8b37323daa33b96918933ad629d7c709c3cb6679`; release discipline base `c777da1`
- 제외: `NHA10-001–012`·`N10-002–011`의 `0.10.0-test`
- `0.9.4` source knowledge scan correction 완료: `--candidates`와 `--apply`의 store-level validation parity, claim-bound credential diagnostic, canonical provenance summary의 human review ID false-positive 제거. source build로 61개 current-truth fact·architecture claim을 source-private `aigent-hive` collection에 재적용, automatic shared promotion `0건`, generation 115와 PortareFolium explicit retrieval 확인. published `0.9.3` stable artifact 변경 없음; installed `0.9.3` binary의 correction은 `0.9.4-test` publication 뒤 검증 필요
- `0.9.4` 응답 언어 계약 완료: 원본 `.agents`와 소비자 하네스에 ASD-STE100 영어·한국어 의미 중심 문장 기준 반영. `user-setup` 4개 투영 byte 일치, 렌더된 소비자 `AGENTS.md`·31개 focused 시험·문서 말투·Markdown 링크·Source Wiki 색인 재생성·lint 검사 통과
- `0.9.4` 응답 언어 보강 완료: 치환 가능한 영어 혼용·혼합 합성어·강조용 영어 괄호·영어 어순 직역 금지와 자연스러운 한국어 대체 예시를 원본·소비자 계약에 반영. 정적 계약·네 Skill 투영 byte 일치·렌더된 소비자 `AGENTS.md` 수명주기 31개 시험 통과, 문서 말투 finding `0`
- `0.9.4` 작업 폴더 수명주기 완료: 일반 작업의 단일 기준 작업 폴더 우선, 안전한 순차 처리 불가한 병렬 독립 변경 때만 추가 작업 폴더 허용, 원격 도달성·미반영 작업 없음 확인 뒤 즉시 제거 규칙·정적 계약 반영. `develop` 원격 반영·기준 작업 폴더 전환·`develop-release-integration` 임시 작업 폴더 제거 확인
- `0.9.4` 전체 Skill 식별자 표시 완료: current 26개 Hive Skill의 catalog·front matter·Codex metadata·plugin·Copier projection 설명 첫머리를 `(skill-id)`로 통일. `hive-projection` 34개, `hive-cli user_install` 84개, Skill·setup 적합성 검사 통과. third-party·사용자 작성 Skill 변경 `0건`
- `0.9.4` 전역 projection validation 정합성 수용 완료: public `0.9.4-test.1` Windows x64의 fresh setup, npm `0.9.3 → 0.9.4-test.1` upgrade, preserving reinstall, `hive install --validate`, `hive setup --validate` 수렴. local managed edit·malformed receipt·structured config corruption fail-closed local 회귀 유지
- `0.9.4` 전역 지식 기록 credential 오탐 수용 완료: public `0.9.4-test.1` Windows x64 safe user-root record의 canonical receipt 성공. credential-shaped input은 canonical Markdown·SQLite 변경 없이 `normalized_fact` field·이유 오류 반환
- `0.9.4` 공개 HTML·PDF 지식 기능 안내 완료: `hive-core-features.ko.html`의 지식 기능 한 줄 강조 상자를 다섯 Skill의 정본 ID·사람 중심 기능명·사용 시점·범위·안전 경계·예시 비교표로 교체. 좁은 화면은 항목명과 내용을 짝지은 세로 레이아웃. print CSS는 `.section-head`에만 `break-after: avoid-page` 적용, title 단독 page 방지와 section 전체 이동 없음. HTML에서 PDF 재생성, Poppler 144 dpi 8쪽 render와 desktop·mobile 화면 확인. `HGD94-001–004` 완료
- `0.9.4` GitHub Release 이중 언어 설명 수용 완료: public `v0.9.4-test.1` description과 canonical release note text 일치. English-first 순서, scope 5건·compatibility 2건·verification 2건 fact ID parity 확인
- `0.9.4` 프롬프트 영어 기본값 수용 완료: public `0.9.4-test.1` Korean response environment의 installed directive와 `prompt-refine` 확인. default English·explicit Korean prompt contract 각각 `hive.prompt-refinement-valid`, 설명·질문 언어는 Korean directive 유지
- `0.9.4` 정식 출시 완료: `v0.9.4-test.1` 수용 source `cc50bcb` 뒤 PR #33 merge `8b37323`, stable candidate `31767805733`, publication `31768342121`. [GitHub Release `v0.9.4`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.4)와 여섯 npm package `latest=0.9.4` 확인. Windows x64 public direct installer·npm latest·released zip·integrity bundle `hive.release-verified` source `8b37323`·sequence `13`, release lane 40 pass·Windows 미지원 macOS 7건·POSIX 1건 skip
- 완료: 제품 harness·release와 분리한 source workspace 전용 `update-summary` Skill의 구독자 관점 필터. 설치 product 또는 사용자 작업 방식에 영향이 없는 개발자·기여자 전용 변경은 요약에서 제외
- 완료: 안정판 GitHub Release 뒤 Discord 배너 1건과 `update-summary` 한국어 구독자 요약 1건 전송. 배너 성공 뒤 요약 전송, 시험판 제외, webhook URL은 `release-publication` 환경 비밀 값 한정
- 실제 Discord 전달 시험·표시 수용 완료: `0.9.3` [run `31775328374`](https://github.com/gvm1229/aigent-hive/actions/runs/31775328374)·`0.9.4` [run `31775377264`](https://github.com/gvm1229/aigent-hive/actions/runs/31775377264) 각각 배너 뒤 한국어 요약 전송 성공. 유지보수자 실제 Discord 화면에서 `0.9.3` 뒤 `0.9.4`, 각 배너 아래 해당 요약 표시 확인. 안정판·npm 게시·GitHub Release 생성 `0건`
- npm publication: Trusted Publisher OIDC 유지, `release-publication` 환경의 historical `NPM_TOKEN` 삭제 완료
- 문서 gate: root·번역 README, 설치 안내, 공개 HTML, npm README, plugin metadata,
  문서 색인·명령·version 예시 전수 최신화와 공개 artifact 재검증
- `0.9.2` 안정판 완료: PR #25 전체 CI PASS·merge exact `a502867e6b20e8f22bc014af05ca678f211f40ed`.
  Candidate run `31609928346`·publication run `31611457288`, normal GitHub Release 26개 자산,
  npm `latest=0.9.2` PASS. 이 Windows의 전역 npm 설치와 공개 `install.ps1` 빈 경로 설치에서
  `AIgent Hive v0.9.2 (released 2026-08-13)` 확인. User-scope apply·validate·doctor PASS
- 공개 문서 마감: root·번역 README의 시험판 직접 언급 `0건`, 별도 검증 안내 중립 링크 각
  `1건`. npm `0.9.2` tarball README도 직접 언급 `0건`·링크 `1건`; plugin developer
  `Hojin (Tom) Jeong`·공식 512 px logo 계약 PASS
- Markdown 전용 후속 통합: Source Wiki·문체·링크·가장 가까운 packaging·directive 검사 PASS
  뒤 관련 없는 전체 플랫폼 CI 대기 생략. 완료되지 않은 CI를 통과로 보고하지 않으며 새 시험판·
  안정판 후보·tag·package publication 생성 없음
- QA contributor 등록: 안희준([No-Jyun](https://github.com/No-Jyun)), Windows x64 설치·설정 검증
- `0.9.3` 동결: QA contributor 등록 완료. 유지보수자의 후속 명시적 승인 전 구현·검증·출시 mutation `0건`
- `0.9.3` 재개: 유지보수자 명시 승인 뒤 `codex/0.9.3-native-agents`에서 `develop` 기준 통합.
  Native orchestration event·authority·CLI·feasibility baseline 복원, custom profile의 preview·exact
  digest consent·owned projection·foreign-byte 거부 구현. Ralph loop initialize·validate·recover PASS.
- `0.9.3` 범위 최적화: actual native host child lifecycle·attestation은 default-off `NHA10-001–012`의
  `0.10.0-test` 후보로 이관. `VAL93-*`가 `local-preserved`·formatter validation 계약을, `OPT93-*`가
  risk-tier CI·candidate economy를 단독 소유. release fragment는 public transition만 소유하며 선행
  evidence checklist 재열거 금지.
- `0.9.3` validation·출시 운영 최적화 완료: authenticated `local-preserved` Skill ledger는
  `project upgrade --validate`와 `setup --validate`에서 같은 valid 상태로 수용. stale·forged ledger와
  malformed role JSON은 remediation과 함께 fail-closed. marker-owned `.prettierignore`는 foreign bytes를
  보존하고 CRLF/LF formatter 변환만 정규화. CI는 Markdown-only 단일 lane·product Linux full와
  macOS/Windows smoke로 분리, 최신 branch CI만 취소, release runtime은 weekly/manual로 한정.
  renderer 61·update 54·project lifecycle 9·release contract 26 focused PASS, projection parity·문서 style PASS.

- `0.9.3` source knowledge import·access optimization 완료: source scan의 `.agents/` 등 foreign host
  namespace는 content read 없이 receipt skip. Windows source inventory 824 included·236 skipped 뒤
  reviewed claim 19개를 private source collection에 저장. PortareFolium ordinary `auto` retrieval의
  source-private hit `0건`, explicit `collection:aigent-hive` retrieval의 source collection-only 확인.
  reviewed safe-general decision 2개: scan apply 중 user interruption 없는 `user-root` shared 승격.
  source evidence 무효화 시 파생 shared claim 동반 무효화 regression PASS

- `0.9.3-test` public acceptance 완료: exact `114817677e83aae535bd1f8b47518bf9b6745432` candidate의
  five-platform build·attestation PASS, [GitHub Release `v0.9.3-test`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.3-test)
  및 npm `test=0.9.3-test` 확인. 이 Windows x64 isolated prefix에서 public `0.9.2` clean install,
  `0.9.3-test` upgrade, matching pending receipt recovery, version/date 출력과 receipt 수렴 PASS.
  `latest=0.9.2` 유지. 다음 gate: protected `main` 통합과 stable candidate

- `0.9.3` 안정판 완료: PR #30 merge `e4c5b5e`와 candidate run `31741922500`의 5 native target·
  npm umbrella·direct installer·integrity bundle PASS. publication run `31742946016` 뒤
  [GitHub Release `v0.9.3`](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.3)·npm `latest=0.9.3` 확인.
  이 Windows x64의 public direct installer receipt, global npm install, preserving user reinstall,
  `hive install --validate`, `AIgent Hive v0.9.3 (released 2026-08-14)` PASS. PortareFolium target의
  explicit `collection:aigent-hive` installed retrieval에서 source-private canonical citation 5건 반환

- `0.9.3` projection purge·directive 우선 갱신 완료: historical digest 또는 authenticated ownership으로
  증명되는 retired Hive Skill만 전역·project `.agents/skills`에서 제거. 새 Hive safety·ownership rule과
  직접 충돌하는 Hive-owned directive clause만 incoming 우선으로 갱신. 사용자·foreign·비충돌 byte 보존.
  workspace Rust 559·strict Clippy·Python projection parity와 global setup/project lifecycle 22개,
  human documentation style·Markdown link·Source Wiki lint 통과

- 공개 안내 보강 완료: 세 Hive logo PNG의 pixel-preserving 중심 정렬, `hive-core-features.ko.html`의
  전폭 feature card·use case·강조 정보 구조, 두 HTML의 centered logo 내장과 PDF 재생성. static contract·Chrome
  offline desktop/mobile·core PDF 6페이지 Poppler render PASS. `0.9.3` 출시 전 numbered public test와 remaining
  native/custom-host evidence 필요
  Codex CLI `0.147.0`·Claude Code `2.1.163` 확인. Claude fresh-session probe: CLI 미로그인으로
  model 실행 `0회`, auth 전 MRA actual lifecycle 수용 보류
- Codex custom profile 실제 수용: Windows x64 isolated project에서 Codex `0.147.0`의 fresh
  ephemeral session이 Hive의 `.codex/agents/*.toml` profile을 `--profile`로 발견하고
  Luna·Terra·Sol marker invocation을 각각 실행. `preview→apply→validate`와 profile별 실행은
  PASS. Codex JSON result는 host-signed exact model/effort receipt를 제공하지 않아 MRA-004의
  attestation·NAT-016 envelope 소비 증거로는 사용 불가
- Host feasibility 정본화: Sol Advisor clean-room owner table, Codex·Claude custom-agent
  schema·scope·model/effort·fallback contract, Codex·Claude 한정과 Antigravity unsupported,
  default-off·foreign host-file consent acceptance 완료. Native activation과 signed runtime
  attestation은 Codex·Claude child lifecycle 증거 전 계속 차단
- Custom agent lifecycle 보강: `8fdbba4`·`88bc18f`, no-follow parent handle의 exact-byte claim
  삭제·foreign/symlink 거부·interrupted owned deletion retry 수렴, Judge invocation의 product
  setup 질문·partial reconfiguration projection 동기화. Rust workspace local qualification과
  strict Clippy PASS; host fresh-session evidence는 여전히 별도 수용 대기
- Signed host model catalog 추천 보강: `804fc80`·`1b7b2de`, `hive agent recommend`의 외부 보호
  catalog·분리 attestation·trust root 검증과 exact model/effort/minimum version mapping 결합,
  위조 서명·mapping 누락 실패 폐쇄. CLI 13·core 6·Copier/Rust parity 22·strict Clippy PASS.
  실제 발급 catalog·Codex·Claude fresh-session capability 수용 전 `MRA-024` 유지
- Custom agent decision lifecycle: `e659d87`, `accept` prior 없음·`manual|revise`의 prior digest와
  동일 scope·exact prior request 검증, `--previous-request` 재추천. CLI 14·Copier/Rust parity 22·
  strict Clippy PASS, `MRA-025` 완료
- Custom profile qualification 보강: `3344b5d`·`a86e9e4`, 13개 host lifecycle capability 각각의
  fail-closed activation, Judge `explicit|implicit` schema·default·persisted round-trip·describe
  contract 검증. custom-agent core 6·user-setup 41·strict Clippy PASS, `MRA-029` 완료
- Custom profile hostile matrix: `271ad81`, duplicate role·trigger collision과 missing receipt의
  no-mutation, Judge shadow/downgrade·forged signer·model fallback·stale capability·symlink 거부.
  custom-agent core 7·CLI 15·strict Clippy PASS, `MRA-030` 완료
- Judge invocation policy: `explicit`은 strict terminal Judge만, `implicit`은 material-risk route만
  허용. simple·read-only·format·scheduler·heartbeat·retry·결정적 실패·unsupported host는 모두 거부.
  설정 persistence·natural-language projection과 core 6·user-setup 42 focused PASS, `MRA-022` 완료
- Judge loop quorum 결합: verifier envelope v2는 exact loop subject·quorum request digest·target 밖
  protected trust root를 요구하며, checkpoint와 재검증에서 same v2 Ed25519 quorum authenticated PASS를
  다시 확인. boolean authentication claim은 mutation 전 거부. loop focused 19·strict Clippy PASS;
  실제 signer와 Codex·Claude fresh-session workflow evidence 전 `MRA-021` 유지
- Native orchestration receipt 시험: ACK 유실 `dispatch-uncertain`은 authenticated non-launch proof 전
  재prepare·final result 거부. duplicate/conflict·late receipt·clock rollback·two-scheduler prepare
  경쟁 회귀 통과로 `NAT-022` 완료. stale pointer·wrong session·Stop 100회 중립 응답도 고정,
  cancel/guard control-plane E2E는 host lifecycle 수용과 함께 `NAT-021` 유지
- Native migration 부분 게시 복구: staged `RECOVERY.toml`과 부분 `MIGRATION.md` 뒤 동일 signed
  migration `--recover` 수렴, native event·head 완성·recovery marker 제거·legacy byte 보존. Windows의
  빈 file-name run root 경로 조립 결함 수정과 직접 회귀로 `NAT-023` 완료
- Native role parity: planning·review·QA·research·performance는 하나의 `iterative-execution`
  event·receipt·evidence·usage·cancel·recover 경로 공유. `explicit|implicit` 모두 terminal Judge
  강제, `implicit`의 추가 material-risk route만 허용. 4개 product projection과 static contract 검증으로
  `NAT-020` 완료
- Native control-plane 보강: 다른 run의 malformed graph `CURRENT.md`와 무관하게 exact `--target`
  usage session disable·status 성공, stale pointer byte 보존 회귀 추가. cancel·recover의 host lifecycle
  E2E는 NAT-004·016 수용 증거와 결합 필요
- 지식 Skill 표시 정비: 정본 ID `knowledge-capture|recall|import|promote|maintain` 유지. 한국어
  표시명은 기능명과 ID 병기, 설명 첫머리는 항상 `(정본-ID)`; 자동 기록은 대화 종료 전 나중 작업에
  도움 되는 사실·선호·방식 하나의 안전한 기록, import는 명시 대상 저장소 스캔으로 표현. Codex·Claude·
  Antigravity projection·plugin/template·26개 inventory 수렴, core features HTML·PDF도 같은 지식
  흐름으로 갱신. Rust 34·Copier/Rust parity 22·Python static/inventory 19·문서·Source Wiki gate PASS
- 소비자 하네스 세션 조정 완료: Git 제외 `.hive/runtime/active-sessions/`의 exact target·path 점유,
  parent/child 충돌 거부, close·검증 가능한 stale PID recover, 세 host projection·portable directive
  fallback 구현. `project-setup`·`project-refresh`는 인증된 older base의 directive를 preview·dry-run·exact
  apply로 갱신하며, 직접 모순되는 Hive-owned clause 외 user-authored·foreign·비충돌 bytes 보존. Rust workspace,
  Python 611개 통과·40개 의도적 skip, Copier/Rust parity, Source Wiki·문서 style·링크 검사 PASS
- `0.9.2-test.1` 수용 거부: candidate run `31596919466`·publication run `31597939956`는
  통과했으나 GitHub prerelease에 direct installer 자산 3개가 누락되어 공개 설치 URL이 `404`를
  반환. 게시 workflow·회귀 보정 뒤 `0.9.2-test.2`부터 전체 공개 설치 수용 재수행
- `0.9.2-test.2` 수용 보류: candidate run `31599834995`·publication run `31600929652`,
  GitHub 자산 25개와 npm `test`, Windows clean install·upgrade·recovery·성능·보존 PASS.
  README 두 언어의 stale `test.1` 고정 안내 발견으로 번호 독립 npm `test` 안내 보정 뒤
  `0.9.2-test.3` 전체 문서·배포 수용 대상으로 전환
- `0.9.2-test.3` 기능·문서 수용 완료: candidate run `31602608609`·publication run
  `31603511607`, GitHub 자산 25개·npm `test`·npm README·Windows 공개 설치·plugin 표시·
  user-scope validate·5% usage guard·지식과 preference 보존 PASS. 완료 checklist exact tree의
  최종 `0.9.2-test.4` 뒤 source mutation 없이 protected `main`·stable candidate 진행
- `0.9.2-test.5` 최종 기능 수용 완료: candidate run `31605647532`·publication run
  `31606685534`, main history 동기화 뒤 동일 tree·25개 자산·npm `test` PASS. README 두 언어는
  stable 설치만 노출하고 유지보수자용 중립 링크 1개만 별도 출시 검증 문서로 연결. 기능·설치
  byte 변경 없음; 새 시험판 없이 문서·npm pack·PR CI 뒤 stable 진행
- `0.9.0` stable publication: candidate run `31561636239`, publication run `31562280178`,
  annotated `v0.9.0`, normal GitHub Release, npm `latest=0.9.0`, Windows 전역 설치·validate·
  release date `2026-08-12` 표시 PASS
- `0.9.1` stable publication·Windows 공개 설치 완료. Release candidate run `31578314040`,
  publication run `31579447825`, protected `main` exact `1e5e7b39c17545f8b997f71cdee39e4cd77d2ef2`,
  annotated `v0.9.1`, normal GitHub Release, 여섯 npm package `latest=0.9.1` PASS
- Usage guard 정본 전환 CI 보정 완료: 삭제된 `tests.conformance.test_source_usage_guard`
  호출을 CI·local pre-push에서 제거. exact `1227e95243374757c8d5dc51dd6348da15ec09fe`,
  CI run `31586404076`의 Linux·macOS·Windows 19개 작업 PASS
- npm README 동기화: root English `README.md` 기반 umbrella package README 생성,
  `QA Contributors` 제외와 npm link·asset 변환. `test_npm_packaging` 3개 PASS. `0.9.1`
  candidate tarball·실제 npm registry README 동일성은 `REL9-025–026` 출시 gate에 포함
- Codex plugin 표시 보정: developer `Hojin (Tom) Jeong`, 공식 colored Hive mark의 중앙
  900 px crop→512 px plugin logo, `logo`·`composerIcon`·`brandColor` manifest 계약을
  `REL9-025–026` 출시 gate에 포함
- `KAC-011` 구현 완료: commit `4311cbe`, 미등록 project CLI 자동 user-root 폴백과 전역·project
  지침·두 knowledge Skill 분기 보강. focused Rust 237·Python static 13·strict Clippy PASS
- `0.9.0` 게시 순서 오류의 최종 patch gate `KAC-012`·`REL9-026` 포함 마감.
  이 Windows의 public npm install·user-scope apply·validate, release date `2026-08-12`,
  knowledge 24개·saved preference 보존, retired empty directory·transaction entry `0건` PASS.
  `0.9.1` 뒤 실제 사용의 critical 문제 외 추가 patch 금지 원칙
- Mandatory memory regression: `0.9.0-test.13` operational user guidance의 every-turn
  `hive knowledge remember`·receipt 규칙 부재, localized `knowledge-capture` route 의미 축소.
  `KAC-002–005` 전역 안내·세 host 공통 투영·localized 설명·의미 검증 보정 완료.
  `KAC-006` 격리 user-root 기록·검색 E2E 통과.
  `0.9.0-test.17`의 schema-free `--user-statement` canonical write·derived-index receipt와 Windows preserving
  uninstall→reinstall→`hive install --validate` PASS. authenticated release와 불일치한 구조상 유효 ownership
  manifest: 이미 승인된 user-scope install/update/setup의 자동 preserving reinstall, 추가 승인 질문 없음.
  malformed·path-unsafe manifest, foreign overwrite, material choice: 사용자 결정 유지.
  `0.9.0-test.19` candidate·publication·Windows preserving reinstall·`hive install --validate`,
  embedded release date `2026-08-11` 표시 PASS.
  설치 직후 선택 host의 모든 folder에서 project setup·Hive harness·project marker·collection 없이
  user-root capture 적용. 미등록 target `auto`: user-root·shared 폴백, project-private·confidential 제외.
  [`KAC-001·007–008`](../plans/active/v0.9.0-knowledge-autocapture-regression.md) 수용 완료
  - protected `main` exact `3ebdd57ca4b90539b5de9ec51105d5e2a401dbbb` candidate run
    `31574492371` 7개 작업 PASS
  - Windows user-root preserving reinstall·validate, release date `2026-08-12`, retired empty
    `agents/`·empty transaction `0건`, knowledge 23개·saved preference SHA-256 보존
  - fresh Codex ordinary prompt의 user-root claim·receipt 자동 생성과 별도 session 자동 recall PASS
- `0.9.0-test.18` candidate [run `31525743736`](https://github.com/gvm1229/aigent-hive/actions/runs/31525743736):
  `e8fe91e` 기반, 이후 user-setup 문체·투영 정리 누락으로 취소, publication 0건
- `0.9.0-test.19` candidate [run `31526591402`](https://github.com/gvm1229/aigent-hive/actions/runs/31526591402),
  publication [run `31527961647`](https://github.com/gvm1229/aigent-hive/actions/runs/31527961647): exact
  `089b0717e24c368a1725774aaca0c85ab596df10`, test 게시·Windows 설치·version date·automatic recovery PASS
- Existing stable candidate run `31482918509`: 회귀 보정 전 historical qualification,
  tag·GitHub Release·npm `latest` publication authority 없음
- `develop` CI [run `31430181535`](https://github.com/gvm1229/aigent-hive/actions/runs/31430181535):
  exact `5ffff764bc2aa832863a10d9c01543474b300b51`, 19개 작업 전체 PASS. Copier·문서 스타일·Rust·Ubuntu/macOS/Windows 적합성 PASS
- Native release runtime [run `31428720884`](https://github.com/gvm1229/aigent-hive/actions/runs/31428720884):
  exact `c0ca51aae005cd9f494fd7ef3d2c205e472e610b`, Linux musl x86_64·arm64, macOS x86_64·arm64, Windows x86_64 전체 PASS
- `0.9.0-test.16`: candidate [run `31514244763`](https://github.com/gvm1229/aigent-hive/actions/runs/31514244763),
  exact `d4ffa337586733fabdecf6a8e0eeca309091de1e`; publication [run `31515563254`](https://github.com/gvm1229/aigent-hive/actions/runs/31515563254),
  six npm package `test=0.9.0-test.16`, annotated prerelease, `latest=0.8.0` 유지
- Windows global user install: `test.13` ownership-manifest conflict의 Hive-owned preserving
  uninstall→saved-preference reinstall→`hive install --validate` PASS. `AIgent Hive v0.9.0-test #16`,
  Codex every-turn `remember`·canonical Markdown/derived-index receipt 안내, automatic `knowledge-capture` 표시 확인.
  Fresh Codex session write→next-session recall은 `KAC-007` 대기
- `test.16` embedded release date `2026-08-01`: historical input 오류. published tag·package byte 유지,
  별도 테스트 배포 없이 다음 정상 배포에서 actual UTC date 입력·표시 검증 예정
- Stable publication trust 개정: protected `main` exact tag, same-candidate GitHub Release,
  SHA-256·GitHub attestation, npm OIDC provenance만 필수. macOS ad-hoc·Windows unsigned 공개.
  외부 release trust ceremony·platform certificate gate 제거 작업: `REL9-019–022`
- `main` stable candidate: PR #20 merge `4b3d585f8e5d014a4b282cfeb6f9b2e9f8fb0f84`의
  [run `31482918509`](https://github.com/gvm1229/aigent-hive/actions/runs/31482918509) PASS. 5개 native archive,
  npm 6개 package·direct installer·attestation과 폐기 예정 legacy authorization bundle 생성,
  tag·GitHub Release·npm mutation `0건`
- 현재 Skill 이름: [`docs/skills.md`](../skills.md)의 product-only 22개. retired 이름은
  release·plan·fact의 historical evidence와 rename ledger에만 보존, current guidance 출력 금지
- 다음 경계: Active/current legacy release trust 제거, usage guard·CodexBar·projection purge 보정,
  replacement candidate, protected stable environment의 publication approval 한 번
- `0.9.0` 예외: 실제 Antigravity host 수용과 Claude fixture 공개 제외. Codex 실제 plugin 활성화·global setup은 `REL9-011` 필수 gate. `develop → main` CI는 병합 gate 일시 면제이며 실패·미실행 범위 공개 유지
- Codex marketplace 복구: `0.9.0-test.8`의 미완료 transaction으로 남은 manifest 없는 Hive marketplace entry를
  `hive install --recover`가 Hive-owned root 확인 뒤 조용히 정리·재설치. foreign host entry·knowledge·저장 preference 보존
- Windows `0.9.0-test.13` actual user-root 수용: global npm install, `hive uninstall → install → dry-run → apply →
  validate → install validate`, saved preference SHA-256 `1cb6031da4492e20398eae9dad4e4153ab854c38c6270616084a99e9764b0faf`와
  knowledge 5개 파일 digest 보존, Hive active Skill 22개·retired ID `0건`,
  Korean·bilingual Wiki, usage guard `20%`, Discord persisted 설정, home temporary answer `0건`, `--full` 거부 PASS.
  유지보수자 확인: 새 Codex session 자동 `hive` 탐색·Discord 실제 전달. `KST-006`·`DIS9-010`·`WGS-011`·`REL9-011` 완료
- `hive uninstall`: Hive가 추가한 host activation·projection·package·derived index·backup·runtime 제거 계약.
  `.hive/knowledge/`와 저장 user preference는 항상 보존, `--full`·`-f` 삭제 경로 제공 없음. 저장 preference 재설치: setup 질문 `0건`
- Mac `0.9.0-test.19` audit에서 확인한 retired empty `agents/` leaf 44개와 empty
  `.hive/install-transactions`의 원인을 보정. 인증된 retired file의 owned boundary 안 빈 조상을
  leaf-to-root로 정리하고 transaction journal의 빈 parent를 제거. Knowledge·saved preference·
  foreign byte 보존, `.hive/dev-install`은 별도 developer rollback state로 일반 purge 제외
- Agent 자율 실행 지속: 이전 시험판 작업에서 Agent 소유 회귀 정리·검증·push·후보·게시가
  남은 중간 보고 종료 판단 오류. [`agent-autonomous-continuation.md`](../plans/active/agent-autonomous-continuation.md)의
  `AAC-001–008`: source·소비자 프로젝트·전역 설정 지시문 terminal state·closure gate·session record·static regression 보강 완료
- 이전 expedited acceptance의 usage guard disabled 기본은 superseded. 현재 기본: 활성화 권장·
  신속 `20%`. Normal setup의 CodexBar 노출 `0건`, Codex native-only probe의 allowlisted 실패 뒤에만
  별도 사용·설치 동의
- `REL9-030`: Python 전수 module의 단일 lane manifest·owner·contract·release gate, Rust command·CI job 대장,
  `os × lane` CI matrix·Windows 시간 기록 완료. Python 순차 477.77초 대비 matrix critical path 262.03초 모델,
  45.2% 단축. Phase 4 repository fixture: ignored `tests/work/hive-phase4-<random>`만 사용
- Discord 수용: 실제·시험 알림의 동일 renderer·선택 필드·선택 언어, 시험 알림 첫 줄의 변경 안내 고지, 첨부 화면의 webhook 전달
- Discord 알림 형식: 사용자 승인 구역형 Markdown 적용 완료. 사용량·작업 정보·작업 계속 요청 구역, 빈 줄과 이모지·굵은 제목 적용. 밑줄 표기 `0건`
- `0.9.0-test.11`: candidate [run `31372510565`](https://github.com/gvm1229/aigent-hive/actions/runs/31372510565)의
  exact `b0e41f58bd6b73b56cbe92c2b054fb5cefcc9f03`, 5개 native target·npm umbrella·direct installer·attestation PASS.
  publication [run `31373214154`](https://github.com/gvm1229/aigent-hive/actions/runs/31373214154)의 six-package OIDC `test`
  게시, annotated GitHub prerelease, `test=0.9.0-test.11`, `latest=0.8.0` 유지 PASS
- `0.9.0-test.13`: candidate [run `31403054797`](https://github.com/gvm1229/aigent-hive/actions/runs/31403054797)의
  exact `03a16676ebe8873493b85b717befc68983825cdd`, 5개 native target·npm umbrella·direct installer·attestation PASS.
  publication [run `31404195752`](https://github.com/gvm1229/aigent-hive/actions/runs/31404195752)의 six-package OIDC
  `test` 게시, annotated GitHub prerelease, `test=0.9.0-test.13`, `latest=0.8.0` 유지 PASS
- `0.9.0-test.6`: 후보 [run `31294665865`](https://github.com/gvm1229/aigent-hive/actions/runs/31294665865)의 5개 native target·npm 묶음·attestation PASS. 게시 [run `31295045199`](https://github.com/gvm1229/aigent-hive/actions/runs/31295045199)의 여섯 package OIDC publication, annotated `v0.9.0-test.6`, 22-asset GitHub prerelease, 여섯 package `test=0.9.0-test.6`, `latest=0.8.0` 유지 PASS. Windows clean install·fresh Codex session 수용 전 stable 미착수
- `REL9-027`: six-package Trusted Publisher OIDC 실제 게시 PASS. `BGR-012–013` source 응답의 내부 용어 억제·사용자 영향 우선 설명과 local `-dev → 0.9.0-test.6` user-scope validation 완료. `SIL-001–006` public Skill identity·localization·retired-ID cleanup과 `0.9.0-test.5` 독립 시험 게시 완료. `N10-001` Notion 사용자 노출 차단 완료. `DIS9-004`의 부분 설정 전체 목록·Discord 하위 항목 표시 계약 구현. 다음 작업: `DIS9-002–010` Discord 연결 UX. Notion end-to-end 기능은 `0.10.0-test`까지 보류
- Skill 최종 결정: [`docs/skills.md`](../skills.md)의 product-only 22개. Source 개발도 설치 product
  Skill과 repository directive 사용, tracked source Skill 최종 `0건`. 신규 product `ship`·
  `amend-directive`; `source-review`는 Wiki 조회·기본 read-only 도구, `source-knowledge`는 세
  knowledge Skill·`hive source-wiki` CLI로 분리. retired-ID ledger·all-host projection·Copier ledger 회귀 완료
- 사용량 보호: 활성화 권장·신속 기본 `20%`로 보정 완료. Custom threshold·등록 project별 더
  보수적인 override·effective `max` 유지. CodexBar: post-init native 실제 실패 뒤에만 별도 동의
- 외부 중지 경계: `main` PR·review, protected stable publication approval,
  exact `1.0.0` 사용자 authority
- Plan load: compact `docs/plans/PLAN.md` + `docs/plans/phases/07-public-qualification.md`
  + `docs/plans/active/plugin-project-lifecycle.md`
  + `docs/plans/active/native-usage-sensor.md`
  + `docs/plans/active/user-onboarding-shared-index.md`
  + `docs/plans/active/source-docs-wiki.md`
  + `docs/plans/active/windows-shell-install.md`
  + `docs/plans/active/documentation-style.md` + `docs/plans/active/security-review.md`
  + `docs/plans/active/docs-wiki-migration.md`
  + `docs/plans/active/release-0.8.0.md`
  + `docs/plans/active/v0.9.0-loop-wiki-skills.md`
  + `docs/plans/active/v0.9.0-global-knowledge-rag.md`
  + `docs/plans/active/v0.9.0-knowledge-autocapture-regression.md`
  + `docs/plans/active/v0.9.0-knowledge-portability-scan.md`
  + `docs/plans/active/native-iterative-execution.md`
  + `docs/plans/active/model-routed-custom-subagents.md`
  + `docs/plans/active/prompt-refine-auto-routing.md`
  + `docs/plans/active/v0.9.0-test-finalization.md`
  + `docs/plans/active/release-0.9.0.md`
  + `docs/plans/active/test-release-setup-routing.md`
  + `docs/plans/active/bootstrap-global-setup-recovery.md`
  + `docs/plans/active/korean-setup-terminology.md`
  + `docs/plans/active/global-skill-selection.md`
  + `docs/plans/active/skill-identity-localization.md`
  + `docs/plans/active/usage-guard-policy.md`
  + `docs/plans/active/discord-onboarding-v09.md`
  + `docs/plans/active/windows-global-setup-hardening.md`
  + `docs/plans/active/v0.10.0-notion-candidate.md`
- 공개 한국어 HTML 안내: `PHG-001–005` 완료. 핵심 기능·간단 설치 HTML commit
  `a9224cc`, 벌집 금색 accent, 기존 logo, 상호 link·local asset·LXML parse PASS.
  디자인 원칙 `docs/guides/public-html-design-principles.md`, 설치 3단계의 host별 `--host`
  반복 예시·복수 `selected_hosts` 계약 확인. README branding: 기존 commit `245ae80`
- 공개 HTML 독립 공유: `PHG-006` 완료. 정본 PNG 원본 byte 내장, system font, network·file-relative resource 0건, 프로젝트 밖 Edge desktop·mobile render PASS
- 복수 호스트 사용자 설치: `MHI-001–005` 완료.
  `--hosts codex,claude`·반복 `--host codex --host claude`, 전체 dry-run preflight,
  입력 순서 실행과 부분 실패 aggregate JSON 구현·Rust 전체 328개·strict clippy 통과,
  commits `4f97787`·`6bab86b`·`ff1a28a`·`565b41f`; 공백 포함 CSV는 전체 argument 따옴표 표기,
  `develop` push 완료
- Plan completion: canonical checklist `441/528` 완료, `87`개 미완료, `83.5%`
- 출시 분리: `0.9.0`의 user-visible Wiki는 local Markdown 정본과 SQLite projection만 제공.
  Notion backend·host browser OAuth·freshness·write-through·사용자 문서는 `0.10.0-test`까지 보류.
  Discord webhook 대화·시험 알림, HTML 안내, project·run·요청·progress payload는 `DIS9-*` 후속 범위
- Korean setup 용어: `KST-006` Windows actual 수용 완료. `Skill → 기술` 번역 차단,
  product-only 22개 Skill·global user context 보존·canonical/plugin projection parity 유지
- Global Skill selection: `a30eb47`로 profile-bound recommended suite 제거, 새 setup 기본
  all built-in·개별 Skill toggle·one-entry-per-line 적용. 기존 recommended closure는 saved
  answer validate에서만 해석, 새 `all|individual` preview·approval 전 활성 Skill 추가 0건.
  project recommendation은 분리 catalog 유지
- Skill identity·localization: 22개 consumer Skill의 public short name 적용 완료. 후속 catalog는
  source·product 이름 분리와 24개 target identity 승인 완료, `SIL-008` 실제 이관 대기. Host invocation:
  `aigent-hive:<short-name>`. Canonical `retired-names.yml`: retired ID→current ID mapping·collision
  reservation·saved selection migration. 삭제 authority: frozen historical release inventory 또는 installed
  ownership manifest의 exact byte 검증만 허용. `0.8.0` 세 host의 historic Skill path 전체 삭제와 변조
  path no-write conflict 회귀 확인. [candidate `31183471023`](https://github.com/gvm1229/aigent-hive/actions/runs/31183471023)
  5 native target·npm umbrella PASS. [publication `31184578205`](https://github.com/gvm1229/aigent-hive/actions/runs/31184578205)
  registry-token fallback PASS; six npm package `test=0.9.0-test.5`, `latest=0.8.0`, annotated
  `v0.9.0-test.5`·22-asset prerelease 확인. Trusted Publishing은 새 scoped platform package에 `404` 반환
- Global Skill test release: candidate `31134306991` 5 native target·npm umbrella/direct
  installer PASS, publication `31135040224` PASS. 여섯 package `test=0.9.0-test.4`,
  `latest=0.8.0`; annotated `v0.9.0-test.4`·22-asset prerelease와 isolated CLI
  `AIgent Hive v0.9.0-test #4 · developer test build (released 2026-08-07)` 확인. Trusted
  Publisher registry 404는 bootstrap registry-token fallback으로 안전하게 해소
- Developer binary: `scripts/dev-install.sh`의 sandbox·global·CAS rollback과 `product-dev`
  version identity, isolated global activation·rollback 회귀 검증 완료. Local `-dev` binary는
  internally reproducible prior manifest·live byte 일치 때만 developer-only global projection
  refresh 허용, public stable·test binary의 fail-closed 보존
- Global setup refresh: authenticated user install·saved-answer projection drift의 dry-run·apply·
  revalidate 자동 처리. Review-only yes/no 질문 제거, local edit·인증 실패·별도 권한 경계만 질문
- Legacy setup recovery: schema-1 `0.7.0`의 19개 legacy projection은 saved preference·path
  inventory·live digest 일치 뒤 schema 2 base로 이관. later Codex metadata는 추가 처리,
  pre-schema-2 local edit·unknown inventory는 write 0건. `0.9.0-test.3` Codex host inventory도
  frozen `setup-hive` digest 기반 인증 복구. local user-scope apply·validate PASS, stale
  Antigravity `0.7.0` Hive-only projection은 recoverable Trash 이동, canonical knowledge·index 보존
- Source Wiki route: source marker 확인 뒤 `hive source-wiki query` 사용,
  consumer `hive knowledge retrieve`의 source root 호출 금지·static contract 36개 PASS
- Fresh clone: exact `6761f0b`, Rust format·strict Clippy·workspace all-feature, Python 677개 PASS·platform skip 5개
- Test candidate: [run `30771098518`](https://github.com/gvm1229/aigent-hive/actions/runs/30771098518), exact `6761f0b`, 5 target·npm umbrella PASS
- Latest local Windows: Rust workspace 459개 실행·통과. Python 적합성 670개 발견 중
  628개 실제 실행·통과, 42개 미실행. 미실행 범위: 관리자 권한 없는 Windows의
  symbolic link 생성 제약 16개, POSIX·Unix 전용 동작 19개, macOS 전용 설치·서명
  동작 7개. 운영체제 판별: Windows. 미실행 42개: 이 컴퓨터에서 검증 완료로
  판단할 근거 불충분. PowerShell 5.1·7.6.4 installer와 `cmd.exe`
  bootstrap 계약은 이 Windows 컴퓨터에서 실제 실행·통과
- Latest native remote: exact `420e244`의 candidate run `30657669889`,
  macOS·Linux·Windows 5/5와 npm umbrella PASS
- Latest npm publication: run `30658188721`, exact `0.8.0` 여섯 package,
  `latest=0.8.0`, 기존 `test=0.8.0-test.1`, provenance PASS
- Actual Windows public install: npm·CMD clean install, repeat, pending receipt recovery,
  product·receipt `0.8.0`, npm·direct SHA-256
  `330f4e0c8da5b6347400b9b16a9f76b2fb4f94406a2eacfe8c641367ca344ef9`
- Native Goal routing: legacy `PLAN.md` checkbox 문구를 `phases/07-public-qualification.md`,
  `active/plugin-project-lifecycle.md`, `active/native-usage-sensor.md`,
  `active/user-onboarding-shared-index.md`, `active/source-docs-wiki.md`,
  `active/windows-shell-install.md`,
  `active/documentation-style.md`, `active/security-review.md`,
  `active/docs-wiki-migration.md`,
  `active/release-0.8.0.md`, `active/v0.9.0-loop-wiki-skills.md`,
  `active/v0.9.0-global-knowledge-rag.md`, `active/native-iterative-execution.md`,
  `active/model-routed-custom-subagents.md`로 해석

## Hive-native 반복 실행 전환

- 상태: RALPLAN-DR·Architect·Critic 승인과 정본 plan·proposed ADR 완료, host feasibility 미착수
- 결정: [`ADR-0019`](../decisions/ADR-0019-hive-native-iterative-execution.md) proposed
- Active fragment:
  [`native-iterative-execution.md`](../plans/active/native-iterative-execution.md)
- Hive 소유 목표: event reducer·logical scheduler·lease·receipt·cancel·team·multi-goal state
- Host 소유 유지: model call·model/subagent process·native task identity·envelope consume
- 신규 경계: OMX·OMC functional dependency 없음, provider API·credential·direct process spawn 없음
- Authority: selected session pointer는 selector only; exact target·event head·control epoch·one-time authority 필수
- Incident regression: wrong pointer + Stop 100회 canonical mutation `0건`, cancel·guard·recover 독립 접근
- 불확실 dispatch: qualified non-launch proof 없는 automatic reclaim `0건`, `dispatch-uncertain` 중지
- Legacy run: read-only provenance, migration은 새 native identity와 원본 byte 불변
- 다음 작업: NAT-002–005와 MRA-001–006 capability·Sol Advisor parity 재분류,
  세 host orchestration과 Codex·Claude exact-model lifecycle feasibility spike
- Activation gate: feasibility·ADR acceptance·schema·security qualification 전 default-off

## Model-routed custom subagent

- 상태: `0.9.0` 실행 계획 활성, 구현 미착수
- 결정: [`ADR-0019`](../decisions/ADR-0019-hive-native-iterative-execution.md) proposed
- Active fragment:
  [`model-routed-custom-subagents.md`](../plans/active/model-routed-custom-subagents.md)
- 지원: OpenAI Codex·Claude Code. Antigravity는 근거 있는 custom-agent surface 확보 전 unsupported
- 목표: Sol Advisor의 orchestrator→routine/complex implementer→reserved independent Judge 흐름 clean-room 동등 구현
- Model authority: role별 exact model ID·thinking level 고정, runtime receipt 불일치 결과 fail-closed
- Scope: user·project canonical role과 host projection, project precedence, preview·명시적 동의·non-clobber
- Built-in 후보: routine·complex implementer, design·article·research specialist,
  user-scope reserved `hive-independent-judge`
- Auto-call: Skill·role description 기반 semantic route, simple·작은 단일 단계·증명 불가 task 제외
- 생성 Skill: 목적 우선 질문 뒤 이름·양쪽 host model/effort·scope·권한 추천,
  `1 수락 | 2 수동 | 3 수정`, 적용 뒤 동일 auto-route registry 통합
- Judge 정책: setup의 `explicit`은 strict iterative·team·multi-goal terminal gate만,
  `implicit`은 strict gate + 일반 material-risk route. Natural-language reconfigure 지원
- Judge 경계: Codex `gpt-5.6-sol/high` 후보, Claude exact profile 검증 대기, project shadow 금지.
  Agent는 verdict만 생성하고 외부 signer가 Ed25519 private key 소유
- Token 경계: scheduler tick·heartbeat·retry별 Judge `0건`; dispatch 전 usage guard,
  strict gate 제한 시 성공 우회 없이 pending·usage-limited 중지
- 다음 작업: MRA-001–006 Codex·Claude 공식·실제 lifecycle와 Sol Advisor 기능 동등성 검증
- Activation gate: 양쪽 host fresh-session E2E·exact attestation·ownership consent·hostile test 전 default-off

## v0.9.0 구현

- 상태: source 구현·local qualification 완료, publication 미실행
- 결정: [`ADR-0015`](../decisions/ADR-0015-host-native-skill-composition.md) accepted
- Active fragment:
  [`v0.9.0-loop-wiki-skills.md`](../plans/active/v0.9.0-loop-wiki-skills.md)
- 범위: `hive-loop-engineering`, `hive-wiki`, `ai-slop-cleaner`,
  `best-practice-research`, 기존 run·role·usage·judge Skill 조합
- Loop 계약: host-native subagent·goal·hook capability, DAG·cycle detection·bounded
  retry·evidence edge·independent verification·dynamic steering·terminal state
- Wiki 계약: `add|query|lint|list|read|delete|refresh`, keyword·tag·category,
  taxonomy, `[[wikilink]]`, agent-reviewed quick-add
- Utility 계약: 회귀 시험 우선 code cleanup·fallback 분류·변경 파일 한정,
  읽기 전용 bounded 연구·공식 source 우선·저장소 사실 분리·handoff
- 채택 계약: 전체 OMX·OMC Skill·adapter의 `adopt|merge|exclude` 근거표와
  비중복·사용자 승인·license·보안·conformance gate
- 완료 기준선: scheduler·model runtime·tmux·Stop continuation·`omx_wiki`·`.omx|.omc`·
  `omx|omc` command·자동 adapter 우선권·raw session 자동 수집 0건
- 후속 정책: scheduler·iterative·team·multi-goal non-goal은 ADR-0019로 superseded,
  model runtime·provider API·direct process spawn 금지는 유지
- 실행 결과: V9-001–025 완료, host-native 기본값 전환과 세 host projection PASS

## `0.9.0-test` 기능 마감

- 상태: `TST9-001–018` 구현·검증 완료, public test publication 대기
- 결정: [`ADR-0018`](../decisions/ADR-0018-notion-wiki-backend.md) accepted, Notion 공개 범위 `0.10.0-test` 보류
- Active fragment:
  [`v0.9.0-test-finalization.md`](../plans/active/v0.9.0-test-finalization.md)
- Wiki backend: local Markdown 정본·user-root SQLite projection
- Notion typed core: `0.10.0-test` 후보로 보류, `0.9.0` user setup·help·README·release note 노출 없음
- Discord: usage guard 중단의 optional outbound, Claude inbound official plugin 위임,
  Codex inbound official capability 전 `unsupported`
- 다음 작업: `REL9-014–015` public test acceptance·retention 관찰

## v0.9.0 시험·정식 릴리스

- 상태: candidate·fresh clone qualification PASS, public test prerelease·npm 게시 완료;
  수용 관찰 진행
- 사용자 authority: 분리된 시험·정식 `0.9.0` 계획과 원격 `develop` push 승인
- 결정: [`ADR-0017`](../decisions/ADR-0017-0.9-full-release.md) accepted
- Active fragment: [`release-0.9.0.md`](../plans/active/release-0.9.0.md)
- 변경점: [`docs/releases/0.9.0.md`](../releases/0.9.0.md)
- Test identity: 기본 `0.9.0-test|test`, 추가 시험 시에만 `0.9.0-test.N|test`,
  기존 `latest` 불변
- Stable identity: 시험 수용 뒤 별도 protected `main` exact commit·annotated `v0.9.0`·
  GitHub normal Release·npm `0.9.0|latest`
- Parity: 시험·정식 기능·명령·기본값 동일, 시험 전용 기능 0건
- 공통 문제 보고: 명시적 preview·collect·export, 자동 업로드·raw prompt 기본 수집 0건
- Candidate: run `30771098518`, exact `6761f0b`, 5 target·npm umbrella·direct installer PASS
- Host preflight: Codex `0.146.0` dry-run PASS; Antigravity `1.1.9` unowned
  `aigent-hive` namespace conflict, host apply·update 보류
- Test workflow registration: [#16](https://github.com/gvm1229/aigent-hive/pull/16) `main` merge 완료
- Release surface: [#17](https://github.com/gvm1229/aigent-hive/pull/17) CI·review·`main` merge 대기,
  `release-publication` secret 유지·reviewer 0명과 future Deployment record 생성 비활성화
- Test dispatch: [run `30789141992`](https://github.com/gvm1229/aigent-hive/actions/runs/30789141992)
  `dist/...` Git remote parse failure, 첫 npm 게시 전 중단, `latest`·tag·GitHub Release mutation `0건`
- Test retry: [run `30808850724`](https://github.com/gvm1229/aigent-hive/actions/runs/30808850724)
  `./dist/...` local file spec 뒤 first npm publish `404`, version mutation `0건`
- Bootstrap retry: [run `30890841117`](https://github.com/gvm1229/aigent-hive/actions/runs/30890841117)는
  여섯 npm `0.9.0-test` publish와 `test=0.9.0-test`, `latest=0.8.0` verification까지 PASS.
  마지막 tag/Release는 GitHub App token의 workflow-tag 권한 거부로 실패
- Actual prerelease: authenticated maintainer recovery로 `6761f0b` annotated
  `v0.9.0-test`, [GitHub prerelease](https://github.com/gvm1229/aigent-hive/releases/tag/v0.9.0-test),
  22 assets 생성 완료. npm public install `0.8.0 → 0.9.0-test`와 CLI 실행 확인
- App automation: reviewer 0명. client ID Variable·private key Secret, `contents|workflows: write`
  installation token, credential-free checkout 적용. [candidate `31042797141`](https://github.com/gvm1229/aigent-hive/actions/runs/31042797141)
  `dd0224a`와 [publication `31043631056`](https://github.com/gvm1229/aigent-hive/actions/runs/31043631056) PASS.
  `0.9.0-test.1` 여섯 npm package `test`, `latest=0.8.0`, annotated tag·22-asset prerelease 확인
- Corrected test publication: [candidate `31082481203`](https://github.com/gvm1229/aigent-hive/actions/runs/31082481203)
  `6980e8b`의 5 native target·npm umbrella PASS; [publication `31083602464`](https://github.com/gvm1229/aigent-hive/actions/runs/31083602464)
  `0.9.0-test.2` 여섯 package `test`, `latest=0.8.0`, annotated tag·22-asset prerelease PASS.
  Isolated npm install `--version`: `AIgent Hive v0.9.0-test #2 · developer test build (released 2026-08-06)`
- Trusted publishing exception: [run `31083140684`](https://github.com/gvm1229/aigent-hive/actions/runs/31083140684)의
  2회 `@aigent-hive/darwin-arm64` registry `404`, npm package·tag·GitHub Release mutation 0건;
  existing registry-auth fallback으로 `31083602464` 게시 완료. npm trusted publisher binding 재정비 뒤 token 없는 경로 재검증 필요
- Source baseline: `6980e8b38c08a9ebe483a4ffa7937f70999d63a5`, `develop` 포함
- Setup-routing test publication: [candidate `31090062784`](https://github.com/gvm1229/aigent-hive/actions/runs/31090062784)
  와 [publication `31090917408`](https://github.com/gvm1229/aigent-hive/actions/runs/31090917408) PASS.
  exact `5341bdf3562cb1ed8fdd3323965cc5f529649107`, 여섯 npm package
  `test=0.9.0-test.3`·`latest=0.8.0`, annotated `v0.9.0-test.3`, 22-asset prerelease.
  Existing Codex legacy user installation의 source test.3 dry-run authenticated preview PASS.
- Global Skill test publication: [candidate `31134306991`](https://github.com/gvm1229/aigent-hive/actions/runs/31134306991)
  5 native target·npm umbrella/direct installer PASS; [publication `31135040224`](https://github.com/gvm1229/aigent-hive/actions/runs/31135040224)
  PASS. `dc4466d42f4d3c4b71472e1ee8e6f27b58b2212a`, 여섯 package `test=0.9.0-test.4`,
  `latest=0.8.0`, annotated `v0.9.0-test.4`·22-asset prerelease와 isolated CLI #4 확인
- `staging`: 현재 release flow에 불필요하여 생성 0건
- Production gate: 5개 native target·6개 npm package·3개 installer, SHA-256·GitHub attestation·
  npm OIDC provenance, public install·`0.8.0 → 0.9.0` update
- 다음 작업: `REL9-014–015` public acceptance·retention 관찰; stable은 별도 main 후보까지 시작 금지

## Prompt refine 자동 routing

- 상태: `PRF-001–012` 구현·검증 완료
- Active fragment:
  [`prompt-refine-auto-routing.md`](../plans/active/prompt-refine-auto-routing.md)
- 원인: source·consumer policy와 Skill catalog의 explicit-only·suggestion-only 계약
- 결함: explicit invocation 뒤 imperative payload를 실행 승인으로 오해한 host-level
  mode boundary
- 목표: material ambiguity 자동 `refine-only`, refined prompt 제시 뒤
  `awaiting-approval`, exact 후속 승인 전 side effect 0건
- 보존: simple/editless question·clear work route, prompt-classifier hook 금지,
  frozen `0.7.0|0.8.0` Skill bytes
- 다음 작업: test publication 뒤 public parity 수용 관찰

## v0.9.0 전역 knowledge RAG

- 상태: RAG-001–020 완료, publication 미실행
- 결정: [`ADR-0016`](../decisions/ADR-0016-global-knowledge-rag.md) accepted
- Active fragment:
  [`v0.9.0-global-knowledge-rag.md`](../plans/active/v0.9.0-global-knowledge-rag.md)
- 기존 기반: user-root Markdown·단일 SQLite, FTS5·tag·alias·BM25,
  visibility-aware shared project query
- 구현: 모든 질문의 bounded retrieval preflight, named project scope, durable user statement의
  mandatory write, citation-ready chunk result와 fresh-session recall
- DB: SQLite derived boundary, chunk·generation·dirty journal, 검증된 resident generation
- 성능: 50,000 chunk cold p95 `163.3569ms`, warm p95 `0.1178ms`
- 이식: `.hivekb` canonical bundle export·import 뒤 destination SQLite rebuild
- 수집: directory별 table 대신 stable `collection_id`, explicit `hive-knowledge-scan`과
  project claim → reusable candidate 2단계 review
- 검색 Skill: 새 find Skill 없이 기존 `hive-knowledge-query`의 질문·research·work
  bounded automatic route
- 안전 경계: Wiki opt-out, raw transcript·secret·credential·SQLite-only fact 0건

## v0.9.0 knowledge 이식·directory scan

- 상태: KPX-001–018 완료, publication 미실행
- Active fragment:
  [`v0.9.0-knowledge-portability-scan.md`](../plans/active/v0.9.0-knowledge-portability-scan.md)
- Research:
  [`knowledge-portability-ingestion-retrieval.md`](../research/knowledge-portability-ingestion-retrieval.md)
- Bundle: deterministic ZIP + versioned manifest·SHA-256, SQLite·runtime·absolute path 제외
- Collection: fixed normalized schema, stable ID, detached destination mapping
- Scan: tracked-first inventory, claim kind·assertion status, evidence-qualified convention,
  기존 promote의 consolidated consent 재사용
- Retrieval: existing query Skill single owner, turn당 1회 top 5·byte budget,
  retrieved instruction authority 0건
- Qualification: 100 collection·50,000 chunk export p95 `1066.9209ms`,
  import+rebuild p95 `3255.1537ms`
- Adversarial review: overlap·schema growth·archive path·secret·poisoning·truth overclaim·
  path portability·context overload finding 교정, 사용자 결정 잔여 0건

## `docs/` Wiki 전환

- 결정: [`ADR-0014`](../decisions/ADR-0014-docs-wiki-architecture.md)
- Active fragment:
  [`docs-wiki-migration.md`](../plans/active/docs-wiki-migration.md)
- 유지: 간결한 English·Korean README와 빈 QA Contributors 표
- 복원 source: 간소화 직전 Git `README.md`
- 목표 구조: `docs/00-home.md`, `docs/01-index.md`, topic MOC,
  `docs/facts/{en,ko}` atomic pair
- 제거 완료: standalone source-Wiki directory를 tracked tree에서 제거
- 보존 원칙: valid knowledge 이동 우선, deprecated·incorrect·superseded knowledge만 제거
- AI directive: 간소화 전 durable claim inventory, replacement locator와 docs home·index
  도달성 확인, Git history recoverability 적용
- Human Wiki: `docs/00-home.md`, `docs/01-index.md`, topic MOC, product overview,
  development guide에서 간소화 직전 README knowledge 복원
- Atomic knowledge: `docs/facts/en`·`ko` 40개 exact pair, primary fact 1개,
  cross-link와 source digest

## Windows shell 설치 경계

- Consumer runtime: `hive.exe`·installed harness의 PowerShell dependency 없음
- Consumer direct install: Windows 기본 `powershell.exe` 5.1 지원
- Consumer `cmd.exe`: exact-version PowerShell 5.1 bootstrap 호출 명령 지원
- Consumer PowerShell 7: dependency·탐지 경고·설치 prompt 없음
- Source Windows: PowerShell 7.6.4 LTS 개발·release dependency
- Source dependency setup: exact command·package·scope preview, 명시적 동의,
  Microsoft 지원 installer 위임, Hive update·uninstall 없음
- Current host evidence: Windows PowerShell `5.1.26100.8875`, PowerShell `7.6.4`,
  Rust·Cargo `1.97.1`, Copier `9.17.0`, pip `26.1.2`
- Current implementation: `[IO.File]::Replace` atomic overwrite, shell-independent
  UTF-8, `cmd.exe` bootstrap, source dependency preview·동의·재검증
- Local evidence: Rust workspace 전체 PASS, Phase 6 Windows contract 21개 실행,
  platform 비대상 8개 expected skip와 나머지 PASS
- Active fragment:
  [`windows-shell-install.md`](../plans/active/windows-shell-install.md)
- Decision:
  [`ADR-0013`](../decisions/ADR-0013-0.8-release-scope.md)

## Global onboarding·shared index audit

Target: `0.8.0`

| 요청 범위 | 현재 구현 |
| --- | --- |
| User install | 세 host minimal bootstrap 뒤 mandatory global setup |
| Mandatory global setup | `setup-hive`와 user-scope setup CLI |
| Language·user profile·persona·multi-host | 첫 질문 language, 이후 선택 언어의 signed catalog one-question sequence |
| Update 확인 | Explicit opt-in daily check, offline 뒤 다음 host session 재시도 |
| Skill suite 선택 | Recommended 또는 individual, dependency closure preview |
| Wiki opt-out | Default-on, 언제든 disable/enable, Markdown 보존 |
| Usage guard 선택 | 활성화 권장, 신속 `20%`, Custom 사용자 선택 한도, failure-only fallback consent |
| User marker | `AIGENT-HIVE:USER:START|END` append·owned replace |
| User `.agents` | Provider-neutral directive·selected Skill projection |
| Root knowledge | `~/.hive/knowledge` + disposable root SQLite |
| Project setup mode | `expedited|custom`, 양쪽 모두 project kind 필수 |
| Project type | Required project identity·domain profile |
| Project index | User-root 단일 SQLite, project DB 생성 없음 |
| Initial global expedited | Language와 update-check consent 뒤 나머지 default 적용 |
| Project auto onboarding | Global 상속·canonical evidence·unresolved-only 질문 |
| Wiki task-fact capture | Wiki enabled material-task 종료 시 검토된 결과·도구·기준·요청 요약 자동 기록 |

완료 evidence:

- Global setup state·schema·catalog·selected projection
- Wiki disable의 Skill·operation 차단과 canonical Markdown 보존
- Usage consent의 runtime sensor·fallback 제어
- Project activation + root registry/index 연결 rollback
- Connected `0.7.0 → 0.8.0` preference 보존, unconnected setup-required fail-closed,
  legacy project DB cleanup과 거부 시 전체 install tree 무변경
- Codex·Antigravity expedited/custom connected matrix 4/4
- Initial expedited fixed defaults와 `auto-setup-harness` zero-question inference 구현
- Initial setup의 첫 질문 `English|한국어`, 이후 setup Skill 질문·preview와
  user directive·host guidance의 선택 언어 적용
- Initial setup update-check consent, 성공 확인 뒤 24시간 throttle, offline·malformed
  결과 무기록과 다음 host session retry, check-only no-install 구현
- Wiki disable 시 0건, enable 시 agent-reviewed bounded task-fact completion capture
- Auto Skill canonical·plugin·source·Codex·Claude projection parity
- 실제 Windows 11 x86_64 Codex user install·validate, recommended global setup,
  zero-question project auto onboarding, user-root shared index 재빌드·lint PASS
- Same-version repeat update·recover 뒤 user install·project harness 재검증 PASS
- Skill validator PASS, `hive-cli` 223/223와 version integration PASS,
  `hive-render` 59/59, Wiki·static contract 65/65 PASS
- Signed `0.8.0` release activation은 Phase 7 외부 gate
- Decision:
  [`ADR-0012`](../decisions/ADR-0012-global-onboarding-shared-index.md)
- Active fragment:
  [`user-onboarding-shared-index.md`](../plans/active/user-onboarding-shared-index.md)

## Source docs Wiki

- Canonical path: `docs/facts/en/`, `docs/facts/ko/`
- 금지 path: `omx_wiki/`, `.omx/wiki/`, source root의 consumer `.hive/knowledge/`
- Current OMX/OMC: 신규 workflow dependency 제거 결정, legacy foreign provenance만 보존
- 장기 방향: Hive-native provider-neutral orchestration과 explicit legacy migration
- Consumer reuse: `hive-wiki` core와 capture·maintenance·query 안전 계약
- Skill reuse: shared canonical `harness/skills/`, exact source `.agents/skills/` projection
- 현재 상태: 영어 46개·한국어 46개 atomic fact, exact pair 46개와 source-confined
  CLI·Skill·material-task completion capture 구현 완료
- Derived source Wiki index: `docs/facts/` 46 pair 기준 rebuild 완료
- Current logical digest:
  `sha256:71c830f55adaf85d92c58c1a8ff3ebfe816789bd70e857e2a8c1dc47791dc502`
- 현재 검증: fact schema·pair·source digest·body limit·문서 graph 시험과 current
  `target/debug/hive` Source Wiki lint·index·영어·한국어 query PASS
- SQLite binary digest는 invocation-local evidence이며 정본·clean-copy equivalence 기준이
  아님. Logical digest와 query 결과가 rebuild equivalence 기준
- Marketing deck 재개 record:
  [`aigent-hive-marketing-deck.md`](artifacts/aigent-hive-marketing-deck.md)
- LumaDeck 사용·생성 기준·초기 요청 요약:
  [`marketing-deck-record.md`](../facts/ko/marketing-deck-record.md)
- Current Wiki tests: `hive-wiki` 33/33, Source Wiki conformance 재검증 PASS
- OMX Wiki Skill 제외 이유·향후 OMX/OMC retirement 시 knowledge migration 0건:
  [`ADR-0011`](../decisions/ADR-0011-source-wiki-independence.md)
- Active fragment:
  [`source-docs-wiki.md`](../plans/active/source-docs-wiki.md)

## 세 host native usage sensor

- Codex: `codex-cli 0.145.0` app-server native primary와 process identity·bounded JSONL
  adapter 구현
- Claude Code: host-owned `/statusline` opt-in용 sanitized 5-hour·7-day capture 구현,
  `~/.claude/settings.json` mutation 0회, 실제 Pro/Max qualification 잔여
- Antigravity CLI `1.1.7`: native machine sensor `unsupported`, qualified CodexBar
  fallback 구현
- 실제 Antigravity fallback: CodexBar `0.45.2`, `default`·
  `antigravity-claude-gpt` provider-defined pool, threshold `10%`, selected window
  `multiple`, exit `0`, raw payload persistence 0건
- 세 provider 공통 CodexBar fallback-only, native limited 뒤 fallback 우회 0회
- CodexBar 미설치 notification·fixed command preview·explicit current-action consent 구현
- Deferred: 실제 Claude Pro/Max parity와 future Antigravity native fixture
- Active implementation fragment:
  [`native-usage-sensor.md`](../plans/active/native-usage-sensor.md)
- Decision:
  [`ADR-0010`](../decisions/ADR-0010-native-first-usage-sensors.md)

## 구현 완료 범위

| Phase | 완료 범위 |
| --- | --- |
| 1 | 결정적 setup, staging, ownership, conflict·rollback, host projection |
| 2 | canonical Markdown knowledge, disposable SQLite index, rebuild·suppression |
| 3 | portable Skill routing, simple-question isolation, prompt refinement, OMX/OMC precedence |
| 4 | persistent role, durable run, fresh-session recovery, owner continuity |
| 5 | subscription usage policy, one-shot dispatch authorization, authenticated judge quorum |
| 6 | verifier-only signed release, update·migration·backup·crash recovery, installer ownership |
| 7 local | shipping one-shot usage gate, 세 host projection, provenance verifier, fault injection |

구현 완료된 v0.9 기준선:

- Provider API·SDK·credential path 없음
- Model runtime·provider session engine·direct model/subagent process launcher 없음
- Native scheduler·iterative·team·multi-goal: ADR-0019의 default-off 후속 계획, 현재 release 범위 밖
- Source workspace, release bundle, installed consumer harness의 물리·논리 분리
- Release private key 생성·읽기·저장·signing 없음
- Canonical state: tracked Markdown·YAML·TOML
- SQLite: 삭제·재생성 가능한 local index
- OMX/OMC namespace와 host-global configuration의 Hive 소유권 없음

## User plugin·project lifecycle review

현재 구현:

- Codex·Claude Code·Antigravity native plugin package와 user-scope install/update.
  Antigravity는 Hive-owned source package와 `agy`-owned staging·registry를 분리
- User guidance marker append·own-block replace·foreign byte 보존
- Project `.agents/directives`, portable `.agents/skills`, Claude `.claude/skills` adapter
- `setup-harness` expedited/custom sequence, project canonical knowledge와
  user-root 단일 SQLite·explicit promotion
- Historical exact base, unmodified replace, modified local-priority three-way merge
- Durable journal, executable-mode backup, host-state compensation과 crash recovery
- Source `hive-prompt-refine` projection, 현재 explicit refine-only routing,
  material ambiguity automatic refine-only 전환 계획

`0.8.0` npm 배포 gap:

- Interactive owner-aware `hive update`
- Linux x86_64·arm64 musl native build·install·runtime qualification
- `aigent-hive@0.8.0|latest` package family와 registry publication
- Unix·PowerShell·CMD npm-backed 직접 installer
- 5개 target SHA-256·GitHub artifact attestation·npm binary identity
- Exact product·npm `0.8.0` candidate qualification과 `latest` publication

Pre-1.0 비차단 deferred:

- 실제 Claude Code install/update E2E
- 실제 Claude Pro/Max quota usage parity
- Optional macOS·Windows publisher signing 실제 도입

실제 current-host evidence:

- Hive CLI: `~/.local/bin/hive`, `hive --version|-v|-V` 모두
  `hive 0.7.0 (released 2026-07-24)` 출력
- 기존 signed user harness에 현재 미서명 source를 덮어쓰는 setup preview는 ownership
  manifest 불일치로 exit `5`; 안전 경계 우회 0회
- Codex `0.145.0`: install→validate→update→validate PASS,
  `aigent-hive@aigent-hive` `0.7.0` enabled와 exact local source 확인
- Codex fresh ephemeral session: detail-poor ordinary request의 optional refine 제안,
  automatic rewrite 0회, safe read-only discovery 후 empty-workspace 중단
- Antigravity `agy 1.1.7`: support range `>=1.1.7 <1.2.0`, 기존
  directory-scan `0.7.0` migration dry-run→install→validate→repeat update→validate PASS
- Antigravity native discovery: `agy plugin list` import 등록 PASS, Hive source와
  host staging 16/16 exact path·byte parity, full-tree validation PASS,
  host staging의 Hive ledger ownership 0건
- Claude Code: executable·authenticated Pro/Max session 부재

결정:

- [`ADR-0009`](../decisions/ADR-0009-user-plugin-project-knowledge-boundary.md)
- Active implementation fragment:
  [`plugin-project-lifecycle.md`](../plans/active/plugin-project-lifecycle.md)

계획 evidence:

- Active fragment 8 KiB 제한 충족
- Canonical checklist `223/291`, active checklist ID 중복 0건
- Root English 159줄·Korean 155줄 README, 상호 language link와 빈 QA 표 PASS
- Phase 3 static contract 41/41, documentation style regression 18/18 PASS
- Human documentation inventory 1,285/1,285 review, finding 0건
- Markdown link conformance PASS

## Phase 7 shipping usage gate

### 구현

- Built-in `hive-usage-guard` source, template mirror, 세 host projection과 active Skill
  ledger
- Typed CLI: `hive usage enforce|status|threshold|session`
- 새 automatic dispatch 직전 one-shot `enforce`; 일반 응답·manual·non-dispatch 호출 없음
- Exit `0`은 session-bound preflight-only; 별도 automatic resume의
  `enforced=true`, `outcome=authorized`, authorization ID 1개·brief 1개만 dispatch 허용
- Current halt 우선, exit `3`은 해당 dispatch 차단, session disable은 authorization 아님
- Host-scoped session digest:
  `SHA-256(primary_host || NUL || exact_session_id)`
- Current process ID 결합, 다른 host·session·process의 override·marker replay 거부
- Explicit current-session disable 확인 필수; enable·toggle과 새 session default-enable
- Current valid halt marker를 sensor보다 먼저 확인하고 반복 호출에서 sensor 재사용 금지
- Account digest 생략 시 qualified sensor의 unique account만 허용; 0개·복수 fail-closed
- Quota pool별 provider-defined window 단독 적용; cadence window는 session 우선,
  session 부재 시 weekly fallback; 모든 pool 통과 필수
- Allowed: exit `0`, halt marker 없음
- Limited·unknown: sanitized marker의 optimistic atomic publication, exit `3`
- Marker 내용: host scope, session digest, PID, decision, window, threshold, measured time,
  evidence digest, revision
- Raw account, raw session ID, CodexBar payload 저장·출력 없음
- Installed `primary_host`와 pinned run·capability host 불일치 차단; Codex app-server,
  Claude opt-in status-line capture, Antigravity truthful native unsupported를 구분하고
  allowlisted unavailable·unsupported·malformed에서만 CodexBar fallback
- Fallback hook, prompt rewrite, Skill activation, watcher, subagent, orchestration,
  Stop continuation 설치·실행 없음
- OMX/OMC cancellation 결과: 보조 evidence only; halt marker나 durable goal/task 상태
  대체 불가

### Local qualification evidence

- Rust workspace 390/390:
  `hive-cli` 185, `hive-core` 62, `hive-projection` 22, `hive-render` 51,
  `hive-update` 63, `hive-wiki` 7
- Python conformance 524개 실행, 523 PASS, Windows `pwsh` 전용 1개 expected skip
- Native hostile sensor·fallback, Phase 3 projection과 설치본 usage guard source-target regression PASS
- 독립 Antigravity code·test review: PASS, actionable finding 0건

Pre-1.0 비차단 deferred:

- 실제 Claude app session E2E
- 실제 Claude Pro/Max qualified subscription usage sensor
- Optional macOS·Windows publisher signing 실제 도입

## Source 개발 usage safeguard

- 정본 결정: 설치된 `hive usage` 단일 사용
- 사용자 선택 global threshold: remaining `5%`
- 제거 완료: source 전용 Python gate·15초 background watcher·source 전용 scratch policy,
  매 tool 경계의 중복 gate
- 유지 경계: automatic dispatch 직전 session-bound one-shot enforce, native sensor 우선,
  명시적 session control, raw account·session identifier 저장 금지
- 구현 완료: 명시적 global threshold 저장과 registered project override 분리. `hive-source.json`
  source workspace만 설치 product global guard 사용. Project harness 없는 자체 `AGENTS.md`·빈 folder는
  guard 전체 비활성, halt·threshold mutation·session override·runtime file `0건`
- Windows 실제 수용: global `20% → 5%` 원자 갱신, source `status`·native `enforce`
  threshold `5%` PASS, source `.hive/` 생성 `0건`
- 회귀 증거: Rust usage control 10개·global projection binding 1개, Python 대상 분류·정적
  계약 39개 통과, Windows 조건부 1개 예상 건너뜀
- UX 경계: non-Hive guard 비활성과 setup-free Hive Skill 사용 가능 여부 분리. Project state
  workflow만 한 번의 활성화 승인과 자동 capability·run bootstrap 소유
- 전체 검증: Rust workspace·format·strict Clippy PASS. Python 적합성 602개 중 562개 PASS,
  Windows 조건부 40개 예상 건너뜀
- 현재 Windows 설치: `0.9.1-dev` build date `2026-08-12`, global threshold `5%`, user
  projection validate PASS. 공개 `0.9.1` native binary는 `.hive/dev-install/original`에 복구용 보존

## 사람용 문서 style

- Source directive: `.agents/directives/08-human-documentation-style.md`
- Consumer projection: `harness/template/AGENTS.md.jinja`, compiled renderer,
  `docs/guidance-schema.md`
- 대화 언어: 선택 언어로 질문·응답 전체 통일
- 한국어 대화: 고유명사·제품명·패키지명·명령어·코드 식별자·경로·스키마 키·정확한
  화면 문구·뚜렷한 한국어 대체어가 없는 용어만 영어 유지
- 영어 대화: 정확한 한국어 이름·문자열·인용문·사용자 보존 요청을 제외하고 영어로 통일
- 소비자 전역 지침: 한국어 선택 시 대체 가능한 일반 영어 단어의 한영 혼용 금지
- 한국어 설명문: 짧은 heading·bullet·table·checklist와 의미 중심 명사구 우선
- Declarative·conversational sentence-form과 기계적 nominalization 금지
- Exact bad/good 21쌍, authored callout·blockquote 적용, 비제한 규칙 명시
- Conversational imperative prompt sample은 path·line·reason·line digest allowlist로만 보존
- Exact external quote·UI prompt·protocol·fixture만 path·line·reason·line digest 예외
- Checker: `scripts/check-human-documentation-style.py`
- Regression: `tests/conformance/test_human_documentation_style.py`
- Independent semantic review PASS, residual finding 0건
- 최종 completion 조건: fresh inventory 전수 review, finding 0건, stale exception 0건,
  source/template/generated parity

## Phase 6 release·update truth

- Legacy release verifier·authorization path: 삭제 대기 `REL9-019–022`
- 유지: local bundle version·length·SHA-256, transactional backup·atomic activation·failure
  rollback·crash recovery, same-major migration
- Distribution trust: GitHub attestation 또는 npm registry integrity·OIDC provenance

### Version·migration

- Compiled historical surface와 signed cumulative inventory의 독립 release classification
- Feature: exact next minor; compatible fix: exact next patch
- Same-major breaking change: major `0`에서도 거부
- Major target 자동 추론 없음; exact user target과 별도 confirmation 필수
- Signed metadata가 선택 가능한 compiled route:
  `same-major-render-v1|cross-major-system-representation-v1`
- Downloaded script·DLL·dylib·WASM·argv migration 실행 금지
- Supported `0.1.0`–`0.6.0` generation의 same-major dry-run·apply corpus
- Cross-major protected project/docs/preferences/Markdown과 foreign marker byte 보존

### Backup·activation·recovery

- Verification·classification·route selection·dry-run 전 target mutation 0건
- Changed owned path와 canonical config/team/run/knowledge의 self-digested backup
- SQLite/WAL/SHM/journal, runtime, backup, `.omx/.omc` 제외
- Durable journal과 exact dry-run plan/tree 기반 atomic activation
- Before/after digest에서만 rollback 또는 forward completion
- Concurrent third digest 보존과 conflict
- Canonical text에서 SQLite rebuild
- Exact 7일 초과 unreferenced backup만 재검증 후 정리

### Local evidence

- SEC-001·SEC-003 current regression: `hive-cli` 166/166,
  `hive-render` 51/51, `hive-update` 63/63
- SEC-001·SEC-003 strict Clippy와 독립 재review: PASS
- Phase 6 static·CLI conformance: 8/8
- Phase 4 run lifecycle: Rust 10/10, Python 29/29
- Upgrade/migration fault injection: activation failure, concurrent user edit, forged recovery,
  cross-major preservation PASS

## Version parity

다음 표면의 `0.7.0` 동기화:

- Root Cargo workspace와 Cargo.lock의 Hive packages
- Compiled `hive --version`
- Release manifest, migration table과 signed surface fixture
- Copier/Rust installed `.hive/config/harness.toml`
- Harness release metadata, README, PLAN, CURRENT와 version lifecycle ADR

`0.6.0 → 0.7.0`: signed release/update, safe migration·backup·recovery와 release
packaging을 추가한 backward-compatible feature minor. Major 변경·추론 없음.

## 현재 검증 상태

Global onboarding·shared index local qualification PASS:

- Strict Clippy all targets·all features와 format check PASS
- Rust workspace 477/477
- Python conformance 576개 발견 중 575개 실행·통과. Windows가 아닌 환경에서만
  가능한 `pwsh` 전용 검사 1개는 현재 Windows 환경에서 미실행
- Shared index 동일 입력 재실행 byte-exact no-op, `changed_paths=[]`
- Codex·Antigravity expedited/custom connected onboarding matrix 4/4
- 독립 final blocker review의 critical·high·medium·low finding 0건

Source docs Wiki targeted qualification historical PASS:

- `hive-wiki` 33/33, Source Wiki conformance 재검증 PASS
- Canonical fact 74개, bilingual pair 37개
- `lint` finding·warning 0건, 영어·한국어 query PASS
- Index 삭제 뒤 query fail-closed exit `5`, logical digest·query equivalence rebuild PASS
- Ignored index·persistent lock의 Git 추적 0건
- 당시 Full Python conformance 565개 발견 중 528개 실행·통과, 37개 미실행.
  미실행 범위는 현재 Windows에서 권한 없이 만들 수 없는 symbolic link와
  POSIX·macOS 전용 동작. 해당 동작의 Windows 검증 완료 근거로 사용 금지

### macOS Apple Silicon local release qualification historical CLEAR

- Host: Apple M2, macOS 26.5.2, native `arm64`
- Tested source: `ba798d8`
- 상태: historical evidence. Current candidate
  `28f1c366aa06a609b443724decc474cb7718ea8a` 재검증 필요
- Locked `aarch64-apple-darwin` release build·version·Mach-O architecture PASS
- Release strict Clippy·workspace strict Clippy·format PASS
- Rust workspace 236/236
- Deterministic release archive 2회 byte-identical
- Binary SHA-256:
  `914b684da0c28da1914121ffc43a7331828a11ef13ef7b1159adc05fe445eda3`
- Archive SHA-256:
  `bde2c886c6d475b4a1a564ba0df33eaa9b6fb4a1b49ca49a7f2a896aa586a54b`
- Actual archive direct-install fixture, ownership receipt, repeat install PASS
- Installed binary setup dry-run·apply·validate PASS
- Phase 6 release/update 15 PASS, Windows-only `pwsh` 1 skip
- Phase 1 setup 31/31
- Protected 경계: Developer ID signing·notarization·GitHub attestation 미실행
- Local signature observation: linker ad-hoc, `TeamIdentifier` 없음, Gatekeeper 거부

Current remote qualification evidence:

- Current native source:
  `baff938b99967b4830eee79daa6c4477a607f427`
- Native release runtime:
  [run `30581894132`](https://github.com/gvm1229/aigent-hive/actions/runs/30581894132),
  macOS arm64·Intel, Linux musl x86_64·arm64, Windows x86_64 5/5 job PASS
- Linux 두 target: locked release build, ELF architecture·static linkage, package layout,
  archive digest·실행, isolated Antigravity install lifecycle PASS
- P7-040 current clean-clone gate 충족
- P7-043 Linux x86_64·arm64 musl qualification 충족

검증 경계:

- Local Phase 6 계약: Windows 적용 대상 21개 실행·통과. macOS 전용 8개는 현재
  Windows 환경에서 실행 불가하여 미실행이며 macOS 동작은 이 결과로 미검증
- Direct installer의 같은 owner parent handle-pinning race

## `0.8.0` npm 배포 완료

- P7-044 public npm package family와 native smoke 완료
- P7-045 npm-backed Unix·PowerShell·CMD installer와 digest·owner receipt 검증 완료
- P7-049 설치 소유자 기반 대화형 `hive update` 완료
- P7-020 5개 target archive·npm tarball provenance 완료
- P7-018 exact `0.8.0` release candidate qualification 완료
- P7-037 GitHub Release 없이 npm `0.8.0|latest` publication·clean install 완료

배포 증거:

- Current source·npm product version `0.8.0`
- 사용자 지정 순서: npm `0.8.0` 배포 성공 뒤 `develop` → `main` 병합
- Historical candidate authority: PR·필수 상태 검사·삭제·강제 push 차단이 적용된
  exact `develop`
- Historical GitHub ruleset: `0.8.0` candidate 당시 `develop` 보호 활성
- Current publication environment: `release-publication` 필수 검토자 `gvm1229` 설정
  확인, 자기 배포 승인 차단 비활성
- Historical branch policy: `codex/release-0.8.0` 임시 branch와 `develop` 대상 PR
  사용자 예외 승인
- First candidate run `30633581092`: exact `develop` commit `1031ff0`, 5개 native
  target·6개 npm tarball PASS
- First publication run `30634201469`: `release-publication` 승인 PASS,
  `docs/releases/0.8.0.md` 누락으로 npm 게시 전 실패, npm publish 실행 0건
- Corrective branch: 제품 후보 출시 안내, 명확한 게시 선행 검사 오류, Codex process
  교체 뒤 source watcher 복구와 회귀 시험
- `release.yml`: protected exact `420e244`, run `30657669889` PASS
- `release-publish.yml`: GitHub Release 0건과 6개 package `latest` publication 계약으로
  전환. 성공한 정확한 `develop` 후보만 허용. 최초 등록은 명시적
  `bootstrap_with_token=true`·임시 `NPM_TOKEN`, 이후는 OIDC 전용
- Direct installer: exact `0.8.0` unpkg bootstrap, scoped npm tarball digest,
  native 제품·receipt `0.8.0`, polluted `PSModulePath` CMD 회귀 PASS
- GitHub repository environment `release-publication` 1개, 최초 등록용
  `NPM_TOKEN` secret 1개
- Local npm: Node.js `24.13.1`, npm `11.17.0`, registry 사용자 `gvm1229`
- Public registry: umbrella·5개 scoped package exact·latest `0.8.0`, test
  `0.8.0-test.1`
- Local Windows npm baseline: actual `0.8.0` binary의 platform·umbrella pack,
  isolated global install과 digest
  `a8bdb5d7dd42965ec6f4d2f1f334a4ee4184a7f659f09cb92caf794b96524b0d`
  byte identity PASS
- Interactive activation: npm `test` 확인, npm·direct owner 인증, 선택 언어 prompt,
  명시적 수락 뒤 exact adapter 실행·owner와 package version 재검증 PASS

Pre-1.0 비차단 deferred:

- Optional macOS Developer ID signing·notarization
- Optional Windows Authenticode
- 실제 Claude subscription-backed install·usage parity
- GitHub normal release와 Git tag
- Exact `1.0.0` 사용자 지시 전 stable major preparation 금지

## 다음 action

1. V9-025 orchestration owner 전환과 capability inventory
2. KPX-001–007 portable bundle·collection schema·safe import
3. RAG-001–020 automatic retrieval·mandatory capture·freshness
4. KPX-008–018 directory scan·promotion·automatic query
5. V9-001–024 loop·Wiki·utility Skill·전체 qualification
