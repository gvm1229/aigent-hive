# Phase 1. Deterministic setup renderer — `0.2.0`

> 상태: 완료 기록; 일반 goal 실행에서 load 제외

- [x] `hive-render` crate 추가
- [x] Copier/Rust parity corpus
- [x] `orchestration_layer` setup preference를 schema/Copier/template/Skill에서 제거
- [x] host capability resolver와 `available|absent|incompatible|unknown` evidence contract
- [x] fallback hook capability preview·consent schema와 `.hive/config/approved-hooks.yml`
- [x] staging render와 ownership validator
- [x] ownership-class 보존과 target read 전 no-follow guard
- [x] 충돌하지 않는 exclusive temp, transactional rollback과 rollback-failure code
- [x] shared marker three-way merge
- [x] setup answer migration
- [x] role seed materializer와 idempotent/reconfigure fixture
- [x] RFC 8785 Skill consent verifier와 tamper fixture
- [x] fallback hook typed execution·revoke와 complete installed validation
- [x] schema-valid `UnknownAction` JSON과 write-free failure
- [x] `hive setup --dry-run|apply|validate`
