# Raw

원본 또는 정규화한 source material 보관 위치.

- 기존 source를 조용히 수정하지 않음
- 변경된 source는 새 revision으로 수집
- 폐기된 source는 active tree에서 삭제
- 재유입 방지는 `../suppression.yml`의 최소 fingerprint로 처리
- 기밀정보와 credential 저장 금지
