# 0.11.1 부속 자료 폴더 정리 회귀

## 실제 발견과 보존

- 2026-10-05 승인된 현재 사용자 test.7 설치 완료, 지식 502개·설정 2개·Hive 영역 밖 지침 보존; [실제 결과](../../tests/results/legacy/de02a504d7f71cc60c20.md)
- WProject의 미수정 Hive 사본 정리 중 `references/SKILL.md`라는 가상 경로 검사로 실패
- Windows에서 발견한 공통 코드 오류, 자동 원복 뒤 기존 경로 35개 지문·사용자 AGENTS의 Git 차이 없음·사용자 파일 2개·관련 없는 문서 3개 보존, 복구 기록 없음; [실패와 원복](../../tests/results/legacy/2e2073a5bbd14cd5306d.md)
- 실제 사용자 프로젝트를 시험 자료로 복사하거나 소유권을 수동 변경하는 우회 없음

## 수정

- `prune_empty_project_skill_ancestors`: 검증한 실제 삭제 파일을 각 상위 폴더의 확인 근거로 전달
- `remove_empty_project_owned_dir`: 파일 경로 허용·같은 스킬 안의 상위 폴더·깊이 확인, 연결 경로 미추적·빈 폴더만 삭제 유지
- 새 소유권이나 파일 종류 허용 없음, 사용자 자료가 남은 폴더 보존
- 공개 수용에 전역 usage-guard 제공·프로젝트 부속 자료 복원·다시 전역 제공으로 전환하는 전체 과정 추가
- 첫 보강 시험 자료에서 전역 usage-guard 선택이 빠져 정리 조건 미성립, 선택 추가 뒤 실제 test.7 오류 재현과 수정 개발 빌드 통과 확인

## 실제 검증

| 범위 | 실제 Windows 실행 | 한계 |
| --- | --- | --- |
| 관련 CLI·과거 갱신 | [62개 통과](../../tests/results/runs/20261004T153237-d56207e1a059.md) | 합성 프로젝트, 실제 사용자 적용과 구분 |
| Rust 전체 | [976개 통과·4개 수동 조건 제외](../../tests/results/runs/20261004T154327-9266b13c31b0.md) | 제외된 실런타임·성능 시험은 이 결과의 증명 범위 밖 |
| Python 전체 | [969개 중 925개 통과·44개 조건 제외](../../tests/results/runs/20261004T160222-20da65cdebb5.md) | POSIX·macOS·심볼릭 링크 권한 등의 조건별 제외 |
| Rust 정적 검사 | [전체 Clippy 종료 코드 0](../../tests/results/runs/20261004T153548-e447e95553bb.md) | 기존 외부 process-wrap 경고는 수정 범위 밖 |
| 보강한 실제 CLI 수용 | [수정 개발 빌드 통과](../../tests/results/legacy/29acf32b56fe05bc84e4.md) | Codex 관리 응답은 모의, 공개 파일·실제 모델 호출은 별도 |

- [조건별 제외와 보강 검사 요약](../../tests/results/legacy/6bafc654121b96ebd45f.md)
- 전체 검사 후 보강한 공개 수용과 정리 진단은 해당 검사로 다시 확인, 제품 수정 이후 새 공개 test.8 수용 필요
- 현재 사용자 WProject 적용 성공과 실제 선택 스킬 호출은 아직 미증명
