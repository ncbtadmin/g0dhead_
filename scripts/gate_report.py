#!/usr/bin/env python3
"""gate_report.py — the verification gate's report producer (SLICE_10 §3; H3(6)).

The format belongs to the dev-pipeline SKILL.md; only a producer can keep it —
nine slices of hand-composed blocks eroded field by field, one slice was gated
in fact but never reported in form, and the erosion's cause was hand
composition. This script runs doc 00 §4's three steps (the Rust override),
parses their real output, and emits the block — including the step count and
the unverifiable line — so no future block is hand-composed.

Usage:  python scripts/gate_report.py
Runs from the repo root (or locates it from its own path). Prints the block
and writes it to docs/dev/GATE_REPORT.txt (UTF-8, no BOM, LF — the R2
discipline). Exit code 0 iff the gate passed WHOLE: a run whose tests skipped
for a missing DATABASE_URL is not a full gate pass and fails here.

The test step runs multithreaded (fast); any binary that comes back red is
reran single-threaded in its own serial bucket before a regression is
suspected (SLICE_10 §8), because the shared live Railway DB makes parallel
runs of the singleton-touching tests (bias, rebalance, concordat adoption)
nondeterministic. A binary that fails even single-threaded is a real
failure and keeps the gate red; a binary that clears on the serial rerun is
reported as such, never silently.
"""

import os
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / "docs" / "dev" / "GATE_REPORT.txt"
CARGO = "cargo"

STEPS = [
    ("cargo fmt --check", [CARGO, "fmt", "--check"]),
    ("cargo clippy --workspace --all-targets", [CARGO, "clippy", "--workspace", "--all-targets"]),
    # --no-fail-fast is load-bearing here: cargo's default stops at the FIRST
    # red test binary, so a single early shared-DB flake would abort the run
    # and hide every binary after it — and the serial-rerun pass below would
    # only ever see (and clear) that one aborting binary, a false PASS. With
    # --no-fail-fast the whole suite executes and every flaky binary is
    # collected, so each gets its own serial rerun.
    ("cargo test --workspace --no-fail-fast", [CARGO, "test", "--workspace", "--no-fail-fast"]),
]


def run(cmd):
    env = dict(os.environ)
    env["PATH"] = r"C:\Users\mbc21\.cargo\bin;" + env.get("PATH", "")
    proc = subprocess.run(
        cmd, cwd=ROOT, capture_output=True, text=True, encoding="utf-8", errors="replace", env=env
    )
    return proc.returncode, (proc.stdout or "") + (proc.stderr or "")


def summarize_tests(output):
    """Sum cargo test's per-binary 'test result:' lines; surface DB skips."""
    passed = failed = ignored = binaries = 0
    for m in re.finditer(
        r"^test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored", output, re.M
    ):
        binaries += 1
        passed += int(m.group(2))
        failed += int(m.group(3))
        ignored += int(m.group(4))
    db_skips = len(re.findall(r"SKIP: DATABASE_URL unset", output))
    return passed, failed, ignored, binaries, db_skips


def rerun_specs(output):
    """cargo names each failed test binary in a hint line:
    ``error: test failed, to rerun pass `-p <pkg> --test <name>` ``. Pull the
    unique rerun specs so each red binary can go back through its own serial
    bucket (SLICE_10 §8) before a regression is suspected."""
    seen, specs = set(), []
    for m in re.finditer(r"to rerun pass `([^`]+)`", output):
        spec = m.group(1).strip()
        if spec not in seen:
            seen.add(spec)
            specs.append(spec)
    return specs


def main():
    lines = ["== dev-pipeline gate: g0dhead_ ==", "languages: rust (doc 00 §4 override)"]
    all_pass = True
    step_count = 0

    for label, cmd in STEPS:
        code, output = run(cmd)
        step_count += 1
        if label.endswith("--check"):
            detail = "all files formatted" if code == 0 else "formatting drift (see cargo fmt)"
        elif "clippy" in label:
            warnings = len(re.findall(r"^warning(?::|\[)", output, re.M))
            detail = (
                f"clean (workspace lints deny warnings; {warnings} warnings)"
                if code == 0
                else "lints failed"
            )
        else:
            passed, failed, ignored, binaries, db_skips = summarize_tests(output)
            detail = f"{passed} passed; {failed} failed; {ignored} ignored across {binaries} binaries"
            if db_skips:
                detail += f"; {db_skips} DATABASE_URL skips — NOT a full gate pass"
                code = code or 1
            elif code != 0:
                # The multithreaded pass left binaries red. On a shared live
                # DB that is singleton contention (bias/rebalance/concordat),
                # not a defect — the suite is green single-threaded (SLICE_10
                # §8). Rerun each failed binary in its own serial bucket and
                # clear it only if it then passes green; a binary that fails
                # even single-threaded is a real regression and stays red.
                specs = rerun_specs(output)
                cleared, still_red = [], []
                for spec in specs:
                    rc, rout = run(
                        [CARGO, "test", *spec.split(), "--no-fail-fast", "--", "--test-threads=1"]
                    )
                    _, rf, _, rb, rs = summarize_tests(rout)
                    if rc == 0 and rf == 0 and rs == 0 and rb >= 1:
                        cleared.append(spec)
                    else:
                        still_red.append((spec, rout))
                if specs and not still_red:
                    code = 0
                    detail += (
                        f"; {len(cleared)} binary(ies) reran single-threaded to clear "
                        "documented shared-DB singleton contention (SLICE_10 §8): "
                        + ", ".join(f"`{s}`" for s in cleared)
                        + " — all green on serial rerun"
                    )
                elif still_red:
                    detail += "; serial rerun did NOT clear " + ", ".join(
                        f"`{s}`" for s, _ in still_red
                    )
                    output = still_red[0][1]  # surface the real failure's tail
        status = "PASS" if code == 0 else "FAIL"
        if code != 0:
            all_pass = False
            tail = "\n".join(output.strip().splitlines()[-25:])
            detail += "\n" + "\n".join(f"    | {line}" for line in tail.splitlines())
        lines.append(f"[{status}] {label}: {detail}")
        if code != 0 and "fmt" not in label and "clippy" not in label:
            break  # a failed test step ends the gate; fmt/clippy failures still show the rest

    lines.append(f"result: {'PASS (' + str(step_count) + ' steps)' if all_pass else 'FAIL'}")
    lines.append(
        "unverifiable in sandbox: n/a — execution locus is the host against live "
        "Railway Postgres (doc 00 §4, fourth override); a run without DATABASE_URL "
        "is reported above as NOT a full gate pass"
    )

    block = "\n".join(lines) + "\n"
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write(block)
    sys.stdout.write(block)
    return 0 if all_pass else 1


if __name__ == "__main__":
    sys.exit(main())
