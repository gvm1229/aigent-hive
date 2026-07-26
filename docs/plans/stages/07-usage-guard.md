# Stage 7. Subscription usage guard

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

기본 설정은 신뢰 가능한 sensor가 보고한 `remaining <= 10%`일 때 새 자율 delegation 또는 loop iteration을 시작하지 않는 것.

Hive는 provider API를 호출 금지. Sensor 후보는 host가 로컬로 노출하는 command/file/status 또는 CodexBar 같은 별도 local tool.

Snapshot freshness:

- 새 delegation 직전 sample
- 한 번의 host dispatch가 발생하면 즉시 만료
- dispatch가 없더라도 sensor가 선언한 TTL 또는 Hive 최대 TTL 중 짧은 값 사용
- missing, stale, account/window 불일치, 역행 값은 `usage_unknown`

`usage_unknown`에서는 automatic continuation fail-closed. 이미 시작된 dispatch를 강제 종료한다고 주장하지 않으며, 다음 dispatch 전 새 snapshot이 필요.

#### 구현

`UsageSnapshot` 최소 필드:

- sensor ID/version
- host/account scope
- quota window
- remaining percent
- measured at
- expires at
- source confidence

CodexBar adapter는 pinned-qualified CLI의 `usage` JSON에서 active account와 quota
pool·window를 검증. `guard` 결과만으로는 account와 freshness를 증명할 수 없으므로
정본 snapshot으로 사용 금지. Cadence pool은 session window 우선, session 부재
시에만 weekly fallback. Provider-defined window는 같은 pool의 cadence window와
공존 금지. 모든 pool의 선택 window가 유효하고 threshold를 통과해야 permit 발급.
선택 가능한 window가 없거나 어느 pool이든
`remaining <= installed threshold`이면 새 permit을 발급 금지. Default
installed threshold는 `10%`.

Adapter는 side-effect-free local command만 실행. Hive는 model call retry를 하지 않으며 local sensor read도 bounded attempt 후 unknown 처리.

`hive run resume`은 기본 manual intent에서 sensor를 읽지 않고 기존 prepare-only
recovery를 유지. Explicit automatic intent는 account digest와 selected active
role 하나를 요구하고 installed `.hive/config/harness.toml`의
`usage_stop_remaining_percent`를 권위값으로 사용. Optional threshold override는
설치값과 exact하게 같을 때만 허용. Durable run, owner, role과 evidence 검증 뒤
fresh CodexBar snapshot을 평가.

Selected prior snapshot vector는 Git에서 제외된 Hive-owned
`.hive/runtime/usage-history/`에만 bounded·integrity-bound record로 저장. 이후
measurement/reset 역행과 같은 reset의 remaining 증가는 `usage_unknown`.
Permit은 dispatch brief 준비 closure 직전에 한 번 소비하며 exact run
revision·role·brief당 deterministic authorization 하나와 brief 하나만 발급.
같은 binding의 재발급, limited, unknown 또는 expired 결과는 brief 없이 recovery
data만 반환. 이 integration은 model이나 subagent를 spawn 금지.

Hive의 보호 범위: 같은 authorization의 재발급 거부. Caller가 이미 capture한 JSON의
Hive 외부 replay 차단은 범위 밖. 실제 host/orchestration owner의 dispatch
boundary에서 authorization ID exactly-once 소비 필수.

#### 완료 조건

- [x] API endpoint와 provider SDK dependency 0개
- [x] installed threshold와 결합된 10% 경계에서 새 automatic dispatch 0개
- [x] stale/missing/mismatched/regressing sensor가 product path에서 `usage_unknown`
- [x] sensor가 없는 host에서 enforcement 가능하다고 표시 금지
- [x] CodexBar가 없어도 core setup·memory·update 동작
