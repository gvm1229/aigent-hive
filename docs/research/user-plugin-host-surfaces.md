# User plugin host surface 조사

- 조사일: 2026-07-27
- 범위: Codex·Claude Code·Antigravity user plugin, global guidance, global·project Skill
- 결론: 세 host native plugin packaging 채택, host별 guidance marker append, project
  portable `.agents/skills/` 유지

## Capability matrix

| Host | Native plugin package | User install·discovery | Global guidance | Global Skill | Project Skill | Qualification |
| --- | --- | --- | --- | --- | --- | --- |
| Codex CLI | `.codex-plugin/plugin.json`, `skills/` | local marketplace 등록 후 `codex plugin add` | `~/.codex/AGENTS.override.md` 우선, 없으면 `~/.codex/AGENTS.md` | `$HOME/.agents/skills/` | `<repo>/.agents/skills/` | local `0.145.0` fixed argv 성공 |
| Claude Code | `.claude-plugin/plugin.json`, `skills/` | marketplace 등록·`--scope user` install | `~/.claude/CLAUDE.md` | `~/.claude/skills/` | `<repo>/.claude/skills/` | 공식 CLI contract 확인, local executable 부재 |
| Antigravity | root `plugin.json`, `skills/` | `agy plugin validate`·`agy plugin install`·registry import | `~/.gemini/GEMINI.md` | `~/.gemini/config/skills/` | `<repo>/.agents/skills/` | local authenticated `agy 1.1.7` install·repeat update 성공 |

## Hive adapter

### Codex

- Package: `~/.hive/marketplaces/codex/plugins/aigent-hive/`
- Marketplace: `~/.hive/marketplaces/codex/.agents/plugins/marketplace.json`
- Activation:
  `codex plugin marketplace add <marketplace-root> --json` →
  `codex plugin add aigent-hive@aigent-hive --json`
- Guidance target: active non-empty `AGENTS.override.md` 우선, 그 외 `AGENTS.md`
- Existing OMX·foreign bytes 보존과 `AIGENT-HIVE:USER` own-block 교체

### Claude Code

- Package: `~/.hive/marketplaces/claude/plugins/aigent-hive/`
- Marketplace: `~/.hive/marketplaces/claude/.claude-plugin/marketplace.json`
- Activation: plugin validate → marketplace add/update → user-scope install/update
- Guidance target: `~/.claude/CLAUDE.md`
- Existing OMC·foreign bytes 보존과 `AIGENT-HIVE:USER` own-block 교체
- Actual Claude executable qualification: release 보호 환경 과제

### Antigravity

- Hive-owned source package:
  `~/.hive/marketplaces/antigravity/plugins/aigent-hive/`
- Host-owned staging package: `~/.gemini/config/plugins/aigent-hive/`
- Required marker: root `plugin.json`
- Activation: `agy plugin validate <source-package>` →
  `agy plugin install <source-package>`
- Discovery proof: `agy plugin list`의 `imports[].name == "aigent-hive"`
- Rollback: 신규 등록은 `agy plugin uninstall aigent-hive`, 기존 등록 갱신은
  복구된 이전 source package 재설치
- Compatibility projection: `~/.gemini/config/skills/`
- Guidance target: `~/.gemini/GEMINI.md`
- Ownership exclusion: `~/.gemini/config/import_manifest.json`과 staging package는
  `agy` 소유, Hive ownership manifest 제외
- Mutation 전 host staging 전체 경로·바이트를 authenticated prior package와 비교하고
  unknown file·directory·symlink는 conflict로 보존
- 기존 `0.7.0` directory-scan 설치는 고정 source-release digest inventory로만
  native registry 설치로 이동

## Version boundary

| Surface | Supported range metadata | Evidence |
| --- | --- | --- |
| Codex plugin | `>=0.145.0 <1.0.0` | local CLI help·install command qualification |
| Claude Code plugin | `>=2.1.0 <3.0.0` | current plugin marketplace·scope documentation |
| Antigravity plugin | `>=1.1.7 <1.2.0` | local `agy 1.1.7`, fixed argv install·list·repeat update |
| Antigravity usage sensor | CLI `1.1.7` | 별도 usage TUI 조사 |

## Local qualification

- 실제 사용자 설치 migration dry-run·apply·validate 성공
- `agy plugin list`: `source=antigravity`, `components=["skills"]`
- Source package 16개와 host staging 16개 exact path·byte parity
- 동일 `0.7.0` repeat update와 validate 성공
- `--validate`는 native registry와 host staging 전체 parity도 read-only 검증
- Same source/staging path: first install의 source self-overwrite와 second install
  failure, path separation 필수
- Claude Code는 executable·subscription 부재로 실제 install/update 미검증

## 근거

- [OpenAI Codex AGENTS.md](https://developers.openai.com/codex/guides/agents-md)
- [OpenAI Codex Skills](https://developers.openai.com/codex/skills)
- [OpenAI Codex app-server plugin methods](https://github.com/openai/codex/blob/main/codex-rs/app-server/README.md)
- [Anthropic Claude Code marketplace](https://code.claude.com/docs/en/plugin-marketplaces)
- [Anthropic Claude Code plugin reference](https://code.claude.com/docs/en/plugins-reference)
- [Anthropic Claude Code memory](https://code.claude.com/docs/en/memory)
- [Anthropic Claude Code Skills](https://code.claude.com/docs/en/skills)
- [Google Antigravity CLI plugins](https://antigravity.google/docs/cli/plugins)
- [Google Antigravity CLI migration](https://antigravity.google/docs/cli/gcli-migration)
- [Firebase Antigravity CLI extensions](https://firebase.google.com/docs/ai-assistance/gcli-extension?hl=en)
- [Google Antigravity Skills](https://antigravity.google/docs/skills?app=antigravity-ide)
- [Google Antigravity Rules](https://antigravity.google/docs/ide/rules)
