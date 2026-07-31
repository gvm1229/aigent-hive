# Consumer guidance marker contract

## 목적

Hive는 provider-neutral consumer guidance를 shared `AGENTS.md`의 exact marker block으로 projection. Host별 entrypoint는 이 공통 contract를 발견하는 얇은 adapter이며 별도 orchestration 정본에서 제외.

```text
<!-- AIGENT-HIVE:START -->
... Hive-owned guidance ...
<!-- AIGENT-HIVE:END -->
```

Marker 이름은 `AIGENT-HIVE`로 고정. Hive가 소유하는 범위는 start marker의 첫 byte부터 end marker의 마지막 byte까지 하나의 정상적인 block뿐.

## Merge와 ownership

- Marker가 없으면 기존 bytes 뒤에 Hive block을 추가.
- 정상 marker가 하나 있으면 그 block만 교체.
- Marker 밖 user bytes와 third-party marker bytes는 opaque로 취급하고 byte-for-byte 보존.
- Hive marker가 중복, 중첩 또는 한쪽만 있으면 추정 복구하지 않고 conflict로 중지.
- Hive의 external marker 의미 parse와 external namespace 소유권 manifest 편입 금지.

Renderer는 `AGENTS.md` 조회 전에 target-relative path와 모든 ancestor의 non-symlink 상태를 확인. Symlink, FIFO, directory 또는 다른 non-file이 shared path를 차지하면 외부 target bytes 조회 없이 conflict 종료.

Apply는 충돌하지 않는 임의 이름의 exclusive temp file에서 새 bytes를 flush·sync한 뒤 교체. 중간 operation이나 설치 후 검증이 실패하면 transaction 시작 전의 managed bytes와 생성 directory를 복원. Rollback 자체가 실패하면 성공으로 축소하지 않고 `hive.activation-rollback-failed`를 반환.

Consumer guidance에는 project/profile, primary host, resolved owner와 resolution evidence digest처럼 installed harness에서 재현 가능한 최소 정보만 배치. Canonical setup/config/consent는 `.hive/`의 tracked YAML/TOML이며 marker prose의 machine authority 대체는 불가.

## Usage automatic-dispatch gate

- 일반 응답, manual 작업과 non-dispatch action: `hive usage enforce` 호출 없음
- 새 automatic dispatch 직전 exact current host session ID·process ID로 one-shot
  `hive usage enforce` 실행
- Current halt marker 우선; exit `3`이면 해당 automatic dispatch 차단
- Exit `0`: session-bound preflight 통과만 의미, dispatch authorization 아님
- 실제 authorization: 별도 `hive run resume --dispatch-intent automatic` 결과의 `data.usage_guard.enforced=true`, `outcome=authorized`, authorization ID 1개와 dispatch brief 정확히 1개
- 명시적인 current-session disable과 confirmation flag: preflight 우회만 허용, dispatch authorization 효과 없음
- Installed `primary_host`와 pinned run·capability host 일치 필수; Non-Codex automatic dispatch는 qualified local sensor 전까지 fail-closed
- 명백한 threshold·disable·enable intent: Skill 이름이나 고정 문구 목록이 아닌 의미 기반 판별
- Enable 뒤 current halt marker가 남은 경우 즉시 재차단; 다른 host·session·PID marker는 stale 또는 absent
- `status`는 조회 전용이며 automatic-dispatch preflight 대체 불가
- OMX/OMC cancellation 결과는 보조 evidence일 뿐 halt marker나 durable goal/task 상태 대체 불가
- Fallback hook, prompt rewrite, Skill activation, watcher, orchestration과 Stop continuation 없음

`hive setup --validate`는 현재 marker block이 supplied setup answer와 capability evidence에서 재현한 exact block인지 확인. Missing, stale, malformed marker는 marker 밖 bytes를 바꾸지 않은 채 verification failure.

## 편집 규율

- 모든 편집 전 `.hive/directives/00-editing-discipline.md` 전문 확인
- 네 section 전부 Hive contract 안의 최우선 편집 규율로 적용
- 축약·요약·누락·대체 금지
- 원문의 `# CLAUDE.md` heading은 Claude 전용 범위가 아니며 Codex·Claude·Gemini Antigravity에 동일 적용
- 상위 instruction과 Hive security·ownership·credential·production 경계 우선

## 남은 작업 안내

- 남은 작업 목록·인계 전: 범위 안에서 안전하게 자동 처리 가능한 작업 선행 완료
- 사용자 권한·자격 증명·보호된 외부 변경·제품 결정 필요 작업만 사용자 단계로 유지
- 각 사용자 단계: 정확한 위치·명령 또는 행동, 예상 결과·반환 증거, 자동 처리 불가 이유
- 실패·불가능 작업: 원인과 해결 경로 분리

## 사람용 문서 스타일

Consumer marker의 사람용 문서 작성 기본값:

- 사용자가 다른 언어를 명시하지 않은 경우 간결한 한국어
- 선택 언어를 질문과 응답 전체에서 일관되게 사용
- 한국어에서는 고유명사, 제품·패키지 이름, 명령어, 코드 식별자, 경로, 스키마 키,
  정확한 화면 문구, 뚜렷한 한국어 대체어가 없는 용어만 영어 유지
- 대체 가능한 일반 영어 단어의 한영 혼용 금지
- 영어에서는 정확한 한국어 이름·문자열·인용문·사용자 보존 요청을 제외하고 영어로 통일
- 짧은 heading, bullet, table, checklist 우선
- `추가`, `정리`, `검증`, `확인`, `적용` 같은 명사형·동작 명사형 우선
- 설명문을 끝내는 `~다`, `~한다`, `~된다`, `~이다`, `~있다`, `~없다`, `~않는다`,
  `~했다`, `~됐다`, `~합니다`, `~됩니다`, `~해요` 금지. 예시는 제한 목록 아님
- 단순 suffix 교체로 만든 `~음`·attached `~ㅁ` 금지. Korean stem, mixed
  English-Korean form, state+copula와 possibility clause 포함
- authored 설명의 conversational imperative `~줘`, attached `~해` 금지. 예시는
  제한 목록 아님
- authored callout·blockquote도 같은 규칙 적용. Blockquote 표시는 exact quote 증거 아님
- exact 외부 인용·UI prompt·protocol·fixture만 path·line·reason·line digest 예외 허용
- code identifier, schema key, path, command, product name, exact UI label 원형 유지
- teaching note와 handoff는 지나친 축약보다 이해 우선

정확한 bad/good 예시:

| 금지 | 권장 |
| --- | --- |
| `Aigent Hive는 provider-neutral 로컬 agent harness다.` | `Aigent Hive: provider-neutral 로컬 agent harness` |
| `Product version은 0.7.0이다.` | `Product version: 0.7.0` |
| `Release 계약이 구현됐다.` | `Release 계약 구현 완료` |
| `API key를 요청하거나 저장하지 않는다.` | `API key 요청·저장 없음` |
| `이 기능을 사용합니다.` | `기능 사용` |
| `다음 단계에서 검증해요.` | `다음 단계: 검증` |
| `검증이 필요합니다.` | `검증 필요` |
| `업데이트가 완료되었습니다.` | `업데이트 완료` |
| `Release 계약이 구현됐음.` | `Release 계약 구현 완료` |
| `API key를 요청하거나 저장하지 않음.` | `API key 요청·저장 없음` |
| `Status는 INDETERMINATE다.` | `Status: INDETERMINATE` |
| `문서를 읽음.` | `문서 확인` |
| `작업이 끝남.` | `작업 완료` |
| `연결이 닫힘.` | `연결 종료` |
| `설정 값을 가짐.` | `설정 값 보유` |
| `정책을 따름.` | `정책 준수` |
| `compile됨.` | `compile 완료` |
| `검증할 수 있음.` | `검증 가능` |
| `검증할 수 없음.` | `검증 불가` |
| `문서를 보여 줘.` | `문서 확인 요청` |
| `기능을 사용해.` | `기능 사용 요청` |

Authored blockquote 대조:

```text
금지: > 현재 상태는 0.7.0이다.
권장: > 현재 상태: 0.7.0
```

결과 보고 규칙:

- 통과·실패·건너뜀·연기·미검증·미지원 결과: 대상 범위, 정확한 이유, 현재
  호스트·운영체제와의 관계, 실제 실행 여부, 증명 범위와 미증명 범위를 모두 명시
- `Windows 전용`, `Unix 전용` 같은 표현: 현재 운영체제에서 실행 또는 건너뛴 쪽과
  그 이유를 함께 명시
- 해석에 필요한 한정어를 간결함을 이유로 생략 금지

## Artifact 경계

| Artifact | Guidance 역할 |
| --- | --- |
| Hive source workspace | template, renderer, schema와 conformance test의 정본 |
| Release bundle | versioned compiled template와 ownership metadata 운반 |
| Installed consumer harness | 해당 consumer project의 exact Hive marker와 `.hive/` canonical config |

Source 개발 지침인 root `AGENTS.md`와 `.agents/`의 release 또는 consumer marker 복사 금지. Installed harness state의 source 역수입과 release bundle의 mutable user state 저장소 사용 금지.

## External runtime 경계

Hive marker는 compatible OMX/OMC precedence와 non-clobber 규칙만 설명. `.omx/`, `.omc/`, plugin cache, host-global configuration, session manifest, external runtime marker의 Hive state import·생성·수정·삭제·소유권 주장 금지.

Fallback hook guidance도 Hive data-integrity capability의 조건부 consent 경계만 기술. External runtime의 plan, Ralph, team, Skill routing, prompt classification 또는 continuation state projection 금지.
