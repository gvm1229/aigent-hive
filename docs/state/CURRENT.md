# 현재 상태

- 제품 버전: `0.15.2`
- 공개 안정판: `0.15.1` (2026-11-15). 이 PC 사용자 갱신 실패·Claude·Antigravity 미설치 확인 ([실측](../research/overhaul/OV152-001.md))
- 작업: 0.15.2 호스트 설치 복구. [소유 계획](../plans/active/0.15.2-host-install.md)·[수정 근거](../research/overhaul/OV152-002-005.md)
- 현재 요청: 다섯 호스트의 Hive 플러그인·Skill 확인, 실패 수정 뒤 새 정식 출시. 재정비 단계 5 이후 보류 유지
- 구독자 글: 0.15.1 안내 전송 완료, 0.15.2 안내는 정식 게시와 함께
- 고정 입력: [판별 기준선](../research/overhaul/OVX-001.md)·[검색 기준선](../research/overhaul/OVX-002.md)
- 비추적 조사 2026-11-11.md·2026-11-12.md 편집·커밋·삭제 제외
- [정식 0.15.1 근거와 한계](../research/overhaul/OV151-010.md)·[이전 정식 근거](../research/overhaul/OV150-013.md) 보존

- [최종 CLI·안내 검증](../research/overhaul/stage-4-human-output.md): Rust 1,475개 통과·5개 수동 제외, Python 1,211개 중 1,167개 통과·44개 제외. fmt·clippy·문서·문체·지침·계획·Source Wiki 통과, Windows 밖 실행·실제 설치·Discord 전송 미검증
- [이전 판별 검토·측정](../research/overhaul/stage-4-review.md): 보류 종류 19/20·출력 사실 53/53, 고정 검색 22/40의 당시 근거 보존. 감지기·검색 구현·동결 정답 변경 없음
- 이번 고정 실행 파일 사본 3개·184,858,624바이트 정리. 전체 산출물의 기존 검토 필요 467개·보존 1개·최소 29.52GiB·미측정 21경로는 별도 조치 필요, 임의 삭제 없음


## 생성된 구현·수용 항목

<!-- HIVE:PLAN-STATE:START -->
- 구현 목표: `0.15.2`
- 현재 등록 항목: 6/9 완료

- `agent-owned`: `OV152-006`, `OV152-007`, `OV152-008`
- `awaiting-user-authority`: 없음
- `awaiting-external-evidence`: 없음
- `blocked`: 없음
<!-- HIVE:PLAN-STATE:END -->

- [단계 3 종료 상태·유효한 근거와 한계](../archive/state/0.15.0-closeout.md)
- [기준 main CI 실패·보완](../research/overhaul/stage-4-main-ci.md)
