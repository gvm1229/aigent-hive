# Graphify 잔여 범위 0.11.2

> Plan version: 0.11.2
> Scope: product

기존 코드 전용 추출·Markdown 관계·벡터 검색의 완료 유지. 미완료 확장만 소유. [조사](../../research/followup-feasibility-0.11.2.md)·[결정](../../decisions/ADR-0025-0.11.2-scope.md).

## 기준

- [ ] [GPH112-001] 고정 후보의 증분/전체 동등성과 모음별 권한 격리
  - state: agent-owned
- [ ] [GPH112-002] 호스트 소유 문서 의미 추출과 출처 검증·가져오기
  - state: agent-owned; depends: GPH112-001
- [ ] [GPH112-003] 30개 관계 질문·고유 5만 청크 성능·복구의 Windows 수용
  - state: agent-owned; depends: GPH112-001,GPH112-002

## GPH112-001

- 읽기: `crates/hive-wiki/src/graphify.rs::normalize_graphify_code`, `graph.rs`, `scripts/qualify-source-graph.py`, `scripts/build-graphify-wheel-lock.py`, [기존 실패](../../research/graphify-0.10-feasibility.md)
- 첫 조사: 조회된 0.9.76 후보의 정확한 배포 버전·커밋·라이선스·의존 파일 확정. 가변 브랜치의 현재 숫자만으로 기존 0.9.47 교체 금지
- 시험: 기존 함수 추가 표본에 추가·수정·삭제·이름 변경 사례 보강, 증분/전체 결과의 노드·관계·정규화 경로 동등성 확인
- 결정: 후보 통과 시 고정 의존 파일로 갱신. 실패 시 Hive 소유의 범위별 전체 재생성 또는 결정적 갱신 설계를 비교하고 구현 전 이 단계 갱신
- 격리: source·shared·project-private·confidential별 물리 세대와 조회 권한 검사. upstream `global`의 단일 공유 그래프 사용 금지
- 완료: 정확한 의존 파일과 Windows의 동등성·경로 탈출·권한 누출 반례 0건. 실패 후보는 출하 제외, 기존 정상 기능 유지

## GPH112-002

- 최종 결정: 활성 범위의 `after-capture` 자동 분석. 원문·색인 저장 완료 뒤 `graph_update`의 `unchanged|pending|disabled`와 변경 지문 제공
- 공개 명령: 기존 graph 명령에 `prepare|apply`·`host-semantic` 엔진 추가, Source Wiki와 소비자 범위 분리
- prepare: 권한 확인 문서 최대 10개·본문 합계 16KiB·근거 위치·변경 지문. 분석 1회·형식 교정 1회
- apply: 현재 원문·권한·입력/결과 지문·근거 위치 재검증 뒤 완성 세대만 원자 활성화. 관계는 파생 자료, 원문 재저장 금지
- 동일 지문 무작업·문서별 최신 변경 합치기. 분석 실패·사용량 제한은 대기 보존, 다음 허용된 지식 처리에서 재개
- 삭제 후 상태·조회는 현재 권한과 지문이 맞는 관계만 반환. 분석 문서가 없는 정리는 `cleanup_required`로 표시, 빈 결과 적용으로 파생 자료만 정리
- `remember`뿐 아니라 명시적 파일 수집의 `ingest/add` 성공에도 `graph_update` 반환. 색인 완료 뒤 실제 모음·공개 범위에 연결, 동일 원문·Wiki 재수집은 새 분석 제외

- 읽기·변경: 기존 graph 세대·출처·영수증 검증, `harness/skills/knowledge-scan/`의 호스트 소유 자료 검토 경계. 정확한 명령·자료 형식은 첫 설계 검토에서 확정
- 방식: 활성 호스트가 제한된 문서에서 근거 구절·관계 후보·출처 지문 생성, Hive가 검증 후 파생 그래프에 가져오기. Hive의 모델 호출·API 키·새 추론 서버 제외
- 자료: 모음·문서 상대 위치·내용 지문·관계 종류·근거 위치·검토 상태. 추정 관계와 명시 관계 구분, 검토 전 사실 승격 제외
- 반례: 원문 지시문 주입, 출처 부재·오래된 원문·권한 변경·다른 모음·허위 관계. 현재 승인·정본 확인 실패 시 활성 세대 변경 0건
- 완료: 실제 호스트가 만든 제한된 합성 문서 결과와 가져오기 검증. 원문·FTS 무변경, 실패 시 이전 정상 세대 유지

## GPH112-003

- 기존 30개 질문: source·소비자·전역 각 10개. 의미 추출·코드 경로·직접 사실의 기대 결과를 구현 조정 전 고정
- 고유 50,000청크: 중복 문서 반복과 구분. 생성·조회·증분·메모리·디스크·관계 정답률 측정
- 수치 기준: 직접 사실 30/30·근거 관계 27/30 이상·새 CLI 조회 p95 2초 이하. 고유 5만 청크의 생성 시간·RAM·디스크 별도 측정, 측정 뒤 기준 완화 금지
- 복구: 중단 재개·삭제 반영·원문 변경·손상 색인·도우미 부재, Markdown·FTS 보존과 기존 버전 갱신 확인
- Obsidian: 기존 Markdown 열기와 내부 링크 사용 안내, Hive의 출처·권한과 일반 그래프의 차이 명시. 별도 플러그인·설정 변경·동기화 제외
- 완료: Windows의 전체 근거와 Linux 공개 검사 인계. macOS는 QLF112-005의 최종 수용

## 보존

- sqlite-vec·MiniLM 구현의 재개나 엔진 재선정 제외
- [엔진 비교](../../research/vector-memory-0.10-feasibility-2026-08-22.md)와 초기 실패·최종 채택 기록 보존
- 기존 Graphify 코드 전용 동작·명시적 Markdown 그래프의 회귀 금지
