# Harness source

이 디렉터리는 소비자 프로젝트에 설치할 harness의 canonical source.

`harness/**`와 여기에서 생성된 Aigent Hive 소유 파일·marker block의 라이선스:
Apache-2.0. 소비자 프로젝트의 기존 source, 문서, 설정, data 라이선스는 변경 대상에서
제외. 생성 결과에는 `.hive/LICENSE-AIGENT-HIVE.txt` 포함.

- `template/`: Copier authoring·CI render source
- `skills/`: release가 host별로 projection할 portable Skill source
- `projections/`: host별 얇은 projection 규칙
- `profiles/`: domain profile 확장점
- `manifest.toml`: 생성 경로와 ownership 계약

루트 `.agents/`의 Hive 개발 지침은 이 디렉터리 복사 대상에서 제외. 소비자 setup은
release에 포함된 Rust renderer가 수행하며 Python과 Copier 설치 요구 없음.
Static Copier tree와 role seed 같은 dynamic materializer output은 별도 conformance fixture로 검증.
