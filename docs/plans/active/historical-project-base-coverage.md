# 과거 프로젝트 기준본 수용 범위 보강

> Checklist IDs: `HBC95-001`–`HBC95-006`
> Target: `0.9.5` compatible patch
> Scope: shipped project upgrade의 과거 full project-base 식별·인증·검증 범위

## 문제 기준

`0.9.4` migration table의 same-major source range에 `0.9.2`가 포함되지만, compiled binary에는
그 release의 full project base가 없음. `0.9.2` ledger에는 Skill 외 directive·shared marker가 함께
있어 Skill-only historical fallback 적용 불가. 결과: `scan`·`dry-run` 단계의
`hive.update-migration-unsupported`, `apply` 미실행.

## 불변 조건

- Project base version field 단독 신뢰 금지. exact historical bytes·ownership·digest 인증 전 `apply` 금지
- Migration range 선언과 binary 내 인증 가능 historical base의 일대일 대응
- 과거 release artifact의 임의 download·실행 금지. release bundle에 포함된 정본만 사용
- Hive-owned 경로 외 user·foreign byte 보존. 기준본 누락·tamper 상태: mutation 없는 실패
- 실제 소비자 workspace import 금지. 최소 synthetic fixture와 release artifact 검사 사용

## 실행 checklist

- [ ] `HBC95-001` shipped full project-base release registry 작성: migration source로 선언한 모든
  release의 exact rendered base·digest·required render input 수집. 우선 회귀 기준 `0.9.1`·`0.9.2`·`0.9.3`
- [ ] `HBC95-002` registry 단일 소비 경로 연결: `historical_project_upgrade_candidate_in`와
  `authenticate_historical_base`의 version lookup·full-ledger 인증·diagnostic을 같은 registry로 통합
- [ ] `HBC95-003` migration-table build/validation gate 추가: declared `from_min`–`from_max`의 각
  shipped source release에 registry entry가 없으면 bundle 생성·release gate 실패 또는 range 축소
- [ ] `HBC95-004` `0.9.2` synthetic consumer fixture 회귀 추가: `scan`·`dry-run`·`apply`·`validate`
  전체 수명주기, modified Hive-owned projection의 three-way merge, user·foreign byte 보존, rollback 검증
- [ ] `HBC95-005` declared source range matrix 회귀 추가: 현재 target으로의 각 source release
  candidate authentication·apply validation, missing·tampered baseline의 no-mutation 실패, packaged binary 포함 확인
- [ ] `HBC95-006` release evidence·문서 수용: `0.9.5-test.N`에서 public `0.9.2 → 0.9.5-test.N`
  project upgrade 수용, registry coverage report·migration range parity 기록, stable publication 전 gate 연결

## 완료 기준

- `0.9.2` full project base를 가진 synthetic project의 `scan`·`dry-run`·`apply`·`validate` 성공
- binary가 현재 migration range의 모든 declared source version을 exact base로 인증
- exact base가 없거나 digest가 불일치한 project: `apply` 전 중단, Hive-owned·user·foreign 파일 변경 `0건`
- migration range와 registry coverage의 불일치: CI와 release bundle gate에서 실패
- public test artifact와 stable candidate에서 같은 matrix evidence 보관

## 출시 경계

`HBC95-001`–`HBC95-005`는 source implementation·local verification 범위. `HBC95-006`의 public
test와 stable publication은 release authority·external evidence 필요. version field 수동 변경,
consumer `project-base.json` 편집, `0.9.3` 경유 upgrade는 복구 경로 아님.
