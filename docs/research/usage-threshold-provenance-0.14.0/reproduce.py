"""Synthetic 0.14.0 threshold/progress diagnosis; no user data or provider calls."""
from __future__ import annotations

import argparse
import copy
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

import yaml

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT))


def sha(data):
    return "sha256:" + hashlib.sha256(data).hexdigest()


def decision(origin="explicit"):
    return {"origin": origin, "input_channel": "migration" if origin == "legacy-unknown" else "terminal",
            "scope": "user", "question_id": None, "question_revision": None,
            "policy_version": "0.14.0", "changed_in_version": "0.14.0",
            "decided_at": None if origin == "legacy-unknown" else 1, "value_digest": sha(b"20")}


def configuration(origin=None):
    value = copy.deepcopy(json.loads((ROOT / "tests/fixtures/interactive-cli/model-fixed-choices.json").read_text("utf-8"))["user-quick"])
    value["schema_version"] = 2
    value["wiki"]["enabled"] = False
    value["usage_guard"].update(warnings_enabled=True,
        preflight={"threshold_enabled": False, "reset_enabled": False, "zero_quota_enabled": False, "observation_failure": "warn"},
        midway={"enabled": False, "threshold_enabled": False, "reset_enabled": False, "zero_quota_enabled": False, "observation_failure": "warn"})
    if origin:
        value["preference_decisions"] = {"/usage_guard/stop_remaining_percent": decision(origin)}
    return value


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    args = parser.parse_args()
    binary = args.binary.resolve(strict=True)
    work = args.work.absolute()
    assert not work.exists() and work.resolve().is_relative_to((ROOT / "tests/work").resolve())
    for parent in work.parents:
        if parent.exists():
            assert not parent.is_symlink() and not parent.is_junction()
        if parent == ROOT:
            break
    work.mkdir(parents=True)
    env = {k:v for k,v in os.environ.items() if not k.startswith(("HIVE_", "CODEX_"))}
    for key in ["HOME", "USERPROFILE", "APPDATA", "LOCALAPPDATA", "CODEX_HOME", "TEMP", "TMP", "HIVE_INSTALL_ROOT", "HIVE_CACHE_ROOT"]:
        path=work / "environment" / key.lower(); path.mkdir(parents=True)
        env[key]=str(path)
    fake_bin=work / "fake-bin"; fake_bin.mkdir(); env["PATH"]=str(fake_bin)
    # Reuse the existing deterministic host stub; never execute the real Codex host.
    from tests.conformance.integration.test_project_lifecycle import ProjectLifecycleConformance
    stub=ProjectLifecycleConformance("runTest")
    stub.fake_bin=fake_bin; stub.work_root=work; stub.host_log=work/"host-commands.jsonl"
    stub._write_fake_host("codex")
    env["HIVE_TEST_HOST_LOG"]=str(stub.host_log)
    rows=[]

    def run(label, root, arguments, expected=None):
        path=root / ".hive/config/user-setup.yml"
        before=path.read_bytes()
        pending=root / ".hive/config/setup-progress/user.yml"
        had_pending=pending.exists()
        pending_phase=yaml.safe_load(pending.read_text("utf-8")).get("phase") if had_pending else None
        command=[str(binary),*map(str,arguments),"--user-root",str(root),"--output","json"]
        result=subprocess.run(command,cwd=work,env={**env,"HIVE_TEST_USER_ROOT":str(root)},capture_output=True,text=True,encoding="utf-8",timeout=180)
        value=json.loads(result.stdout); assert result.returncode==value["exit_code"]
        after=path.read_bytes(); current=yaml.safe_load(after)
        record=current.get("preference_decisions",{}).get("/usage_guard/stop_remaining_percent")
        number=current["usage_guard"]["stop_remaining_percent"]
        row={"case":label,"exit_code":value["exit_code"],"code":value["code"],
             "message":value["message"],"diagnostic":(value.get("data") or {}).get("diagnostic"),
             "pending_before":had_pending,"pending_phase_before":pending_phase,"pending_after":pending.exists(),"threshold_after":number,
             "config_hash_before":sha(before),"config_hash_after":sha(after),"config_unchanged":before==after,
             "provenance_matches_threshold":None if record is None else record["value_digest"]==sha(str(number).encode())}
        rows.append(row)
        (work / "observations.json").write_text(json.dumps({"platform":sys.platform,"binary_sha256":sha(binary.read_bytes()),"synthetic_only":True,"cases":rows},ensure_ascii=False,indent=2)+"\n",encoding="utf-8")
        print(label,value["exit_code"],value["code"],"threshold",number,"pending_before",had_pending,flush=True)
        if expected is not None:
            assert result.returncode==expected,(label,value["message"],row["diagnostic"])
        return value

    def root_for(name,origin=None):
        root=work / name; path=root / ".hive/config/user-setup.yml"; path.parent.mkdir(parents=True)
        path.write_text(yaml.safe_dump(configuration(origin),allow_unicode=True,sort_keys=False),encoding="utf-8")
        return root

    arguments=["usage","threshold","--remaining-percent","25"]
    root=root_for("no-provenance")
    run("no-provenance-no-pending",root,arguments,0)
    for origin in ["explicit","legacy-unknown"]:
        root=root_for("with-"+origin,origin)
        run(origin+"-no-pending",root,arguments,3)
        assert rows[-1]["config_unchanged"] and not rows[-1]["pending_before"] and not rows[-1]["pending_after"]
    root=root_for("same-value","explicit")
    run("same-value-noop",root,["usage","threshold","--remaining-percent","20"],0)
    assert rows[-1]["config_unchanged"]
    patch=work / "patch.json"
    patch.write_text(json.dumps({"schema_version":1,"kind":"hive.setup-answer-patch","answers":{"usage_guard":{"stop_remaining_percent":25}}}),encoding="utf-8")
    root=root_for("file-patch","explicit")
    run("file-patch-no-pending",root,["setup","--scope","user","--answers",patch,"--input-channel","agent","--apply"],0)
    assert rows[-1]["threshold_after"]==25 and rows[-1]["provenance_matches_threshold"]
    run("dedicated-after-correct-file-patch",root,["usage","threshold","--remaining-percent","26"],3)
    assert rows[-1]["config_unchanged"] and rows[-1]["pending_phase_before"]=="complete"
    root=root_for("pending-other-intent","explicit")
    full=work / "current.json"; full.write_text(json.dumps(configuration("explicit")),encoding="utf-8")
    run("create-synthetic-pending",root,["setup","--progress","save","--scope","user","--step","usage-enforcement","--answers",full],0)
    path=root / ".hive/config/setup-progress/user.yml"; saved=yaml.safe_load(path.read_text("utf-8")); saved["phase"]="applying"; path.write_text(yaml.safe_dump(saved,allow_unicode=True,sort_keys=False),encoding="utf-8")
    run("file-patch-other-pending-intent",root,["setup","--scope","user","--answers",patch,"--input-channel","agent","--apply"],3)
    assert rows[-1]["config_unchanged"] and rows[-1]["pending_before"]
    run("threshold-with-pending",root,arguments,3)
    assert rows[-1]["config_unchanged"]


if __name__=="__main__":
    main()
