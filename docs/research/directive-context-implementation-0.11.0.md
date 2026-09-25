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
