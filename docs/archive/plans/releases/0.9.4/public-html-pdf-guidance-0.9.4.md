# `0.9.4` 공개 HTML·PDF 지식 기능 안내

> Checklist owner: `HGD94-*`
> 대상: `0.9.4` patch
> 표면: `docs/hive-core-features.ko.html`와 파생 PDF

## 문제

현재 핵심 기능 안내의 지식 기능: 한 줄 강조 상자. Skill별 목적·사용 시점·입력 범위·예시 구분 부족.

현재 파생 PDF: section title만 한 페이지 끝에 남고 해당 section의 본문은 다음 페이지로 이동하는
경우 존재. 제목과 본문 일부가 같은 페이지에 있으면 현 위치 유지 필요.

## 원칙

- 지식 기능: 강조 상자 대신 표. 정본 ID·기능명·언제 사용·무엇을 하는지·사용 예시 비교
- `knowledge-capture`, `knowledge-recall`, `knowledge-import`, `knowledge-promote`,
  `knowledge-maintain`의 목적·대상·실행 시점·안전 경계를 서로 구별 가능하게 설명
- PDF page break: section title 단독일 때만 다음 페이지 이동. 제목과 section content 일부가 함께
  배치 가능한 경우 강제 이동 금지
- HTML 정본 우선. PDF는 HTML에서 재생성. 외부 resource dependency 추가 금지

## Checklist

- [x] [HGD94-001] `hive-core-features.ko.html` 지식 기능 강조 상자 제거와 다섯 Skill 비교표 추가
- [x] [HGD94-002] 비교표 각 행의 정본 ID·사람 중심 기능명·적합한 사용 시점·작업 범위·구체적
  사용 예시·비밀 값과 원문 기록 금지 경계 추가
- [x] [HGD94-003] 인쇄 CSS의 section title 단독 page 방지. title과 content 일부 동시 배치 가능 시
  현 page 유지 regression 추가
- [x] [HGD94-004] HTML static·desktop/mobile render, 재생성 PDF Poppler page render와 title/content
  page adjacency visual inspection

## 완료 증거

- `knowledge-capture`, `knowledge-recall`, `knowledge-import`, `knowledge-promote`,
  `knowledge-maintain`의 정본 ID·용도·범위·안전 경계·예시를 HTML 표와 좁은 화면의 항목별 읽기
  레이아웃으로 구현
- print CSS는 `.section-head`에만 `break-after: avoid-page` 적용, section·표 전체 강제 page 이동
  미적용
- `python -m unittest tests.conformance.test_phase3_static_contracts.Phase3SchemaContract.test_public_core_features_compares_knowledge_skills_and_keeps_print_heading_with_content -v` 통과
- Chrome headless에서 HTML 정본으로 PDF 재생성. Poppler `pdfinfo` 8쪽과 144 dpi 8쪽 PNG
  확인, 02·03·04 section title과 각각 뒤따르는 content의 같은 page 배치
- Chrome headless desktop 1280 px·mobile 500 px render에서 표의 넓은 화면 열과 좁은 화면
  항목별 레이아웃을 확인

## 수락 기준

- 사용자: 다섯 지식 Skill 중 필요한 기능과 사용 시점 선택 가능
- 지식 기능 표: 가로 폭 `100%`, 좁은 화면에서 읽기 가능한 responsive layout
- PDF: section title 단독 page `0건`; title과 같은 page의 content 일부 허용
- HTML·PDF의 지식 기능 설명과 현재 product Skill 의미 일치

## 범위 제외

- 지식 Skill 동작 변경
- 다른 공개 HTML의 정보 구조 재설계
- PDF 전용 수동 편집
