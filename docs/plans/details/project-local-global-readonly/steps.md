# 단계별 구현 인계 후보

[설계](design.md) · [수용·이전](acceptance.md). 모든 단계의 소유자: 다음 구현 담당자. 현재는 구현 승인 대기. 새 제품 버전은 사용자 선정 전 미정.

## KB-P0: 추가 옵션과 기존 기능 보존의 기준 고정

- 선행: 다음 사용자 구현 승인, 현재 소스·계획 재대조
- 읽기: 현재 setup/knowledge/store 경로와 [보존 회귀 계약](compatibility.md). 변경 후보는 새 모드의 분기와 전용 시험, 기존 경로의 동작 수정 제외
- 순서: 공개0.14.0의 가짜 입력·기존 결과 기준 고정 → 미선택·명시 shared·두 방식 공존 비교 절차 작성 → 새 모드에만 경로·권한 적용
- 공통 연결 D1·D2와 기존 공유 수집의 검증 순서 수정은 [별도 작업](../../backlog/knowledge-attachment-consistency.md). 이번 작업의 의존성·완료 조건에서 제외
- 문서: 실제 저장 위치와 새 선택 범위 표시. 기존 모드에서 해제 후 재활성화 방법·승격 의미 변경 금지. 과거 기준 파일 보존
- 검증: 호환성 C1–C5의 기존 기준 실행. 제품 구현 후 같은 입력·선택·의도·부수 효과 재대조
- 완료: 기존 경로 변경 없이 추가할 위치와 보호 시험 확정. 공통 결함 해결·새 모드 제품 완료 주장 제외

## KB-P1: 설정과 중앙 권한

- 선행: P0, [설계](design.md)의 단일 모드·호환 계약
- 읽기·변경: `schemas/setup-answers.schema.json`, `hive-render/src/lib.rs`의 답안·유효 설정·투영, `harness/setup/questions.yml`, 프로젝트 질문·파일 입력·기존 설정·활성화 모듈
- 새 중앙 모듈 제안: `hive-core/src/knowledge_policy.rs`, `hive-cli/src/knowledge/context.rs`. 목적: 검증된 프로젝트·읽기 출처·쓰기 대상·정책 지문 결정
- 순서: 기존 입력은 재기록 없이 기존 shared 의미 유지 → 명시 local 선택만 새 판 저장 → 기존 정책 변경은 미리보기 승인 확인 → 모드 결정 뒤 진행 저장 위치 선택 → local 모드의 모든 쓰기 진입점에 결합 권한 요구
- `user_setup.rs::project_preferences`, `project/file_input.rs::file_scope`, `ProgressStore::open`, `activation::apply`의 사용자 루트 필수·전역 진행 기록 의존 분리. 전역 부재 시 로컬 명시 설정 허용
- `main.rs::reconcile_project_registry`: local 모드에서는 전역 등록 호출 없음. `--dry-run`은 실제 예정 경로·접근권한·부수 효과 표시
- 시험: 명시 target·현재 폴더·다른 user-root·인자 생략·요청 위조·정책 변경·경로 교체·재개 불일치의 실패 전 쓰기 0건. 기존 schema 1·2 기본값 회귀
- 완료: local 모드에서 전역 쓰기 인터페이스 선택 불가, shared 모드는 기존 공개 API·권한·명령 인자 유지. 경로 선택만으로 기능 완료 표시 금지

## KB-P2: 로컬 정본·파생 자료·작업 경로

- 선행: P1
- 변경: `hive-wiki/src/store.rs`, `shared.rs`, `rag.rs`, `store/freshness.rs`, `knowledge_lock.rs`; CLI `knowledge.rs`, `knowledge/remember.rs`, `knowledge_scan.rs`, `knowledge_transfer.rs`
- 순서: 사용자 루트 전용 자료 구성과 게시 수단 분리 → 로컬 self collection의 ID 보존 → 로컬 Claims·Wiki·Raw 게시 → 로컬 색인·변경 기록·잠금 → CLI 수집·기억·검색·관리 연결
- 파생 기능: `hive-cli/src/vector.rs`, `knowledge/semantic_graph.rs`, `hive-wiki/src/vector.rs`, `semantic_graph.rs`, `graph.rs`의 로컬 권한·출력 경로 연결
- `scan --apply`의 로컬 검토 저장 유지, 전역 자동 승격 시도 거부. 삭제·억제·이전·수리·느린 초기화 경로도 동일 권한 사용
- 세션: `session.rs`의 기존 프로젝트 내부 경로 유지. 설정·업그레이드·상태·세션 조합에서 불필요한 전역 동기화가 없는지 실행으로 확인
- 시험: 가짜 한국어 ingest/capture/query, 중복·교체 이력, 전체 로컬 색인 재구축, 그래프·벡터의 동의·실패·중단·재개. 전역 쓰기 시도 계측 포함
- 완료: 관련 결과와 중간 자료 전부 프로젝트 아래, 전역 읽기 없이도 로컬 기능 사용 가능. 전역 쓰기가 필요한 복구로의 자동 폴백 금지

## KB-P3: 전역 스냅샷과 통합 조회

- 선행: P2
- 변경: 새 `hive-wiki/src/read_snapshot.rs` 제안, 기존 store 검색 검증·`rag` 검색·CLI `knowledge/retrieve.rs` 재사용
- 순서: 무쓰기 파일 읽기 객체 → 게시 상태·출처 검증 → 다시 읽기 한도 → 기존 공개 범위 필터 → 두 결과의 메모리 합산·예산·출처 표시
- 새 모드의 query/list/read만 같은 검증된 읽기 객체 사용. 기존 shared 조회·잠금·오래된 색인 의미 유지, 새 모드의 최신성 진단과 구분
- 전역 부재·오래됨·손상·부수 파일·형식 불일치의 이유별 출력. 전역 필요 요청은 실패, 자동 범위는 로컬 결과와 축소 진단
- 시험: 읽기 전용 ACL 환경과 쓰기 시도 감시, 바이트·속성·디렉터리 사건 비교. 성공·실패·재시도·동시 전역 게시·프로세스 중단 모두 포함
- 완료: 전역 잠금·승인 기록·측정·캐시를 포함한 Hive 기인 전역 쓰기 0건. unrelated 프로젝트 검색에 로컬 표식 0건

## KB-P4: 이전·안내·수용

- 선행: P3, [수용·이전](acceptance.md)과 [기존 동작 보존](compatibility.md)의 전체 조건. 공통 연결 상태 수정은 선행 조건에서 제외
- 변경: 프로젝트 업그레이드·재설정·설정 미리보기·요약, CLI 도움말, 생성 README·프로젝트 지침, `harness/skills/knowledge-*`, project/user setup·refresh 및 관련 스킬
- 정본 스킬에서 수정 후 기존 투영 도구로 각 호스트·플러그인 사본 생성. 별도 새 스킬 목록·역사적 기준 파일 편집 금지
- 출력: 유효 모드, 로컬 정본·파생 경로, 전역 읽기 출처·상태·권한, 예정 쓰기, 해제 상태. `configuration valid`와 실행으로 확인한 무쓰기의 명확한 구분
- 선택적 이전의 preview/apply/recovery와 9개 수용 묶음 연결. 기존 shared·인증된0.7 이하 호환 경로 회귀 포함
- 완료: 새 버전의 번호 공개 시험에서 지원 운영체제·호스트별 실제 결과 기록. 문서·스키마 성공만으로 제품 경계 통과 처리 금지
- 출시·설치: 구현 승인과 별개인 해당 버전의 사용자 명시 승인 필요. 실제 지식 이전도 별도 승인 전 미실행
