# 개발 설치의 무삭제 공개판 전환

> Plan version: 0.13.0
> Scope: product

- 대상: 현재 Windows의 관리된0.12.1-dev 설치 → 이미 공개한0.13.0
- 승인: 사용자 데이터·설정 삭제 없는 갱신과 개발 설치 절차 수정. 새 공개 버전·출시·Codex 종료·추가 중단 시험 제외
- 제품 파일 변경 없음. 소스 전용 개발·복구 스크립트 변경과 실제 설치 수용 범위

## 실행 순서

- [x] [DEVREC-001] `scripts/dev-install.sh`의 전역 실행 파일 덮어쓰기와 과거 감시 배포 진입 차단. `--sandbox` 유지, 과거 설치용 `--rollback` 유지. 전역 거부 전 빌드·파일 변경 없음 회귀
  - state: complete; evidence: repo:docs/research/developer-install-recovery.md#sha256:552390190deb914cf2dfd84106461b994b7ee3587a5ce954102cfa62f74d9925
- [x] [DEVREC-002] `scripts/recover-dev-install.py`로 관리된 개발 실행 파일 지문·설치 검증 뒤 정확한 Hive 생성 파일과 호스트 소유 기록만 외부 백업으로 이동. 지식·설정·공유 지침·감시·실행 기록 미이동. 변조·링크·경로 탈출·중복·중간 실패의 거부와 복원 시험
  - state: complete; evidence: repo:docs/research/developer-install-recovery.md#sha256:552390190deb914cf2dfd84106461b994b7ee3587a5ce954102cfa62f74d9925
- [x] [DEVREC-003] 실제 Windows에서 위 백업 후 기존 관리 원복으로0.12.0 실행 파일 복원. Hive 저장소 등록만 공식 해제 후 일반 설치 미리보기·적용·검증으로 동일 설정의0.12.0 투영 재생성, `hive update`로0.13.0 갱신. 제거·보존 재설치 경로는 거부. 새 CLI의 이전 투영 인증 거부는 보관한 공식 원본으로 생성 파일만 재검증·백업하여0.13.0 일반 설치로 복구. 후속 `hive update` 정상 실행과 지식·사용자 설정·감시 설정·Codex 프로세스 보존 확인
  - state: complete; evidence: repo:docs/research/developer-install-recovery.md#sha256:552390190deb914cf2dfd84106461b994b7ee3587a5ce954102cfa62f74d9925

## 복구 경계

- 사전 설치 인증은 실제 개발 CLI의 `install --validate` 담당. 단순 JSON 구조를 인증으로 취급하지 않는 경계
- 외부 백업에 원본과 이동 목록 보존. 이동 전 전체 경로·지문 재검사, 다른 파일이 차지한 복원 위치 덮어쓰기 금지
- 첫 공식 재생성 전에 실패하면 이동 파일 복원.0.12.0 재생성 성공 뒤 갱신 실패면 검증된0.12.0과 백업 유지, 불일치 개발 상태로 무조건 회귀 금지
- 기존 공개0.13.0의 인증·갱신 코드 수정 없음. 공식 패키지 설치와 사용자 투영 생성은 기존 CLI 담당
- 시험·수용 증거: 격리 경로 안전성 회귀, 실제 Windows 명령 결과, 개인 경로·내용을 뺀 보존 집계. 실제 제공자 초기화·다른 OS 복구의 실측 범위 제외
