# 한국어 언어 core

## 역할

한국어 생성·윤문의 공통 계약:

```text
host draft
  → profile별 결정적 inspect
  → host-owned 국소 rewrite 0–1회
  → 원본·후보 deterministic verify
  → 통과 후보 또는 정확한 draft fallback
```

Hive: model 호출·재작성 주체 아님. 활성 host: 초안과 필요한 국소 재작성 소유. Rust core:
빈도·리듬·보호 span·change rate·touch rate·서법 보존 판정 소유.

## Profile

| Profile | 대상 | 핵심 경계 |
| --- | --- | --- |
| `response` | 일반 응답 | 쉬운 말·짧은 응답 과윤문 방지 |
| `release-note` | Discord·출시 안내 | 변경 main list·scenario sublist |
| `documentation` | README·guide·Wiki | 제목·링크·계약·경고 보존 |
| `technical` | CLI·오류·schema | code·field·path·진단 byte 보존 |
| `verbatim` | 직접 인용·법률·원문 요청 | 검사만 허용, rewrite 금지 |

## CLI

```text
hive korean inspect --target <consumer> --profile <profile> --input <file> --output json
hive korean verify --target <consumer> --profile <profile> --before <file> --after <file> --output json
hive korean sanitize --input <file> --output-file <file> --output json

hive korean pack check --output json
hive korean pack status --target <consumer> --output json
hive korean pack preview --target <consumer> --candidate <pack-dir> --output json
hive korean pack activate --target <consumer> --candidate <pack-dir> \
  --consent-digest <sha256:...> --confirm-pack --output json
hive korean pack rollback --target <consumer> --output json
```

`verify` 실패 결과: `fallback_required=true`. Gate 자체의 text repair 금지.

`--target` 생략 시 현재 폴더 기준. 소비자 프로젝트의 활성 팩을 검사·비교·hook이 함께 사용하며,
source workspace는 내장 팩 사용. 활성 팩 손상 시 오류 반환, 내장 팩으로 조용히 대체 금지.
결정적 검사는 의미 보존의 충분조건 아님. 부정문 변경은 보수적으로 거부하고, 통과 후보도
host가 주체·주장·수치·출처를 원문과 비교. 불확실한 경우 원문 유지.

## Host 적용

- Codex: 공통 directive와 final self-review
- Claude: fresh `Stop` capability·exact consent 기반 `validate-korean-output`, 최대 retry 1회
- Antigravity: fresh `AfterAgent` capability·exact consent 기반 같은 adapter
- 미지원·미검증 event: instruction-only, hook 활성화 없음

Hook: 최종 응답 직접 교체 주장 없음. 첫 실패 때 국소 재작성 요청, 두 번째 실패 때 정확한 draft
fallback 안내와 종료 허용.

## Language pack

기본 pack: `im-not-ai 2.3.2@0ac1e84f92334f9696e69184478f91c1c6f1dc5e` 기반 Hive 변환
version 1. 정본 경로: `harness/language-packs/im-not-ai/2.3.2/`.

- Upstream tree·license·선별 source·rules SHA-256 고정
- `rules-data|engine-code|host-surface` 분류
- Symlink·raw install script·floating ref·자동 activation 금지
- `check`: 고정 HTTPS metadata의 version 확인만 수행
- `preview`: 규칙 구조·지원 검사 종류·안전 상한·manifest 일치를 확인한 뒤 동의 digest 발급
- `activate`: 경로 잠금과 격리 staging 검증 뒤 원자 활성화. 기존 generation 재사용 전 byte 재확인
- `rollback`: 이전 generation 재검증 뒤 복구. 최초 활성화라면 내장 팩 복구. 손상된 현재 팩도 복구 가능
- 같은 팩 재활성화: 이전 복구 지점 유지

## 무결성

보호 대상: 사실·주장·서법·수치·날짜·단위·version·고유명사·인용·링크·Markdown·code·명령·
경로·목록·출처.

금지 목적: watermark 우회, detector 점수 최적화, 의무 고지 제거, 출처 은폐, 거짓 인간 저자
표시. 허용 text hygiene: zero-width·bidi control 제거와 modern Hangul NFD→NFC.
