# 벡터 사용 안내와 사용자 답변

> Checklist owner: `VON10-*`
> 대상: `0.10.0`의 선택형 의미 검색 안내·설정·재수용

## Checklist

- [x] [VON10-001] 전역 사용자 기능 답변 정본과 `status|claim|answer|prompt` CLI 계약
- [x] [VON10-002] 신규 빠른·사용자 지정 설정의 마지막 벡터 질문
- [x] [VON10-003] 갱신 뒤 최초 지침 로드의 단일 질문과 세 host 투영
- [x] [VON10-004] yes·no·미응답·동시 세션·재설치·직접 갱신 보존
- [x] [VON10-005] 전역·기존 공유 모음 고정, 비공개·기밀·새 모음 제외 안내문
- [x] [VON10-006] 미리보기 동의·지원 Python·설치·갱신·재개·FTS 복귀 연결
- [ ] [VON10-007] 구조·보안·upgrade·projection·실행 회귀와 다음 공개 시험 수용
- [x] [VON10-008] 사용자용 `0.10.0` 업데이트 내역 검토 초안과 release-note 문체 검증

## 결정

- 기존 `user-setup.yml`은 변경하지 않고 `.hive/config/user-feature-answers.yml`에 답변 저장
- 답변: `unanswered|yes|no`; 사용 의사와 runtime·색인 준비 상태 분리
- 기본 대상: user-root와 현재 등록된 shared 모음; project-private·confidential·새 모음 자동 포함 금지
- yes 뒤 새 세션에 제출하는 고정 범위·동의 지문 안내문이 설치 실행 권한
- no는 재질문과 자동 의미 검색 권유만 중지. 원본·FTS·기존 파생 자료 삭제 금지
- 제품 변경이므로 `0.10.0-test.6` 수용은 재사용 불가. 다음 공개 시험 수용 필요

## 구현 근거

- `.hive/config/user-feature-answers.yml`: `yes|no` 답변, 도입 version, 답변 시각
- 15분 local claim과 compare-and-swap: 동시 세션의 중복 질문 방지, host session 식별자 저장 없음
- `prompt`: user-root·연결된 shared 모음의 고정 목록과 `setup_request_digest`; private·confidential·새 모음 제외
- 재설치·복구의 명시 보존 목록에 사용자 기능 답변 추가, 기존 `user-setup.yml` 바이트 불변
- Windows local Rust 5개·Python CLI/결과 형식/세 host source 투영 8개 통과. 전체 세 host 설치 투영·운영체제 공개 수용: `VON10-007`에서 재확인
