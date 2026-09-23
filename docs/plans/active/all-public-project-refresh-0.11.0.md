# 모든 공개 버전의 프로젝트 갱신 보장

> Plan version: 0.11.0
> Scope: product

## 범위

- 최초 공개 안정판 0.9.1부터 현재 목표 이전의 모든 공개 안정판 자동 열거, 향후 출시에도 누락 거부
- 각 버전의 갱신·사용자 수정 보존·실패 복구 검증. 기준 자료 존재와 실행 성공의 구분
- 기존 사용자 파일·동결 기준본 보존. 제품 변경 시 다음 번호 공개 시험판 검증, 안정판 게시 제외
- 검증 완료 뒤 모든 작업 브랜치의 커밋을 develop에 보존하고 로컬·원격 main·develop만 유지

## 완료 기준

- [x] [APC-001] 모든 공개 이전 버전의 자동 열거와 누락 차단
  - state: complete; evidence: repo:tests/results/runs/20260923T040856-d5791cfd29fa.md#sha256:274243f9e83d6b4be05fa44e9cc6acb253eb9c0143284d6821d3d32095772cfc
- [x] [APC-002] 버전별 실제 갱신·보존·실패 복구와 지속 검사 연결
  - state: complete; depends: APC-001; evidence: repo:tests/results/runs/20260923T040856-d5791cfd29fa.md#sha256:274243f9e83d6b4be05fa44e9cc6acb253eb9c0143284d6821d3d32095772cfc
- [ ] [APC-003] 전체 회귀·독립 검증과 공개 산출물 검사
  - state: agent-owned; depends: APC-002
- [ ] [APC-004] 작업·커밋 손실 없는 develop 통합과 로컬·원격 브랜치 정리
  - state: agent-owned; depends: APC-003

## 검증 경계

- [검증 설계와 범위](../../research/all-public-project-refresh-0.11.0.md)

- 검사한 운영체제·실행 파일·출발 버전·실행 단계와 실패·제외 사유를 결과에 기록
- 전체 공개 안정판 목록은 태그에서 도출. 범위 축소·누락·필수 기준본 부재는 통과 금지
- 원래 브랜치별 끝 커밋의 develop 포함 여부 확인 후 삭제. 미커밋 변경·작업 폴더·원격 신규 변경 재확인
