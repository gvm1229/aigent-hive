# 2026-10-05 추가 정리 검토

- 사용자 요청: 남은 프로젝트 용량의 추가 축소
- 실제 조사: Codex·Windows, 현재 소스의 활성 CLI는 저장소 밖 npm 소유 파일
- 대상: 미사용 Cargo 산출물, 과거 합성 검색 시험의 모델·Python 환경 복제본·파생 SQLite와 상태 사본
- Cargo 확인: 별도 시험 빌드의 CACHEDIR.TAG·rustc 정보·release 폴더, 현재 시험·제품 소스에서 과거 경로 참조 없음
- 모델 확인: 남기는 다운로드 영수증의 모델 이름·버전·크기·지문, 별도 보존한 결과와 생성·측정 스크립트
- 검색 사본 확인: audit/reference 폴더의 SQLite와 active/checkpoint/runtime 등의 파생 상태, 원본 Markdown·Python·요약 JSON은 별도 경로에 보존
- 현재 빌드 target/debug는 이후 전체 검사에 사용하므로 제외, 다른 빌드 폴더도 살아 있는 소비자 발견 시 보존
- 삭제 전 기존 정리 도구의 소유권·Git·근거·현재 프로세스·연결 경로·목록 지문 검사 필수
- 원시 캐시 직접 복구 보장 없음; 재생성에는 필요한 소스·입력·도구 필요, 원래 정확한 빌드 바이트 보장과 구분
- 현재 설치·WProject·외부 실행 상태·소스 코드·동결 원본 변경 제외
- 과거 파일 존재로 과거 시험 통과 추론 없음, 이번 검토는 삭제 용도 확인 범위

## 정확한 대상

- `target/release`
- `target/x86_64-pc-windows-gnu`
- `tests/work/knowledge-transfer-baseline-target`
- `tests/work/knowledge-transfer-native/imported-user/.hive/index`
- `tests/work/knowledge-transfer-release-target`
- `tests/work/scope-audit-20260828/actual-minilm-sqlite-vec-mmap.sqlite3`
- `tests/work/scope-audit-20260828/actual-minilm-sqlite-vec.sqlite3`
- `tests/work/scope-audit-20260828/e5-base/model_qint8_avx512_vnni.onnx`
- `tests/work/scope-audit-20260828/e5-small/model_qint8_avx512_vnni.onnx`
- `tests/work/scope-audit-20260828/minilm-vectors-50000.npy`
- `tests/work/scope-audit-20260828/mrl/onnx/model.onnx`
- `tests/work/scope-audit-20260828/mrl/onnx/model_int8.onnx`
- `tests/work/scope-audit-20260828/mrl/vectors-50000.npy`
- `tests/work/scope-audit-20260828/potion/onnx/model.onnx`
- `tests/work/scope-audit-20260828/potion/vectors-50000.npy`
- `tests/work/scope-audit-20260828/product-download-cache`
- `tests/work/scope-audit-20260828/product-runtime-1/model`
- `tests/work/scope-audit-20260828/product-runtime-1/site`
- `tests/work/scope-audit-20260828/product-runtime-1/wheels`
- `tests/work/scope-audit-20260828/product-runtime-2/model`
- `tests/work/scope-audit-20260828/product-runtime-2/site`
- `tests/work/scope-audit-20260828/product-runtime-2/wheels`
- `tests/work/scope-audit-20260828/vector-cli-runtime-5mzoqrna/.agents/work/vector`
- `tests/work/scope-audit-20260828/vector-cli-runtime-iz9955o1/.agents/work/source-wiki`
- `tests/work/scope-audit-20260828/vector-scale-100/batch-cli-acceptance/physical-snapshots`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/cpu-affinity/p-only/audit-reference`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/cpu-affinity/p-only/incremental/recovery/audit-unpublished`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/shared-list-next/incremental100/audit-reference`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/shared-list-next/incremental100/reference-after-incremental`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/shared-list-next/wave4-next/final-acceptance/fresh50k-measurement/audit-reference`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/shared-list-next/wave4-next/final-acceptance/incremental-next/incremental100-measurement/before-reference`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/shared-list-next/wave4-next/final-acceptance/incremental-next/reference-after-incremental`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/shared-list-next/wave4-next/reference-after-full`
- `tests/work/scope-audit-20260828/vector-scale-100/optimized-100/shared-list-next/wave4-next/reference-after-incremental`
- `tests/work/scope-audit-20260828/vector-scale-100/static-potion/document-vectors.npy`
- `tests/work/scope-audit-20260828/vector-scale-100/static-potion/e5-small-followup/document-vectors.npy`
- `tests/work/scope-audit-20260828/vector-scale-100/static-potion/e5-small-followup/original-query-vectors.npy`
- `tests/work/scope-audit-20260828/vector-scale-100/static-potion/model.safetensors`
- `tests/work/scope-audit-20260828/vector-scale-100/static-potion/original-query-vectors.npy`
- `tests/work/scope-audit-20260828/vector-scale-100/static-potion/superseded-unk-removal/document-vectors.npy`
- `tests/work/scope-audit-20260828/vector-scale-100/static-potion/superseded-unk-removal/original-query-vectors.npy`
- `tests/work/vector-native-query-probe/target`
- `tests/work/vector-research/engine-data`
- `tests/work/vector-research/qdrant-edge-data`
- `tests/work/vector-research/qdrant-edge-data-named`

## 추가 확인

- 같은 합성 원본 조사 폴더의 `.agents/work/vector`에 남은 파생 모델·실행 환경 복제본, 원본 docs와 제어 기록은 보존
- `tests/work/scope-audit-20260828/vector-cli-runtime-iz9955o1/.agents/work/vector`
