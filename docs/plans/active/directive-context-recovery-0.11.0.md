# 반복 압축 뒤 지침 복구

> Plan version: 0.11.0
> Scope: product

- 제품: `0.11.0`, 다음 공개 수용: `0.11.0-test.9`
- 근거: [상세 구현안](../directive-context-recovery-0.11.0.md), [공식 조사](../../research/directive-context-hooks-2026-09-25.md)
- 사용자 승인: 2026-09-25 제품 구현. 설치·안정판 공개의 별도 승인 경계 유지

## 기준

- [x] [DCR-001] 승인된 Markdown 구절 선택·내용 지문·경로·출력 상한 검증, 원문 전체 반복·추가 모델 호출 제외
  - state: complete; evidence: repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e
- [x] [DCR-002] Codex 시작·재개·매 압축 복원, 부모 세션 공유·같은 턴 재압축의 누락 방지
  - state: complete; evidence: repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e
- [x] [DCR-003] 경로별 상세 안내·명시적 파일 보호와 기존 변경 검사 결합, 정상 범위 최소 출력
  - state: complete; evidence: repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e
- [x] [DCR-004] 미리보기·정확한 승인·갱신·철회·구버전 설정 보존과 스킬 안내
  - state: complete; evidence: repo:tests/results/runs/20260925T215447-f7c2db4f183c.md#sha256:ce0e26feed4eca13cff58142ea42a09f6c15751b7dd0b2251677e3f84c5c7500
- [x] [DCR-005] 반복 0·1·3·5·10회와 손상·변조·예산·우회 경계 시험, 비용·의미 준수의 증거 분리
  - state: complete; evidence: repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e
- [ ] [DCR-006] 전체 회귀·test.9·실제 반복 압축의 의미 준수와 모델 사용량 수용
  - state: agent-owned; reason: 실제 계측 완료, 수정 코드의 전체 회귀와 다음 공개 시험 수용 필요

## 구현 선택

- 기존 훅 승인의 선택 기능으로 짧은 정본 구절 목록을 등록. 구성은 TOML, 안내 원문은 기존 Markdown
- 명시적으로 승인한 구절의 내용 지문 변경은 재검토 대상. 임의 저장소 내용의 자동 고우선순위 주입 금지
- 동일 이벤트 ID 부재 시 재전달 우선. 전역 세션 완료 캐시나 대화 원문 수집 제외
- 경로별 안내는 해당 편집에서만 전달. 신뢰할 압축 세대 ID가 없으므로 같은 경로의 다음 편집에서 무조건 생략하는 캐시 제외
- 실제 호스트·모델 준수와 CLI 재생 시험을 구분. 다른 운영체제 미실행 결과의 통과 주장 금지

## 현재 증거

- Windows CLI 훅 계약 26개: 25개 통과, 심볼릭 링크 생성 권한이 없는 1개 제외
- 설정·전달·차단의 프로그램 검증. 실제 호스트 압축·모델 준수의 증명은 별도

- test.8 공개 실행 파일의 세 운영체제별 새 훅 계약 각각 26개 통과. 실제 Codex 반복 압축·의미 준수·모델 사용량만 별도 승인 후 확인

- 2026-09-26 Windows Codex 실제 수동 압축 시험: 사용자 훅 4개 신뢰 보고 뒤 새 대화에서 시작·누적 1·3·5·10회 확인. 앱의 완료 사건 총 10개를 직접 조회
- 각 확인 지점의 일반 파일 생성 5/5 허용, 합성 보호 파일 수정 5/5 거부. 보호 파일의 기존 33바이트 유지 직접 확인, 한국어 응답 5/5 유지. 시작 파일 13바이트, 압축 1·3·5회 파일 각 21바이트, 10회 파일 22바이트
- 복원 안내 수신 5/5는 지침 재독 없는 모델 자체 보고. 매 압축 사건의 전달 경로 독립 증명·전체 지침 준수·실제 장기 작업 성능과 구분. 두 번째 압축 등 행동 검사를 넣지 않은 중간 사건은 완료만 확인
- 신뢰 이전 생성 대화는 압축 1회 뒤 합성 보호 편집 허용으로 거부 시험 실패, 안내 전달 미확인. 신뢰 이후 새 대화와의 차이 확인, 내부 원인 미확정. 실패 파일 바이트 보존
- 동일 등록 명령의 직접 합성 호출: 복원 안내 2,351바이트와 보호 거부 확인. 실제 모델 사용량·앱 전달의 대체 증거에서 제외
- 시험 후 공식 철회 완료: `.codex/hooks.json`과 이번 임시 `.hive-context.toml` 부재 확인. `.agents/policy-hooks/codex.json`의 철회 영수증과 합성 시험 파일 보존; 사용자 전역 훅 신뢰 기록 변경 없음
- 남은 근거: 자동 압축·한 턴 내 연속 압축·하위 작업·재개에 대한 실제 호스트 행동, 같은 조건의 기존/전체/선택 전달 비교, 입력·출력·캐시 토큰과 실제 지연. 현재 앱 도구에는 압축 실행·상세 토큰 관측 경로 없음. 10회 수동 시험을 해당 조건의 통과로 대체하지 않는 경계
- 다음: 지원된 호스트 실행·계측 경로로 위 근거 확보 후 DCR-006/RFR-001 최종 판정. 추가 수동 압축 요청 없음, 안정판 완료 주장 제외

- [추가 계측 결과](../../research/directive-context-instrumented-tests-2026-09-26.md): 한 턴 자동 압축 3회·토큰 관측 확인, 재등록 결함 수정과 관련 검사 27개 통과·1개 제외. 신뢰 후 Hive 자동 복원·문구 비용 비교·철회 완료. 수정 제품의 추가 출시 검증 필요
