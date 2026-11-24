#!/usr/bin/env python3
"""Verification of docs/derivation/02_cost_and_subsidy.md (cost model and government co-funding).

Run with the project verification interpreter, for example

    python -I math/verify/sympy_subsidy.py

The script has three parts.

Part A uses SymPy to check every algebraic identity that the derivation relies on
(annuity identities, the DCF bucket factor and its gap to the ledger factor, the
waterfall identity, the closed-form break-even shares, the monotonicity and convexity
signs, the three interest bases).

Part B rebuilds the zero-subsidy L5 ledger and the discounted cash flow from the
committed results/Economic_Ledger.csv for FOAK, BOAK and both tenth-of-a-kind routes
at 25 y and 60 y, and compares them with the stored LAC_DCF, the stored gap and the
stored gap terms.

Part C applies the grant model through four independent routes (closed form, ledger
lines, dated cash-flow list, year-by-year funded-balance recursion) and checks that
they agree, that the limits behave, and that the break-even shares solve LAC = target.

Nothing here calls the Rust engine. Annual abatement A is read from the CSV only as a
placeholder symbol value: the carbon rebuild replaces it later and nothing below
depends on how it is obtained.

Exit status 0 means every check passed.
"""
from __future__ import annotations

import csv
import pathlib
import re
import sys

import mpmath as mp
import sympy as sp

mp.mp.dps = 40
ROOT = pathlib.Path(__file__).resolve().parents[2]
CSV_PATH = ROOT / "results" / "Economic_Ledger.csv"
TOML_PATH = ROOT / "engine" / "data" / "parameters.toml"

RESULTS: list[tuple[str, bool, str, str]] = []


def check(cid: str, text: str, ok: bool, detail: str = "") -> None:
    RESULTS.append((cid, bool(ok), text, detail))
    tag = "PASS" if ok else "FAIL"
    print(f"[{tag}] {cid}: {text}" + (f"  ({detail})" if detail else ""))


def show(title: str) -> None:
    print("\n" + "=" * 78 + f"\n{title}\n" + "=" * 78)


# ---------------------------------------------------------------------------
# Parameters, read from the registry so that nothing is typed twice
# ---------------------------------------------------------------------------
def read_params(path: pathlib.Path) -> dict[str, mp.mpf]:
    out: dict[str, mp.mpf] = {}
    text = path.read_text(encoding="utf-8")
    for block in text.split("[[param]]")[1:]:
        m = re.search(r'^id\s*=\s*"([^"]+)"', block, re.M)
        v = re.search(r"^value\s*=\s*([-+0-9.eE]+)\s*$", block, re.M)
        if m and v:
            out[m.group(1)] = mp.mpf(v.group(1))
    return out


P = read_params(TOML_PATH)
I = P["DISCOUNT"]  # real discount rate i
NC = P["CONSTRUCTION_YEARS"]  # t_c, construction time in years
TU = int(P["ITEM_LIFE_Y"])  # t_u, replacement interval of the 25 y items
FOM = P["CAPTURE_FOM"]  # fixed O&M share of capital (also used for the recycle train)
TGT = P["TARGET_COST"]  # T, S$/t
W = int(mp.floor(NC))  # whole construction years w
Q = NC - W  # part year q
V = 1 / (1 + I)
STAGES = ["FOAK", "BOAK", "10OAK_P46", "10OAK_P47"]
LIVES = [25, 60]
SHARES = [mp.mpf(x) for x in ("0", "0.25", "0.5", "0.75")]


# ---------------------------------------------------------------------------
# Numerical building blocks (mpmath)
# ---------------------------------------------------------------------------
def crf(t):  # eq L.1
    return I / (1 - (1 + I) ** (-t))


def sff(t):  # eq L.3
    return I / ((1 + I) ** t - 1)


def alpha(L):  # annuity factor, sum of V^y for y = 1..L
    return (1 - V**L) / I


def f_idc(t):  # eq L.4 evaluated at construction length t
    return ((1 + I) ** t - 1) / (I * t)


def h_close(u):  # private reactor PV factor after a grant covering a fraction u of the reactor
    return ((1 + I) ** (NC * (1 - u)) - 1) / (I * NC)


def f_dcf():  # S.0.1, the year-end bucket factor of dcf.rs uniform_spend
    return (Q * (1 + I) ** W + ((1 + I) ** W - 1) / I) / NC


def rho_R(L, tu=TU):  # S.0.3 replacement factor of dcf.rs with_replacement
    n = -(-L // tu)
    s = sum(V ** (m * tu) for m in range(n))
    unused = n * tu - L
    if unused > 0:
        s -= mp.mpf(unused) / tu * V**L
    return s


FI = f_idc(NC)
FD = f_dcf()


def buckets(C):
    """uniform_spend(C, t_c): list of (year, amount). Part year first."""
    out = []
    if Q > 0:
        out.append((-W, C * Q / NC))
    for k in range(W):
        out.append((k - W + 1, C / NC))
    return out


# ---------------------------------------------------------------------------
# CSV access
# ---------------------------------------------------------------------------
LINE_IDS = [
    "GAS", "REACTOR_CAP", "CAPTURE_CAP", "CAPTURE_FOM", "TS", "REACTOR_OM", "IHX",
    "LICENCE", "DECOM", "IDC", "POWER_ISLAND", "RECYCLE_TRAIN", "LOST_STEAM", "BASE_POWER",
]
ROWS = list(csv.DictReader(CSV_PATH.open(newline="", encoding="utf-8")))


class Point:
    def __init__(self, stage: str, life: int):
        sel = [
            r for r in ROWS
            if r["run_id"] == "B_REC_F" and r["stage"] == stage and r["life_y"] == str(life)
            and r["n"] == "1.0000" and r["reactor_charged_mwth"] == "210.00"
        ]
        self.stage, self.L = stage, life
        self.lines = {r["line_id"]: mp.mpf(r["value_sgdm_y"]) for r in sel if r["level"] != "DCF"}
        self.dcf = {r["line_id"]: mp.mpf(r["dcf_value"]) for r in sel if r["level"] == "DCF" and r["dcf_value"]}
        self.a = mp.mpf(sel[0]["abatement_t_y"])  # placeholder value of the symbol A, t/y
        self.l5_csv = mp.mpf([r for r in sel if r["level"] == "L5"][-1]["cum_s_per_t"])
        # capital items rebuilt from the annualised lines, with the factor the engine used
        self.C = self.lines["REACTOR_CAP"] / crf(life)
        self.Kcap = self.lines["CAPTURE_CAP"] / crf(TU)
        self.Kihx = self.lines["IHX"] / crf(TU)
        self.Klic = self.lines["LICENCE"] / crf(life)
        self.Kpi = self.lines["POWER_ISLAND"] / crf(life)
        self.Krec = self.lines["RECYCLE_TRAIN"] / (crf(TU) + FOM)
        self.D = self.lines["DECOM"] / sff(life)
        self.Y = sum(self.lines[k] for k in ("GAS", "CAPTURE_FOM", "TS", "REACTOR_OM", "LOST_STEAM", "BASE_POWER"))
        self.l5_0 = sum(self.lines[k] for k in LINE_IDS) * mp.mpf(10) ** 6 / self.a  # S$/t
        self.kappa = crf(life) * mp.mpf(10) ** 6 / self.a  # S$/t per S$M of one-off year-0 value

    @property
    def X_B(self):
        return self.Kihx + self.Klic + self.Kpi

    def eligible(self, d):
        X = self.X_B if d == "B" else mp.mpf(0)
        return self.C + X, X


POINTS = [Point(st, L) for st in STAGES for L in LIVES]


def dcf_zero_flows(pt: Point):
    """Differential flows NUSH2 minus SMR, S$M, as (year, amount) with a label. SMR island cancels."""
    L = pt.L
    fl = []
    fl += [("REACTOR_CAP", t, b) for t, b in buckets(pt.C)]
    for lab, K in (("CAPTURE_CAP", pt.Kcap), ("IHX", pt.Kihx), ("RECYCLE_TRAIN", pt.Krec)):
        n = -(-L // TU)
        for m in range(n):
            fl.append((lab, m * TU, K))
        unused = n * TU - L
        if unused > 0:
            fl.append((lab, L, -K * mp.mpf(unused) / TU))
    fl.append(("LICENCE", 0, pt.Klic))
    fl.append(("POWER_ISLAND", 0, pt.Kpi))
    fl.append(("DECOM", L, pt.D))
    annual = pt.Y + FOM * pt.Krec  # capture FOM is inside Y; recycle FOM added here
    for y in range(1, L + 1):
        fl.append(("ANNUAL", y, annual))
    return fl


def pv_of(flows):
    return sum(amount * (1 + I) ** (-year) for _, year, amount in flows)


# ===========================================================================
# PART A, SymPy identities
# ===========================================================================
def symbolic_part() -> None:
    show("PART A. SymPy identities")
    i = sp.symbols("i", positive=True)
    Ls, tu, tcs, uu, ss = sp.symbols("L t_u t_c u s", positive=True)
    qq = sp.symbols("q", nonnegative=True)
    ww = sp.symbols("w", integer=True, nonnegative=True)
    v = 1 / (1 + i)

    def zero(expr, name, subs_sets=None):
        """Symbolic zero test with a numerical fallback that is reported as such."""
        e = sp.simplify(expr)
        if e == 0:
            check(name, "symbolic zero", True)
            return
        pts = subs_sets or []
        ok = bool(pts)
        for sub in pts:
            val = sp.N(expr.subs(sub), 30)
            ok = ok and abs(val) < sp.Float("1e-25")
        check(name, "numerical spot check only (SymPy did not reduce it to 0)", ok, f"{len(pts)} points")

    # A1 annuity identities (eq L.1, L.3)
    crf_s = i / (1 - (1 + i) ** (-Ls))
    sff_s = i / ((1 + i) ** Ls - 1)
    alpha_s = (1 - v**Ls) / i
    zero(crf_s * alpha_s - 1, "A1 CRF * alpha = 1")
    zero(v**Ls * crf_s - sff_s, "A2 v^L CRF = SFF  (decommissioning identity, S.0.4)")

    # A3 geometric bucket sum by induction
    closed = lambda n: ((1 + i) ** n - 1) / i
    zero(closed(ww + 1) - closed(ww) - (1 + i) ** ww, "A3a geometric sum, induction step")
    zero(closed(sp.Integer(0)), "A3b geometric sum, base case")

    # A4 f_D - f_IDC closed form (S.0.2)
    N = ww + qq
    fD = (qq * (1 + i) ** ww + ((1 + i) ** ww - 1) / i) / N
    fI = ((1 + i) ** N - 1) / (i * N)
    claim = (1 + i) ** ww * (1 + qq * i - (1 + i) ** qq) / (N * i)
    spots = [{i: sp.Rational(8, 100), ww: 5, qq: sp.Rational(1, 2)},
             {i: sp.Rational(5, 100), ww: 3, qq: sp.Rational(1, 3)},
             {i: sp.Rational(10, 100), ww: 7, qq: sp.Rational(9, 10)}]
    zero(sp.expand_power_exp(sp.expand(fD - fI - claim)), "A4 f_D - f_IDC = (1+i)^w [1+q i-(1+i)^q]/(N i)", spots)

    # A5 sign of the gap, by concavity
    g = 1 + qq * i - (1 + i) ** qq
    g2 = sp.diff(g, qq, 2)
    zero(g2 + sp.log(1 + i) ** 2 * (1 + i) ** qq, "A5a g''(q) = -ln(1+i)^2 (1+i)^q")
    check("A5b", "g(0) = 0 and g(1) = 0 and g'' < 0 (SymPy sign inference), so g >= 0 on [0,1] and f_D >= f_IDC",
          sp.simplify(g.subs(qq, 0)) == 0 and sp.simplify(g.subs(qq, 1)) == 0
          and (-sp.log(1 + i) ** 2 * (1 + i) ** qq).is_negative is True)

    # A6 series of the gap to leading order (label: series)
    ser = sp.series((1 + i) ** qq, i, 0, 4).removeO()
    g_ser = sp.expand(1 + qq * i - ser)
    lead = sp.simplify(g_ser.coeff(i, 2) - qq * (1 - qq) / 2)
    check("A6", "g(q) = q(1-q) i^2/2 + O(i^3)  (series)", lead == 0)

    # A7 waterfall identity (S.3): (1-u) f_IDC(t_c (1-u)) = h(u)
    fI_t = lambda t: ((1 + i) ** t - 1) / (i * t)
    h = ((1 + i) ** (tcs * (1 - uu)) - 1) / (i * tcs)
    zero((1 - uu) * fI_t(tcs * (1 - uu)) - h, "A7a (1-u) f_IDC(t_c(1-u)) = h(u)")
    zero(h.subs(uu, 0) - fI_t(tcs), "A7b h(0) = f_IDC(t_c)")
    check("A7c", "h(1) = 0", sp.simplify(h.subs(uu, 1)) == 0)
    hp = sp.diff(h, uu)
    zero(hp + sp.log(1 + i) * (1 + i) ** (tcs * (1 - uu)) / i, "A7d h'(u) = -ln(1+i)(1+i)^(t_c(1-u))/i < 0")
    hpp = sp.diff(h, uu, 2)
    zero(hpp - tcs * sp.log(1 + i) ** 2 * (1 + i) ** (tcs * (1 - uu)) / i, "A7e h''(u) > 0 (convex)")

    # A8 progressive affinity (S.7)
    kap, Cc, X, f, L0, T = sp.symbols("kappa C X f L0 T", positive=True)
    lac_prog = L0 - ss * kap * (Cc * f + X)
    check("A8a", "progressive LAC is affine: d2/ds2 = 0", sp.diff(lac_prog, ss, 2) == 0)
    s_star = (L0 - T) / (kap * (Cc * f + X))
    zero(lac_prog.subs(ss, s_star) - T, "A8b progressive s* solves LAC(s*) = T")

    # A9 waterfall ledger break-even (S.7)
    E = Cc + X
    G = ss * E
    u = G / Cc
    fI_c = sp.Symbol("fI", positive=True)  # f_IDC(t_c) as a symbol
    hh = lambda uu_: (sp.exp(tcs * (1 - uu_) * sp.log(1 + i)) - 1) / (i * tcs)
    Hs = fI_c - (L0 - T) / (kap * Cc)
    u_star = 1 - sp.log(1 + i * tcs * Hs) / (tcs * sp.log(1 + i))
    resid = hh(u_star) - Hs
    zero(sp.simplify(sp.expand_log(resid, force=True)), "A9a h(u*) = H*  (u* is the exact root of the reactor regime)")
    s_star_w = u_star * Cc / E
    check("A9b", "waterfall s* = u* C/E, with G = s E", sp.simplify(s_star_w * E / Cc - u_star) == 0)

    # A10 the three interest bases close (S.5)
    c = sp.symbols("C", positive=True)
    B1 = c * (fI_c - 1)
    # progressive: B2 = (1-s) B1; B3 = s B1
    check("A10a", "progressive: B1 - B2 = B3 with B2 = (1-s)B1 and B3 = s B1", sp.simplify(B1 - (1 - ss) * B1 - ss * B1) == 0)
    # at close: B2 = C h(u) - C (1-u); B3 = C (f - h(u) - u)
    B2c = c * hh(uu) - c * (1 - uu)
    B3c = c * (fI_c - hh(uu) - uu)
    check("A10b", "at close: B1 - B2 = B3", sp.simplify(sp.expand(B1 - B2c - B3c)) == 0)

    # A11 dated-flow bound (not the default): sign change and the s = 1 failure
    Gd = ss * c
    idc_d = c * (fI_c - 1) - Gd * ((1 + i) ** tcs - 1)
    s0 = sp.solve(sp.Eq(idc_d, 0), ss)[0]
    check("A11a", "dated-flow IDC changes sign at s0 = (f-1)/((1+i)^t_c - 1)",
          sp.simplify(s0 - (fI_c - 1) / ((1 + i) ** tcs - 1)) == 0)
    pv_priv_d = c * fI_c - Gd * (1 + i) ** tcs
    check("A11b", "dated-flow private PV at s=1 equals C (f - (1+i)^t_c)",
          sp.simplify(pv_priv_d.subs(ss, 1) - c * (fI_c - (1 + i) ** tcs)) == 0)

    # A12 the bucket waterfall splits each bucket exactly (S.3a)
    S_prev, b, Gs = sp.symbols("S_prev b G", nonnegative=True)
    g_k = sp.Min(b, sp.Max(0, Gs - S_prev))
    p_k = sp.Min(b, sp.Max(0, S_prev + b - Gs))
    ok = True
    for sp_v, bv, gv in [(0, 3, 1), (0, 3, 3), (0, 3, 7), (2, 3, 1), (2, 3, 4), (2, 3, 9), (5, 2, 5), (5, 2, 6)]:
        ok = ok and (g_k + p_k - b).subs({S_prev: sp_v, b: bv, Gs: gv}) == 0
    check("A12", "g_k + p_k = b_k for the bucket waterfall (8 cases covering all regimes)", ok)


# ===========================================================================
# PART B, zero-subsidy reproduction of the committed CSV
# ===========================================================================
def part_b() -> None:
    show("PART B. Zero-subsidy reproduction of results/Economic_Ledger.csv")
    print(f"i = {mp.nstr(I,6)}, t_c = {mp.nstr(NC,6)}, w = {W}, q = {mp.nstr(Q,6)}, t_u = {TU}, T = {mp.nstr(TGT,6)}")
    print(f"CRF(25) = {mp.nstr(crf(25),10)}   CRF(60) = {mp.nstr(crf(60),10)}   SFF(25) = {mp.nstr(sff(25),8)}")
    print(f"f_IDC = {mp.nstr(FI,12)}   f_D = {mp.nstr(FD,12)}   f_D - f_IDC = {mp.nstr(FD-FI,10)}")
    claim = (1 + I) ** W * (1 + Q * I - (1 + I) ** Q) / (NC * I)
    check("B0a", "f_D - f_IDC closed form numerically", abs(FD - FI - claim) < mp.mpf("1e-35"), mp.nstr(FD - FI, 10))
    lead = (1 + I) ** W * Q * (1 - Q) * I / (2 * NC)
    print(f"leading-order series of the gap: {mp.nstr(lead,8)} (exact {mp.nstr(FD-FI,8)}, ratio {mp.nstr(lead/(FD-FI),6)})")
    print(f"rho_R(25) = {mp.nstr(rho_R(25),10)}  rho_R(60) = {mp.nstr(rho_R(60),10)}")

    tol_csv = mp.mpf("2e-5")  # the CSV keeps 6 decimals of S$M/y per line
    lf10 = mp.mpf("0.9") ** (mp.log(10) / mp.log(2))
    base = {pt.L: pt for pt in POINTS if pt.stage == "BOAK"}
    print("\nstage      L   C_R(S$M)   L5 recomputed   L5 CSV       LAC_DCF model  LAC_DCF CSV   gap model  gap CSV")
    for pt in POINTS:
        # ladder, S$/t
        check(f"B1 {pt.stage} {pt.L}y", "L5 = sum of lines x 1e6 / A equals the CSV cumulative value",
              abs(pt.l5_0 - pt.l5_csv) < tol_csv, mp.nstr(pt.l5_0 - pt.l5_csv, 3))
        # IDC line = C (f_IDC - 1) CRF
        idc_model = pt.C * (FI - 1) * crf(pt.L)
        check(f"B2 {pt.stage} {pt.L}y", "IDC line = C_R (f_IDC - 1) CRF(L)", abs(idc_model - pt.lines["IDC"]) < mp.mpf("2e-6"),
              mp.nstr(idc_model - pt.lines["IDC"], 3))
        # DCF rebuilt from flows
        pv = pv_of(dcf_zero_flows(pt))
        lac = pv * mp.mpf(10) ** 6 / (pt.a * alpha(pt.L))
        check(f"B3 {pt.stage} {pt.L}y", "LAC_DCF rebuilt from dated flows equals the CSV value",
              abs(lac - pt.dcf["LAC_DCF"]) < tol_csv, mp.nstr(lac - pt.dcf["LAC_DCF"], 3))
        # closed form (S.0.5): Y + CRF(L)(C fD + (Kcap+Kihx+Krec) rho + Klic + Kpi + D v^L) + FOM Krec
        cf = (pt.Y + crf(pt.L) * (pt.C * FD + (pt.Kcap + pt.Kihx + pt.Krec) * rho_R(pt.L) + pt.Klic + pt.Kpi + pt.D * V**pt.L)
              + FOM * pt.Krec) * mp.mpf(10) ** 6 / pt.a
        check(f"B4 {pt.stage} {pt.L}y", "LAC_DCF closed form (S.0.5) equals the flow-by-flow PV",
              abs(cf - lac) < mp.mpf("1e-30"), mp.nstr(cf - lac, 3))
        # gap and gap terms
        k = mp.mpf(10) ** 6 / pt.a
        gap_react = pt.C * crf(pt.L) * (FD - FI) * k
        gap_cap = pt.Kcap * (crf(pt.L) * rho_R(pt.L) - crf(TU)) * k
        gap_ihx = pt.Kihx * (crf(pt.L) * rho_R(pt.L) - crf(TU)) * k
        gap_rec = pt.Krec * (crf(pt.L) * rho_R(pt.L) - crf(TU)) * k
        gap_model = gap_react + gap_cap + gap_ihx + gap_rec
        check(f"B5 {pt.stage} {pt.L}y", "gap term REACTOR_CAP_WITH_IDC = C_R CRF(L)(f_D - f_IDC) 1e6/A",
              abs(gap_react - pt.dcf["GAP_REACTOR_CAP_WITH_IDC"]) < tol_csv, mp.nstr(gap_react - pt.dcf["GAP_REACTOR_CAP_WITH_IDC"], 3))
        check(f"B6 {pt.stage} {pt.L}y", "replacement gap terms CAPTURE_CAP, IHX, RECYCLE_TRAIN",
              abs(gap_cap - pt.dcf["GAP_CAPTURE_CAP"]) < tol_csv and abs(gap_ihx - pt.dcf["GAP_IHX"]) < tol_csv
              and abs(gap_rec - pt.dcf["GAP_RECYCLE_TRAIN"]) < tol_csv,
              f"{mp.nstr(gap_cap-pt.dcf['GAP_CAPTURE_CAP'],3)}, {mp.nstr(gap_ihx-pt.dcf['GAP_IHX'],3)}, {mp.nstr(gap_rec-pt.dcf['GAP_RECYCLE_TRAIN'],3)}")
        other_zero = all(abs(pt.dcf[k2]) < tol_csv for k2 in ("GAP_GAS", "GAP_CAPTURE_FOM", "GAP_TS", "GAP_REACTOR_OM",
                                                          "GAP_LICENCE", "GAP_DECOM", "GAP_POWER_ISLAND", "GAP_STEAM_AND_POWER"))
        check(f"B7 {pt.stage} {pt.L}y", "all other gap terms are zero (annual lines, licence, power island, decommissioning)", other_zero)
        check(f"B8 {pt.stage} {pt.L}y", "total gap model equals stored LAC_L5_GAP", abs(gap_model - pt.dcf["LAC_L5_GAP"]) < tol_csv,
              mp.nstr(gap_model - pt.dcf["LAC_L5_GAP"], 3))
        print(f"{pt.stage:10s} {pt.L:3d}  {mp.nstr(pt.C,8):>9s}  {mp.nstr(pt.l5_0,10):>13s}  {mp.nstr(pt.l5_csv,10):>11s}  "
              f"{mp.nstr(lac,10):>13s}  {mp.nstr(pt.dcf['LAC_DCF'],10):>11s}  {mp.nstr(gap_model,7):>8s}  {mp.nstr(pt.dcf['LAC_L5_GAP'],7):>8s}")
    # stage multipliers (eq L.7 to L.9)
    for L in LIVES:
        b = next(p for p in POINTS if p.stage == "BOAK" and p.L == L)
        f_ = next(p for p in POINTS if p.stage == "FOAK" and p.L == L)
        a_ = next(p for p in POINTS if p.stage == "10OAK_P46" and p.L == L)
        c_ = next(p for p in POINTS if p.stage == "10OAK_P47" and p.L == L)
        ok = (abs(f_.C / b.C - P["FOAK_CENTRAL"]) < mp.mpf("1e-7") and abs(a_.C / b.C - lf10) < mp.mpf("1e-7")
              and abs(c_.C / b.C - P["FOAK_CENTRAL"] * lf10) < mp.mpf("1e-7"))
        check(f"B9 {L}y", "reactor capital ratios FOAK/BOAK = 1.6, P46/BOAK = lf10, P47/BOAK = 1.6 lf10", ok)
    c_boak = P["OCC_CENTRAL"] * P["REACTOR_COST_REF_SIZE"] * 1000 / mp.mpf(10) ** 6 * P["F_2019USD_TO_2025SGD"] * (
        P["REACTOR_RATED_MWTH"] / P["REACTOR_COST_REF_SIZE"])
    b25 = next(p for p in POINTS if p.stage == "BOAK" and p.L == 25)
    check("B10", "BOAK reactor capital from the registry (eq L.5, L.6, n=1) equals the CSV-derived value",
          abs(c_boak - b25.C) < mp.mpf("1e-5"), f"{mp.nstr(c_boak,10)} vs {mp.nstr(b25.C,10)}")
    # the 25 y gap is entirely reactor IDC
    check("B11", "at 25 y the whole gap is the reactor IDC term",
          all(abs(p.dcf["LAC_L5_GAP"] - p.dcf["GAP_REACTOR_CAP_WITH_IDC"]) < tol_csv for p in POINTS if p.L == 25))
    # capex of the other items, for the eligible-set table
    print("\nBOAK 25 y capital items rebuilt from the ledger lines (S$M):")
    for lab, val in (("reactor C_R", b25.C), ("capture K_cap", b25.Kcap), ("IHX K_ihx", b25.Kihx), ("licence K_lic", b25.Klic),
                     ("power island K_pi", b25.Kpi), ("recycle K_rec", b25.Krec), ("decommissioning D (liability)", b25.D)):
        print(f"  {lab:32s} {mp.nstr(val,8)}")
    ktot = b25.C + b25.Kcap + b25.Kihx + b25.Klic + b25.Kpi + b25.Krec
    print(f"  total capital K_tot (overnight, ex IDC, ex decommissioning) = {mp.nstr(ktot,8)}")
    print(f"  E_A = {mp.nstr(b25.C,8)}   E_B = {mp.nstr(b25.C+b25.X_B,8)}   X_B = {mp.nstr(b25.X_B,8)}")


# ===========================================================================
# PART C, the grant model through four routes
# ===========================================================================
def unit_k(pt):  # S$/t per S$M of annual cost
    return mp.mpf(10) ** 6 / pt.a


# ---- Route 1, closed forms ------------------------------------------------
def pd_closed(C, G):
    """P_D(G): PV at year 0 of the private reactor flows of the DCF after a grant G applied
    to the earliest spend. Piecewise closed form in units of C/N. Reactor grant only (G <= C)."""
    if G >= C:
        return mp.mpf(0)
    gam = G * NC / C  # grant in units of C/N
    if gam <= Q:
        rem, e = Q - gam, W
    else:
        j = int(mp.floor(gam - Q))
        r = gam - Q - j
        rem, e = 1 - r, W - 1 - j
    return (C / NC) * (rem * (1 + I) ** e + ((1 + I) ** e - 1) / I)


def lac5_closed(pt, s, d, timing):
    E, X = pt.eligible(d)
    G = s * E
    if timing == "prog":
        red = pt.kappa * (s * (pt.C * FI + X))
    else:
        u = min(G / pt.C, mp.mpf(1))
        red = pt.kappa * (pt.C * (FI - h_close(u)) + max(mp.mpf(0), G - pt.C))
    return pt.l5_0 - red


def lacd_closed(pt, s, d, timing, base):
    E, X = pt.eligible(d)
    G = s * E
    if timing == "prog":
        red = pt.kappa * (s * (pt.C * FD + X))
    else:
        red = pt.kappa * (pt.C * FD - pd_closed(pt.C, min(G, pt.C)) + max(mp.mpf(0), G - pt.C))
    return base - red


# ---- Route 2, ledger lines built from private capital ----------------------
def lac5_lines(pt, s, d, timing):
    E, X = pt.eligible(d)
    G = s * E
    lines = dict(pt.lines)
    cl = crf(pt.L)
    idc0 = pt.C * (FI - 1) * cl  # model value of the zero-subsidy IDC line (the CSV line differs by its rounding)
    if timing == "prog":
        Kp = pt.C * (1 - s)
        lines["REACTOR_CAP"] = Kp * cl
        lines["IDC"] = pt.lines["IDC"] + (Kp * (FI - 1) * cl - idc0)
        if d == "B":
            lines["IHX"] = pt.Kihx * crf(TU) - s * pt.Kihx * cl  # one-off grant, annualised over the project life
            lines["LICENCE"] = (1 - s) * pt.Klic * cl
            lines["POWER_ISLAND"] = (1 - s) * pt.Kpi * cl
    else:
        GR = min(G, pt.C)
        Kp = pt.C - GR
        tp = NC * (1 - GR / pt.C)
        lines["REACTOR_CAP"] = Kp * cl
        idc_new = Kp * (f_idc(tp) - 1) * cl if tp > 0 else mp.mpf(0)
        lines["IDC"] = pt.lines["IDC"] + (idc_new - idc0)
        lines["GRANT_LATER"] = -max(mp.mpf(0), G - pt.C) * cl
    return sum(lines.values()) * mp.mpf(10) ** 6 / pt.a


# ---- Route 3, dated cash-flow list ----------------------------------------
def grant_flows(pt, s, d, timing):
    """Negative GRANT flows, S$M, at the dates where the grant is applied."""
    E, X = pt.eligible(d)
    G = s * E
    out = []
    bk = buckets(pt.C)
    if timing == "prog":
        out += [("GRANT", t, -s * b) for t, b in bk]
        if d == "B":
            out += [("GRANT", 0, -s * K) for K in (pt.Kihx, pt.Klic, pt.Kpi)]
    else:
        left = G
        for t, b in bk:
            g = min(b, left)
            out.append(("GRANT", t, -g))
            left -= g
        if left > 0:
            out.append(("GRANT", 0, -left))
    return out


def lacd_flows(pt, s, d, timing):
    fl = dcf_zero_flows(pt) + grant_flows(pt, s, d, timing)
    return pv_of(fl) * mp.mpf(10) ** 6 / (pt.a * alpha(pt.L))


def lacd_flows_extra(pt, extra):
    """LAC_D of the zero-subsidy flows plus an arbitrary list of extra dated flows (used for the W4 mistake of S.5b)."""
    return pv_of(dcf_zero_flows(pt) + list(extra)) * mp.mpf(10) ** 6 / (pt.a * alpha(pt.L))


# ---- Route 4, year-by-year funded balance ---------------------------------
def funded_balance(pt, s, d, timing, mode=None):
    """Return the list of (year, B) of the private reactor balance, and B(0).
    mode 'dated' is the optional bound in which idle grant cash earns the discount rate."""
    E, X = pt.eligible(d)
    G = s * E
    bk = buckets(pt.C)
    B, prev, traj = mp.mpf(0), None, []
    account = G if (timing == "close") else mp.mpf(0)
    for t, b in bk:
        if prev is not None:
            B *= (1 + I) ** (t - prev)
        if mode == "dated":
            draw = b
        elif timing == "prog":
            draw = b * (1 - s)
        else:
            use = min(b, account)
            account -= use
            draw = b - use
        B += draw
        prev = t
        traj.append((t, B))
    if mode == "dated":
        # the lump G sits at t_c before the first bucket and compounds at the discount rate
        traj = [(t, Bt - G * (1 + I) ** (t + NC)) for t, Bt in traj]
    return traj, traj[-1][1]


def lacd_balance(pt, s, d, timing, base):
    E, X = pt.eligible(d)
    G = s * E
    _, B0 = funded_balance(pt, s, d, timing)
    if timing == "prog":
        credit_other = s * X
    else:
        credit_other = max(mp.mpf(0), G - pt.C)
    return base - pt.kappa * (pt.C * FD - B0 + credit_other)


def part_c() -> None:
    show("PART C. Grant model, four routes")
    tol = mp.mpf("1e-25")
    base_dcf = {}
    for pt in POINTS:
        base_dcf[(pt.stage, pt.L)] = pv_of(dcf_zero_flows(pt)) * mp.mpf(10) ** 6 / (pt.a * alpha(pt.L))

    worst = {"l5": mp.mpf(0), "dcf_cf": mp.mpf(0), "dcf_bal": mp.mpf(0)}
    ncomp = 0
    for pt in POINTS:
        b0 = base_dcf[(pt.stage, pt.L)]
        for d in ("A", "B"):
            for timing in ("prog", "close"):
                for s in SHARES + [mp.mpf("0.1"), mp.mpf("0.9"), mp.mpf(1)]:
                    r1 = lac5_closed(pt, s, d, timing)
                    r2 = lac5_lines(pt, s, d, timing)
                    c1 = lacd_closed(pt, s, d, timing, b0)
                    c3 = lacd_flows(pt, s, d, timing)
                    c4 = lacd_balance(pt, s, d, timing, b0)
                    worst["l5"] = max(worst["l5"], abs(r1 - r2))
                    worst["dcf_cf"] = max(worst["dcf_cf"], abs(c1 - c3))
                    worst["dcf_bal"] = max(worst["dcf_bal"], abs(c3 - c4))
                    ncomp += 1
    check("C1", f"L5: closed form (S.6a) equals the line-by-line ledger over {ncomp} cases", worst["l5"] < tol, mp.nstr(worst["l5"], 3))
    check("C2", "DCF: closed form (S.6b) equals the dated cash-flow list", worst["dcf_cf"] < tol, mp.nstr(worst["dcf_cf"], 3))
    check("C3", "DCF: dated cash-flow list equals the year-by-year funded-balance recursion", worst["dcf_bal"] < tol, mp.nstr(worst["dcf_bal"], 3))

    # s = 0 reproduces the existing cost, and the two timings agree at s = 0
    ok0 = True
    ok0b = True
    for pt in POINTS:
        b0 = base_dcf[(pt.stage, pt.L)]
        for d in ("A", "B"):
            for timing in ("prog", "close"):
                ok0 = ok0 and abs(lac5_closed(pt, mp.mpf(0), d, timing) - pt.l5_0) < tol
                ok0 = ok0 and abs(lacd_closed(pt, mp.mpf(0), d, timing, b0) - b0) < tol
                ok0 = ok0 and abs(lacd_flows(pt, mp.mpf(0), d, timing) - b0) < tol
        ok0b = ok0b and abs(pt.l5_0 - pt.l5_csv) < mp.mpf("2e-5") and abs(b0 - pt.dcf["LAC_DCF"]) < mp.mpf("2e-5")
    check("C4", "s = 0 changes nothing: every route returns the zero-subsidy L5 and DCF value", ok0)
    check("C5", "the zero-subsidy values equal the committed CSV (tolerance of its 6-decimal rounding)", ok0b)

    # s = 1 on definition A removes exactly the reactor lines (read straight from the CSV)
    ok1 = True
    for pt in POINTS:
        removed = (pt.lines["REACTOR_CAP"] + pt.lines["IDC"]) * mp.mpf(10) ** 6 / pt.a
        for timing in ("prog", "close"):
            ok1 = ok1 and abs((pt.l5_0 - lac5_closed(pt, mp.mpf(1), "A", timing)) - removed) < mp.mpf("1e-5")
    check("C6", "s = 1 on definition A removes exactly the REACTOR_CAP and IDC lines of the CSV (both timings)", ok1)

    # gap scaling for the progressive grant (S.6): gap_react(s) = (1-s) gap_react(0)
    okg = True
    for pt in POINTS:
        b0 = base_dcf[(pt.stage, pt.L)]
        gap0 = b0 - pt.l5_0
        for s in SHARES:
            gap_s = lacd_closed(pt, s, "A", "prog", b0) - lac5_closed(pt, s, "A", "prog")
            react0 = pt.C * crf(pt.L) * (FD - FI) * unit_k(pt)
            okg = okg and abs(gap_s - (gap0 - s * react0)) < tol
    check("C7", "progressive grant: gap(s) = gap(0) - s x reactor IDC gap term, i.e. the reactor term scales by (1-s)", okg)

    # at close: the gap between methods across s
    maxgap, arg = mp.mpf(0), None
    for pt in POINTS:
        b0 = base_dcf[(pt.stage, pt.L)]
        for k in range(0, 101):
            s = mp.mpf(k) / 100
            for d in ("A", "B"):
                gp = abs(lacd_closed(pt, s, d, "close", b0) - lac5_closed(pt, s, d, "close"))
                if gp > maxgap:
                    maxgap, arg = gp, (pt.stage, pt.L, d, s)
    print(f"largest |LAC_DCF - L5| at close over s in [0,1], both definitions: {mp.nstr(maxgap,5)} S$/t at {arg[0]} {arg[1]} y def {arg[2]} s={mp.nstr(arg[3],3)}")
    check("C8", "at close the two methods stay within 1 S$/t for every s", maxgap < 1, mp.nstr(maxgap, 4))
    in_band = True
    for pt in POINTS:
        b0 = base_dcf[(pt.stage, pt.L)]
        gap0 = b0 - pt.l5_0
        for k in range(0, 101):
            s = mp.mpf(k) / 100
            for d in ("A", "B"):
                gp = lacd_closed(pt, s, d, "close", b0) - lac5_closed(pt, s, d, "close")
                # the 2e-5 slack is the 6-decimal rounding of the CSV lines that define both zero-subsidy values
                in_band = in_band and (-mp.mpf("2e-5") <= gp <= gap0 + mp.mpf("2e-5"))
    check("C8b", "at close the method gap stays between 0 and its zero-subsidy value for every s on a 101-point grid, both definitions", in_band)

    # monotone and convex (S.9a): second differences, by regime
    okm, okc_prog, okc_dcf, okc_l5a, okc_l5b = True, True, True, True, True
    floor_ = -mp.mpf("1e-25")
    for pt in POINTS:
        b0 = base_dcf[(pt.stage, pt.L)]
        for d in ("A", "B"):
            E, X = pt.eligible(d)
            kink = int(mp.floor(200 * pt.C / E))  # grid index of s = C/E, where u = 1
            for timing in ("prog", "close"):
                vals5 = [lac5_closed(pt, mp.mpf(k) / 200, d, timing) for k in range(0, 201)]
                valsD = [lacd_closed(pt, mp.mpf(k) / 200, d, timing, b0) for k in range(0, 201)]
                for seq in (vals5, valsD):
                    okm = okm and all(seq[k + 1] < seq[k] for k in range(200))
                sd2 = lambda seq, k: seq[k + 2] - 2 * seq[k + 1] + seq[k]
                if timing == "prog":
                    okc_prog = okc_prog and all(abs(sd2(vals5, k)) < tol and abs(sd2(valsD, k)) < tol for k in range(199))
                else:
                    okc_dcf = okc_dcf and all(sd2(valsD, k) >= floor_ for k in range(199))
                    if d == "A":
                        okc_l5a = okc_l5a and all(sd2(vals5, k) >= floor_ for k in range(199))
                    else:
                        okc_l5b = okc_l5b and all(sd2(vals5, k) >= floor_ for k in range(0, max(kink - 1, 0)))
    check("C9", "LAC is strictly decreasing in s for both methods, timings and definitions (201-point scan)", okm)
    check("C10a", "progressive grant: LAC is affine, so its second differences vanish (both methods)", okc_prog)
    check("C10b", "at close: the DCF is convex in s over [0,1] (piecewise linear, slopes shrink as the grant reaches later buckets)", okc_dcf)
    check("C10c", "at close, definition A: the ledger form is convex over [0,1]", okc_l5a)
    check("C10d", "at close, definition B: the ledger form is convex up to s = C_R/E (u = 1)", okc_l5b)
    # the kink at u = 1 of the smooth ledger form (definition B only): slope ratio ln(1+i)/i
    ptk = next(p for p in POINTS if p.stage == "BOAK" and p.L == 25)
    Ek, Xk = ptk.eligible("B")
    sk = ptk.C / Ek
    de = mp.mpf("1e-12")
    sl_left = (lac5_closed(ptk, sk, "B", "close") - lac5_closed(ptk, sk - de, "B", "close")) / de
    sl_right = (lac5_closed(ptk, sk + de, "B", "close") - lac5_closed(ptk, sk, "B", "close")) / de
    ratio = sl_left / sl_right
    print(f"ledger slope ratio left/right of u = 1 (definition B): {mp.nstr(ratio,8)}  vs ln(1+i)/i = {mp.nstr(mp.log(1+I)/I,8)}")
    check("C10e", "ledger kink at u = 1: slope ratio equals ln(1+i)/i (concave kink of relative size 1 - ln(1+i)/i)",
          abs(ratio - mp.log(1 + I) / I) < mp.mpf("1e-8"))

    # affinity: progressive affine, close not
    pt = next(p for p in POINTS if p.stage == "BOAK" and p.L == 25)
    sd = lambda timing: (lac5_closed(pt, mp.mpf("0.75"), "A", timing) - 2 * lac5_closed(pt, mp.mpf("0.5"), "A", timing)
                         + lac5_closed(pt, mp.mpf("0.25"), "A", timing))
    check("C11", "second difference at s = 0.25, 0.5, 0.75: zero for progressive, positive at close",
          abs(sd("prog")) < tol and sd("close") > mp.mpf("0.01"), f"prog {mp.nstr(sd('prog'),3)}, close {mp.nstr(sd('close'),5)}")

    # break-even shares
    show("Break-even shares s* (definition A, B; progressive, at close; L5, DCF)  [placeholder A from the legacy CSV]")
    hdr = f"{'stage':10s} {'L':>3s} | {'A prog L5':>9s} {'A prog DCF':>10s} {'A cls L5':>9s} {'A cls DCF':>9s} | {'B prog L5':>9s} {'B prog DCF':>10s} {'B cls L5':>9s} {'B cls DCF':>9s} | dated A"
    print(hdr)
    okroot, okstrict = True, True
    for pt in POINTS:
        b0 = base_dcf[(pt.stage, pt.L)]
        row = []
        for d in ("A", "B"):
            E, X = pt.eligible(d)
            for timing in ("prog", "close"):
                f5 = lambda s: lac5_closed(pt, s, d, timing)
                fd = lambda s: lacd_closed(pt, s, d, timing, b0)
                vals = []
                for fun, kind in ((f5, "L5"), (fd, "DCF")):
                    # numerical root by bisection on [0, 1] (fallback)
                    if fun(mp.mpf(1)) >= TGT:
                        sroot = None
                    elif fun(mp.mpf(0)) < TGT:
                        sroot = mp.mpf(0)
                    else:
                        lo, hi = mp.mpf(0), mp.mpf(1)
                        for _ in range(200):
                            mid = (lo + hi) / 2
                            if fun(mid) < TGT:
                                hi = mid
                            else:
                                lo = mid
                        sroot = hi
                    # closed form where one exists
                    if kind == "L5" and sroot is not None and sroot > 0:
                        if timing == "prog":
                            sc = (pt.l5_0 - TGT) / (pt.kappa * (pt.C * FI + X))
                        else:
                            Hs = FI - (pt.l5_0 - TGT) / (pt.kappa * pt.C)
                            if Hs >= 0:
                                us = 1 - mp.log(1 + I * NC * Hs) / (NC * mp.log(1 + I))
                                sc = us * pt.C / E
                            else:
                                sc = (pt.C + (pt.l5_0 - TGT - pt.kappa * pt.C * FI) / pt.kappa) / E
                        okroot = okroot and abs(sc - sroot) < mp.mpf("1e-30")
                    if kind == "DCF" and timing == "prog" and sroot is not None and sroot > 0:
                        sc = (b0 - TGT) / (pt.kappa * (pt.C * FD + X))
                        okroot = okroot and abs(sc - sroot) < mp.mpf("1e-30")
                    if sroot is not None and sroot > 0:
                        okstrict = okstrict and fun(sroot) < TGT and fun(sroot - mp.mpf("1e-12")) > TGT
                    vals.append(sroot)
                row += vals
        dated = (pt.l5_0 - TGT) / (pt.kappa * pt.C * (1 + I) ** NC)
        fmt = lambda x: "  >1  " if x is None else mp.nstr(x, 5)
        print(f"{pt.stage:10s} {pt.L:3d} | {fmt(row[0]):>9s} {fmt(row[1]):>10s} {fmt(row[2]):>9s} {fmt(row[3]):>9s} | "
              f"{fmt(row[4]):>9s} {fmt(row[5]):>10s} {fmt(row[6]):>9s} {fmt(row[7]):>9s} | {mp.nstr(dated,5)}")
    check("C12", "closed-form s* equals the bisection root (progressive L5 and DCF, at-close L5)", okroot)
    check("C13", "strict test: LAC(s*) < T and LAC(s* - 1e-12) > T (the infimum is approached from the passing side)", okstrict)

    # the cases of s* (S.7e), shown by rescaling the placeholder abatement A
    def scaled(pt0, factor):
        import copy
        q_ = copy.copy(pt0)
        q_.a = pt0.a * factor
        q_.l5_0 = pt0.l5_0 / factor
        q_.kappa = pt0.kappa / factor
        return q_

    def classify(pt0, factor, d="A", timing="prog"):
        q_ = scaled(pt0, factor)
        E_, X_ = q_.eligible(d)
        sst = (q_.l5_0 - TGT) / (q_.kappa * (q_.C * FI + X_))
        grid_pass = [lac5_closed(q_, sg, d, timing) < TGT for sg in SHARES]
        if sst < 0:
            case = "s* < 0: target met without a grant"
        elif sst == 0:
            case = "s* = 0: LAC(0) = T, strict test fails at s = 0, every s > 0 passes"
        elif sst <= mp.mpf("0.75"):
            case = "0 < s* <= 0.75: some grid shares pass"
        elif sst < 1:
            case = "0.75 < s* < 1: no grid share passes, a larger share does"
        else:
            case = "s* >= 1: unreachable on this eligible set"
        return sst, case, grid_pass

    boak25 = next(p for p in POINTS if p.stage == "BOAK" and p.L == 25)
    foak25 = next(p for p in POINTS if p.stage == "FOAK" and p.L == 25)
    cases = [(boak25, mp.mpf(2)), (boak25, mp.mpf(1)), (foak25, mp.mpf(1)), (boak25, mp.mpf("0.4"))]
    print("\nCases of s* (progressive, definition A, L5), rescaling the placeholder abatement:")
    results_cls = []
    for pt0, fac in cases:
        sst, case, gp = classify(pt0, fac)
        results_cls.append((sst, case, gp))
        print(f"  {pt0.stage} {pt0.L} y, A x {mp.nstr(fac,3)}: s* = {mp.nstr(sst,6)}  {case}  grid pass (0, .25, .5, .75) = {gp}")
    # exact tie LAC(0) = T
    tie = scaled(boak25, boak25.l5_0 / TGT)
    ssst, case_t, gp_t = classify(boak25, boak25.l5_0 / TGT)
    print(f"  tie: LAC(0) = {mp.nstr(tie.l5_0,12)}, s* = {mp.nstr(ssst,3)}  grid pass = {gp_t}")
    check("C19", "case analysis of s*: negative, inside the grid range, beyond 75 percent, above 1, and the exact tie",
          results_cls[0][0] < 0 and results_cls[0][2] == [True, True, True, True]
          and 0 < results_cls[1][0] < mp.mpf("0.75") and results_cls[1][2] == [False, False, False, True]
          and mp.mpf("0.75") < results_cls[2][0] < 1 and results_cls[2][2] == [False, False, False, False]
          and results_cls[3][0] > 1
          and abs(ssst) < mp.mpf("1e-30") and gp_t == [False, True, True, True])

    # allowance form: s* = (dC5(0) - T A 1e-6) / (CRF (C f_IDC + X)), where A enters only through T A
    pt0 = boak25
    allow = (sum(pt0.lines[k] for k in LINE_IDS) - TGT * pt0.a / mp.mpf(10) ** 6) / (crf(pt0.L) * pt0.C * FI)
    check("C20", "allowance form of s*: the abatement A enters only through the allowed annual cost T A 1e-6",
          abs(allow - (pt0.l5_0 - TGT) / (pt0.kappa * pt0.C * FI)) < mp.mpf("1e-30"))

    # DCF at close: exact piecewise-linear root versus bisection
    okpw = True
    for pt in POINTS:
        b0 = base_dcf[(pt.stage, pt.L)]
        for d in ("A", "B"):
            E, X = pt.eligible(d)
            fun = lambda s: lacd_closed(pt, s, d, "close", b0)
            knots = [mp.mpf(0)]
            cum = mp.mpf(0)
            for _, b in buckets(pt.C):
                cum += b
                knots.append(cum / E)
            knots.append(mp.mpf(1))
            knots = sorted(set(k for k in knots if 0 <= k <= 1))
            root = None
            for ka, kb in zip(knots[:-1], knots[1:]):
                fa, fb = fun(ka), fun(kb)
                if fa >= TGT > fb or (fa > TGT >= fb):
                    root = ka + (fa - TGT) / (fa - fb) * (kb - ka)
                    break
            if root is not None:
                lo, hi = mp.mpf(0), mp.mpf(1)
                for _ in range(200):
                    mid = (lo + hi) / 2
                    if fun(mid) < TGT:
                        hi = mid
                    else:
                        lo = mid
                okpw = okpw and abs(root - hi) < mp.mpf("1e-25")
    check("C14", "DCF at close: piecewise-linear exact root equals the bisection root", okpw)

    # the wrong ways of applying s (S.5)
    show("Worked example, BOAK 25 y, definition A, s = 0.5 (placeholder A)")
    pt = next(p for p in POINTS if p.stage == "BOAK" and p.L == 25)
    b0 = base_dcf[(pt.stage, pt.L)]
    s = mp.mpf("0.5")
    G = s * pt.C
    u = G / pt.C
    idc1 = pt.C * (FI - 1)
    idc2_prog = (1 - s) * pt.C * (FI - 1)
    idc2_close = pt.C * h_close(u) - pt.C * (1 - u)
    print(f"C_R = {mp.nstr(pt.C,8)}  G = {mp.nstr(G,8)}  f_IDC = {mp.nstr(FI,8)}  h(0.5) = {mp.nstr(h_close(u),8)}")
    print(f"IDC basis 1 (gross spend)             = {mp.nstr(idc1,8)} S$M")
    print(f"IDC basis 2 private, progressive      = {mp.nstr(idc2_prog,8)} S$M   saving (basis 3) = {mp.nstr(idc1-idc2_prog,8)}")
    print(f"IDC basis 2 private, at close         = {mp.nstr(idc2_close,8)} S$M   saving (basis 3) = {mp.nstr(idc1-idc2_close,8)}")
    idc_dated = idc1 - G * ((1 + I) ** NC - 1)
    print(f"dated-flow bound (not default)        = {mp.nstr(idc_dated,8)} S$M (negative = interest income); sign change at s0 = {mp.nstr((FI-1)/((1+I)**NC-1),6)}")
    print(f"L5 zero {mp.nstr(pt.l5_0,8)}; progressive {mp.nstr(lac5_closed(pt,s,'A','prog'),8)}; at close {mp.nstr(lac5_closed(pt,s,'A','close'),8)}")
    print(f"DCF zero {mp.nstr(b0,8)}; progressive {mp.nstr(lacd_closed(pt,s,'A','prog',b0),8)}; at close {mp.nstr(lacd_closed(pt,s,'A','close',b0),8)}")
    wrong_base = pt.kappa * G * (FI - 1)
    print(f"mistake W1/W2 (grant base includes IDC, or IDC reduced twice): L5 too low by {mp.nstr(wrong_base,6)} S$/t")
    wrong_timing = pt.kappa * pt.C * ((FI - h_close(u)) - s * FI)
    print(f"mistake W3 (progressive rule applied to an at-close grant): L5 too high by {mp.nstr(wrong_timing,6)} S$/t")
    check("C15", "W1/W2 magnitude equals s C (f-1) kappa and is the gap between a PV-based and a nominal grant base",
          abs(wrong_base - (pt.kappa * s * pt.C * FI - pt.kappa * s * pt.C)) < tol)
    # W4 (S.5b): the share of a year-0 present value, dated at financial close and compounded again.
    # Built as an explicit dated GRANT flow in the DCF (a flow t_c before year 0 is worth a (1+i)^t_c times its amount at year 0)
    # and as the ledger credit, then compared with the correct progressive reduction.
    correct_prog5 = pt.l5_0 - lac5_closed(pt, s, "A", "prog")
    correct_progD = b0 - lacd_closed(pt, s, "A", "prog", b0)
    w4_flow = [("GRANT", -NC, -s * pt.C * FD)]
    w4_D = b0 - lacd_flows_extra(pt, w4_flow)
    w4_5 = pt.kappa * s * pt.C * FI * (1 + I) ** NC
    print(f"mistake W4 (share of the year-0 PV, dated at close and compounded again): reduction {mp.nstr(w4_5,8)} S$/t "
          f"against the correct {mp.nstr(correct_prog5,8)}; excess {mp.nstr(w4_5 - correct_prog5,8)} S$/t "
          f"({mp.nstr(100 * (w4_5 - correct_prog5) / pt.l5_0,4)} percent of L5(0))")
    check("C15b", "W4 counts the interest twice: the credit is (1+i)^t_c times the correct progressive credit, excess 37.985 S$/t (BOAK 25 y, A, s=0.5)",
          abs(w4_5 / correct_prog5 - (1 + I) ** NC) < mp.mpf("1e-20") and abs(w4_D / correct_progD - (1 + I) ** NC) < mp.mpf("1e-20")
          and abs((w4_5 - correct_prog5) - mp.mpf("37.985")) < mp.mpf("5e-4"), mp.nstr(w4_5 - correct_prog5, 8))

    show("Funded balance of the private reactor funding, BOAK 25 y, s = 0.5, S$M (year-end)")
    gross, _ = funded_balance(pt, mp.mpf(0), "A", "prog")
    prog, _ = funded_balance(pt, s, "A", "prog")
    close, _ = funded_balance(pt, s, "A", "close")
    dated, _ = funded_balance(pt, s, "A", "close", mode="dated")
    print(f"{'year':>5s} {'gross':>10s} {'progressive':>12s} {'at close':>10s} {'dated bound':>12s}")
    for (t, g_), (_, p_), (_, c_), (_, d_) in zip(gross, prog, close, dated):
        print(f"{t:5d} {mp.nstr(g_,8):>10s} {mp.nstr(p_,8):>12s} {mp.nstr(c_,8):>10s} {mp.nstr(d_,8):>12s}")
    check("C16", "balance at year 0: gross = C f_D, progressive = (1-s) C f_D, close = P_D(G)",
          abs(gross[-1][1] - pt.C * FD) < tol and abs(prog[-1][1] - (1 - s) * pt.C * FD) < tol
          and abs(close[-1][1] - pd_closed(pt.C, G)) < tol)
    check("C17", "the dated-flow bound holds a negative balance in the early years (a cash surplus earning interest) and gives negative private IDC",
          min(b_ for _, b_ in dated) < 0 and idc_dated < 0, f"min balance {mp.nstr(min(b_ for _, b_ in dated),6)}, IDC {mp.nstr(idc_dated,6)}")
    s_zero = FI / (1 + I) ** NC
    print(f"dated bound, definition A: private PV at year 0 changes sign at s = f_IDC/(1+i)^t_c = {mp.nstr(s_zero,6)}; at s = 1 it is {mp.nstr(pt.C*(FI-(1+I)**NC),8)} S$M")
    check("C18", "dated bound at s = 1 overshoots: it removes more than the reactor capital and its interest",
          pt.C * (FI - (1 + I) ** NC) < 0)

    # public present value at close versus the private benefit (informational, S.8)
    pub_close = G * (1 + I) ** NC
    priv_benefit = pt.C * FD - pd_closed(pt.C, G)
    print(f"state-side PV at year 0 of a grant paid at close = {mp.nstr(pub_close,8)} S$M; private benefit = {mp.nstr(priv_benefit,8)} S$M; "
          f"carry of idle cash = {mp.nstr(pub_close-priv_benefit,8)} S$M")
    pub_prog = s * pt.C * FD
    check("C21", "state-side PV exceeds the private benefit at close by the idle-cash carry; progressive timing has no leak",
          pub_close > priv_benefit and abs(pub_prog - (pt.C * FD - (1 - s) * pt.C * FD)) < tol)

    # ordering of the break-even shares: progressive > at close > dated bound (L5, definition A and B where the dated bound applies)
    order_ok = True
    for p_ in POINTS:
        for d in ("A", "B"):
            E_, X_ = p_.eligible(d)
            s_prog = (p_.l5_0 - TGT) / (p_.kappa * (p_.C * FI + X_))
            Hs = FI - (p_.l5_0 - TGT) / (p_.kappa * p_.C)
            if Hs >= 0:
                s_cls = (1 - mp.log(1 + I * NC * Hs) / (NC * mp.log(1 + I))) * p_.C / E_
            else:
                s_cls = (p_.C + (p_.l5_0 - TGT - p_.kappa * p_.C * FI) / p_.kappa) / E_
            s_dat = (p_.l5_0 - TGT) / (p_.kappa * (p_.C * (1 + I) ** NC + X_))
            order_ok = order_ok and (s_prog > s_cls > s_dat)
    check("C22", "ordering of break-even shares on all 8 points and both definitions: progressive > at close > dated bound", order_ok)

    # S.3c step 7: the smooth ledger form has a small negative private interest when the private window is shorter than 1 y;
    # the dated balance of S.3a never does.
    pt23 = next(p for p in POINTS if p.stage == "BOAK" and p.L == 25)
    idc2 = lambda u_: pt23.C * (h_close(u_) - (1 - u_))
    u_min = 1 - mp.log(I / mp.log(1 + I)) / (NC * mp.log(1 + I))
    grid_u = [mp.mpf(k) / 2000 for k in range(0, 2001)]
    neg_ok = all((idc2(x) < 0) == (x > 1 - 1 / NC) for x in grid_u if abs(x - (1 - 1 / NC)) > mp.mpf("1e-9") and x < 1)
    zero_ok = abs(idc2(1 - 1 / NC)) < mp.mpf("1e-30") and abs(idc2(mp.mpf(1))) < mp.mpf("1e-30")
    min_ok = all(idc2(x) >= idc2(u_min) - mp.mpf("1e-30") for x in grid_u) and abs((idc2(u_min) / pt23.C) + mp.mpf("0.0017489707")) < mp.mpf("1e-9")
    bal_ok = True
    for k_ in range(0, 401):
        s_ = mp.mpf(k_) / 400
        _, b0_ = funded_balance(pt23, s_, "A", "close")
        bal_ok = bal_ok and (b0_ - pt23.C * (1 - s_) >= -mp.mpf("1e-30"))
    print(f"ledger private interest at close: zero at u = 1 - 1/t_c = {mp.nstr(1 - 1 / NC,6)}, minimum {mp.nstr(idc2(u_min),6)} S$M "
          f"({mp.nstr(idc2(u_min) / pt23.C,6)} of C_R) at u = {mp.nstr(u_min,6)}; LAC effect {mp.nstr(pt23.kappa * idc2(u_min),5)} S$/t")
    check("C23", "S.3c step 7: ledger IDC2 < 0 exactly for 1 - 1/t_c < u < 1 (minimum -0.0017490 C_R at u_min), dated-balance IDC2 >= 0 for every grant",
          neg_ok and zero_ok and min_ok and bal_ok)

    show("LAC at the grid shares, BOAK 25 y (placeholder A), S$/t")
    print(f"zero subsidy: L5 {mp.nstr(pt.l5_0,8)}  DCF {mp.nstr(b0,8)}")
    for d in ("A", "B"):
        for timing in ("prog", "close"):
            r5 = [mp.nstr(lac5_closed(pt, sg, d, timing), 7) for sg in SHARES[1:]]
            rd = [mp.nstr(lacd_closed(pt, sg, d, timing, b0), 7) for sg in SHARES[1:]]
            fl = ["pass" if (lac5_closed(pt, sg, d, timing) < TGT and lacd_closed(pt, sg, d, timing, b0) < TGT) else "fail" for sg in SHARES[1:]]
            print(f"  def {d} {timing:5s}  L5 {r5}  DCF {rd}  both pass: {fl}")

    show("Reported quantities, BOAK 25 y, s = 0.5 (S.8), placeholder A")
    for d in ("A", "B"):
        E, X = pt.eligible(d)
        Gd = s * E
        ktot = pt.C + pt.Kcap + pt.Kihx + pt.Klic + pt.Kpi + pt.Krec
        idc_g = pt.C * (FI - 1)
        print(f"def {d}: K_tot = {mp.nstr(ktot,8)}  TCR gross = {mp.nstr(ktot+idc_g,8)}  E = {mp.nstr(E,8)}  G = {mp.nstr(Gd,8)}  K_priv = {mp.nstr(ktot-Gd,8)}")
        for timing in ("prog", "close"):
            ledger_net = lac5_closed(pt, s, d, timing) * pt.a / mp.mpf(10) ** 6
            print(f"   {timing:5s}: dC_gross = {mp.nstr(pt.l5_0*pt.a/mp.mpf(10)**6,8)} S$M/y  dC_net = {mp.nstr(ledger_net,8)} S$M/y"
                  f"  LAC5 gross {mp.nstr(pt.l5_0,7)} net {mp.nstr(lac5_closed(pt,s,d,timing),7)};  DCF gross {mp.nstr(b0,7)} net {mp.nstr(lacd_closed(pt,s,d,timing,b0),7)}")


def main() -> int:
    symbolic_part()
    part_b()
    part_c()
    failed = [r for r in RESULTS if not r[1]]
    show("SUMMARY")
    print(f"{len(RESULTS)} checks, {len(RESULTS) - len(failed)} passed, {len(failed)} failed")
    for cid, _, text, detail in failed:
        print(f"FAILED {cid}: {text} {detail}")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
