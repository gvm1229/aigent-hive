# 0.11.0 지침 복구 구현 검증

- 소유 기준: [DCR-001–006](../plans/active/directive-context-recovery-0.11.0.md)
- 제품 코드: `c00202a0`, Windows 11·Codex CLI 0.155.1 설치 환경
- 이번 자동 검증: CLI·생성된 훅 명령과 파일 상태. 실제 Codex 모델의 압축 실행·준수는 미검증

## 구현

- 사용자 소유 `.hive-context.toml`의 승인 지문과 선택한 Markdown 원문 구절 지문 검증
- 시작·재개·매 압축과 Codex 하위 작업 시작의 핵심 복원, 경로별 상세 안내와 명시적 보호 경로 검사
- 부모 세션 공유·같은 턴·원문 불변을 이유로 재전달 생략 없음. 원문 대화 수집·추가 모델 호출 없음
- 승인 화면과 등록 명령에 같은 읽기 결과 사용. 이후 원문 변경은 새 내용의 무단 주입 대신 확인 안내
- 설정 손상은 파일 검사 거부, 조회·철회 유지. 원문 구절만 바뀌면 무관한 편집 유지
- 출력 4,096바이트 상한. Codex의 추가 요약·잘림은 이 제한이 적용된 선택형 전달에서만 해제
- 작업 폴더가 등록 범위 밖이면 구절 전달 없음. Windows 대소문자·경로 이름 변경·끝 점의 보호 우회 검사
- Antigravity 기존 파일 보호 유지, 압축 복구 미지원. Claude 실제 수용은 기존 제외 범위

## 자동 검증 근거

- [최종 훅 계약](../../tests/results/runs/20260925T094207-137ebc825bc3.md): 26개 중 25개 통과·1개 제외. Windows 심볼릭 링크 생성 권한 부재로 해당 경우 미실행
- [전체 Python](../../tests/results/runs/20260925T092457-b057d4446057.md): 보완 전 `b71c0988`의 951개 중 907개 통과·44개 제외. 운영체제·권한·선택 실행 조건별 미실행 유지
- 보완 뒤 전체 Rust: [검증 기록](../../tests/results/runs/20260925T094249-9b5ca064d39b.md). Clippy 전체 검사 통과, 외부 고정 소스의 기존 경고 보존
- `0.9.1` 이후 공개 안정판의 갱신·수정 보존·실패 복구와 스킬 배포본 일치 검사 통과
- 이전부터 있던 소스 지침 크기 초과는 출시 안내 규칙을 조건부 참조로 이동하여 해결. 유효 규칙·원문 예시 보존, 허용량 확대 없음
- 위 결과는 실제 모델 준수·다른 운영체제 실행의 대체 근거에서 제외. 보완 뒤 정확한 커밋의 전체 검사는 원격 CI에서 별도 확인

## 비용 표본

```json
{
  "base": {
    "median_ms": 433.86,
    "p95_ms": 457.984,
    "samples": 30
  },
  "selected": {
    "median_ms": 429.22,
    "p95_ms": 466.505,
    "samples": 30
  },
  "p95_change_percent": 1.86,
  "restore_utf8_bytes": 527,
  "full_fixture_utf8_bytes": 10560,
  "unrelated_edit_context_bytes": 0,
  "model_calls": 0,
  "tokens": "unmeasured",
  "source_commit": "c00202a0b9ca58f5dae075a91ab5ccf33d1b9bf2",
  "binary_sha256": "93299f31a981880354339e1add32bdb529e5b92d0fe13e4ecfa42299704a791a",
  "platform": "Windows 11, generated PowerShell hook launcher; no live model",
  "method": "five warmups and thirty interleaved samples per condition; no concurrent test load"
}
```

같은 새 실행 파일에서 복원 미선택·선택 조건을 비교한 생성된 PowerShell 훅 명령의 측정. 조건별 준비 5회·교차 표본 30회, 동시 시험 부하 없음.
정상 편집 p95 증가 1.86%로 초기 10% 목표 이내. 합성 자료의 선택된 안내 527바이트·전체 문서 10,560바이트 비교, 실제 입력 토큰·캐시·과금 절감률의 증명 제외.

## 남은 실제 수용

- Codex에서 승인·로드된 정확한 훅으로 시작·재개·수동/자동 압축·같은 턴 재압축·하위 작업 확인
- 반복 5회·강화 10회 뒤 실제 언어·사용자 수정 보존·승인 경계·현재 작업 준수와 모델 사용량 비교
- CLI 입력 재생이나 지침 암송만으로 실제 준수 통과 판정 금지
- 시험판·운영체제별 산출물 수용과 실제 호스트 수용은 별도 기록. 안정판 승인·전역 설치·예시 프로젝트 변경 제외

## 공개 시험판 수용

- [정확한 제품 커밋 CI](https://github.com/gvm1229/aigent-hive/actions/runs/36120562980)와 [최종 검사 절차 CI](https://github.com/gvm1229/aigent-hive/actions/runs/36122144539) 통과
- [후보](https://github.com/gvm1229/aigent-hive/actions/runs/36120562717), [게시](https://github.com/gvm1229/aigent-hive/actions/runs/36122418779), [공개 수용](https://github.com/gvm1229/aigent-hive/actions/runs/36123198461) 통과
- 공개 버전 `0.11.0-test.8`, 소스 `983135554cfb12a18dca48babd579dfac993e2da`, 제품 지문 `sha256:fe34b0cc45ff0a3c68130e194603c137fbeec94dbbd911925feab3852d454508`
- GitHub 시험판과 npm test 일치, latest `0.10.3` 유지. 안정판·전역 설치·Discord 전송 없음
- 공개 실행 파일의 새 훅 계약: Windows Server 2025 x64·macOS 15.7.9 arm64·Linux glibc 호스트의 musl x64에서 각각 26개 전부 통과
- [공개 계약 기록 20260925T101807-90319487f894](../../tests/results/runs/20260925T101807-90319487f894.md)
- [공개 계약 기록 20260925T101815-c8a0f1172cf7](../../tests/results/runs/20260925T101815-c8a0f1172cf7.md)
- [공개 계약 기록 20260925T101844-2cf14b91409b](../../tests/results/runs/20260925T101844-2cf14b91409b.md)
- [로컬 Windows 11 공개 실행 파일](../../tests/results/runs/20260925T101039-af58391772ab.md): 25개 통과·심볼릭 링크 권한 부재 1개 제외. CI Windows의 해당 사례 통과와 로컬 권한 부재는 별도 판정
- 앱의 실제 실행 파일 `0.155.0-alpha.16.4`에서 생성한 공개 프로토콜에 SubagentStart·additionalContextLimit·thread/compact/start 선언 확인. 전역 CLI `0.155.1`과 구분, 실제 훅 실행·모델 준수의 증명 제외

## 유지보수 기록

- 최초 게시의 npm 채널 전파 확인 실패 뒤 같은 후보·같은 바이트의 복구 게시 성공. 공개 패키지 변경·안정판 태그 변경 없음
- 공개 검사 단계 추가 뒤 기존 지식의 원본 지문 불일치 수정. 최종 관련 문서 검사와 CI 통과
- 실제 앱 설정은 미변경. 현재 저장소의 임시 활성화·새 검증 대화·시험 뒤 철회를 위한 미리보기 준비, 사용자 승인 대기
