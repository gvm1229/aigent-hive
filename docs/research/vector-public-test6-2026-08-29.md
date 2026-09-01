# 벡터 포함 공개 시험판 수용

## 결과

- 공개 시험판: `0.10.0-test.6`, 제품 `0.10.0`, 배포일 2026-08-29
- 정확한 소스: `d331dc879cf51eab078c5e189b2fe7b8d729e541`
- [전체 CI](https://github.com/gvm1229/aigent-hive/actions/runs/33190877319), [후보](https://github.com/gvm1229/aigent-hive/actions/runs/33191658413), [게시](https://github.com/gvm1229/aigent-hive/actions/runs/33193036233), [공개 설치 수용](https://github.com/gvm1229/aigent-hive/actions/runs/33193323751) 통과
- [GitHub 공개 시험판](https://github.com/gvm1229/aigent-hive/releases/tag/v0.10.0-test.6): 비공개 초안 아님, 시험판 표시 확인
- npm 정확 버전·시험 채널 `0.10.0-test.6`, 안정 채널 `latest=0.9.5` 독립 확인
- 안정판 `0.10.0` 게시·설치·`main` 통합 없음
- 파일 지문과 운영체제별 증거: [수용 대장](evidence/vector-public-test6-2026-08-29.json)

## 실제 실행 범위

| 환경 | 공개 바이너리 | 결과 |
| --- | --- | --- |
| Windows Server 2025 x64 | `win32-x64` | 설치·한국어·벡터·할당된 실제 자식 취소/복구 통과 |
| macOS 15.7.7 arm64 | `darwin-arm64` | 설치·한국어·벡터 통과 |
| Ubuntu 계열 x64·glibc Python | `linux-x64` musl CLI | 설치·한국어·벡터 통과 |

- 공통: 정확한 설치 동의, 실제 MiniLM·sqlite-vec, 의미 인용, 무변경 재사용, 손상 시 FTS 복귀, 비활성화, 이전 세대 복구
- 지식 보존: 원본·FTS 바이트 유지, 묶음 파일의 벡터 제외, 새 로컬 루트에서 원문 8개 복원·벡터 신규 생성
- 공유 모음 3개·24청크: 물리 분리와 증분 1개 재계산, 원본·FTS 보존
- 소스 81청크: 첫 실행과 새 생성 2회의 시간 제한 재개 관찰, 완성·원본 보존
- 한국어: 활성 규칙 반영, 잘못된 규칙 거부, 검증 승인/거부, 이전 언어 팩 복구, 제공자 API·키 접근 0건

## 증명 범위와 한계

- 독립 벡터 실행기의 `public package identity` 미증명 표시는 원본 그대로 보존. 바깥 공개 수용 절차의 npm 버전·채널·태그·설치 바이너리 결합으로 공개 패키지 신원 확인
- 새 루트 가져오기는 각 운영체제에서 실제 실행. 동일 묶음 파일을 서로 다른 물리 기기 사이에 전송하는 실험과 구분
- 이 공개 시험은 소규모 기능 수용. 5만 개 성능·비공개/기밀 격리는 [별도 보존 근거](vector-product-integration-2026-08-28.md)와 [승인된 성능 정책](vector-acceptance-2026-08-29.md) 적용
- Unix의 실제 모델 자식 취소와 Windows 프로세스 생성→Job 할당 사이의 원자성은 이번 공개 시험에서 미증명. Windows의 이미 할당된 실제 자식 종료·기존 세대 보존·복구만 직접 관찰
- Intel Mac·Linux arm64 CLI 후보 빌드 통과와 벡터 설치 수용은 별개. Alpine의 musl Python도 현재 벡터 지원 대상 밖
- 사용자 전역 지식·실제 소비자 프로젝트·전역 설치 변경 없음. 모든 수용 자료는 격리된 합성 자료

## 검사와 시행착오 보존

- 최종 Windows 전체 Rust 893개 통과·수동 실행 전용 4개 제외
- Windows 전체 Python 789개: 748 통과·운영체제/권한 조건 41개 제외·실패/누락 0개. 건너뛴 항목은 다른 환경의 성공으로 대체 집계 금지
- 정확 목록 대조: `tests/work/scope-audit-20260828/vector-test6-reconciled-tests.json`
- `test.5` 후보 `33188952482`: 게시 전 취소. 초기 CI의 시험 폴더·Unix 경로·코드 검사·문서 검사 오류 보존, 수정 뒤 최종 CI 통과
- 직접 공개 수용 dispatch의 기본 브랜치 미등록 404: 소스 변경 없이 기존 `release-runtime.yml`의 공개 시험 입력으로 같은 수용 절차 실행
- 현재 상태의 이전 기록: [보존본](../archive/state/0.10.0-vector-before-test6.md)
- 모델의 요청 사이 메모리 유지는 선택적 성능 개선으로 생략. 기능·안전 기준 변경 없음
