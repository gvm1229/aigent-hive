# 여러 지식 묶음 `test.8` 수용

## 배포 식별

- 제품: `0.10.0`; 공개 패키지: `0.10.0-test.8`; 배포일: `2026-08-31`
- 제품 소스: `6d33557c0dec41b5df8f0ae82c2db0f97f58f312`
- [구현 CI](https://github.com/gvm1229/aigent-hive/actions/runs/33381544513): 전체 필수 검사·세 방향 다중 묶음 수용 통과
- [후보](https://github.com/gvm1229/aigent-hive/actions/runs/33382232083): 다섯 native 대상·npm 묶음 통과
- [게시](https://github.com/gvm1229/aigent-hive/actions/runs/33383120615): 정확 후보 파일의 test 채널 게시 통과
- [공개 설치](https://github.com/gvm1229/aigent-hive/actions/runs/33383382234): Windows x64·macOS arm64·Linux musl x64 통과
- npm: `test=0.10.0-test.8`, `latest=0.9.5`
- GitHub: [`v0.10.0-test.8` 시험판](https://github.com/gvm1229/aigent-hive/releases/tag/v0.10.0-test.8), 태그 소스 일치
- 안정판 tag·게시·설치·`main` 통합·Discord 전송 없음

## 여러 묶음 교차 운영체제 수용

| 생산 | 수신 | 최종 원문 | 자동 중복 정리 | 원문·FTS |
| --- | --- | ---: | ---: | --- |
| Windows | macOS | 100 | 11개 | 일치·통과 |
| Windows | Linux | 100 | 11개 | 일치·통과 |
| macOS | Windows | 100 | 11개 | 일치·통과 |

- 수신 영수증 SHA-256: macOS→Windows `41b64f2fdfcef4fb27d52279cf849ed201670fc4089f4bd7ce31e5665b71d7ee`, Windows→macOS `982c0fdec51831f03786567718c38d790c457f3430516c0769943b00e83fd248`, Windows→Linux `102cf0e24c3732d08205905d6e74f03e3974f9bddab1a074df42fe368e824d4c`
- 공개 설치 native 영수증 SHA-256: Windows `5c727881fc2f1e64629debee2993a66a154419af85ab0dc327ce3eadcf9c4392`, macOS `b24b0e815f56f12560b2bf1199e0f66b645f40367f9bbc19f6eb9afc278d2e60`, Linux `0f182a1eeda80b6e3763adb8d617c1f8897e83ae40589c8fb28aa062dc008d90`

## 증명 범위와 한계

- 합성 Markdown·격리 사용자 루트·실제 GitHub 운영체제 수신 검증
- 입력 묶음 변경·다른 운영체제의 같은 파일 전달·원문 합집합·FTS 확인
- 실제 사용자 USB·파일 공유 도구의 전송 품질, 모든 권한·디스크 상태, 5,000파일의 교차 운영체제 성능 보장 제외
- 공개 설치 수용의 vector 검증: 새 루트에서 원문 8개 복원·새 임베딩 생성·FTS 보존. 다중 묶음의 교차 운영체제 이전 근거와 구분
