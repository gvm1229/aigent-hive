# Binary update 뒤 사용자 투영 자동 갱신

> Checklist IDs: `AUP95-001`–`AUP95-007`
> Target: `0.9.5` compatible patch
> Scope: interactive bare `hive update`의 authenticated binary replacement 뒤 saved user-scope projection refresh

## 문제 기준

Bare `hive update`는 npm 또는 direct install owner의 binary package만 교체. 사용자 directive·Skill·host
plugin projection은 별도 `hive install --scope user ... --apply` 전까지 기존 release 상태. Binary version
성공 표시와 실제 Hive harness 상태의 불일치·추가 사용자 명령 혼동.

## 범위 복원 규칙

- Last command-line syntax의 byte 재생 불필요. semantic scope 정본: valid `user-setup.yml`의
  `selected_hosts`와 host별 authenticated `.hive/install/<host>.json`의 교집합
- `--host` 반복·`--hosts` CSV의 표현 차이 무관. 복원 결과: canonical `--hosts <ordered-hosts>` 사용
- setup·manifest 부재·parse failure·ownership authentication failure·empty intersection: host default 금지,
  projection mutation `0건`, binary update 상태와 manual recovery command 분리 표시
- 사용자 갱신 동의 전 resolved host list·projection refresh 포함 범위 표시. `--check` 자동 설치 금지 유지
- 새 binary만 projection refresh 실행. 이전 process의 embedded harness·PATH 우연 resolution 사용 금지

## 실행 checklist

- [x] `AUP95-001` trusted refresh-scope resolver 구현: canonical user root·operational `selected_hosts`·per-host
  install manifest의 schema·ownership·release authentication 검증과 stable ordered intersection 반환
- [x] `AUP95-002` interactive prompt 보강: package owner·exact target·resolved host list·post-update
  projection refresh 범위 표시. safe scope 부재 시 binary-only outcome과 이유·manual command preview 표시
- [x] `AUP95-003` owner install 뒤 exact refreshed executable handoff 구현: target binding 재검증 뒤
  new executable로 `install --scope user --hosts <resolved> --apply --output json` 실행, old process 사용 금지
- [x] `AUP95-004` projection apply 뒤 same resolved scope `--validate` 실행과 structured child result 검증.
  host별 transactional preflight·backup·foreign byte 보존은 existing user-install contract 재사용
- [x] `AUP95-005` outcome contract 구현: binary·projection 모두 성공, binary-only safe skip, binary 성공 뒤
  projection 실패의 세 상태 분리. 실패 상태: binary rollback 주장 금지·exact recovery command·changed-host evidence 표시
- [x] `AUP95-006` unit·integration matrix 추가: `codex,claude` saved scope 보존, single host, missing
  manifest, malformed/tampered setup·manifest, empty intersection, child exit/invalid JSON, npm·direct owner,
  old-executable rejection, no default-host fallback

## 명시 보류 배포 수용

`AUP95-007` public `0.9.5-test.N` multi-host bare update 수용은 현재 유지보수자 경계에 따라
공개 시험판·정식 배포와 함께 보류. 완료·통과 주장 금지. 재승인 뒤
[`release-0.9.5-stable-publication.md`](release-0.9.5-stable-publication.md)의 `REL95-*`와 함께
수행.

## 완료 기준

- 이전 설치가 `codex,claude`인 user root: bare update 뒤 두 host projection 모두 새 release와 validate 결과 일치
- 이전 설치가 `codex`만인 user root: Claude activation·파일 변경 `0건`
- trusted scope 결정을 못 하는 상태: binary만 authenticated owner로 갱신 가능, host projection mutation `0건`,
  결과에 정확한 skipped reason·recovery command
- package owner·target binary·child executable 불일치 또는 child failure: success 오표시 금지, foreign byte 보존
- release artifact의 npm·direct owner와 public test contract evidence는 공개 배포 재승인 뒤 수행

## 출시 경계

`AUP95-001`–`AUP95-006`: source implementation·local verification 범위. `AUP95-007`: 현재 명시 보류,
재승인 뒤 external public test evidence와 stable publication authority 필요. 과거 literal install flag 기록 부재는 blocker 아님;
authenticated semantic scope 부재만 automatic projection refresh blocker.
