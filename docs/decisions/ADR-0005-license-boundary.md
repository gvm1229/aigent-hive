# ADR-0005: Source와 출하 harness의 라이선스를 분리한다

- 상태: Accepted
- 기준일: 2026-07-23

## 맥락

Aigent Hive CLI와 source의 비공개 파생 배포에는 copyleft를 적용하되, 소비자 프로젝트에 설치되는 harness가 그 프로젝트의 라이선스를 제약해서는 안 된다. 하나의 라이선스를 저장소 전체에 적용하면 두 목표를 동시에 충족하기 어렵다.

## 결정

- `harness/**`를 제외한 저장소 source는 `GPL-3.0-only`로 배포한다.
- `harness/**`와 여기에서 생성된 Aigent Hive 소유 파일·exact marker block은 `Apache-2.0`으로 배포한다.
- 소비자 프로젝트의 기존 source, 문서, 설정과 data는 Aigent Hive가 재라이선스하지 않는다.
- `LICENSES/`에 두 라이선스 전문을 보관하고 `REUSE.toml`을 file-scope 정본으로 사용한다.
- Rust package metadata는 `GPL-3.0-only`를 선언한다.
- 생성된 harness는 프로젝트 root가 아닌 `.hive/LICENSE-AIGENT-HIVE.txt`에 Apache-2.0 전문을 포함한다.
- release bundle은 두 라이선스 전문과 scope mapping을 모두 포함해야 한다.

## 결과

- CLI/source의 GPL-covered 수정판을 배포하는 주체는 GPLv3 의무를 따라야 한다.
- 사용자는 Apache-2.0 harness를 상용·비공개 프로젝트에 포함할 수 있다.
- 생성된 harness는 소비자 프로젝트의 root license를 생성·수정하지 않는다.
- renderer, updater와 release packager는 ownership뿐 아니라 license scope도 보존해야 한다.
