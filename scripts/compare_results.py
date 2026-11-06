#!/usr/bin/env python3
"""Numeric diff of two results folders.

Compares every CSV that either folder holds, row by row and cell by cell. Text
cells must be equal. Numeric cells must agree within the largest of: one unit
in the last digit either file prints, so a value on a rounding edge may flip
by one printed unit between platforms; an absolute 1e-6 (--abs), the unit of
the six decimals the ledgers print, so round-off residuals of order 1e-9 that
the solvers leave in closure and error columns compare as equal; and a
relative 1e-9. Columns named in --ignore (default: commit) are skipped.

--names restricts the comparison to the named files. --subset compares by key
instead of by position: every row of the reference folder must be present in
the regenerated folder with the same cells, and rows that only the regenerated
folder holds are counted and not failed. It is the mode that compares a
regenerated legacy export with the snapshot under engine/vv/data/legacy_results,
since later work adds rows to the tables. --allow-cell FILE:KEY:COLUMN names one
cell that may differ in subset mode, such as a count of registry entries that
grows when an input is added.

Usage: python scripts/compare_results.py results target/results
Exit 0 when the folders agree, 1 on any difference or missing file.
"""
import argparse
import csv
import io
import math
import re
import sys
from pathlib import Path

EXPECTED = (
    "Economic_Ledger.csv",
    "Emissions_Ledger.csv",
    "Final_Design_Basis.csv",
    "Integrated_Energy_Balance.csv",
    "Integrated_Mass_Balance.csv",
    "Sensitivity_Results.csv",
    "Subsidy_Scenarios.csv",
    "Validation_Results.csv",
)
NUM = re.compile(r"^[+-]?(?:\d+\.?\d*|\.\d+)(?:[eE][+-]?\d+)?$")
REL = 1e-9
ABS = 1e-6


def read(path):
    text = path.read_text(encoding="utf-8")
    return list(csv.reader(io.StringIO(text)))


def last_unit(s):
    """Size of one unit in the last printed digit of a numeric string."""
    s = s.strip().lstrip("+-")
    exp = 0
    if "e" in s.lower():
        s, e = re.split("[eE]", s)
        exp = int(e)
    decimals = len(s.split(".", 1)[1]) if "." in s else 0
    return 10.0 ** (exp - decimals)


def same(a, b, abs_tol=ABS):
    """(equal, numeric) for one pair of cells."""
    if a == b:
        return True, NUM.match(a) is not None
    if NUM.match(a) and NUM.match(b):
        x, y = float(a), float(b)
        if math.isnan(x) or math.isnan(y):
            return False, True
        tol = max(max(last_unit(a), last_unit(b)) * (1 + 1e-9), abs_tol, REL * max(abs(x), abs(y)))
        return abs(x - y) <= tol, True
    return False, False


def compare(name, a_rows, b_rows, ignore, limit, abs_tol=ABS):
    out = []
    if not a_rows or not b_rows:
        return [f"{name}: empty file"], 0, 0
    if a_rows[0] != b_rows[0]:
        return [f"{name}: header differs: {a_rows[0]} against {b_rows[0]}"], 0, 0
    header = a_rows[0]
    skip = {i for i, h in enumerate(header) if h in ignore}
    if len(a_rows) != len(b_rows):
        out.append(f"{name}: {len(a_rows) - 1} rows against {len(b_rows) - 1}")
    cells = flips = 0
    for r, (ra, rb) in enumerate(zip(a_rows[1:], b_rows[1:]), start=2):
        if len(ra) != len(rb):
            out.append(f"{name} line {r}: {len(ra)} cells against {len(rb)}")
            continue
        for i, (x, y) in enumerate(zip(ra, rb)):
            if i in skip:
                continue
            cells += 1
            ok, numeric = same(x, y, abs_tol)
            if ok and numeric and x != y:
                flips += 1
            if not ok:
                key = ",".join(ra[:2])
                out.append(f"{name} line {r} [{key}] {header[i]}: {x!r} against {y!r}")
                if len(out) >= limit:
                    out.append(f"{name}: further differences not listed")
                    return out, cells, flips
    return out, cells, flips


# Leading columns that identify a row of each table, for --subset.
KEY_COLUMNS = {
    "Economic_Ledger.csv": 7,
    "Emissions_Ledger.csv": 4,
    "Final_Design_Basis.csv": 1,
    "Integrated_Energy_Balance.csv": 2,
    "Integrated_Mass_Balance.csv": 2,
    "Sensitivity_Results.csv": 7,
    "Subsidy_Scenarios.csv": 4,
    "Validation_Results.csv": 1,
}


def keyed(rows, width):
    """Map (key cells, occurrence) to row, so repeated keys stay distinct."""
    out = {}
    seen = {}
    for row in rows[1:]:
        key = tuple(row[:width])
        n = seen.get(key, 0)
        seen[key] = n + 1
        out[(key, n)] = row
    return out


def compare_subset(name, a_rows, b_rows, ignore, limit, abs_tol, allowed):
    """Every reference row must be in the regenerated table with equal cells."""
    if not a_rows or not b_rows:
        return [f"{name}: empty file"], 0, 0, 0
    ha, hb = a_rows[0], b_rows[0]
    missing_cols = [h for h in ha if h not in hb]
    if missing_cols:
        return [f"{name}: columns missing from the regenerated table: {missing_cols}"], 0, 0, 0
    width = KEY_COLUMNS.get(name, 1)
    ka, kb = keyed(a_rows, width), keyed(b_rows, width)
    out = []
    cells = flips = 0
    col_b = {h: i for i, h in enumerate(hb)}
    for (key, n), ra in ka.items():
        rb = kb.get((key, n))
        label = ",".join(key[:2])
        if rb is None:
            out.append(f"{name} [{label}] occurrence {n}: row missing from the regenerated table")
        else:
            for i, h in enumerate(ha):
                if h in ignore:
                    continue
                x, y = ra[i], rb[col_b[h]]
                cells += 1
                ok, numeric = same(x, y, abs_tol)
                if ok and numeric and x != y:
                    flips += 1
                if not ok and (name, key[0], h) in allowed:
                    continue
                if not ok:
                    out.append(f"{name} [{label}] {h}: {x!r} against {y!r}")
        if len(out) >= limit:
            out.append(f"{name}: further differences not listed")
            return out, cells, flips, 0
    extra = len(set(kb) - set(ka))
    return out, cells, flips, extra


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("a", type=Path, help="reference folder, normally results")
    ap.add_argument("b", type=Path, help="regenerated folder")
    ap.add_argument("--ignore", nargs="*", default=["commit"], help="column names to skip")
    ap.add_argument("--limit", type=int, default=40, help="differences listed per file")
    ap.add_argument("--abs", type=float, default=ABS, dest="abs_tol", help="absolute tolerance, default 1e-6")
    ap.add_argument("--names", nargs="*", default=None, help="compare only these file names")
    ap.add_argument("--subset", action="store_true", help="keyed compare; rows only in b are counted, not failed")
    ap.add_argument(
        "--allow-cell",
        action="append",
        default=[],
        metavar="FILE:KEY:COLUMN",
        help="a cell that may differ in --subset mode, named by file, first key cell and column",
    )
    args = ap.parse_args(argv)
    names = sorted({p.name for p in args.a.glob("*.csv")} | {p.name for p in args.b.glob("*.csv")} | set(EXPECTED))
    if args.names is not None:
        names = sorted(args.names)
    allowed = set()
    for spec in args.allow_cell:
        parts = spec.split(":")
        if len(parts) != 3:
            ap.error(f"--allow-cell wants FILE:KEY:COLUMN, got {spec!r}")
        allowed.add(tuple(parts))
    fails = []
    for name in names:
        pa, pb = args.a / name, args.b / name
        if not pa.is_file() or not pb.is_file():
            missing = [str(p) for p in (pa, pb) if not p.is_file()]
            fails.append(f"{name}: missing in {', '.join(missing)}")
            print(f"{name}: MISSING")
            continue
        if args.subset:
            diffs, cells, flips, extra = compare_subset(
                name, read(pa), read(pb), set(args.ignore), args.limit, args.abs_tol, allowed
            )
            state = "differs" if diffs else "agrees"
            print(
                f"{name}: {state}, {cells} cells compared, {flips} unequal as text but within "
                f"tolerance, {extra} rows only in the regenerated table"
            )
            for d in diffs:
                print(f"  {d}")
            fails.extend(diffs)
            continue
        diffs, cells, flips = compare(name, read(pa), read(pb), set(args.ignore), args.limit, args.abs_tol)
        state = "differs" if diffs else "agrees"
        print(f"{name}: {state}, {cells} cells compared, {flips} unequal as text but within tolerance")
        for d in diffs:
            print(f"  {d}")
        fails.extend(diffs)
    if fails:
        print(f"FAIL: {len(fails)} difference(s)")
        return 1
    print("PASS")
    return 0


if __name__ == "__main__":
    sys.exit(main())
