---
name: custom-subagent-create
description: 목적부터 확인해 Codex와 Claude 양쪽에서 같은 정의로 동작하는 Hive 사용자 정의 에이전트 추천·동의·생성
---

# Hive 사용자 정의 에이전트 생성

목적·범위·권한·양쪽 host mapping이 모두 확정된 경우만 사용자 정의 에이전트 생성

## 흐름

1. 목적과 `user|project` 범위 확인. 단순 질문·모호한 목적·host 지원 미확인 상태의 자동 생성 제외
2. 추천 준비

   ```text
   hive agent recommend --purpose <목적> --scope <user|project> --catalog <외부-서명-카탈로그.json> --catalog-attestation <분리-서명.json> --trust-root <보호된-신뢰-루트.toml> --output json
   ```

3. 표시된 선택지 중 하나 수용

   - `accept`: 추천 request 그대로 수용
   - `manual`: 이름·양쪽 exact model/effort·범위·권한·trigger와 이전 `decision_digest`를 포함한 request 작성
   - `revise`: 이전 request를 지정해 변경 목적의 새 추천 준비

   ```text
   hive agent recommend --purpose <수정-목적> --scope <기존-범위> --catalog <외부-서명-카탈로그.json> --catalog-attestation <분리-서명.json> --trust-root <보호된-신뢰-루트.toml> --previous-request <이전-creation.json> --output json
   ```

4. request의 `decision_digest`와 정확히 일치하는 동의 뒤 생성

   ```text
   hive agent create --request <creation.json> --root <user-root|project-root> --accept-decision-digest <sha256:...> [--prior-decision <이전-creation.json>] --output json
   ```

5. 생성 뒤 `hive agent validate`, fresh host session discovery, exact runtime attestation 확인

## 경계

- Codex·Claude mapping 모두 없는 request 거부
- 외부 보호 경로의 서명된 호스트 모델 카탈로그·분리 attestation·trust root 없이는 추천 없음
- 카탈로그 서명 또는 exact model/effort/minimum version mapping 불일치 시 실패 폐쇄
- `manual|revise` request의 prior digest·scope·지정 이전 request 불일치 시 생성 거부
- `hive-independent-judge` 생성·변경·project shadow 금지
- Hive는 recommendation·정본 저장·projection·검증만 수행. provider API·credential·model 또는 subagent process 실행 없음
- preview·digest 동의·Hive ownership ledger 없는 host 파일 변경 없음
- host capability가 `unsupported|unverified`이면 생성 후 activation 주장 없음
