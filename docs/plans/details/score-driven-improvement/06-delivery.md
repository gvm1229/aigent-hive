# 06 — 출하·호환·실제 수용

- 상위: [최종 계획](../../score-driven-improvement-0.14.0.md)
- 소유 항목: QSI-010·011
- 아래 시험은 구현 단계의 실행 대상

## QSI-010 — 소비자 하네스와 소스 개발에 제공

- 선행: QSI-003·006·009
- 기존 수정: harness/skills/verified-workflow/SKILL.md, run-checkpoint/SKILL.md, knowledge-capture/SKILL.md, knowledge-promote/SKILL.md
- 신규: harness/skills/verified-workflow/references/quality-evaluation.md
- 신규: harness/skills/run-checkpoint/references/quality-report.md
- 신규: harness/skills/knowledge-capture/references/improvement-experience.md
- 신규 공통 자료: harness/evaluation/profiles/의 네 평가표, 해당 스키마·설명
- 기존 수정: crates/hive-projection/src/lib.rs의 참조 파일 투영, crates/hive-render/src/lib.rs·activation.rs의 소유 목록·검증
- source 연결: .agents/directives/03-workflow.md의 선택형 평가 경로와 소스 전용 저장 경계
- 문서: 신규 docs/guides/score-driven-improvement.md; 제품 사용 안내는 소비자에게 필요한 명령·결과·한계만 포함

### 투영·활성화

- 정본은 harness/. 기존 생성 경로를 통해 plugins·template의 필요한 투영 동기화
- 소비자 기본 평가표 위치 .hive/evaluation/profiles/, 소유권 목록에 등록
- 사용자 정의 평가표는 명시한 별도 파일을 compile 입력으로 사용, 제품 갱신의 자동 덮어쓰기 제외
- 신규 스킬 이름·별도 분류기·항상 실행되는 평가 훅 추가 없음
- 기본은 평가를 시작하지 않는 상태. 명시적 평가 요청에는 observe, 승인된 실행과 교정·호스트 증거가 있을 때만 enforce
- 하네스 자동 실험은 실행별 명시 활성. 단순 질문·기존 프로젝트 설정·업데이트에 모델 호출이나 새 필수 질문 추가 없음
- 소비자 설치·갱신 시 평가표 자료 제공과 자동 실행 활성화를 구분
- 소스 개발은 같은 계산기·평가표 사용, .hive 없이 SourceQualityStore로 기록
- 사용자 안내: 두 자리 점수·미채점·참고·완료·미완료 구분, 불필요한 내부 설명 제외

### 검사·완료

- cargo test -p hive-projection
- cargo test -p hive-render
- python -m unittest tests.conformance.contracts.test_skill_projections tests.conformance.contracts.test_hive_skills
- 계약 시험: 기본 자동 모델 호출0회, 소스와 소비자 점수 일치, 사용자 파일 보존, 현재 투영 일치
- 완료: 배포 자료만으로 명시적 평가 사용 가능, 설정만으로 실행 자격을 입증했다고 표시하지 않는 결과
- 실패 복구: 기존 설치 보존, 새 선택형 기능만 비활성·구체적 원인 보고

## QSI-011 — 자료 호환·갱신·동시 변경

- 선행: QSI-002·006·010
- 기존 변경 범위: loop-graph.schema.json의 v1/v2 분기, CLI VerifierEnvelope의 v1/v2/v3 분기, projection·activation의 새 자료 등록
- 기존 run-status·host-orchestration-receipt·Judge 판정 형식의 바이트 계약 유지
- 새 품질 데이터는 선택형 연결과 별도 자료. 구형 실행에 품질 필드를 강제로 삽입하지 않는 이전
- 신규 tests/conformance/integration/test_quality_upgrade.py를 integration lane 등록
- 신규 fixtures: tests/fixtures/quality/legacy/, hostile/, resume/

### 필수 호환 사례

| 사례 | 기대 결과 |
| --- | --- |
| 기존 품질 기능 없는 실행 | 기존 판정·복구 유지, 자동 채점 없음 |
| v1 graph에 품질 필드 위장 | 거부·무변경 |
| v2 품질 필수 graph에서 gate 제거 | 거부·무변경 |
| 구형 CLI로 새 graph 읽기 | 명확한 미지원, 품질 조건을 버린 실행 금지 |
| 평가표·모델·산출물·범위 변경 | 이전 교정·결과 무효화, 재확인 필요 |
| 새 평가표를 포함한 제품 갱신 | 기존 사용자 평가표·경험·역할·설정 보존 |
| 적용 직전 사용자 편집 | 예상 지문 불일치로 거부 |
| 일부 파일 변경 뒤 실패 | 기존 transaction의 정확 복구, 미복구 증거 보존 |
| 취소·재개·늦은 결과 | 공통 epoch·영수증 계약, 중복 적용0건 |

- python -m unittest tests.conformance.integration.test_quality_upgrade
- 기존 보안·설치·업그레이드·Judge·loop 관련 회귀를 변경 범위에 맞춰 함께 실행
- 완료: 예전 실행의 무변경 호환과 새 조건의 무시 불가. 실패 시 새 활성화만 중지

실제 호스트·운영체제·공개 시험 수용은 [QSI-012](07-qualification.md) 소유.
