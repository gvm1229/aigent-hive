# ADR-0005: 전체 source와 출하 harness의 Apache-2.0 적용

- 상태: Accepted
- 기준일: 2026-07-23

## 맥락

Aigent Hive는 대규모 사용자에게 배포되는 provider-neutral CLI와 통합용 harness.
Host, plugin, 상용·비공개 프로젝트의 별도 법적 결합 범위 해석 없는 채택 가능성과
기여자 특허 권리의 명시적 처리가 필요.

## 결정

- 저장소의 CLI, source, 문서와 `harness/**`를 모두 `Apache-2.0`으로 배포.
- 여기에서 생성된 Aigent Hive 소유 파일·exact marker block도 `Apache-2.0`으로 배포.
- 소비자 프로젝트의 기존 source, 문서, 설정과 data는 Aigent Hive 재라이선스 대상에서 제외.
- `LICENSES/Apache-2.0.txt`에 REUSE canonical 전문을 보관하고 `REUSE.toml`을 file-scope 정본으로 사용.
- GitHub license 감지를 위해 root `LICENSE`는 수정하지 않은 Apache-2.0 전문만 포함.
- `harness/LICENSE`는 출하 source의 Apache-2.0 전문을 제공.
- Rust package metadata는 `Apache-2.0`을 선언.
- 생성된 harness는 프로젝트 root가 아닌 `.hive/LICENSE-AIGENT-HIVE.txt`에 Apache-2.0 전문을 포함.
- release bundle의 Apache-2.0 전문과 scope mapping 포함 필수.

## 결과

- 사용자의 CLI·harness 상용·비공개 프로젝트 사용과 수정·배포 허용.
- 배포자의 Apache-2.0 저작권·라이선스 고지, 변경 기록, NOTICE 조건 준수 필수.
- 기여자의 명시적 특허 허여와 방어적 종료 조항 적용.
- 생성된 harness에 의한 소비자 프로젝트 root license 생성·수정 없음.
- renderer, updater, release packager의 ownership과 license notice 보존 필수.
