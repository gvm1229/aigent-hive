# `0.9.4` 전체 Skill 식별자 표시

> Checklist owner: `SID94-*`
> 대상: `0.9.4` patch
> 선행: 지식 Skill 다섯 개의 `(knowledge-*)` 표시는 `0.9.3`에 완료

## 문제

`0.9.3`: 지식 Skill만 설명 첫머리의 정본 영문 ID 표시.
다른 Hive Skill: 한국어 이름만 표시. 동일 화면의 정본 호출 이름 식별 불일치.

## 원칙

- Hive가 제공하는 모든 활성 Skill 설명은 선택 언어와 관계없이 `(skill-id)`로 시작
- 표시명은 사람이 읽기 쉬운 현재 언어 유지. 호출·설정·문서 호환을 위한 정본 ID는 변경 금지
- Hive가 소유하지 않은 third-party Skill과 사용자 작성 Skill 변경 금지
- host별 별도 표시 API를 새로 만들지 않고 canonical catalog·projection을 단일 원본으로 사용

## Checklist

- [ ] [SID94-001] 모든 활성 Hive Skill의 canonical catalog·source front matter 설명 첫머리에
  정확한 `(skill-id)` 표시
- [ ] [SID94-002] Codex·Claude·Antigravity user·project projection에서 선택 언어별 설명과
  `(skill-id)`가 byte parity로 수렴하고, 기존 호출·설정 ID 유지
- [ ] [SID94-003] 전체 Hive Skill inventory regression과 user install·update·purge 수용으로
  Hive-owned Skill만 변경, third-party·사용자 작성 Skill 변경 `0건` 확인

## 수락 기준

- Codex의 한국어 Skill 화면에서 모든 Hive Skill 설명 첫 단어가 `(skill-id)`
- 지식 Skill의 사람 중심 이름과 `1건 기록`·저장소 스캔 같은 기존 기능 설명 보존
- cache 삭제나 Codex 재시작 없이 새 설치·갱신 결과가 canonical projection과 일치

## 범위 제외

- Skill ID rename
- third-party 또는 사용자 작성 Skill 설명 변경
- host 전용 표시 확장 기능
