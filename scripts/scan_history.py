#!/usr/bin/env python3
"""Scan the Git history of a branch for local absolute paths, personal e-mail
addresses and private course or matriculation identifiers.

This is the in-repository substitute for the external pre-push history scan
that the completion package names (prompt/PROGRESS.md records the decision).
Run it before every push, after scripts/check_clean.py and the Rust text guard
(`cargo test -p nush2_plant --test no_internal_pointers`).

What it reads, for every commit reachable from REV (default HEAD):
- the commit message and the author and committer e-mail addresses;
- every blob of every tree, once per blob id. Text blobs are scanned as text.
  Office files (zip containers) are opened and their XML parts scanned. PDF and
  other binary blobs get the local-path rules only, on their raw bytes; their
  number is printed so that the gap is visible.

The path, profile, matriculation, cohort, schedule, allocation and declaration
rules are imported from scripts/check_clean.py, so the two scans share one
definition and this file holds no literal pattern that could trip either scan.
Two rules are added here. E-mail addresses are hits unless they are the
owner's GitHub noreply address (OWNER_MAIL) or one of the service addresses in
SERVICE_MAIL; another account's users.noreply.github.com address is a hit.
Module-style course codes (two or three capitals, four digits, an optional
capital) are hits unless they are one of the reviewed technical identifiers in
TECHNICAL_IDS, each of which names a reactor design, a price basis, a cited
report or a citation key. Add to that list only after reading the context.

Hits print the commit, the path and the rule, never the matched text, so the
output can be kept in a log without repeating what it found. Use --show to
print the matched text when reviewing locally.

Usage: python scripts/scan_history.py [REV] [--show]
       python scripts/scan_history.py --self-test   each rule fires on a planted sample
Exit 0 clean, 1 on any hit, 2 on a Git error.
"""
import io
import re
import subprocess
import sys
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts"))
import check_clean  # noqa: E402  (shared rule definitions)

# Allowed e-mail addresses. Only the repository owner's GitHub noreply address
# (with or without the numeric account id) and the service addresses that the
# recorded history needs. Any other users.noreply.github.com address names a
# different GitHub account and is a hit.
OWNER_MAIL = re.compile(r"(?:\d+\+)?weewzr@users\.noreply\.github\.com")
SERVICE_MAIL = frozenset({"noreply@github.com", "noreply@anthropic.com"})

TEXT_RULES = ("path", "profile", "drive", "matric", "cohort", "schedule", "allocation", "declaration")
BINARY_RULES = ("path", "profile")
EMAIL = re.compile(r"[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}")
COURSE = re.compile(r"\b[A-Z]{2,3}\d{4}[A-Z]?\b")
# Reviewed matches of COURSE that are public technical identifiers, not course codes.
TECHNICAL_IDS = {
    "AP1000": "reactor design (Westinghouse)",
    "HPR1000": "reactor design (Hualong One)",
    "EUR2014": "price basis of the IEAGHG costs",
    "TE1236": "source key of the reforming equilibrium benchmark",
    "BSL2002": "citation key, Bird, Stewart and Lightfoot",
    "ML2509": "NRC ADAMS folder in a cited URL",
}


def allowed(label, text):
    if label == "email":
        low = text.lower()
        return low in SERVICE_MAIL or OWNER_MAIL.fullmatch(low) is not None
    if label == "course":
        return text in TECHNICAL_IDS
    return False


def git(*args, data=None):
    p = subprocess.run(["git", "-C", str(ROOT), *args], input=data, capture_output=True, check=False)
    if p.returncode != 0:
        sys.stderr.write(p.stderr.decode("utf-8", "replace"))
        sys.exit(2)
    return p.stdout


def rules():
    out = {k: check_clean.decode(v) for k, v in check_clean.ENCODED.items() if k in TEXT_RULES}
    missing = set(TEXT_RULES) - set(out)
    if missing:
        sys.stderr.write(f"check_clean.py lacks rules {sorted(missing)}\n")
        sys.exit(2)
    out["email"] = EMAIL
    out["course"] = COURSE
    return out


def text_hits(text, rx_all, names):
    found = []
    for label in names:
        rx = rx_all[label]
        for m in rx.finditer(text):
            if allowed(label, m.group(0)):
                continue
            found.append((label, m.group(0)))
            break
    return found


def is_binary(data):
    return b"\0" in data[:8000]


def blobs_of(rev):
    """Yield (blob id, first path seen) for every blob reachable from rev."""
    seen = set()
    for line in git("rev-list", "--objects", rev).decode("utf-8", "replace").splitlines():
        parts = line.split(" ", 1)
        if len(parts) == 2 and parts[0] not in seen:
            seen.add(parts[0])
            yield parts[0], parts[1]


def read_blobs(ids):
    """Read many objects through one cat-file process; yield (id, type, bytes)."""
    proc = subprocess.Popen(["git", "-C", str(ROOT), "cat-file", "--batch"],
                            stdin=subprocess.PIPE, stdout=subprocess.PIPE)
    for oid in ids:
        proc.stdin.write((oid + "\n").encode())
        proc.stdin.flush()
        header = proc.stdout.readline().decode().split()
        if len(header) < 3:
            continue
        size = int(header[2])
        body = proc.stdout.read(size)
        proc.stdout.read(1)
        yield oid, header[1], body
    proc.stdin.close()
    proc.wait()


def self_test():
    """Each rule must fire on a planted sample and stay quiet on an allowed one.
    Samples are joined at run time so this file does not match its own rules."""
    rx = rules()
    j = "".join
    planted = {
        "path": j(["C:", "\\", "Users", "\\x"]),
        "profile": j(["App", "Data"]),
        "drive": j(["/", "home", "/someone"]),
        "matric": j(["A", "1234567", "X"]),
        "cohort": j(["group", " 5"]),
        "schedule": j(["dead", "line"]),
        "allocation": j(["speaker", " allocation"]),
        "declaration": j(["AI", " declaration"]),
        "email": j(["someone", "@", "mail.test"]),
        "course": j(["XY", "1234"]),
    }
    # Further samples that must fire: another account's noreply address.
    planted_more = [
        ("email", j(["999+someone", "@", "users.noreply.github.com"])),
        ("email", j(["weewzr.fake", "@", "users.noreply.github.com"])),
    ]
    quiet = {
        "email": j(["123+weewzr", "@", "users.noreply.github.com"]),
        "course": "AP1000",
    }
    bad = 0
    for label, sample in planted.items():
        got = [lab for lab, _ in text_hits(sample, rx, [label])]
        print(f"{'ok  ' if got else 'FAIL'} {label} fires on a planted sample")
        bad += 0 if got else 1
    for label, sample in planted_more:
        got = [lab for lab, _ in text_hits(sample, rx, [label])]
        print(f"{'ok  ' if got else 'FAIL'} {label} fires on another account's noreply address")
        bad += 0 if got else 1
    for label, sample in quiet.items():
        got = [lab for lab, _ in text_hits(sample, rx, [label])]
        print(f"{'FAIL' if got else 'ok  '} {label} stays quiet on an allowed sample")
        bad += 1 if got else 0
    return 1 if bad else 0


def main(argv):
    if "--self-test" in argv:
        return self_test()
    show = "--show" in argv
    args = [a for a in argv if not a.startswith("--")]
    rev = args[0] if args else "HEAD"
    rx = rules()
    text_names = list(TEXT_RULES) + ["email", "course"]
    hits = []

    # Commit messages and identities.
    log = git("log", "--format=%H%x00%ae%x00%ce%x00%B%x01", rev).decode("utf-8", "replace")
    commits = [c for c in log.split("\x01") if c.strip()]
    for c in commits:
        sha, ae, ce, body = c.lstrip("\n").split("\x00", 3)
        for who in (ae, ce):
            if not allowed("email", who.strip()):
                hits.append((sha[:7], "(identity)", "email", who))
        for label, m in text_hits(body, rx, text_names):
            hits.append((sha[:7], "(message)", label, m))

    # Blobs.
    paths = dict(blobs_of(rev))
    ids = list(paths)
    n_text = n_zip = n_raw = 0
    for oid, typ, data in read_blobs(ids):
        if typ != "blob":
            continue
        where = paths.get(oid, "?")
        if data[:2] == b"PK":
            try:
                with zipfile.ZipFile(io.BytesIO(data)) as z:
                    n_zip += 1
                    for member in z.namelist():
                        if member.endswith((".xml", ".rels", ".txt")):
                            t = z.read(member).decode("utf-8", "replace")
                            for label, m in text_hits(t, rx, text_names):
                                hits.append((oid[:7], f"{where}!{member}", label, m))
                    continue
            except zipfile.BadZipFile:
                pass
        if is_binary(data):
            n_raw += 1
            t = data.decode("latin-1")
            for label, m in text_hits(t, rx, BINARY_RULES):
                hits.append((oid[:7], where, label, m))
            continue
        n_text += 1
        t = data.decode("utf-8", "replace")
        for label, m in text_hits(t, rx, text_names):
            hits.append((oid[:7], where, label, m))

    print(f"rev {rev}: {len(commits)} commits, {len(ids)} objects; blobs scanned as text {n_text}, "
          f"zip containers opened {n_zip}, other binaries path rules only {n_raw}; {len(hits)} hits")
    for obj, where, label, m in hits:
        print(f"{label}: {obj}: {where}" + (f": {m}" if show else ""))
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
