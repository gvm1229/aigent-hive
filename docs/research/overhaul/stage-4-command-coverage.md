# CLI 명령과 두 언어 출력 범위

- 실행 환경: Codex·Windows 11 10.0.26300. 사람용 기본값과 명시적 JSON의 별도 검사
- 아래 표: 공통 출력기의 등록 문구별 첫 문장 표본. 조회 값·파일·범위·다음 행동은 실제 결과에 따라 추가
- 기계 지원 범위 유지: doctor·check-target·버전의 기존 JSON 미지원 거부 유지. 내부 `__` 작업자와 `policy hook --stdin-json`의 명시적 기계 프로토콜은 원래 바이트 유지
- 공개 계열 실제 실행: 오류 계열 28개 × 2언어 × 기본/JSON, doctor·check-target, storage 성공, hook 비활성 기본/JSON 바이트, 버전 별칭 3개 × 2언어. 설치·변경 시험은 격리된 기존 회귀와 결합, 개인 설치 변경 없음

## 동작별 첫 문장

| action | 한국어 표본 | English sample |
| --- | --- | --- |
| `AnswerSimpleQuestion` | `완료. 간단한 질문 처리 완료.` | `Success. Question processed.` |
| `DescribeHiveSetupQuestion` | `완료. 설정 질문 확인 완료.` | `Success. Setup question inspected.` |
| `DescribeHiveSetupSection` | `완료. 설정 항목 확인 완료.` | `Success. Setup section inspected.` |
| `NotionKnowledge` | `완료. Notion 지식 요청 처리 완료.` | `Success. Notion knowledge request processed.` |
| `RoleWork` | `완료. 역할 작업 처리 완료.` | `Success. Role work processed.` |
| `RunWork` | `완료. 작업 기록 처리 완료.` | `Success. Work record processed.` |
| `ScanProjectUpgrade` | `완료. 프로젝트 갱신 대상 확인 완료.` | `Success. Project update targets inspected.` |
| `ValidateProjectUpgrade` | `완료. 프로젝트 갱신 확인 완료.` | `Success. Project update checked.` |
| `VectorKnowledge` | `완료. 의미 기반 지식 검색 처리 완료.` | `Success. Meaning-based knowledge search processed.` |
| `CheckRunClosure` | `완료. 작업 종료 조건 확인 완료.` | `Success. Work completion conditions checked.` |
| `CheckRunContinuation` | `완료. 작업 계속 조건 확인 완료.` | `Success. Work continuation conditions checked.` |
| `CheckpointRun` | `완료. 작업 진행 저장 완료.` | `Success. Work progress saved.` |
| `ProbeNativeUsage` | `완료. 앱의 사용량 조회 처리 완료.` | `Success. Host usage inspected.` |
| `RecoverHiveUser` | `완료. 사용자 설정 복구 처리 완료.` | `Success. User setup recovery processed.` |
| `RecoverProjectUpgrade` | `완료. 프로젝트 갱신 복구 처리 완료.` | `Success. Project update recovery processed.` |
| `RefinePrompt` | `완료. 요청 다듬기 처리 완료.` | `Success. Request refinement processed.` |
| `ResumeWork` | `완료. 저장된 작업 재개 처리 완료.` | `Success. Saved work resume processed.` |
| `UnknownAction` | `처리 실패: 명령 처리. 사용할 수 없는 명령이나 옵션입니다. --help에서 사용할 이름을 확인하세요.` | `Could not complete: Command processing. Unknown top-level action: example.` |
| `ValidateHiveUser` | `완료. 사용자 설치 확인 완료.` | `Success. User installation checked.` |
| `InspectInstallation` | `완료. Hive 설치 상태 확인 완료.` | `Success. Hive installation inspected.` |
| `CheckTarget` | `완료. 대상 폴더 확인 완료.` | `Success. Target folder checked.` |
| `SetupHarness` | `완료. 프로젝트 설정 처리 완료.` | `Success. Project setup processed.` |
| `SetupHiveUser` | `완료. 사용자 설정 처리 완료.` | `Success. User setup processed.` |
| `DescribeHiveUserSetup` | `완료. 설정 질문과 선택지 확인 완료.` | `Success. Setup choices inspected.` |
| `ManageHiveUserFeature` | `완료. 선택 기능 설정 처리 완료.` | `Success. Optional feature settings processed.` |
| `InstallHiveUser` | `완료. 사용자 환경 설치 처리 완료.` | `Success. User installation processed.` |
| `UninstallHiveUser` | `완료. Hive 제거 처리 완료.` | `Success. Hive removal processed.` |
| `UpdateHiveUser` | `완료. 사용자 환경 갱신 처리 완료.` | `Success. User update processed.` |
| `UpdateHarness` | `완료. Hive 갱신 처리 완료.` | `Success. Hive update processed.` |
| `UpdateProjectHarness` | `완료. 프로젝트 갱신 처리 완료.` | `Success. Project update processed.` |
| `CheckHiveUpdate` | `완료. 새 버전 확인 완료.` | `Success. New version checked.` |
| `VerifyRelease` | `완료. 배포 파일 확인 완료.` | `Success. Release files checked.` |
| `RouteAgent` | `완료. 요청에 맞는 작업 경로 선택 완료.` | `Success. Work route selected.` |
| `ValidatePrompt` | `완료. 다듬은 요청 확인 완료.` | `Success. Refined request checked.` |
| `ApprovePrompt` | `완료. 다듬은 요청 승인 처리 완료.` | `Success. Refined request approval processed.` |
| `EvaluateHook` | `완료. 연결된 작업의 허용 여부 확인 완료.` | `Success. Hook request checked.` |
| `EvaluatePolicy` | `완료. 안전 규칙 확인 완료.` | `Success. Safety policy checked.` |
| `ConfigurePolicyHooks` | `완료. 안전 규칙 연결 설정 처리 완료.` | `Success. Safety hook settings processed.` |
| `ReviewPolicyCandidate` | `완료. 안전 규칙 검토 처리 완료.` | `Success. Policy review processed.` |
| `CheckUsage` | `완료. 사용량 보호 확인 완료.` | `Success. Usage protection checked.` |
| `ShowUsageStatus` | `완료. 사용량과 보호 상태 확인 완료.` | `Success. Usage and protection inspected.` |
| `CaptureUsage` | `완료. 사용량 관측 처리 완료.` | `Success. Usage observation processed.` |
| `SetUsageThreshold` | `변경 완료. 사용량 보호 기준을 30%에서 10%로 바꿨습니다.` | `Success. Hive usage guard threshold lowered from 30% to 10%.` |
| `ControlUsageSession` | `완료. 현재 세션의 사용량 보호 설정 처리 완료.` | `Success. Session usage protection processed.` |
| `ManageUsageMonitor` | `완료. 상시 사용량 감시 설정 확인 완료.` | `Success. Background usage monitoring inspected.` |
| `InstallUsageFallback` | `완료. 대체 사용량 확인 도구 설치 처리 완료.` | `Success. Fallback usage tool installation processed.` |
| `SummarizeTokens` | `완료. 토큰 사용량 합산 완료.` | `Success. Token usage summarized.` |
| `ReportStorage` | `완료. 저장 공간 사용량 확인 완료.` | `Success. Storage use inspected.` |
| `PruneStorage` | `완료. 정리 대상 처리 완료.` | `Success. Storage cleanup processed.` |
| `CoordinateSession` | `완료. 파일 수정 예약 처리 완료.` | `Success. File edit reservation processed.` |
| `BeginSession` | `완료. 세션 시작과 파일 예약 처리 완료.` | `Success. Session and file reservation started.` |
| `CheckSession` | `완료. 세션과 파일 예약 확인 완료.` | `Success. Session reservations checked.` |
| `UpdateSession` | `완료. 세션 파일 예약 갱신 완료.` | `Success. Session reservations updated.` |
| `CloseSession` | `완료. 세션 파일 예약 해제 완료.` | `Success. Session reservations closed.` |
| `RecoverSession` | `완료. 끝난 세션의 파일 예약 복구 완료.` | `Success. Ended session reservations recovered.` |
| `ValidateRole` | `완료. 역할 설정 확인 완료.` | `Success. Role settings checked.` |
| `RecordRoleHandoff` | `완료. 역할 인계 기록 완료.` | `Success. Role handoff recorded.` |
| `VerifyWork` | `완료. 작업 결과 확인 완료.` | `Success. Work result checked.` |
| `InitializeLoop` | `완료. 반복 작업 초기화 완료.` | `Success. Repeated work initialized.` |
| `ValidateLoop` | `완료. 반복 작업 상태 확인 완료.` | `Success. Repeated work state checked.` |
| `CheckpointLoop` | `완료. 반복 작업 진행 저장 완료.` | `Success. Repeated work progress saved.` |
| `SteerLoop` | `완료. 반복 작업 방향 변경 처리 완료.` | `Success. Repeated work direction processed.` |
| `PrepareLoopDispatch` | `완료. 반복 작업의 전달 준비 완료.` | `Success. Repeated work delivery prepared.` |
| `RecoverLoop` | `완료. 반복 작업 복구 처리 완료.` | `Success. Repeated work recovery processed.` |
| `OrchestrationStatus` | `완료. 작업 조율 상태 확인 완료.` | `Success. Coordination state inspected.` |
| `OrchestrationPlan` | `완료. 작업 조율 계획 처리 완료.` | `Success. Coordination plan processed.` |
| `OrchestrationDispatch` | `완료. 작업 전달 준비 처리 완료.` | `Success. Work delivery preparation processed.` |
| `OrchestrationReceipt` | `완료. 작업 수신 기록 처리 완료.` | `Success. Work receipt processed.` |
| `OrchestrationCancel` | `완료. 작업 취소 처리 완료.` | `Success. Work cancellation processed.` |
| `OrchestrationRecover` | `완료. 작업 조율 복구 처리 완료.` | `Success. Coordination recovery processed.` |
| `OrchestrationAuthority` | `완료. 작업 승인 정보 처리 완료.` | `Success. Work approval processed.` |
| `OrchestrationAuthorityIssue` | `완료. 작업 승인 발급 처리 완료.` | `Success. Work approval issued.` |
| `OrchestrationAuthorityRevoke` | `완료. 작업 승인 철회 처리 완료.` | `Success. Work approval revoked.` |
| `OrchestrationMigrate` | `완료. 작업 기록 이전 처리 완료.` | `Success. Work records migrated.` |
| `PackageJudge` | `완료. 독립 검토 자료 준비 완료.` | `Success. Independent review material prepared.` |
| `RecordJudgeReceipt` | `완료. 독립 검토 결과 기록 완료.` | `Success. Independent review result recorded.` |
| `EvaluateJudgeQuorum` | `완료. 독립 검토의 동의 수 확인 완료.` | `Success. Independent review agreement checked.` |
| `KoreanLanguage` | `완료. 한국어 문장 처리 완료.` | `Success. Korean text processed.` |
| `InspectKorean` | `완료. 한국어 문장 검사 완료.` | `Success. Korean text inspected.` |
| `VerifyKorean` | `완료. 문장 의미와 보존 조건 확인 완료.` | `Success. Text preservation checked.` |
| `SanitizeKorean` | `완료. 한국어 문장 정리 완료.` | `Success. Korean text cleaned.` |
| `PreviewKoreanPack` | `완료. 한국어 규칙 미리보기 완료.` | `Success. Korean rules previewed.` |
| `ActivateKoreanPack` | `완료. 한국어 규칙 적용 완료.` | `Success. Korean rules activated.` |
| `CheckKoreanPack` | `완료. 한국어 규칙 확인 완료.` | `Success. Korean rules checked.` |
| `StatusKoreanPack` | `완료. 한국어 규칙 상태 확인 완료.` | `Success. Korean rules status inspected.` |
| `RollbackKoreanPack` | `완료. 한국어 규칙 되돌림 완료.` | `Success. Korean rules rolled back.` |
| `InspectDiscordInbound` | `완료. Discord 요청 수신 설정 확인 완료.` | `Success. Discord inbound settings inspected.` |
| `TestDiscordWebhook` | `완료. Discord 알림 시험 처리 완료.` | `Success. Discord notification test processed.` |
| `DeveloperReport` | `완료. 문제 보고 자료 처리 완료.` | `Success. Problem report processed.` |
| `PreviewDeveloperReport` | `완료. 문제 보고 자료 미리보기 완료.` | `Success. Problem report previewed.` |
| `CollectDeveloperReport` | `완료. 문제 보고 자료 수집 완료.` | `Success. Problem report collected.` |
| `ExportDeveloperReport` | `완료. 문제 보고 자료 내보내기 완료.` | `Success. Problem report exported.` |
| `RecommendAgent` | `완료. 작업자 추천 완료.` | `Success. Worker recommendation prepared.` |
| `CreateAgent` | `완료. 맞춤 작업자 생성 완료.` | `Success. Custom worker created.` |
| `PreviewAgent` | `완료. 맞춤 작업자 미리보기 완료.` | `Success. Custom worker previewed.` |
| `ValidateAgent` | `완료. 맞춤 작업자 확인 완료.` | `Success. Custom worker checked.` |
| `PreflightAgent` | `완료. 맞춤 작업자 실행 전 확인 완료.` | `Success. Custom worker preflight checked.` |
| `ApplyAgent` | `완료. 맞춤 작업자 적용 완료.` | `Success. Custom worker applied.` |
| `AttestAgent` | `완료. 맞춤 작업자 증명 확인 완료.` | `Success. Custom worker evidence checked.` |
| `DisableAgent` | `완료. 맞춤 작업자 비활성 처리 완료.` | `Success. Custom worker disabled.` |
| `RemoveAgent` | `완료. 맞춤 작업자 제거 처리 완료.` | `Success. Custom worker removed.` |
| `Summon` | `완료. 역할별 작업 준비 완료, 자동 실행 없음.` | `Success. Role work prepared; automatic execution remains disabled.` |
| `roster` | `완료. 역할별 모델과 설정 확인 완료.` | `Success. Role models and settings inspected.` |
| `summon` | `완료. 역할별 작업 준비 완료, 자동 실행 없음.` | `Success. Role work prepared; automatic execution remains disabled.` |
| `AddKnowledge` | `완료. 지식 추가 처리 완료.` | `Success. Knowledge add processed.` |
| `AuthorizeConfidentialKnowledge` | `완료. 지식 기밀 사용 승인 처리 완료.` | `Success. Knowledge authorizeconfidential processed.` |
| `DeleteKnowledge` | `완료. 지식 삭제 처리 완료.` | `Success. Knowledge delete processed.` |
| `ExportKnowledge` | `완료. 지식 내보내기 처리 완료.` | `Success. Knowledge export processed.` |
| `ImportKnowledge` | `완료. 지식 가져오기 처리 완료.` | `Success. Knowledge import processed.` |
| `IngestKnowledge` | `완료. 지식 원문 반영 처리 완료.` | `Success. Knowledge ingest processed.` |
| `LintKnowledge` | `완료. 지식 검사 처리 완료.` | `Success. Knowledge lint processed.` |
| `ListKnowledge` | `완료. 지식 목록 조회 처리 완료.` | `Success. Knowledge list processed.` |
| `MapKnowledge` | `완료. 지식 연결 처리 완료.` | `Success. Knowledge map processed.` |
| `PromoteKnowledge` | `완료. 지식 공유 처리 완료.` | `Success. Knowledge promote processed.` |
| `QueryKnowledge` | `완료. 지식 검색 처리 완료.` | `Success. Knowledge query processed.` |
| `ReadKnowledge` | `완료. 지식 읽기 처리 완료.` | `Success. Knowledge read processed.` |
| `RebuildKnowledge` | `완료. 지식 색인 재생성 처리 완료.` | `Success. Knowledge rebuild processed.` |
| `RefreshKnowledge` | `완료. 지식 갱신 처리 완료.` | `Success. Knowledge refresh processed.` |
| `RememberKnowledge` | `완료. 지식 기록 처리 완료.` | `Success. Knowledge remember processed.` |
| `RetrieveKnowledge` | `완료. 지식 관련 자료 검색 처리 완료.` | `Success. Knowledge retrieve processed.` |
| `ScanKnowledge` | `완료. 지식 폴더 조사 처리 완료.` | `Success. Knowledge scan processed.` |
| `SuppressKnowledge` | `완료. 지식 검색 제외 처리 완료.` | `Success. Knowledge suppress processed.` |
| `TransferKnowledge` | `완료. 지식 이전 처리 완료.` | `Success. Knowledge transfer processed.` |
| `SyncNotionKnowledge` | `완료. 지식 Notion 동기화 처리 완료.` | `Success. Knowledge syncnotion processed.` |
| `RetrieveNotionKnowledge` | `완료. 지식 Notion 검색 처리 완료.` | `Success. Knowledge retrievenotion processed.` |
| `WriteThroughNotionKnowledge` | `완료. 지식 Notion 기록 처리 완료.` | `Success. Knowledge writethroughnotion processed.` |
| `MapKnowledgeCollection` | `완료. 지식 모음 연결 처리 완료.` | `Success. Knowledge collection mapped.` |
| `RebuildKnowledgeIndex` | `완료. 지식 검색 색인 재생성 완료.` | `Success. Knowledge search index rebuilt.` |
| `LintSourceWiki` | `완료. 소스 위키 검사 완료.` | `Success. Source Wiki checked.` |
| `QuerySourceWiki` | `완료. 소스 위키 검색 완료.` | `Success. Source Wiki searched.` |
| `RebuildSourceWikiIndex` | `완료. 소스 위키 색인 재생성 완료.` | `Success. Source Wiki index rebuilt.` |
| `InspectUserSetupProgress` | `완료. 사용자 설정 진행 확인 완료.` | `Success. User setup progress inspect processed.` |
| `SaveUserSetupProgress` | `완료. 사용자 설정 진행 저장 완료.` | `Success. User setup progress save processed.` |
| `ClearUserSetupProgress` | `완료. 사용자 설정 진행 초기화 완료.` | `Success. User setup progress clear processed.` |
| `InspectProjectSetupProgress` | `완료. 프로젝트 설정 진행 확인 완료.` | `Success. Project setup progress inspect processed.` |
| `SaveProjectSetupProgress` | `완료. 프로젝트 설정 진행 저장 완료.` | `Success. Project setup progress save processed.` |
| `ClearProjectSetupProgress` | `완료. 프로젝트 설정 진행 초기화 완료.` | `Success. Project setup progress clear processed.` |

## 전체 도움말의 명령 형식

- 공개 하위 명령·동작 인자의 조사 기준: 각 처리기의 USAGE·HELP 상수와 개별 정책 도움말. 신 이름 9개·한글·별칭은 정본 표의 소유 명령으로 전달, 별도 결과 계약 없음
- 도움말·버전은 요청한 설명·메타데이터, 내부 작업자는 프로그램 간 응답. 사람이 직접 쓰는 ActionResult 결과의 기본 JSON과 구분

### crates/hive-cli/src/custom_agent_cli.rs · USAGE

```text
Preview, recommend, create, apply, validate, attest, route, or remove one consented Hive custom-agent profile.

USAGE:
hive agent preview --profile <profile.json> --root <dir> [--output json]
hive agent recommend --purpose <text> --scope <user|project> --catalog <catalog.json> --catalog-attestation <attestation.json> --trust-root <trust-root.toml> [--previous-request <creation.json>] [--output json]
hive agent create --request <creation.json> --root <dir> --accept-decision-digest <sha256> [--prior-decision <creation.json>] [--output json]
hive agent apply --profile <profile.json> --root <dir> --accept-definition-digest <sha256> [--output json]
hive agent validate --profile <profile.json> --root <dir> [--output json]
hive agent preflight --profile <profile.json> --host <codex|claude> --capabilities <capability.json> [--output json]
hive agent attest --profile <profile.json> --host <codex|claude> --receipt <attestation.json> [--output json]
hive agent route --user-root <dir> --project-root <dir> --request <text> [--output json]
hive agent disable --role <role-id> --scope <user|project> --root <dir> --accept-definition-digest <sha256> [--output json]
hive agent remove --role <role-id> --scope <user|project> --root <dir> --accept-definition-digest <sha256> [--output json]
```

### crates/hive-cli/src/discord.rs · DISCORD_USAGE

```text
Inspect the host-owned Discord inbound continuation boundary.

USAGE:
    hive discord inbound --host codex|claude|antigravity [--output json]
    hive discord test --webhook-env <ENVIRONMENT_NAME> [--language en|ko] [--fields <field,...>] [--output json]

Discord notification delivery remains outbound-only. Claude inbound handling is
delegated to the official Claude Discord Channel plugin. Codex continuation is
unsupported until an official compatible capability is verified.
```

### crates/hive-cli/src/judge.rs · PACKAGE_USAGE

```text
Build one digest-bound clean-context judge package without mutation.

USAGE:
    hive judge package --target <dir> --request <request.json> [--output json]
```

### crates/hive-cli/src/judge.rs · QUORUM_USAGE

```text
Aggregate independent final judge verdicts without mutation.

USAGE:
    hive judge quorum --target <dir> --request <request.json> --trust-root <external-protected.toml> [--output json]
```

### crates/hive-cli/src/judge.rs · RECEIPT_USAGE

```text
Validate one host-owned adversarial Judge launch and result without mutation.

USAGE:
    hive judge receipt --target <dir> --request <receipt.json> --package <package.json> --assignment <assignment.json> [--output json]
```

### crates/hive-cli/src/knowledge.rs · KNOWLEDGE_USAGE

```text
Canonical Markdown knowledge and disposable SQLite index.

USAGE:
    hive knowledge ingest --target <dir> --source <file> --wiki <draft.md> [--user-root <dir>] [--output json]
    hive knowledge add --target <dir> --source <file> --wiki <draft.md> [--quick] [--user-root <dir>] [--output json]
    hive knowledge query --target <dir> (--text <query>|--tag <tag>|--category <category>) [--limit <1..100>] [--user-root <dir>] [--output json]
    hive knowledge list --target <dir> [--tag <tag>] [--category <category>] [--limit <1..100>] [--user-root <dir>] [--output json]
    hive knowledge read --target <dir> --page-id <id> [--user-root <dir>] [--output json]
    hive knowledge promote --target <project> --user-root <dir> --page-id <id> --category fact|preference|workflow (--dry-run|--apply) [--output json]
    hive knowledge promote --user-root <dir> --collection <id-or-alias> [--review-id <id>] --dry-run [--output json]
    hive knowledge promote --user-root <dir> --collection <id-or-alias> --review-id <id> --expected-source-digest <sha256:...> --confirm-global-promotion --apply [--output json]
    hive knowledge lint --target <dir> [--user-root <dir>] [--output json]
    hive knowledge delete --target <dir> --page-id <id> --reason <text> [--replacement <locator>] --timestamp <RFC3339> [--user-root <dir>] [--output json]
    hive knowledge suppress --target <dir> --fingerprint <sha256:...> --source-locator <locator> --reason <text> [--replacement <locator>] --timestamp <RFC3339> [--user-root <dir>] [--output json]
    hive knowledge remember --user-root <dir> (--request <request.json>|--user-statement <normalized-fact> --claim-key <stable-key> [--kind project-profile|decision|convention|preference|workflow]) [--output json]
    hive knowledge retrieve --user-root <dir> --target <current-dir> (--request <request.json>|--query <text> [--scope <scope>] [--top-k <1..100>] [--byte-budget <bytes>]) [--mode fts|semantic] [--authorization-id <id> --authorization-token <token> --capabilities <json> --usage <json>] [--output json]
    hive knowledge authorize-confidential --user-root <dir> --target <current-dir> --collection <id-or-alias> --query <text> --capabilities <json> --usage <json> --expires-at <unix-seconds> --nonce <nonce> --confirm-current-action [--output json]
    hive knowledge authorize-collection --user-root <dir> --operation attach|map|detach --collection <id-or-alias> [--target <dir>] --expires-at <unix-seconds> --nonce <nonce> --confirm-current-action [--output json]
    hive knowledge collection attach|map --user-root <dir> --collection <id-or-alias> --target <dir> --authorization-id <id> --authorization-token <token> [--output json]
    hive knowledge collection detach --user-root <dir> --collection <id-or-alias> --authorization-id <id> --authorization-token <token> [--output json]
    hive knowledge scan --target <dir> (--inventory|--candidates <review.json>|--apply <review.json>) [--include-untracked] [--prior-inventory <json>] [--user-root <dir>] [--output json]
    hive knowledge export --user-root <dir> --scope global|shared|project:<id>|collection:<id>|all-portable --bundle <path>.hivekb [--replace-backup <file-name>] [--output json]
    hive knowledge import --user-root <dir> --bundle <path>.hivekb (--dry-run|--apply) [--output json]
    hive knowledge transfer export --preview|--apply --user-root <dir> --scope global|shared|project:<id>|collection:<id>|all-portable --bundle <path>.hivekb [--replace-backup <file-name>] [--output json]
    hive knowledge transfer import --preview|--apply [--exclude-conflicts] [--preview-digest <sha256:...> --expected-sha256 <sha256:...>] --user-root <dir> --bundle <path>.hivekb [--output json]
    hive knowledge transfer merge preview --bundle <path>.hivekb --bundle <path>.hivekb [--user-root <dir>] [--output json]
    hive knowledge transfer merge review|apply --bundle <path>.hivekb --bundle <path>.hivekb --preview-digest <sha256:...> --review <review.json> [--review-digest <sha256:...>] [--user-root <dir>] [--output json]
    hive knowledge transfer status --id <transfer-id> [--user-root <dir>] [--output json]
    hive knowledge transfer vector --id <transfer-id> --receipt-digest <sha256:...> --answer yes|no|cancel [--user-root <dir>] [--output json]
    hive knowledge refresh (--target <legacy-project>|--user-root <dir>) [--output json]
    hive knowledge graph preview|enable|status|rebuild|disable|query|export --target <dir> [--scope project] [--engine native-markdown|graphify-code] [--consent-digest <sha256:...>] [--input <graph.json> --receipt <receipt.json>] [--node-id <id>] [--text <query>] [--user-root <dir>] [--format json|html] [--output json]
    hive knowledge graph preview|enable|disable|status|prepare|apply|query --engine host-semantic --target <dir> --user-root <dir> --collection <id> --visibility shared|project-private|confidential --host codex|antigravity [--consent-digest <digest>] [--request-digest <digest> --input <result.json> --receipt <receipt.json>] [--node-id <id>] [--output json]
    hive knowledge vector --help
    hive index rebuild (--target <legacy-project>|--user-root <dir>) [--output json]
```

### crates/hive-cli/src/knowledge.rs · NOTION_KNOWLEDGE_USAGE

```text
hive knowledge notion sync --user-root <dir> --capability <receipt.json> --snapshot <complete-inventory.json> [--output json]
    hive knowledge notion rebuild --user-root <dir> --capability <receipt.json> --snapshot <complete-inventory.json> [--output json]
    hive knowledge notion retrieve --user-root <dir> --capability <receipt.json> --snapshot <complete-inventory.json> (--request <request.json>|--query <text> [--scope global] [--top-k <1..100>] [--byte-budget <bytes>]) [--output json]
    hive knowledge notion write-through --user-root <dir> --capability <receipt.json> --snapshot <complete-inventory.json> --write-receipt <confirmed-write.json> [--output json]
```

### crates/hive-cli/src/korean.rs · USAGE

```text
Inspect and verify Korean text without calling a model provider.

USAGE:
    hive korean inspect --profile response|release-note|documentation|technical|verbatim --input <file> [--target <consumer>] [--output json]
    hive korean verify --profile response|release-note|documentation|technical|verbatim --before <file> --after <file> [--target <consumer>] [--output json]
    hive korean sanitize --input <file> --output-file <file> [--output json]
    hive korean pack check [--output json]
    hive korean pack status --target <consumer> [--output json]
    hive korean pack preview --target <consumer> --candidate <pack-dir> [--output json]
    hive korean pack activate --target <consumer> --candidate <pack-dir> --consent-digest <sha256:...> --confirm-pack [--output json]
    hive korean pack rollback --target <consumer> [--output json]
```

### crates/hive-cli/src/loop_engineering.rs · LOOP_USAGE

```text
Manage a durable prepare-only loop graph.

USAGE:
    hive loop initialize --target <dir> --graph <graph.md> [--output json]
    hive loop validate --target <dir> --run <run-id> [--output json]
    hive loop checkpoint --target <dir> --request <request.json> [--output json]
    hive loop steer --target <dir> --request <request.json> [--output json]
    hive loop prepare --target <dir> --request <request.json> [--output json]
    hive loop recover --target <dir> --run <run-id> [--output json]
```

### crates/hive-cli/src/main.rs · USAGE

```text
Aigent Hive

OUTPUT:
    Readable results use your saved language. Add --output json for machine output.

USAGE:
    hive doctor
    hive roster --help
    hive summon --help
    hive <god> <command> [options]
    hive install --scope user (--host codex|claude|antigravity)...|--hosts <comma-separated-hosts> (--dry-run|--apply|--validate) [--user-root <dir>] [--output json]
    hive uninstall [--user-root <dir>] [--dry-run | --confirm] [--output json]
    hive check-target <path>
    hive setup --help
    hive setup --target <dir> [--user-root <dir>] [--interactive|--non-interactive] [--resume|--dry-run]
    hive setup --target <dir> [--user-root <dir>] (--show|--describe --section <id>) [--output json]
    hive setup --target <dir> --user-root <dir> --show --summary [--output json]
    hive setup --target <dir> --user-root <dir> --describe --question <question-id> [--output json]
    hive setup --target <dir> --answers <yml> --capabilities <json> --user-root <dir> (--dry-run|--apply|--validate) [--reconfigure-role <role-id>]... [--input-channel answers-file|agent] [--output json]
    hive source-wiki lint --target <source-root> [--output json]
    hive source-wiki index --target <source-root> [--output json]
    hive source-wiki query --target <source-root> --language en|ko (--text <query>|--tag <tag>) [--limit <1..100>] [--output json]
    hive source-wiki graph preview|enable|status|rebuild|disable|query|export --target <source-root> [--engine native-markdown|graphify-code] [--node-id <id>] [--text <query>] [--format json|html] [--output json]
    hive source-wiki graph preview|enable|disable|status|prepare|apply|query --engine host-semantic --target <dir> --language en|ko --host codex|antigravity [--consent-digest <digest>] [--request-digest <digest> --input <result.json> --receipt <receipt.json>] [--node-id <id>] [--output json]
    hive update [--channel stable|test] [--user-root <absolute-dir>] [--confirm] [--interactive|--non-interactive] [--output json]
    hive update --check --user-root <absolute-dir> [--output json]
    hive knowledge add|authorize-confidential|collection|delete|export|import|ingest|lint|list|promote|query|read|refresh|remember|retrieve|scan|suppress --help
    hive discord inbound --host codex|claude|antigravity [--output json]
    hive report preview|collect|export --help
    hive project upgrade --target <dir> (--scan|--dry-run|--apply|--validate|--recover) [--skill-merge-inputs <skill>|--skill-merges <json> --approve-skill-merge <digest>] [--output json]
    hive session begin|check|update --target <dir> --host codex|claude|antigravity --session-id <id> --process-id <positive-u32> --path <project-relative-path> [--path <project-relative-path>]... [--output json]
    hive session close --target <dir> --host codex|claude|antigravity --session-id <id> [--output json]
    hive session recover --target <dir> [--output json]
    hive index rebuild --target <dir> [--output json]
    hive route --request <json> [--output json]
    hive prompt validate --request <input.json> --result <result.json> [--output json]
    hive prompt approve --request <input.json> --result <result.json> --digest <sha256:...> --target-host codex|claude|antigravity --confirm-refined-prompt [--output json]
    hive hook --capability <name> --event <event> [--capabilities <fresh-json>] [--input <json>] [--output json]
    hive policy evaluate --input <json> [--output json]
    hive usage check --account-digest <sha256:...> [--threshold <0..99>] [--output json]
    hive usage probe-native --host codex|claude|antigravity [--output json]
    hive usage monitor preview|enable|disable|status --user-root <dir> [--output json]
    hive storage [--user-root <dir>] [--output json]
    hive prune --dry-run|--confirm [--user-root <dir>] [--output json]
    hive usage enforce --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--account-digest <sha256:...>] [--user-root <dir>] [--output json]
    hive usage status --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--user-root <dir>] [--output json]
    hive usage threshold (--target <configured-project>|--user-root <user-root>) --remaining-percent <0..99> [--output json]
    hive usage session --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--user-root <dir>] --action enable|disable|toggle [--confirm-session-disable] [--output json]
    hive usage capture --host claude|antigravity (--target <dir>|--target-from-stdin) --stdin-json [--output json]
    hive usage summarize-tokens --request <normalized-events.json> [--output json]
    hive usage fallback-install --host codex|claude|antigravity (--dry-run|--apply) [--confirm-install] [--output json]
    hive role validate --target <dir> --role <role-id> [--output json]
    hive role handoff --target <dir> --request <request.json> [--output json]
    hive run checkpoint --target <dir> --request <request.json> --capabilities <fresh-json> [--output json]
    hive run resume --target <dir> --run <run-id> --capabilities <fresh-json> [--dispatch-intent manual|automatic] [--account-digest <sha256:...>] [--session-id <host-session-id> --process-id <positive-u32>] [--user-root <dir>] [--role <role-id> [--threshold <0..99>]] [--output json]
    hive run closure --target <dir> --run <run-id> [--output json]
    hive run continuation --target <dir> --run <run-id> --session-id <host-session-id> [--output json]
    hive loop initialize|validate|checkpoint|steer|prepare|recover --help
    hive orchestration status|plan|dispatch|receipt|cancel|recover|authority|migrate --help
    hive judge package --target <dir> --request <json> [--output json]
    hive judge receipt --target <dir> --request <json> --package <json> --assignment <json> [--output json]
    hive judge quorum --target <dir> --request <json> [--output json]
    hive korean inspect|verify|sanitize|pack --help
    hive release verify --bundle <release-dir> [--output json]
    hive update --help
```

### crates/hive-cli/src/main.rs · SETUP_USAGE

```text
Create or validate an installed Aigent Hive consumer harness.

USAGE:
    hive setup --target <dir> [--user-root <dir>] [--interactive|--non-interactive] [--resume|--dry-run]
    hive setup --target <dir> [--user-root <dir>] (--show|--describe --section <id>) [--output json]
    hive setup --target <dir> --answers <yml> --capabilities <json> --user-root <dir> (--dry-run|--apply|--validate) [--reconfigure-role <role-id>]... [--output json]

MODES:
    --summary    Compact status/completion output; not an approval preview
    --dry-run    Render and validate in staging without changing the target
    --apply      Render, validate, and apply only manifest-owned changes
    --validate   Validate the installed harness without changing the target
```

### crates/hive-cli/src/orchestration.rs · ORCHESTRATION_USAGE

```text
Manage provider-neutral durable orchestration state without spawning a host process.

USAGE:
    hive orchestration status --target <dir> --run <id> [--output json]
    hive orchestration plan|dispatch|cancel|recover --target <dir> --run <id> --event <event.toml|json> --expected-head <none|sequence:sha256> --control-epoch <n> --authority <authority.toml|json> --trust-root <external-root.toml|json> --request-digest <sha256> --now <ordered-time> [--output json]
    hive orchestration receipt --target <dir> --run <id> --event <event.toml|json> --receipt <receipt.json> --expected-head <none|sequence:sha256> --control-epoch <n> --authority <authority.toml|json> --trust-root <external-root.toml|json> --request-digest <sha256> --now <ordered-time> [--output json]
    hive orchestration authority issue|revoke --help
    hive orchestration migrate --help
```

### crates/hive-cli/src/report.rs · REPORT_USAGE

```text
Create a local, sanitized report for an Aigent Hive developer.

USAGE:
    hive report preview --target <dir> [--output json]
    hive report collect --target <dir> --issue-code <safe-id> [--summary <manual-summary>] [--output json]
    hive report export --target <dir> --report <report-id> --destination <absolute-new-file> [--output json]

Reports exclude raw prompts and transcripts, redact likely credentials in an
optional manual summary, and never upload automatically.
```

### crates/hive-cli/src/role.rs · VALIDATE_USAGE

```text
Validate one persistent role without mutation.

USAGE:
    hive role validate --target <dir> --role <role-id> [--output json]
```

### crates/hive-cli/src/role.rs · HANDOFF_USAGE

```text
Record one explicit optimistic role handoff transaction.

USAGE:
    hive role handoff --target <dir> --request <request.json> [--output json]
```

### crates/hive-cli/src/roster.rs · USAGE

```text
hive roster [check --role <name> --capabilities <host-report.json>] --target <dir> --user-root <home> [--output json]
```

### crates/hive-cli/src/run.rs · CHECKPOINT_USAGE

```text
Record one optimistic durable run checkpoint.

USAGE:
    hive run checkpoint --target <dir> --request <request.json> --capabilities <fresh-json> [--output json]
```

### crates/hive-cli/src/run.rs · RESUME_USAGE

```text
Inspect a durable run, or prepare one session-bound automatic authorization without spawning.

USAGE:
    hive run resume --target <dir> --run <run-id> --capabilities <fresh-json> [--dispatch-intent manual|automatic] [--account-digest <sha256:...>] [--session-id <host-session-id> --process-id <positive-u32>] [--user-root <dir>] [--role <role-id> [--threshold <0..99>]] [--output json]
    Automatic preparation requires --session-id, --process-id, and --role. Manual inspection remains read-only.
```

### crates/hive-cli/src/run.rs · CLOSURE_USAGE

```text
Calculate whether one durable Hive run may finish without spawning or mutation.

USAGE:
    hive run closure --target <dir> --run <run-id> [--output json]
```

### crates/hive-cli/src/run.rs · CONTINUATION_USAGE

```text
Evaluate one host-owned continuation request without spawning or mutation.

USAGE:
    hive run continuation --target <dir> --run <run-id> --session-id <host-session-id> [--output json]
```

### crates/hive-cli/src/source_wiki.rs · SOURCE_WIKI_USAGE

```text
Provider-neutral bilingual source Wiki and disposable SQLite index.

USAGE:
    hive source-wiki lint --target <source-root> [--output json]
    hive source-wiki index --target <source-root> [--output json]
    hive source-wiki query --target <source-root> --language en|ko (--text <query>|--tag <tag>) [--limit <1..100>] [--output json]
    hive source-wiki graph preview|enable|status|rebuild|disable|query|export --target <source-root> [--engine native-markdown|graphify-code] [--consent-digest <sha256:...>] [--input <graph.json> --receipt <receipt.json>] [--node-id <id>] [--text <query>] [--format json|html] [--output json]
    hive source-wiki graph preview|enable|disable|status|prepare|apply|query --engine host-semantic --target <dir> --language en|ko --host codex|antigravity [--consent-digest <digest>] [--request-digest <digest> --input <result.json> --receipt <receipt.json>] [--node-id <id>] [--output json]
    hive source-wiki vector --help
```

### crates/hive-cli/src/storage.rs · STORAGE_USAGE

```text
Usage:
  hive storage [--user-root <path>] [--output json]
  hive prune --dry-run|--confirm [--user-root <path>] [--output json]

storage reports Hive file bytes by category and changes nothing.
prune lists or removes monitor executables that the saved monitor records do not use.
```

### crates/hive-cli/src/summon.rs · USAGE

```text
hive summon <role|god> --task <text|@project-file> --target <dir> --user-root <home> --run <id> --action <id> --expected-head <head> --control-epoch <n> --authority-id <id> [--output json]
Without --event: inert preview and request digest, no writes.
With --event: also require --authority --trust-root --request-digest --now --capabilities; write prepared inbox only. Host execution remains disabled.
```

### crates/hive-cli/src/update.rs · UPDATE_USAGE

```text
Verify and activate a local Aigent Hive release bundle.

USAGE:
    hive update --target <dir> --bundle <release-dir> (--dry-run|--apply) [--output json]
    hive update --target <dir> --recover [--output json]

BREAKING RELEASES:
    --exact-major-target <X.Y.Z> --major-confirmation <confirmation.json>
```

### crates/hive-cli/src/update.rs · RELEASE_USAGE

```text
Verify a local Aigent Hive release bundle without mutation.

USAGE:
    hive release verify --bundle <release-dir> [--output json]
```

### crates/hive-cli/src/update_activation.rs · UPDATE_USAGE

```text
Update the installed Aigent Hive package and its authenticated user projections.

USAGE:
    hive update [--channel stable|test] [--user-root <absolute-dir>] [--confirm]
                [--interactive | --non-interactive] [--output json]

The default channel is stable. The test channel requires explicit --channel test.
--confirm accepts only the software update. It does not answer setup questions.
--non-interactive and [--output json] never wait for input.
--interactive requires a terminal and conflicts with non-interactive or JSON output.
```

### crates/hive-cli/src/update_discovery.rs · UPDATE_CHECK_USAGE

```text
Check the fixed npm registry metadata endpoint for a newer Aigent Hive version.

USAGE:
    hive update --check --user-root <absolute-dir> [--output json]

The check never installs or activates an update.
```

### crates/hive-cli/src/usage_control.rs · USAGE_CONTROL

```text
Inspect or change the installed usage safeguard.

USAGE:
    hive usage show --host codex|claude|antigravity --user-root <dir> [--account-digest <sha256:...>] [--output json]
    hive usage enforce --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--account-digest <sha256:...>] [--user-root <dir>] [--output json]
    hive usage status --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--user-root <dir>] [--output json]
    hive usage threshold (--target <configured-project>|--user-root <user-root>) --remaining-percent <0..99> [--output json]
    hive usage session --target <dir> --session-id <id> --process-id <positive-u32> [--host codex|claude|antigravity] [--user-root <dir>] --action enable|disable|toggle|enable-reset-guard|disable-reset-guard|enable-zero-quota-guard|disable-zero-quota-guard [--confirm-session-disable] [--confirm-reset-guard-disable] [--confirm-zero-quota-guard-disable] [--output json]
    hive usage capture --host claude|antigravity (--target <dir>|--target-from-stdin) --stdin-json [--output json]
    hive usage summarize-tokens --request <normalized-events.json> [--output json]
```

### crates/hive-cli/src/usage_monitor.rs · HELP

```text
hive usage monitor preview|enable|disable|status --user-root <dir> [--confirm <preview-digest> --confirm-session-interruption] [--output json]
```

### crates/hive-cli/src/user_install.rs · USER_UNINSTALL_USAGE

```text
Remove the user-scope Aigent Hive installation while preserving saved preferences and knowledge.

USAGE:
    hive uninstall [--user-root <absolute-dir>] [--dry-run | --confirm] [--output json]
    hive uninstall --user-root <absolute-dir> --resume <operation-id> [--output json]

RESULT:
    Preview removal and preservation before confirmation. JSON and redirected input do not grant consent.
    Preserve saved preferences, knowledge, lifecycle history, and project source files.
```

### crates/hive-cli/src/user_setup.rs · USER_SETUP_USAGE

```text
Configure or validate Aigent Hive user preferences.

USAGE:
    hive setup --scope user --describe [--output json]
    hive setup --scope user --user-root <dir> --show --summary [--output json]
    hive setup --scope user --user-root <dir> --describe --question <question-id> [--output json]
    hive setup --scope user (--answers|--quick-answers) <yml> (--dry-run|--apply|--validate) [--user-root <dir>] [--input-channel answers-file|agent] [--output json]
    hive setup feature status|claim|prompt --id vector-search [--user-root <dir>] [--output json]
    hive setup feature answer --id vector-search --answer yes|no [--user-root <dir>] [--output json]
    hive setup --progress save --scope user --step <step> (--answers|--quick-answers) <yml> [--user-root <dir>] [--output json]
    hive setup --progress status|clear --scope user [--user-root <dir>] [--output json]
    hive setup --progress save|status|clear --scope project --target <dir> [--step <step> --answers <yml>] [--user-root <dir>] [--output json]

MODES:
    --summary    Bound JSON status/completion output; not an approval preview
    --dry-run    Validate answers and preview the owned user configuration change
    --apply      Validate and atomically apply the owned user configuration
    --validate   Validate the installed operational user configuration
    --index-rebuild-scope <digest>  Approve only the source scope shown by --dry-run; requires --apply
```

### crates/hive-cli/src/vector.rs · USAGE

```text
Optional local semantic search; FTS remains the default.
hive knowledge vector preview|enable|status|rebuild|rollback|disable --user-root <dir> --target <dir> --collection <id> --visibility shared|project-private|confidential [--python <absolute-executable>] [--consent-digest <sha256:...>] [--output json]
hive knowledge vector authorize-build --user-root <dir> --target <dir> --collection <id> --visibility confidential --capabilities <json> --usage <json> --expires-at <unix-seconds> --nonce <nonce> --confirm-current-action [--operation rebuild|rollback] [--output json]
hive source-wiki vector preview|enable|status|rebuild|rollback|disable|query --target <source-root> --language en|ko [--python <absolute-executable>] [--consent-digest <sha256:...>] [--output json]
source query options: --query <text> --top-k <1..100>
rebuild options: --max-seconds <1..60> --workers <1..16> --rebuild-mode resume|fresh
shared rebuild window: replace --collection with --collections <JSON-array-of-1..100-explicit-ids>
confidential rebuild/rollback: --authorization-id <id> --authorization-token <token> --capabilities <json> --usage <json>
```

### crates/hive-cli/src/policy.rs

```text
Policy evaluation and explicit native hook configuration.

hive policy evaluate --input <json> [--output json]
hive policy hook --help
hive policy hooks --help
Evaluation is evidence, never execution authority.
```

### crates/hive-cli/src/policy/configure.rs

```text
hive policy hooks preview|apply|remove|status|recover --host codex|claude|antigravity --target <project> [--output json] [--confirm <preview-digest>] [--review-run <id>]
Optional review-run is for Codex/Claude install previews and binds notices to an existing run's recorded session. Preview is read-only. Apply and removal require their exact preview. Host trust and effect remain separately unverified.
```

### crates/hive-cli/src/run/policy_review.rs

```text
hive run policy-review preview|add|list|accept|revise|reject|cancel --target <dir> --run <id> [--output json]
preview/add: --evaluation <run-relative-file> --rule <registered-rule> --class <failure-class>
add: --confirm <preview-digest>
accept/revise/reject/cancel: --candidate <digest> --confirm <book-digest> [--class <revised-class>]
Explicit bounded review only; acceptance never grants file mutation authority.
```

### crates/hive-cli/src/policy/native.rs

```text
hive policy hook --host codex|claude|antigravity --event PreToolUse|SessionStart|SubagentStart|PreInvocation|Stop --target <dir> --stdin-json [--expected-policy <digest>] [--review-run <id>] [--expected-context <digest>]
Native file-edit policy and optional session-bound review notice; registration and host trust are separate.
```

### crates/hive-cli/src/policy/native.rs

```text
invalid native policy hook arguments; inspect hive policy hook --help
```
