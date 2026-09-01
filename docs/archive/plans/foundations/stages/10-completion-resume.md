# Stage 10. 완료 보고와 재개

> 상태: implemented workflow reference; 이 stage 검토 시에만 load

#### 완성 후 동작

완료 조건:

- 모든 필수 criterion PASS
- deterministic verification PASS
- 필요한 judge quorum PASS
- active write/effect 없음
- current STATUS와 evidence locator 저장
- Wiki update 또는 skip reason 정산

보고:

- 생성·변경·삭제 artifact
- 실행한 검증과 결과
- judge verdict
- 사용량 상태
- memory 변경
- optional remaining work
- 재개가 필요할 때 정확한 next action

#### 구현

- completion report를 tracked Markdown로 생성
- 같은 report hash 중복 생성 방지
- blocker는 원인, 이미 시도한 안전한 대안과 resume condition 기록
- stale run은 current state를 다시 검증한 뒤 재개

#### 완료 조건

- [x] completion claim과 fresh evidence 1:1 연결
- [x] 새 session이 정확한 next action 복구
- [x] blocked 상태를 succeeded로 표시 금지
