# Architecture 안내

현재 동작 구조와 trust boundary의 정본.

| 문서 | 범위 |
| --- | --- |
| [Source layout](source-layout.md) | Source·release·consumer tree와 crate ownership |
| [한국어 언어 core](korean-language-core.md) | 자동 한국어 검사·윤문·pack lifecycle |
| [Role lifecycle](role-lifecycle.md) | Persistent role identity·handoff |
| [Run lifecycle](run-lifecycle.md) | Criterion·checkpoint·event·scheduler·receipt·cancel·resume |
| [Skill consent](skill-consent.md) | Optional Skill approval·activation |
| [Hook consent](hook-consent.md) | Fallback hook preview·consent·inert state |
| [Judge trust boundary](judge-trust-boundary.md) | Clean-context package·Ed25519 quorum |
| [Release·update trust boundary](release-update-trust-boundary.md) | Attestation·local integrity·migration·recovery |

관련 결정: [Decision 안내](../decisions/README.md).
