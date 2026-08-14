# `0.9.4` 전역 projection validation 정합성

> Checklist owner: `UPV94-*`
> 대상: `0.9.4` patch
> 선행: `0.9.3` stable artifact immutable

## 문제

현재 `hive setup --scope user --validate`: 실제 Hive-managed Skill·directive projection이
모두 incoming 값과 일치해도 `.hive/install/user-projection.json`의 바이트 차이만으로 충돌
보고 가능. 같은 입력의 dry-run: 각 projection `unchanged` 보고.

원인: 의미적으로 같은 user setup 확인 뒤 새로 직렬화한 setup 바이트로 receipt 재생성.
receipt의 `setup_digest`만 변경, 정상 설치 invalid 판정.

## 원칙

- user setup·install·update·validate는 같은 Hive-managed projection 상태를 같은 결과로 판정
- configuration 의미가 같고 모든 managed path가 일치하면 receipt 직렬화 차이만으로 실패 금지
- 실제 managed file·ownership manifest·구조화 설정 손상은 fail-closed 유지
- validation: file write·receipt recreation `0회`

## Checklist

- [ ] [UPV94-001] 기존 `0.9.3` 설치 형태를 재현해 managed projection 전부 일치·
  `hive setup --validate` 충돌·dry-run unchanged의 불일치 regression 고정
- [ ] [UPV94-002] validation이 설치된 canonical setup binding 또는 동등한 정규화 binding을
  사용해 정상 projection receipt를 수용
- [ ] [UPV94-003] setup·install·update·validate가 user projection의 local change, ownership
  mismatch, malformed receipt와 실제 structured configuration 손상은 계속 거부
- [ ] [UPV94-004] Rust unit·CLI integration과 Windows x64 설치본 fresh setup·preserving
  reinstall·validate 수용에서 결과 수렴 확인

## 수락 기준

- 설치된 Hive-managed file이 모두 일치한 전역 setup validation 성공
- 같은 상태의 dry-run·validate·install validation 결과 모순 `0건`
- 오류 시 실제 변경·손상 경로와 안전한 복구 경로 표시

## 범위 제외

- 사용자 설정 의미 변경
- user-root 밖 파일 변경
- existing local-preserved project validation 계약 변경
