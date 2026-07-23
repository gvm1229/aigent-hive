# Wiki operation log

Wiki에 반영한 ingest, query file-back, lint repair와 삭제 작업을 간결하게 기록.
삭제된 본문은 복제하지 않음.

Canonical page integration은 한 writer가 `.hive/index/.knowledge.lock` 아래에서
직렬화한다. 여러 read-only extraction 결과는 prepared draft로 전달할 수 있다.
