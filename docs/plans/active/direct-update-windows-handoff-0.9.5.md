# Windows 직접 설치 갱신 인계 `0.9.5`

> Checklist owner: `DUP95-*`
> 대상: `0.9.5` 공개 시험판 직접 설치 갱신
> 선행: `REL95-003` 전용 시험 루트 수용

## 원인

- 직접 설치 소유자가 실행 중인 `hive.exe` 경로에 새 archive 설치 요청
- Windows 실행 파일 잠금으로 현재 프로세스 생존 중 파일 교체 거부
- `--confirm`의 Hive 내부 동의와 직접 설치자의 파일 교체 가능 시점 불일치

## 계획

- [ ] `DUP95-001` Windows 직접 설치 소유자 갱신의 실행 파일 잠금 회피 인계 구현과 단위 회귀
- [ ] `DUP95-002` 다음 번호 public test의 Windows 전용 시험 루트에서 직접 설치 `test.8 → test.9` 갱신·user projection refresh·validate 수용

## 수락 기준

- 직접 설치본 `hive update --channel test --confirm`의 현재 실행 파일 잠금 대기·교체·새 실행 파일 projection refresh 완료
- npm 설치 소유자·macOS/Linux 직접 설치 경로 동작 불변
- 호출자 반환 전 갱신 결과와 projection validation 확인 가능
- 기존 Codex 설정·사용자 Hive root·foreign byte mutation `0건`

