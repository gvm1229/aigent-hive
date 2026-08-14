# `0.9.5` macOS 외부 검증

> Checklist ID: `MAC95-001`
> 대상: 현재 Windows에서 실행 불가한 `0.9.5` source qualification
> Owner: 유지보수자

## 경계

현재 목표는 공개 시험판·정식 배포 없이 로컬 구현과 Windows 검증 완료. macOS 고유 build·archive·설치
검사는 현재 host에서 실행 불가하므로 완료로 표시하지 않음. 이전 macOS 증거는 historical reference일 뿐
현재 `develop` commit의 통과 증거가 아님.

## Checklist

- [ ] `MAC95-001` macOS arm64와 x86_64에서 current `develop` source의 locked release build·format·workspace
  test·macOS 조건부 conformance·archive direct-install·`project upgrade --validate`·`install --scope user --validate`
  실행. exact commit·명령·pass/skip/fail·artifact digest를 `CURRENT.md`에 기록

## 인계 기준

- macOS host와 실제 실행 권한: 유지보수자 소유
- release·npm 게시·GitHub Release 생성: 이 검증의 행동 범위 밖
- 실패 시: exact current commit과 smallest reproducer를 전달하고, agent가 source 재현·수정·Windows 회귀를 재개
