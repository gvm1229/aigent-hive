#!/usr/bin/env python3
"""Check the canonical active plan; --write refreshes only generated sections."""

import argparse
import json
from pathlib import Path
import sys

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from scripts.plan_state import PlanError, load_plan, render


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=ROOT)
    parser.add_argument("--write", action="store_true")
    args = parser.parse_args()
    try:
        plan = load_plan(args.root)
        changes = render(plan, write=args.write)
        status = "updated" if args.write and changes else "stale" if changes else "valid"
        print(json.dumps({"status": status, "version": plan.version,
                          "criteria": len(plan.criteria), "changed_paths": changes}, ensure_ascii=False))
        return int(bool(changes) and not args.write)
    except (PlanError, OSError) as error:
        print(json.dumps({"status": "invalid", "message": str(error)}, ensure_ascii=False))
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
