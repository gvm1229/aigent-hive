#!/usr/bin/env python3
"""Check exact CLI bytes with simulated native registration, without AI calls."""
from __future__ import annotations

import argparse
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile

import yaml

ROOT = Path(__file__).resolve().parents[1]

FAKE = r'''
import json, os
from pathlib import Path
root=Path(os.environ['HIVE_DELIVERY_ROOT'])
file=root/'registration.json'
state=json.loads(file.read_text()) if file.exists() else {'market':False,'plugin':False,'enabled':True}
action=os.environ['HIVE_DELIVERY_ACTION']
market=root/'.hive/marketplaces/codex'
if action=='market-list':
    print(json.dumps({'marketplaces':[{'name':'aigent-hive','root':str(market),'marketplaceSource':{'sourceType':'local','source':str(market)}}] if state['market'] else []}))
elif action=='plugin-list':
    entries=[{'pluginId':'aigent-hive@aigent-hive','name':'aigent-hive','marketplaceName':'aigent-hive','version':os.environ['HIVE_DELIVERY_PRODUCT'],'installed':True,'enabled':state['enabled'],'source':{'source':'local','path':str(market/'plugins/aigent-hive')},'marketplaceSource':{'sourceType':'local','source':str(market)}}] if state['plugin'] else []
    print(json.dumps({'installed':entries,'available':[]}))
else:
    if action=='market-add': state['market']=True
    elif action=='market-remove': state['market']=False
    elif action=='plugin-add':
        if os.environ.get('HIVE_DELIVERY_FAIL')=='1': raise SystemExit(1)
        state['plugin']=True
    elif action=='plugin-remove': state['plugin']=False
    else: raise SystemExit(2)
    file.write_text(json.dumps(state)); print('{}')
'''


def native_registration(bin_dir: Path) -> None:
    helper = bin_dir / "registration.py"
    helper.write_text(FAKE, encoding="utf-8")
    command = f'"{sys.executable}" "{helper}"'
    if os.name == "nt":
        command = f'"{command}"'
    source = r'''
#include <stdlib.h>
#include <stdio.h>
#include <string.h>
int main(int argc, char **argv) {
  const char *action=NULL;
  if(argc==2 && strcmp(argv[1],"--version")==0) { puts("codex-cli 0.999.9"); return 0; }
  if(argc>=3 && strcmp(argv[1],"plugin")==0) {
    if(strcmp(argv[2],"list")==0) action="plugin-list";
    else if(strcmp(argv[2],"install")==0 || strcmp(argv[2],"add")==0) action="plugin-add";
    else if(strcmp(argv[2],"remove")==0) action="plugin-remove";
    else if(argc>=4 && strcmp(argv[2],"marketplace")==0) {
      if(strcmp(argv[3],"list")==0) action="market-list";
      else if(strcmp(argv[3],"add")==0) action="market-add";
      else if(strcmp(argv[3],"remove")==0) action="market-remove";
    }
  }
  if(!action) return 2;
#ifdef _WIN32
  _putenv_s("HIVE_DELIVERY_ACTION",action);
#else
  setenv("HIVE_DELIVERY_ACTION",action,1);
#endif
  return system(COMMAND)==0 ? 0 : 1;
}
'''.replace("COMMAND", json.dumps(command))
    src = bin_dir / "registration.c"
    src.write_text(source, encoding="utf-8")
    compiler = shutil.which("clang") or shutil.which("gcc") or shutil.which("cc")
    if not compiler:
        raise RuntimeError("native registration fixture requires an existing C compiler")
    exe = bin_dir / ("codex.exe" if os.name == "nt" else "codex")
    subprocess.run([compiler, str(src), "-o", str(exe)], check=True, capture_output=True)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--product-version", required=True)
    parser.add_argument("--package-version", required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    scenarios = []
    with tempfile.TemporaryDirectory(prefix="hive-delivery-") as scratch:
        work = Path(scratch).resolve()
        user = work / "user"
        user.mkdir()
        bin_dir = work / "bin"
        bin_dir.mkdir()
        native_registration(bin_dir)
        environment = dict(os.environ, PATH=str(bin_dir) + os.pathsep + os.environ.get("PATH", ""),
                           HIVE_DELIVERY_ROOT=str(user), HIVE_DELIVERY_PRODUCT=args.product_version)
        probe = subprocess.run([str(bin_dir / ("codex.exe" if os.name == "nt" else "codex")), "plugin", "marketplace", "list", "--json"],
                               env=environment, capture_output=True, text=True)
        if probe.returncode:
            raise RuntimeError("registration fixture failed before CLI testing: " + probe.stderr)

        def invoke(*command: str) -> dict:
            result = subprocess.run([str(binary), *command], env=environment, text=True,
                                    encoding="utf-8", capture_output=True, timeout=120)
            if result.returncode:
                raise RuntimeError(f"CLI qualification failed: {command[0]}: {result.stderr.strip()}")
            value = json.loads(result.stdout)
            if value["status"] != "success":
                raise RuntimeError("CLI did not report success")
            return value

        version = subprocess.check_output([str(binary), "--version"], text=True).strip()
        assert args.package_version in version, version
        config = {
            "schema_version": 1, "interface_language": "en", "wiki": {"enabled": False, "language": "both"},
            "profile": {"contexts": ["web-developer"]}, "persona": {"id": "balanced"}, "selected_hosts": ["codex"],
            "skills": {"mode": "individual", "selected": ["user-setup", "prompt-refine"]},
            "usage_guard": {"enabled": False, "stop_remaining_percent": 20, "codexbar_fallback_enabled": False},
        }
        answers = work / "user.yml"
        answers.write_text(yaml.safe_dump(config), encoding="utf-8")
        invoke("setup", "--scope", "user", "--answers", str(answers), "--user-root", str(user), "--apply", "--output", "json")
        assert not (user / ".agents/skills/user-setup/SKILL.md").exists()
        scenarios.append("fresh-user-plugin-only")

        # Simulate an exact older common projection. Never import real user data.
        manifest_path = user / ".hive/install/user-projection.json"
        manifest = json.loads(manifest_path.read_text())
        stock = user / ".hive/marketplaces/codex/plugins/aigent-hive/skills/user-setup"
        for file in stock.rglob("*"):
            if not file.is_file():
                continue
            relative = ".agents/skills/user-setup/" + file.relative_to(stock).as_posix()
            target = user / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            data = file.read_bytes()
            target.write_bytes(data)
            digest = "sha256:" + hashlib.sha256(data).hexdigest()
            manifest["entries"].append({"path": relative, "digest": digest})
            manifest["base_entries"].append({"path": relative, "digest": digest, "content": data.decode("utf-8")})
        manifest.update(product_version="0.11.0", package_version="0.11.0")
        for key in ("entries", "base_entries"):
            manifest[key].sort(key=lambda entry: entry["path"])
        manifest_path.write_text(json.dumps(manifest), encoding="utf-8")
        result = invoke("update", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply", "--output", "json")
        assert not (user / ".agents/skills/user-setup/SKILL.md").exists()
        assert any(".agents/skills/user-setup/SKILL.md" == path for path in result["changed_paths"])
        result = invoke("update", "--scope", "user", "--host", "codex", "--user-root", str(user), "--apply", "--output", "json")
        assert not result["changed_paths"]
        scenarios.append("user-update-automatic-cleanup-and-idempotence")

        project = work / "project"
        project.mkdir()
        foreign = project / "README.md"
        foreign.write_bytes(b"Foreign project content\n")
        project_answers = yaml.safe_load((ROOT / "tests/fixtures/setup/answers-base.yml").read_text())
        project_answers.update(setup_mode="custom", interface_language="en", wiki={"enabled": False, "language": "both"},
                               persona={"id": "balanced"}, skills={"mode": "individual", "selected": ["prompt-refine", "project-setup"]})
        path = work / "project.yml"
        path.write_text(yaml.safe_dump(project_answers), encoding="utf-8")
        invoke("setup", "--target", str(project), "--answers", str(path), "--capabilities", str(ROOT / "tests/fixtures/setup/capabilities-codex-host-native.json"),
               "--user-root", str(user), "--apply", "--output", "json")
        assert not (project / ".agents/skills/prompt-refine/SKILL.md").exists()
        assert (project / ".agents/skills/project-setup/SKILL.md").is_file()
        scenarios.append("project-only-selection-preserved")
        upgrade = ("project", "upgrade", "--target", str(project), "--user-root", str(user))
        state = json.loads((user / "registration.json").read_text())
        state["enabled"] = False
        (user / "registration.json").write_text(json.dumps(state))
        invoke(*upgrade, "--apply", "--output", "json")
        assert (project / ".agents/skills/prompt-refine/SKILL.md").is_file()
        state["enabled"] = True
        (user / "registration.json").write_text(json.dumps(state))
        invoke(*upgrade, "--apply", "--output", "json")
        assert not (project / ".agents/skills/prompt-refine/SKILL.md").exists()
        invoke(*upgrade, "--validate", "--output", "json")
        assert foreign.read_bytes() == b"Foreign project content\n"
        scenarios.append("project-update-restoration-cleanup-and-foreign-preservation")
    report = {"schema_version": 1, "status": "passed", "product_version": args.product_version,
              "package_version": args.package_version, "binary_sha256": hashlib.sha256(binary.read_bytes()).hexdigest(),
              "os": sys.platform, "registration": "simulated-native", "actual_ai_verified": False, "scenarios": scenarios}
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report))


if __name__ == "__main__":
    main()
