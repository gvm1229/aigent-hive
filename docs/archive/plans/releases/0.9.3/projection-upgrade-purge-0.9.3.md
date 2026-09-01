# `0.9.3` projection purge·directive 우선 갱신

> Checklist owner: `PUG93-*`
> 대상: `0.9.3`
> 상태: 구현·current-tree qualification 완료

## 목표

- 전역 사용자와 기존 project harness의 폐기·개명 Hive Skill 수렴 제거
- 새 Hive 안전·소유권 규칙과 직접 충돌하는 Hive-owned directive clause의 제한적 갱신
- 사용자 작성·foreign byte와 비충돌 Hive clause 보존

## 안전 경계

- 삭제 대상: `retired-names.yml`의 retired ID와 authenticated historical Hive digest 또는
  이전 ownership manifest가 함께 확인된 `SKILL.md`
- 이름만 같은 사용자·제3자 Skill, 수정되어 origin을 증명할 수 없는 Skill, 예상 밖 sibling 파일: 삭제 없음
- directive: authenticated Hive projection의 직접 충돌 rule만 incoming 우선. 추가한 사용자 문구,
  foreign block, 비충돌 Hive rule은 byte 보존
- base·digest·preview·atomic apply·rollback 없는 mutation 금지

## Checklist

- [x] [PUG93-001] user setup/refresh의 `.agents/skills` retired-name ledger·historical digest
  purge, manifest-owned deletion, leaf-to-root empty directory 수렴 구현
- [x] [PUG93-002] user projection과 project `hive project upgrade`의 Hive directive direct-conflict
  merge policy 구현. 새 safety·ownership rule 우선, non-conflicting local byte 보존
- [x] [PUG93-003] `project-setup`이 authenticated outdated harness에 동일 preview·dry-run·apply
  경로를 사용하고 project refresh와 결과 일치 확인
- [x] [PUG93-004] retired Hive Skill 삭제·foreign/modified Skill 보존·directive direct conflict·
  disjoint user text·rollback·빈 directory 수렴의 Rust/Python 회귀와 문서 contract 검증

## 출시 연결

- `PUG93-001–004` 완료와 current-tree evidence 전 release fragment의 public qualification 완료 처리 금지
- 제품 byte 변경 뒤 numbered public `0.9.3-test.N`에서 전역 setup·project upgrade·clean install
  수용 필요

## 완료 근거

- Rust workspace `cargo test --workspace` 559개 통과, strict Clippy 통과
- user projection retired Hive digest 삭제·foreign 보존·빈 leaf 정리와 Hive directive·shared marker
  direct-conflict 회귀 통과
- Python projection parity·global setup/project lifecycle 적합성 22개 통과
- human documentation style 0 finding, Markdown link 0 failure, Source Wiki lint error 0건
