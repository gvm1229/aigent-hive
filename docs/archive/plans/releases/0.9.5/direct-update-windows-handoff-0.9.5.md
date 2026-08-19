# 직접 설치 갱신 수용 `0.9.5`

> Checklist owner: `DUP95-*`
> 대상: `0.9.5` 공개 시험판 직접 설치 갱신
> 선행: `REL95-003` 전용 시험 루트 수용

## Windows 원인·수정 근거

- 직접 설치 소유자가 실행 중인 `hive.exe` 경로에 새 archive 설치 요청
- Windows 실행 파일 잠금으로 현재 프로세스 생존 중 파일 교체 거부
- `--confirm`의 Hive 내부 동의와 직접 설치자의 파일 교체 가능 시점 불일치

## 체크리스트

- [x] `DUP95-001` Windows 직접 설치 소유자 갱신의 실행 파일 잠금 회피 인계와 단위 회귀. `test.12 → test.13` 전용 시험 루트의 공개 수용 성공
- [x] `DUP95-002` M2 MacBook Air의 공개 `test.14 → test.15` 직접 설치·시험 채널 갱신·Codex 사용자 투영 최종 검증. isolated root `public-hive-acceptance-51180335c8824f98a673cf764b38f4e6`, setup·install·validate·update-check·update·final validate 성공

## macOS 수용 절차

1. 새 `tests/work/` 디렉터리에서 public `0.9.5-test.14` `install.sh`를 전용 prefix에 설치
2. prefix `hive --version`의 최초 `0.9.5-test.14` 기록
3. 제공 `accept-public-hive.py --mode user` 실행으로 고유 `test_root`의 설정·설치·검증·갱신 확인·시험 채널 갱신·최종 검증 확인
4. prefix `hive --version`의 다음 번호 시험판 기록과 runner JSON의 `mode`, 결과 코드, `test_root` 확인
5. 실패 시 local binary 대체 없이 격리 stderr·test root·직접 설치 영수증·설치기 단계 분류
6. source defect 확인 때만 최소 수정·대상 회귀·새 번호 공개 시험판·Windows/macOS 재수용

## 수락 기준

- 직접 설치본 `hive update --channel test --confirm`의 현재 실행 파일 잠금 대기·교체·새 실행 파일 projection refresh 완료
- npm 설치 소유자·macOS/Linux 직접 설치 경로 동작 불변
- 호출자 반환 전 갱신 결과와 projection validation 확인 가능
- 기존 Codex 설정·사용자 Hive root·foreign byte mutation `0건`

## 재수용 근거

- `test.8 → test.9` 보조 실행 파일의 직접 설치 실패: staged installer `NamedTempFile` handle 유지로 Windows installer read sharing 거부
- `into_temp_path`로 handle 종료와 자동 삭제 ownership 유지
- acceptance runner `update` 결과 코드 누락 보정
- Windows evidence: public `test.12` direct install에서 `test.13` test-channel activation·Codex user projection refresh·validate 성공, isolated root `public-hive-acceptance-d53f0375007140859694e401e52b9d75`
- macOS 공개 `test.12 → test.13` 결과: binary `test.13` activation 성공 뒤 update 내부 `--validate` 결과의 action을 `InstallHiveUser`로 잘못 비교해 실패 보고
- 원본 수정: update 내부 `--apply`는 `InstallHiveUser`, `--validate`는 `ValidateHiveUser` action 비교. macOS `hive-cli` 408 unit·3 integration 통과
- macOS 공개 `test.13 → test.14` 결과: `test.13` 실행 프로세스가 이전 action 비교 규칙을 메모리에 유지해 update 반환 실패. 설치 뒤 `test.14` binary의 `--hosts codex --validate`는 `ValidateHiveUser` 성공 JSON 확인
- macOS 공개 `test.14 → test.15` 결과: public direct installer·시험 채널 update·Codex 사용자 투영 apply/validate 성공. runner JSON의 `mode=user`, 최초·최종 version, 여섯 결과 코드와 dedicated `test_root` 확인
- Windows 재수용: `test.14 → test.15` 실제 Windows x64 dedicated root 증거 대기. macOS 성공은 Windows 실행 파일 잠금·handoff의 대체 근거 아님
