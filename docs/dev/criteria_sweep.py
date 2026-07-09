#!/usr/bin/env python3
"""criteria_sweep.py — the Document 8 narrowness sweep, generated not hand-composed.

Re-runnable against HEAD (PROMPT_D [D3] / PROMPT_H [H7.3]): rows come from the
machine; verdicts come from the adjudication ledger seeded in VERDICTS below and
are otherwise PENDING. A hand-composed table decays the way the slice-6 gate
block decayed; this one diffs.

Usage:  python docs/dev/criteria_sweep.py
Writes docs/dev/CRITERIA_SWEEP.md directly — UTF-8 without BOM, LF endings,
deterministically (R2, 2026-07-09): shell redirection is what put a BOM and
CRLF churn into the first committed copy, so the script owns its own bytes now.
Stdlib only. No cargo, no database, no network.
"""

import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
DOC8 = ROOT / "docs" / "08_phase_b_success_criteria.md"
OUT = ROOT / "docs" / "dev" / "CRITERIA_SWEEP.md"

# Adjudicated verdicts (witnessed in the round-2 ledger and/or ruled in
# docs/dev/PROMPT_G_RULINGS.md). Everything not listed: PENDING (cited) /
# UNWITNESSED (uncited, sections A-H) / DEFERRED (uncited, sections I-J).
VERDICTS = {
    "SC-D01": ("NARROWER", "arch half scans 2 crates by literal + 1 token; G2 widens to discovered crate list"),
    "SC-B04": ("NARROWER", "IPC scan covers godhead-store only; pub-mod half meets; G3 widens + orders the HTTP wall"),
    "SC-H06": ("NARROWER", "sweep = 3 tables x 1 pattern vs 26 tables x secrets::scan; refusal half meets; G4 widens"),
    "SC-C07": ("NARROWER", "3 of 8 entries wall-tested; criterion text narrowed in doc 08 (G6, 2026-07-09); signature-impossible entries await S4 arch pins; threshold/mandate entries pinned to slices 10/11"),
    "SC-E01a": ("MEETS-AS-SPLIT", "shape half of former SC-E01 (split per G5, doc 08 amended 2026-07-09); citing tests still say SC-E01 and are folded here; citation text updates ride with Slice 10"),
    "SC-E05": ("MINTED", "G5 (2026-07-09); construction site: Slice 10 riders (S4) — swallow sites, mount refusal, panicking execute, suite-end sweep"),
    "SC-A08": ("MINTED", "G8 (2026-07-09); construction site: Slice 10 — view-integrity sweep + the one-time archaeology pass (H4 NEW-2)"),
    "SC-H07": ("MINTED", "G2 (2026-07-09); construction site: Slice 10 riders (S4) — workspace fallback-shape arch scan"),
    "SC-I07a": ("MINTED", "G10 (2026-07-09); rides with Slice 10 — actor-class substrate authentication"),
    "SC-I07b": ("MINTED", "G11 (2026-07-09); rides with Slice 10 — admission legibility constants + standing notice"),
    "SC-F06": ("HALF+ANNOTATED", "the G13 model: unmet half named in-test, pinned to the Ollama slice"),
    "SC-K04": ("NARROWER", "sovereign-judgment flip is shape-preserving corruption the re-lint cannot catch; G7 content-hash closes the class"),
    "SC-A05": ("NARROWER", "skew's persisted RefusalRecord carries VALIDATION_FAILED not SCHEMA_MISMATCH (G1 miscode, fix ordered)"),
    "SC-K03": ("NARROWER", "ephemeral error is SchemaMismatch (right); persisted record code is not (same G1 miscode)"),
    "SC-H05": ("NARROWER", "exhaustion witnessed on one guarded path; release_lease unguarded, bias surfaces identity-less (B3)"),
    "SC-G01": ("MEETS-AS-RULED", "lawful wall test per G1; the agent-side mount refusal is IX.3 labor-rule debt owed by the Dogma, pinned to slice 10"),
    "SC-N04": ("MEETS+CLAIMS-SEAM", "G6 formally assigns the SC-C07 'crossing the seam' entry to this criterion's observation-window test"),
}

# Criterion ids may carry a single lowercase suffix (SC-E01a, SC-I07b) since the
# 2026-07-09 amendments.
ID_RE = re.compile(r"^- \*\*(SC-[A-N]\d{2}[a-z]?)\*\*\s*[—-]\s*(.+)$", re.M)
CITE_UPPER_RE = re.compile(r"SC-([A-N]\d{2}[a-z]?)")
CITE_FN_RE = re.compile(r"\bsc_([a-n]\d{2}[a-z]?)")

UNIVERSAL_RE = re.compile(r"\b(every|all |any |no |none|never|nothing|regardless|across any)\b", re.I)
ARCH_RE = re.compile(r"\b(architectural|arch/|compile-time|property test|property:)\b", re.I)


def parse_criteria():
    text = DOC8.read_text(encoding="utf-8")
    out = []
    for m in ID_RE.finditer(text):
        cid, body = m.group(1), m.group(2).strip()
        body = re.sub(r"\*\*", "", body)
        quant = []
        if UNIVERSAL_RE.search(body):
            quant.append("universal")
        if ARCH_RE.search(body):
            quant.append("arch")
        out.append((cid, body, "+".join(quant) or "plain"))
    return out


def find_citations():
    cites = {}
    for path in sorted(ROOT.glob("crates/*/tests/*.rs")):
        rel = path.relative_to(ROOT).as_posix()
        current_fn = "?"
        for lineno, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            fn = re.search(r"\bfn\s+([a-zA-Z0-9_]+)\s*\(", line)
            if fn:
                current_fn = fn.group(1)
            for cid in CITE_UPPER_RE.findall(line):
                cites.setdefault("SC-" + cid, []).append((current_fn, rel, lineno))
            for cid in CITE_FN_RE.findall(line):
                cites.setdefault("SC-" + cid.upper(), []).append((current_fn, rel, lineno))
    return cites


def fold_renamed(cites, ids):
    """A test citing a bare id (SC-E01) whose doc id gained a suffix (SC-E01a)
    folds onto the suffixed id — iff exactly one suffixed variant exists."""
    for cited in sorted(cites):
        if cited in ids:
            continue
        variants = sorted(i for i in ids if len(i) == len(cited) + 1 and i.startswith(cited))
        if len(variants) == 1:
            cites.setdefault(variants[0], []).extend(cites.pop(cited))
    return cites


def main():
    criteria = parse_criteria()
    ids = {cid for cid, _, _ in criteria}
    cites = fold_renamed(find_citations(), ids)
    lines = []
    lines.append("# CRITERIA SWEEP — Document 8 vs the test suite")
    lines.append("")
    lines.append("Generated by `docs/dev/criteria_sweep.py` — re-run against HEAD; do not hand-edit.")
    lines.append(f"Criteria found: {len(criteria)}. Verdicts: seeded from the round-2 adjudication")
    lines.append("ledger + PROMPT_G_RULINGS; PENDING rows are the parallel, non-blocking sweep (S1).")
    lines.append("")
    lines.append("| id | quantifier | citing test(s) | verdict | note |")
    lines.append("|----|-----------|----------------|---------|------|")
    counts = {}
    for cid, body, quant in criteria:
        cs = cites.get(cid, [])
        seen, shown = set(), []
        for fn, rel, ln in cs:
            if fn not in seen:
                seen.add(fn)
                shown.append(f"`{fn}` {rel}:{ln}")
        cite_txt = "<br>".join(shown[:4]) + (f"<br>(+{len(shown)-4} more)" if len(shown) > 4 else "") if shown else "—"
        if cid in VERDICTS:
            verdict, note = VERDICTS[cid]
        elif not cs and cid[3] in "IJ":
            verdict, note = "DEFERRED", "section unbuilt; the one place the bookkeeping is clean (A8)"
        elif not cs:
            verdict, note = "UNWITNESSED", "no test cites this criterion"
        else:
            verdict, note = "PENDING", ""
        counts[verdict] = counts.get(verdict, 0) + 1
        lines.append(f"| **{cid}** | {quant} | {cite_txt} | {verdict} | {note} |")
    lines.append("")
    lines.append("## Tally")
    lines.append("")
    for v in sorted(counts):
        lines.append(f"- {v}: {counts[v]}")
    lines.append(f"- TOTAL: {sum(counts.values())}")
    lines.append("")
    with open(OUT, "w", encoding="utf-8", newline="\n") as f:
        f.write("\n".join(lines))
    sys.stderr.write(f"wrote {OUT} — {sum(counts.values())} criteria\n")
    return 0


if __name__ == "__main__":
    sys.exit(main())
