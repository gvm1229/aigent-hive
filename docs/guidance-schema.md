# Consumer guidance marker contract

## 목적

Hive는 provider-neutral consumer guidance를 shared `AGENTS.md`의 exact marker block으로 projection한다. Host별 entrypoint는 이 공통 contract를 발견하는 얇은 adapter이며 별도 orchestration 정본이 아니다.

```text
<!-- AIGENT-HIVE:START -->
... Hive-owned guidance ...
<!-- AIGENT-HIVE:END -->
```

Marker 이름은 `AIGENT-HIVE`로 고정한다. Hive가 소유하는 범위는 start marker의 첫 byte부터 end marker의 마지막 byte까지 하나의 정상적인 block뿐이다.

## Merge와 ownership

- Marker가 없으면 기존 bytes 뒤에 Hive block을 추가한다.
- 정상 marker가 하나 있으면 그 block만 교체한다.
- Marker 밖 user bytes와 third-party marker bytes는 opaque로 취급하고 byte-for-byte 보존한다.
- Hive marker가 중복, 중첩 또는 한쪽만 있으면 추정 복구하지 않고 conflict로 중지한다.
- Hive는 external marker의 의미를 parse하거나 external namespace를 소유권 manifest로 가져오지 않는다.

Renderer는 `AGENTS.md`를 읽기 전에 target-relative path와 모든 ancestor가 symlink가 아닌지 확인한다. Symlink, FIFO, directory 또는 다른 non-file이 shared path를 차지하면 외부 target bytes를 읽지 않고 conflict로 끝난다.

Apply는 충돌하지 않는 임의 이름의 exclusive temp file에서 새 bytes를 flush·sync한 뒤 교체한다. 중간 operation이나 설치 후 검증이 실패하면 transaction 시작 전의 managed bytes와 생성 directory를 복원한다. Rollback 자체가 실패하면 성공으로 축소하지 않고 `hive.activation-rollback-failed`를 반환한다.

Consumer guidance에는 project/profile, primary host, resolved owner와 resolution evidence digest처럼 installed harness에서 재현 가능한 최소 정보만 둔다. Canonical setup/config/consent는 `.hive/`의 tracked YAML/TOML이며 marker prose가 machine authority를 대체하지 않는다.

`hive setup --validate`는 현재 marker block이 supplied setup answer와 capability evidence에서 재현한 exact block인지 확인한다. Missing, stale, malformed marker는 marker 밖 bytes를 바꾸지 않은 채 verification failure다.

## Artifact 경계

| Artifact | Guidance 역할 |
| --- | --- |
| Hive source workspace | template, renderer, schema와 conformance test의 정본 |
| Release bundle | versioned compiled template와 ownership metadata 운반 |
| Installed consumer harness | 해당 consumer project의 exact Hive marker와 `.hive/` canonical config |

Source 개발 지침인 root `AGENTS.md`와 `.agents/`는 release 또는 consumer marker에 복사하지 않는다. Installed harness state를 source로 역수입하지 않으며 release bundle을 mutable user state 저장소로 사용하지 않는다.

## External runtime 경계

Hive marker는 compatible OMX/OMC precedence와 non-clobber 규칙만 설명한다. `.omx/`, `.omc/`, plugin cache, host-global configuration, session manifest 또는 external runtime marker를 읽어 Hive state로 import하지 않으며 생성·수정·삭제하거나 소유권을 주장하지 않는다.

Fallback hook guidance도 Hive data-integrity capability의 조건부 consent 경계만 기술한다. External runtime의 plan, Ralph, team, Skill routing, prompt classification 또는 continuation state를 projection하지 않는다.
