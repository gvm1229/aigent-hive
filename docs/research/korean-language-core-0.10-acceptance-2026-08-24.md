# 한국어 언어 core 공개 시험 수용

> 확인일: 2026-08-24
> 공개 시험판: `0.10.0-test.2`
> 공개 source: `4b9275ae90c08f31dce82085b9cda939a623a975`

## 공개 artifact

- Candidate run `32676134575`: Windows x64·macOS arm64·macOS x64·Linux musl x64·Linux musl arm64 package와 attestation 통과
- Publication run `32676726910`: npm package 6종과 GitHub prerelease 게시
- npm tag: 6종 모두 `test=0.10.0-test.2`, `latest=0.9.5`
- GitHub tag: `v0.10.0-test.2`, prerelease, source `4b9275ae`
- 게시 뒤 acceptance-only source와 공개 tag 사이 product·package·installer source 변경 `0건`

## 세 운영체제 공개 byte 수용

Public acceptance run `32677477765`에서 registry의 exact npm package를 새 root에 설치한 뒤 실행.

| 대상 | 설치 package | 결과 |
| --- | --- | --- |
| Windows x64 | `@aigent-hive/win32-x64@0.10.0-test.2` | 통과 |
| macOS arm64 | `@aigent-hive/darwin-arm64@0.10.0-test.2` | 통과 |
| Linux musl x64 | `@aigent-hive/linux-x64@0.10.0-test.2` | 통과 |

세 receipt의 공통 결과:

- Binary version: `AIgent Hive v0.10.0-test #2`, release date `2026-08-24`
- Gold inspect: 6건 통과
- 의미·수치·link·인용 보존 후보: 1건 수용
- 수치·link·서법 변경 후보: 1건 거부
- Zero-width·bidi sanitize: 제한된 제어 문자만 제거
- Pack lifecycle: `2.3.2` 활성화, 합성 `2.3.3` staging, `2.3.2` rollback
- Corpus digest: `sha256:360cd49a7fc5eb3b4220824fa464dfaf616ccaf66d70de24f7d409f12ff9d3dd`
- Pack manifest digest: `sha256:50e8bec5fb4c7a479f9e0800f262d49c3e01258ba3c7b9066aab65ba3f7ca34e`
- Provider API call·API key read: 각각 `0건`

## Blind A/B 평가

평가자에게 후보 출처와 정답 side를 숨긴 12쌍 제공. 기준은 자연스러운 한국어와 의미·사실·
수치·명령·인용 보존. Clean-context 독립 verifier가 `A|B|tie|reject`만 선택.

| 문항 | 안전한 자연화 side | Blind 판정 | 결과 |
| --- | --- | --- | --- |
| B01 | B | B | 일치 |
| B02 | A | A | 일치 |
| B03 | B | B | 일치 |
| B04 | A | A | 일치 |
| B05 | B | B | 일치 |
| B06 | 없음: 출처 의미 변경 | reject | drift 차단 |
| B07 | 없음: 행위 의미 변경 | reject | drift 차단 |
| B08 | A | A | 일치 |
| B09 | A | A | 일치 |
| B10 | A | A | 일치 |
| B11 | B | B | 일치 |
| B12 | A | A | 일치 |

집계: 자연화 후보 10/10 선택, 의도한 의미 drift 2/2 거부, 잘못된 선택 0건.

## 증명 경계

- 증명 대상: 고정 corpus의 rule finding, 보존 gate, 공개 binary, pack update·rollback, 대표 문장의 blind 선호
- 세 host 적용 근거: 같은 source의 Codex·Claude·Antigravity projection parity와 direct-upgrade 전체 CI
- 미증명 대상: 모든 미래 한국어 문장에 대한 자연스러움, host model별 생성 품질의 절대 보장, detector 회피
- `im-not-ai` upstream의 향후 version 자동 채택 없음. 새 version은 별도 staging·검증·승인 대상
