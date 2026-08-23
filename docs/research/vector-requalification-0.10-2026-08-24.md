# Vector 재검증 결과

> 판정: `defer`
> 기준 host: Windows x64
> 제품 vector dependency 추가: `0건`
> 정량 근거: [`vector-requalification-windows-2026-08-24.json`](evidence/vector-requalification-windows-2026-08-24.json)

## 결론

FTS 대체와 선택형 vector 제품 계층 모두 이번 범위에서 제외. 의미 검색 품질과 query 속도는
통과했지만, 50,000개 고유 chunk의 실제 embedding full build가 10분 기준을 충족하지 못한 결과.
FTS·native relation graph·Graphify code graph 유지.

## Corpus 분리

| Corpus | 항목 | 고유 text digest | 실제 embedding | 결과 |
| --- | ---: | ---: | ---: | --- |
| 30문서 반복 | 50,000 | 30 | 30 | 5.75초, 활성화 성공 |
| 고유 현실형 chunk | 50,000 | 50,000 | 1,000 probe | batch당 26.54–27.69초 |

고유 corpus의 50,000개 환산값: 약 2,711초. Hard gate `600초` 대비 약 4.5배 초과.
환산값의 full-build 통과 근거 사용 금지. 1,000개 측정 시점에서 mandatory gate 실패로 조기 종료.

## Pipeline 검증

- Content digest cache: 반복 corpus embedding `49,970건` 제거
- Batch checkpoint: 500개 완료 뒤 remaining count 보존
- Resume: 다음 500개부터 재개, 미완료 generation 활성화 `0건`
- 100개 변경: 7.20초
- 10개 추가·10개 삭제: 1.42초, 기존 90개 vector 재사용
- 중단 재개와 one-shot 100개 generation pointer digest 일치
- 여섯 scope: 서로 다른 physical research root
- Provider API·API key·network·background server 사용 `0건`

## Query와 품질

| 기준 | 결과 | Gate |
| --- | ---: | ---: |
| Semantic Recall@10 | 93.3% | 90% 이상 |
| FTS 대비 의미 향상 | +15.0 points | +15.0 이상 |
| Hybrid exact Recall@10 | 100% | 저하 0건 |
| Warm query embedding p95 | 37.31ms | end-to-end 500ms 이하 |
| Cold query embedding | 643.45ms | end-to-end 2초 이하 |

Query 수치: embedding 단계 측정값. 50,000개 실제 embedding generation 미완료로 같은 실제 vector를
사용한 Qdrant Edge·sqlite-vec end-to-end 재비교 미실행. 이전 임의 384차원 vector engine 수치는
저장 계층 참고값으로만 유지.

## Model 비교

FastEmbed `0.8.0`의 다국어 후보:

| Model | 크기 | Dimension | License | 판정 |
| --- | ---: | ---: | --- | --- |
| multilingual MiniLM-L12-v2 | 0.22GB | 384 | Apache-2.0 | 실제 품질·속도 측정, full build 실패 |
| multilingual mpnet-base-v2 | 1.0GB | 768 | Apache-2.0 | 기준 model보다 큰 저장·연산량, 후속 실행 중단 |
| multilingual-e5-large | 2.24GB | 1,024 | MIT | Prefix 계약·가장 큰 저장·연산량, 후속 실행 중단 |

가장 작은 다국어 후보의 full-build 실패 뒤 더 큰 두 후보 다운로드·실행 중단. 영어 전용 소형
model: 한국어·cross-language 품질 조건 불충족으로 제외.

## Gate closure

- `full_build`: 실패
- macOS arm64·Linux musl package 수용: reference Windows mandatory gate 실패 뒤 미실행
- Actual 50,000 embedding 기반 engine·cross-scope ANN: prerequisite 미충족으로 미실행
- 제품 vector schema·helper·fusion·incremental index·bundle 경로: `not-applicable`
- Product dependency·설치 byte·release acceptance 변경: `0건`

다음 재검토 조건: 50,000개 고유 한국어·다국어 chunk를 reference CPU에서 10분 안에 처리하는
더 작은 검증 model 또는 측정 가능한 hardware acceleration 경로.

