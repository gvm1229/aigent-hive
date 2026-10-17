# 0.12.0 새 섹션 공지 전송 확인

- 사용자 승인: 새기능·수정·개선·구분선의870자문구를기본으로확정, develop병합뒤새Discord공지 전송
- 실행: develop605401aec8df6dabadac8ce5a737d1d605353fd5의37454909640 성공. Ubuntu CI 보호환경에서공개정식0.12.0·성공게시37396583273 확인, 실제전송두요청수락
- 내용: 승인한새지문aaeb1a62d22fb6c1e839d6d877d4d8d6618a71194bc4081004651975c7d2d574와870자대조. 출시소스a257b4157048b07b40c53ce0f2d50542267772f5는원래값유지, 새문구소스는develop실행SHA
- 중복: 새지문영수증artifact1개, 지문sha256:338397cad4be0204029008278439425f24366a2b89ae3063e0279f8b210a6d00. 같은문구추가발송 거부조건충족
- 경계: 기존829자공지와영수증은이력보존. 새발송은기존메시지수정·제품/태그재게시·새안정판/시험판·설치가없는독립공지. 실제구독자열람의증명 제외

```json
{
  "schema_version": 1,
  "status": "sent",
  "product_version": "0.12.0",
  "publication_run_id": 37396583273,
  "notification_run_id": 37454909640,
  "release_source_sha": "a257b4157048b07b40c53ce0f2d50542267772f5",
  "summary_source_sha": "605401aec8df6dabadac8ce5a737d1d605353fd5",
  "approved_revision": true,
  "summary_sha256": "aaeb1a62d22fb6c1e839d6d877d4d8d6618a71194bc4081004651975c7d2d574",
  "banner_sha256": "5fb5bf571bf8931230e536092c6788d2cc6f7ef7d55ef8f173d83d20f18fc3a1",
  "summary_characters": 870,
  "accepted_requests": 2,
  "webhook_recorded": false,
  "limits": "Discord accepted banner and summary requests; recipient reading is not verified."
}
```
