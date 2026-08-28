# 고정 외부 소스

## process-wrap 9.1.0

- 원본 패키지 전체·라이선스·저작권 보존. [출처와 파일 지문](process-wrap-provenance.json)
- 수정: Windows 동기 Job 생성의 `make_job_object(handle, false)`를 `true`로 변경
- 목적: Job에 할당된 자식이 부모 Hive의 강제 종료 뒤 남는 결함 수정
- 보장 한계: 프로세스 생성과 Job 할당 사이의 중단 구간은 별도
- 원본의 비활성 선택 기능에서 생기는 unused 경고는 원본 보존 대상, 추가 코드 수정 없음
- upstream 동기 API의 같은 종료 계약 지원 뒤 재검증하여 경로 패치 제거
- 구현 배경: [벡터 검증 기록](../docs/research/vector-product-integration-2026-08-28.md#windows-명령-강제-종료의-자식-잔존-수정-계획)

이 폴더의 원본 README·변경 기록·라이선스는 외부 저작물이며 Hive 문체로 재작성하지 않는 보존 자료.
