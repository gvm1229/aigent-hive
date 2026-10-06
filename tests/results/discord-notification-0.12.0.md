# 0.12.0 누락 공지의 실제 전송 확인

- 사용자 명시 전송 요청: 2026-10-06. 최초누락은 현재Codex의false 선택, 기존게시의 실제전송skipped
- 실행: 보호PR74·CI37449900707 통과, main12654d82187913ff754bcddb2116b39e34698061의 공지전용workflow37450946771 성공. 실제전송은 Ubuntu CI의 보호환경에서 실행, 현재Windows·Codex는영수증검증
- 대상: 공개정식0.12.0·성공게시37396583273·출시소스a257b4157048b07b40c53ce0f2d50542267772f5 대조. 안내문승인·지문·829자·배너지문과중복영수증부재 검증후전송
- 결과: 기존도구의배너먼저·안내문다음 두HTTP요청을 Discord가수락. 모든검증·전송·영수증업로드단계success, 문구·제품·패키지·정식태그 재게시 없음
- 예방: 안정판의전송생략/false를첫단계에서거부, 미승인안내문의외부지문검증은npm게시앞필수. 실제Bash4사례와로컬HTTP를포함한25개시험통과
- 한계: 서버수락 증명, 구독자실제열람·설치 증명 제외. webhook값·본문·대화원문 기록 제외

```json
{
  "schema_version": 1,
  "status": "sent",
  "product_version": "0.12.0",
  "publication_run_id": 37396583273,
  "notification_run_id": 37450946771,
  "release_source_sha": "a257b4157048b07b40c53ce0f2d50542267772f5",
  "summary_sha256": "69eada69306704dcf292ffea1ea29dfe7dfab7239f65f4a5e2dffd7a7dc40156",
  "banner_sha256": "5fb5bf571bf8931230e536092c6788d2cc6f7ef7d55ef8f173d83d20f18fc3a1",
  "summary_characters": 829,
  "accepted_requests": 2,
  "webhook_recorded": false,
  "limits": "Discord accepted banner and summary requests; recipient reading is not verified."
}
```
