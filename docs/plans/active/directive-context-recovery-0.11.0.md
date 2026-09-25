# 반복 압축 뒤 지침 복구

> Plan version: 0.11.0
> Scope: product

- 제품: `0.11.0`, 다음 공개 수용: `0.11.0-test.8`
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
  - state: complete; evidence: repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e
- [x] [DCR-005] 반복 0·1·3·5·10회와 손상·변조·예산·우회 경계 시험, 비용·의미 준수의 증거 분리
  - state: complete; evidence: repo:docs/research/directive-context-implementation-0.11.0.md#sha256:eb38786a9b205550a61a3a9a52f98aac7766b04999dbd31fcdd8bd51e6da2e6e
- [ ] [DCR-006] 전체 회귀·test.8·실제 반복 압축의 의미 준수와 모델 사용량 수용
  - state: awaiting-user-authority; owner: maintainer; reason: 실제 훅 임시 활성화와 새 Codex 검증 대화 승인 대기

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
