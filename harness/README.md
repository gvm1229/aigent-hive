# Harness source

이 디렉터리는 소비자 프로젝트에 설치할 harness의 canonical source다.

`harness/**`와 여기에서 생성된 Aigent Hive 소유 파일·marker block은 Apache-2.0이다. 소비자 프로젝트의 기존 source, 문서, 설정과 data의 라이선스는 변경하지 않는다. 생성 결과에는 `.hive/LICENSE-AIGENT-HIVE.txt`가 포함된다.

- `template/`: Copier authoring·CI render source
- `skills/`: release가 host별로 projection할 portable Skill source
- `projections/`: host별 얇은 projection 규칙
- `profiles/`: domain profile 확장점
- `manifest.toml`: 생성 경로와 ownership 계약

루트 `.agents/`의 Hive 개발 지침은 이 디렉터리로 복사하지 않는다. 소비자 setup은 release에 포함된 Rust renderer가 수행하며 Python이나 Copier 설치를 요구하지 않는다.
Static Copier tree와 role seed 같은 dynamic materializer output은 별도 conformance fixture로 검증한다.
