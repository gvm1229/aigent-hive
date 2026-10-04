# 안정판 게시 전 검사

- 2026-10-05, 현재 Windows·Codex에서 실제 검사 실행
- 게시 37230948639는 영어 VERIFY-02의 25단어 제한 초과로 패키지 업로드 전 중단. 같은 의미·미실행 한계를 보존한 20단어 문구로 수정
- 합성 시험만으로 실제 문서 검사를 대신한 결손 보완: 현재 출시 설명·실제 버전 검증부·길이 위반 검사를 CI에 추가
- [관련 검사 39개](runs/20261004T201501-47906591c166.md) 통과, 제품 바이트 변경 없음
- [배포 묶음 증명](runs/20261004T201223-2b5ff7cb53b1.md)·[전체 순차 증명](runs/20261004T201718-6c2834f8c38e.md) 통과
- 첫 병렬 검증의 Linux arm64 거절 관찰, 오류 원문 미수집으로 원인 미확정. 단독 진단과 전체 순차 실행 성공, 최초 [실패 기록](runs/20261004T201553-0bfe4f80a582.md) 보존
- GitHub 게시 환경의 공지 관련 두 설정 이름 등록 확인, 비밀 값 미열람. 로컬의 승인 지문·881자 문구 형식 검증만 수행, 실제 환경 값과 전송은 게시 절차의 확인 대상
- 검증 대상은 후보 37230009485의 파일, 다음 후보의 새 바이트를 대신 검증하는 근거에서 제외. 새 후보는 같은 검사와 인증된 게시 절차 필수
- 현재 설치 시험판으로 배포 묶음 데이터만 검증, 안정판 설치·실행 시험·모델 호출·공지 전송 없음

```json
{
  "status": "passed",
  "actual_os": "Windows",
  "host": "Codex",
  "candidate_run_id": 37230009485,
  "source_commit": "9d920975189ba553976d437da1e248a2cd0f3c10",
  "checksum_files": 6,
  "attested_archive_count": 12,
  "release_verify_code": "hive.release-verified",
  "release_sequence": 20,
  "stable_skill_ledger": "success",
  "approved_subscriber_sha256": "df5e6899112ad7ec15ab41908ef34218fceafed6383d2b3da061f66a74f917c1",
  "approved_subscriber_characters": 881,
  "model_calls": 0,
  "notification_sent": false,
  "archives": [
    {
      "name": "aigent-hive-0.11.1-aarch64-apple-darwin.tar.gz",
      "sha256": "afca9144afb46cbdf4dc85f79cd4e86f127faec7d8b8fcaa59942db87313cf10",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-darwin-arm64-0.11.1.tgz",
      "sha256": "6e56c86e4630cf4cd5b2acc63ca1d1442616fccc5077e461df371eba22db686b",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-0.11.1-aarch64-unknown-linux-musl.tar.gz",
      "sha256": "09fc00e68c135e162372c71a2a8e05ea4dc127ea8778d5ec40689f45aa53d429",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-linux-arm64-0.11.1.tgz",
      "sha256": "81040ccee1a20efe7b8896891c7ba28dd8ff771291d155333a2367e4c16f8265",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-0.11.1-release-bundle.tar.gz",
      "sha256": "043ca97dcbd1541a73dfb262f5077e0e26c54fd6746710c82b7b560d3be9cec3",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-0.11.1.tgz",
      "sha256": "89bb13160709a8d591b7f000417ff07be40f9bfd06605eedd3213bb68f895e3b",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-0.11.1-x86_64-apple-darwin.tar.gz",
      "sha256": "41521b4ab06f3d76ce538776d477546dc5798ee0896be7f85da95d8786ef379d",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-darwin-x64-0.11.1.tgz",
      "sha256": "df1cacfb73b1fa661ddf4ba057cc363ccf18125e588fce43fa40ab728510745f",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-0.11.1-x86_64-pc-windows-msvc.zip",
      "sha256": "4b02fab0c106cdb448efad2861f92dc213b85fa04b27c515f4dc998568104898",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-win32-x64-0.11.1.tgz",
      "sha256": "3269e145ec32651b515c1af6169026533b88744b57297c87a7bbae521b690d7b",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-0.11.1-x86_64-unknown-linux-musl.tar.gz",
      "sha256": "cb4d54eb0d62e89794ff772d2e6d38feafb34530c9e0f6b95d648d1ea1465a37",
      "github_attestation_verified": true
    },
    {
      "name": "aigent-hive-linux-x64-0.11.1.tgz",
      "sha256": "c23e8df1210faa1acabab39cd66cb3c6bc63dca8f9236f51bf30304b1d941c80",
      "github_attestation_verified": true
    }
  ]
}
```
