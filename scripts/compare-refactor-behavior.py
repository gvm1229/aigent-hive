#!/usr/bin/env python3
"""Compare fixed CLI behavior with preserved binaries; no model or provider calls."""
from pathlib import Path
import argparse,hashlib,json,sys
ROOT=Path(__file__).resolve().parents[1]
sys.path.insert(0,str(ROOT))
from tests.conformance.contracts.test_run_role_contracts import Phase4Contracts,CAPABILITIES,USAGE_ACCOUNT_DIGEST,snapshot_tree
Phase4Contracts.setUpClass()
parser=argparse.ArgumentParser(description=__doc__)
parser.add_argument("--before",type=Path,required=True)
parser.add_argument("--after",type=Path,required=True)
parser.add_argument("--work",type=Path,required=True)
args=parser.parse_args()
work=args.work.resolve()
if not work.is_relative_to((ROOT/"tests/work").resolve()) or work.exists():
    raise ValueError("choose a new exact directory under tests/work")
work.mkdir(parents=True)
binaries={"before":args.before.resolve(),"after":args.after.resolve()}
report={"schema_version":1,"scope":"synthetic CLI behavior, not model behavior or host execution","rows":{},"binary_digests":{k:hashlib.sha256(v.read_bytes()).hexdigest() for k,v in binaries.items()}}
for label,binary in binaries.items():
    c=Phase4Contracts();c.binary=binary;c.work=work/label;c.work.mkdir();c.input_root=c.work/"inputs";c.input_root.mkdir();c.capability_copy_index=0
    c.fake_home=c.work/"home";c.fake_home.mkdir();c.home_snapshot=snapshot_tree(c.fake_home)
    results=[]
    for name,state,sensor in [("manual","executing",None),("blocked","blocked",None),("cancelled","cancelled",None),("automatic-allow","executing","allow"),("automatic-limit","executing","threshold"),("missing-session","executing","allow")]:
        target=c.fresh_target(name);c.ensure_handoff(target)
        request=c.checkpoint_request(state=state,next_action=None if state=="cancelled" else "continue",blocked_criteria=["build","tests"] if state=="blocked" else [],blocker="synthetic wait" if state=="blocked" else None)
        p,value=c.checkpoint(target,request,CAPABILITIES["codex-omx"],name);c.assert_success(p,value)
        prior=snapshot_tree(target);capability=c.fresh_capability_input(CAPABILITIES["codex-omx"],name)
        args=["run","resume","--target",target,"--run","demo","--capabilities",capability,"--output","json"]
        sensor_log=c.work/(name+"-sensor.jsonl")
        env=None
        if sensor:
            args += ["--dispatch-intent","automatic","--account-digest",USAGE_ACCOUNT_DIGEST,"--role","reviewer"]
            if name!="missing-session":
                args += ["--session-id","fixture-session"]
                if label=="after":args += ["--process-id","4242"]
            env=c.fake_codexbar_environment(sensor,log=sensor_log)
        p,value=c.run_cli(*args,extra_environment=env)
        after=snapshot_tree(target)
        assert all(after[path]==entry for path,entry in prior.items()),name
        added=sorted(path for path in after.keys()-prior.keys() if after[path][0]=="file")
        assert all(path.startswith(".hive/runtime/") for path in added)
        row={"case":name,"exit_code":p.returncode,"code":value["code"],"brief_count":len((value.get("data") or {}).get("dispatch_briefs",[])),"new_files":added,"existing_bytes_preserved":True,"sensor_calls":len(sensor_log.read_text().splitlines()) if sensor_log.exists() else 0}
        if name in ("manual","blocked","cancelled"):assert not added
        results.append(row)
        if name=="automatic-allow":
            log_before=row["sensor_calls"];p,value=c.run_cli(*args,extra_environment=env)
            results.append({"case":"automatic-replay","exit_code":p.returncode,"code":value["code"],"brief_count":len(value["data"]["dispatch_briefs"]),"new_files":value["changed_paths"],"existing_bytes_preserved":snapshot_tree(target)==after,"sensor_calls":len(sensor_log.read_text().splitlines())-log_before})
    p,value=c.run_cli("route","--request",ROOT/"tests/fixtures/skills/routes/simple-question.json","--output","json")
    results.append({"case":"simple-question","exit_code":p.returncode,"code":value["code"],"route":value.get("data"),"sensor_calls":0})
    report["rows"][label]=results
before={r["case"]:r for r in report["rows"]["before"]};after={r["case"]:r for r in report["rows"]["after"]}
for name in before:
    if name=="missing-session":
        assert before[name]["exit_code"]==0 and after[name]["exit_code"]==2
        assert after[name]["new_files"]==[] and after[name]["sensor_calls"]==0
    else:
        for key in ("exit_code","code","brief_count","route","sensor_calls"):
            assert before[name].get(key)==after[name].get(key),(name,key,before[name],after[name])
report["preserved_result_cases"]=7
report["approved_compatibility_exception"]="missing-session automatic calls reject before sampling or writing; current automatic input adds required positive PID"
report["runtime_difference"]="current allowed/limited automatic preflight records bounded shared session halt/observation state"
(work/"report.json").write_text(json.dumps(report,indent=2)+"\n",encoding="utf-8")
print("Compared 8 cases: 7 preserved results, 1 approved session-binding exception; prior bytes preserved")
