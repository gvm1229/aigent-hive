# `0.9.5` macOS 외부 검증

> Checklist ID: `MAC95-001`
> 대상: 현재 macOS host의 `0.9.5` source qualification
> Owner: 유지보수자와 agent

## 경계

현재 host: macOS Apple Silicon, Rosetta x86_64 실행 가능. 현재 `develop`의 arm64와 x86_64
release build·archive·설치 수용을 같은 source commit에 결합. 이전 macOS 증거는 historical reference일 뿐
현재 `develop` commit의 통과 증거 아님.

## Checklist

- [x] `MAC95-001` macOS arm64와 x86_64에서 current `develop` source의 locked release build·format·workspace
  test·macOS 조건부 conformance·archive direct-install·`project upgrade --validate`·`install --scope user --validate`
  실행. source `7ad1e58` product bytes, 명령·pass/skip/fail·artifact digest는 `CURRENT.md`에 기록

## 인계 기준

- macOS host와 실제 실행 권한: 유지보수자 승인 범위의 agent 실행
- release·npm 게시·GitHub Release 생성: 이 검증의 행동 범위 밖
- 실패 시: exact current commit과 smallest reproducer를 전달하고, agent가 source 재현·수정·Windows 회귀를 재개
