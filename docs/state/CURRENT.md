# 현재 상태

- 제품 버전: `0.14.2`
- 공개 안정판: `0.14.1`
- 현재: 재정비 단계 1b, `fix/overhaul-1-0.14.2`. [활성 계획](../plans/PLAN.md)·[단계 항목](../plans/active/overhaul-0.14.2.md)
- 완료: OV142-001–006. 큰 파일 스트림 비교·자동/실제 prune 회귀·안내문·로컬 검사. [최종 인계 근거](../research/overhaul/OV142-004.md)
- 다음: OV142-007의 test.1 게시 복구와 경로 보완 뒤 test.2 수용, 이후 정식·PC 갱신. 2026-11-12 버전별 전체 승인
- 검사: Windows Rust 1,412개 통과·5개 제외, Python 다섯 묶음의 최종 범위별 검사 통과. 최초 잠금 오류와 종합 기록의 생성 색인 지문 변경은 [회귀 근거](../research/overhaul/OV142-003.md)에 한계 보존
- 문안: 0.14.2 미공개 초안·223자·SHA-256 파일 유지, 검토 보완의 128 MiB 상한 반영
- 공개 시험: test.1 후보 43155047577 성공, 일부 npm 게시 뒤 linux-x64 미공개. macOS 시험 경로 보완 후 test.2 소스 23107415·후보 43162328466·CI 43162324016 진행. [근거](../research/overhaul/OV142-007.md), 최종 공개 수용 미완료
- OV141-011: 정식 게시·Discord 전송 단계·PC 설치 성공, 감시 파일 6개로 조건 실패. [미완료 근거](../research/overhaul/OV141-011.md) 보존, 해결 소유자 OV142-008
- 0.14.1 중단 문서 복구와 develop 통합 완료: `ead55533`, `2b4428bd`. 기존 approval 작업 트리와 비추적 조사 보존

## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.14.2`
- 현재 등록 항목: 6/8 완료

- `agent-owned`: `OV142-007`, `OV142-008`
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

## 보관 근거

- [0.14.1 상태](../archive/state/0.14.1-incomplete.md)·[재정비 할 일](../plans/overhaul/README.md)
