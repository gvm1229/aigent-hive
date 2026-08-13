# 0.9.3 지식 Skill 이름·표시 정비

- 상태: 진행 중
- 대상: `0.9.3`
- 결정: 실행·설정·기존 prompt 호환을 위한 정본 영문 ID 유지. 한국어 설명 첫머리는 반드시 `(정본-ID)` 표기, 표시명은 사람이 이해할 수 있는 기능명과 ID 병기
- 범위: 현재 product Skill, 설정 catalog·의존성, user·project projection, 공개 한국어 핵심 기능 HTML·PDF, 표시 회귀 검증

## 목표

- 자동 기록 Skill: 대화 종료 전 나중 작업에 도움 되는 안전한 지식 `1건` 선택·기록의 의미 표시
- 저장소 대상으로 읽고 검토하는 Skill: `scan` 동작 명시
- 한국어 host 화면: 한국어 기능명 뒤 정본 영문 ID 노출
- host 별 별도 표시 API 의존성 금지. `name`·`description` front matter와 canonical projection 우선

## Canonical 이름

| 정본 ID | 영문 표시명 | 한국어 표시명 | 역할 |
| --- | --- | --- | --- |
| `knowledge-capture` | Remember useful knowledge (`knowledge-capture`) | 유용한 지식 남기기 (`knowledge-capture`) | Wiki 활성 turn 종료 전 후속 작업에 도움 되는 사실·선호·방식 하나의 안전한 기록 |
| `knowledge-recall` | Search knowledge (`knowledge-recall`) | 지식 찾아보기 (`knowledge-recall`) | 현재 질문·작업에 도움 되는 지식의 제한 조회 |
| `knowledge-promote` | Share knowledge (`knowledge-promote`) | 지식 공유하기 (`knowledge-promote`) | 검토한 사실·선호·방식의 전역 공유 |
| `knowledge-maintain` | Maintain knowledge (`knowledge-maintain`) | 지식 정비하기 (`knowledge-maintain`) | 신뢰 가능한 지식의 검사·검색 색인 재생성·명시 정리 |
| `knowledge-import` | 저장소 지식 스캔 (`knowledge-import`) | 저장소 지식 스캔 (`knowledge-import`) | 명시 대상 저장소 inventory·검토 후 import |

## Checklist

- [x] [KNS93-001] Canonical Skill front matter·catalog의 기능·단위 명시와 정본 ID 보존
  - Evidence: `harness/skills/knowledge-*`와 `catalog.yml`의 정본 ID 유지, 모든 설명 첫머리 `(정본-ID)`와 사람 중심 기능 경계 반영
- [x] [KNS93-002] Codex·Claude·Antigravity projection의 한국어 표시명 영문 ID 병기와 description 첫머리 `(정본-ID)` 동일 노출
  - Evidence: `localized_skill_text`의 세 host projection과 Rust `korean_knowledge_skill_labels_keep_the_canonical_english_id_on_every_host` PASS
- [x] [KNS93-003] user·project setup, selected Skill closure, update·uninstall ownership의 기존 ID 보존·표시 수렴
  - Evidence: `sync-user-plugin.py`, active-skill ledger와 26개 현재 product Skill inventory·projection parity PASS
- [x] [KNS93-004] Rust·Copier·Python parity, Korean display, host projection, 공개 한국어 핵심 기능 HTML·PDF, full documentation·Source Wiki gate
  - Evidence: `cargo test -p hive-projection` 34 PASS, Copier/Rust parity 22 PASS, Python static·inventory 19 PASS, 문서 말투·Markdown link·Source Wiki lint `0 error`; Chrome PDF 4쪽 render 확인

## Acceptance

- 한국어 표시에서 각 지식 Skill의 정본 영문 ID 확인 가능
- 자동 기록 이름·설명에서 turn당 `1건` 한도 확인 가능
- 저장소 import 전 scan 단계와 명시적 검토 경계 확인 가능
- 기존 ID 저장 preference와 host invocation 보존, foreign·third-party Skill 변경 `0건`
- Codex·Claude·Antigravity projection byte parity와 full gate 통과
