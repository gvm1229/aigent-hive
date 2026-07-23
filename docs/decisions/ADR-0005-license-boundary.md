# ADR-0005: 전체 source와 출하 harness에 Apache-2.0을 적용한다

- 상태: Accepted
- 기준일: 2026-07-23

## 맥락

Aigent Hive는 대규모 사용자에게 배포되는 provider-neutral CLI와 통합용 harness다. host, plugin, 상용·비공개 프로젝트가 법적 결합 범위를 별도로 해석하지 않고 채택할 수 있어야 하며, 기여자가 제공하는 특허 권리도 명시적으로 다뤄야 한다.

## 결정

- 저장소의 CLI, source, 문서와 `harness/**`를 모두 `Apache-2.0`으로 배포한다.
- 여기에서 생성된 Aigent Hive 소유 파일·exact marker block도 `Apache-2.0`으로 배포한다.
- 소비자 프로젝트의 기존 source, 문서, 설정과 data는 Aigent Hive가 재라이선스하지 않는다.
- `LICENSES/Apache-2.0.txt`에 REUSE canonical 전문을 보관하고 `REUSE.toml`을 file-scope 정본으로 사용한다.
- GitHub license 감지를 위해 root `LICENSE`는 수정하지 않은 Apache-2.0 전문만 포함한다.
- `harness/LICENSE`는 출하 source의 Apache-2.0 전문을 제공한다.
- Rust package metadata는 `Apache-2.0`을 선언한다.
- 생성된 harness는 프로젝트 root가 아닌 `.hive/LICENSE-AIGENT-HIVE.txt`에 Apache-2.0 전문을 포함한다.
- release bundle은 Apache-2.0 전문과 scope mapping을 포함해야 한다.

## 결과

- 사용자는 CLI와 harness를 상용·비공개 프로젝트에 사용하고 수정·배포할 수 있다.
- 배포자는 Apache-2.0의 저작권·라이선스 고지, 변경 기록과 NOTICE 조건을 따라야 한다.
- 기여자의 명시적 특허 허여와 방어적 종료 조항이 적용된다.
- 생성된 harness는 소비자 프로젝트의 root license를 생성·수정하지 않는다.
- renderer, updater와 release packager는 ownership과 license notice를 보존해야 한다.
