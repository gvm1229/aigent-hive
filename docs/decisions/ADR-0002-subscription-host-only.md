# ADR-0002: subscription host 전용 실행

- 상태: accepted
- 날짜: 2026-07-23

## 결정

Hive는 사용자가 이미 인증한 Codex·Claude Code·Gemini Antigravity host 위에서만 실행. Model API 직접 호출 금지.

## 이유

- 사용자의 정액제 구독 사용
- API credential custody 제거
- provider SDK retry·billing·rate-limit 구현 제거
- host와 external orchestration layer의 session ownership 존중

## 결과

- provider SDK dependency 금지
- API key 질문·설정·환경 전달 금지
- host 인증 실패 시 API fallback 없음
- 모델 호출 retry는 host runtime 책임
