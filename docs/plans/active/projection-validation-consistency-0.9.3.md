# `0.9.3` projection validation 정합성

> Checklist owner: `VAL93-*`
> 대상: `0.9.3`
> 범위: project upgrade의 `local-preserved`와 setup validation, formatter 손상 예방

## 원칙

- `project upgrade --validate`가 기록한 authenticated `local-preserved` Skill은 `setup --validate`에서도 유효 상태
- override ledger·base ledger·현재 local digest가 모두 일치할 때만 허용
- JSON·YAML·Markdown·role profile의 문법 손상은 허용 대상 아님
- root `.prettierignore`의 Hive marker block만 Hive 소유, 기존 formatter 설정 byte 보존

## Checklist

- [x] [VAL93-001] `project-overrides.json`의 canonical ledger·base/local digest·Skill projection path 검증과 `setup --validate` local-preserved acceptance
- [x] [VAL93-002] stale·forged override, 다른 path, digest 불일치와 trailing-comma role profile의 fail-closed·supported remediation receipt
- [x] [VAL93-003] project setup·upgrade의 marker-owned `.prettierignore` Hive projection exclusion과 existing foreign byte preservation
- [x] [VAL93-004] Rust renderer·project lifecycle·three-host projection parity·human guidance regression

## 수락 기준

- 동일 local-preserved installation의 upgrade·setup validate 결과 일치
- user-local Skill content 보존, Hive-owned ledger와 foreign `.prettierignore` bytes 보존
- malformed structured projection은 exact path와 `hive project upgrade --apply` remediation 표시
