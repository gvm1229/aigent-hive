# Aigent Hive licensing

Copyright 2026 Hojin (Tom) Jeong.

Aigent Hive is licensed under **Apache License 2.0** (`Apache-2.0`):

- The license applies to the CLI, repository source, [`../harness/`](../harness/) template source, and Aigent Hive files or exact marker blocks generated from it.
- It permits use, modification, distribution, sublicensing, and commercial use subject to its notice and change-documentation conditions.
- It includes an explicit contributor patent grant and defensive termination clause.
- It does not change the license of a consumer project's own files, documentation, configuration, or data.

GitHub uses the exact Apache-2.0 text in [`../LICENSE`](../LICENSE) to identify the repository license. Shipping source also carries the same text at [`../harness/LICENSE`](../harness/LICENSE), and generated harnesses place it at `.hive/LICENSE-AIGENT-HIVE.txt`.

The file-level mapping is defined in [`../REUSE.toml`](../REUSE.toml), and the REUSE canonical text is [`../LICENSES/Apache-2.0.txt`](../LICENSES/Apache-2.0.txt).

## 동봉한 상류 원본

- 위치: `harness/upstream/<이름>/`. 원본 파일은 수정 없이 바이트 그대로 보관, 원본마다 `LICENSE`(Apache-2.0 원본은 `NOTICE`가 있으면 함께) 동봉
- 고정 commit과 사용 Skill: [`UPSTREAM.lock`](../harness/upstream/UPSTREAM.lock). 출처와 저작권 표시: [`THIRD_PARTY_NOTICES.md`](../harness/upstream/THIRD_PARTY_NOTICES.md)
- 라이선스: oh-my-claudecode `ai-slop-cleaner`·hallmark는 MIT([`LICENSES/MIT.txt`](../LICENSES/MIT.txt)), frontend-design·web-artifacts-builder·impeccable은 Apache-2.0. 파일별 대응은 `REUSE.toml`
- 갱신: `scripts/sync-upstream.py`와 주간 Upstream sync 워크플로가 바뀐 원본을 그대로 교체하는 PR 생성, 자동 병합 없음. Hive 덮어쓰기·충돌 판정·사용자 선호는 원본 폴더 밖에 있어 교체 뒤에도 유지
- `SKILL.md` 본문의 출처 링크 금지
