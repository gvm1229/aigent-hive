# Stable release Discord 구독자 알림

- 상태: 완료
- 제품 version: 다음 안정판부터 적용
- 범위: `release-publication` 환경의 안정판 GitHub Release 뒤 Discord webhook 두 번 전송. 시험판·consumer harness·provider API 범위 제외

## Checklist

- [x] [RDN-001] `update-summary` 결과를 안정판별 한국어 구독자 메시지 정본으로 저장하고 제목·사용자 체감 개선 항목 검증
  - Evidence: `docs/releases/0.9.4.subscriber.ko.md`, `update-summary` stable-release output contract, notifier payload acceptance
- [x] [RDN-002] 배너 PNG 전송 성공 뒤에만 한국어 요약 메시지를 전송하는 fail-closed notifier 구현
  - Evidence: `scripts/publish-stable-discord-update.py` multipart banner request·subsequent JSON summary request
- [x] [RDN-003] `release-publish.yml` stable 분기에서 GitHub Release 생성 성공 뒤 `release-publication` 환경 비밀 값을 notifier에 한정 전달
  - Evidence: stable-only preflight before npm publication, stable-only notifier after `gh release create`
- [x] [RDN-004] 안정판 전용·이미지 우선·첫 요청 실패 뒤 요약 미전송·비밀 값 출력 금지 회귀 검증
  - Evidence: local mock Discord `unittest` 7개 PASS. 보호된 GitHub 환경 실제 전달 시험
    `0.9.3` [run `31775328374`](https://github.com/gvm1229/aigent-hive/actions/runs/31775328374)·`0.9.4`
    [run `31775377264`](https://github.com/gvm1229/aigent-hive/actions/runs/31775377264) 각각 성공. 각 실행의
    배너 우선·한국어 요약 후속 단계 성공, 안정판·npm 게시 `0건`
- [x] [RDN-005] 출시 운영 안내·결정·현재 상태·bilingual fact·Source Wiki 갱신과 문서 검증
  - Evidence: release guide·decision·current state·fact pair, human documentation·Markdown link·Source Wiki gates

## Acceptance

- 안정판별 Discord 메시지 수: 배너 1건·한국어 구독자 요약 1건
- 첫 요청 실패 시 두 번째 요청 `0건`
- 시험판 채널의 Discord 요청 `0건`
- webhook URL: GitHub `release-publication` 환경 비밀 값 한정, 저장소·로그·출력 노출 `0건`
