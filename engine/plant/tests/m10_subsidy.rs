//! M10 tests: the government co-funding model of `docs/derivation/02_cost_and_subsidy.md`.
//!
//! Every test is named after the equation it checks. The closed forms that the
//! tests compare against are written here from the derivation, on registry
//! values and capital items, and never through `nush2_plant::subsidy`. The
//! annual net abatement is the legacy value that the model passes in.

use nush2_plant::anchored::{Balance, Model, Scenario};
use nush2_plant::dcf::{uniform_spend, DcfResult};
use nush2_plant::economics::{CostItems, CostQuery, Stage};
use nush2_plant::params::ParamSet;
use nush2_plant::subsidy::{
    bisect_share, bucket_factor, combine, funded_balance, private_factor_at_close,
    private_pv_closed_form, round_up_share, BreakEven, BreakEvenSet, Definition, GrantQuery,
    SolveRoute, Timing,
};
use nush2_plant::Inputs;
use std::sync::OnceLock;

const LIVES: [f64; 2] = [25.0, 60.0];

struct Ctx {
    p: ParamSet,
    m: Model,
    b: Balance,
    a: f64,
}

fn build(p: ParamSet) -> Ctx {
    let m = Model::new(&p).unwrap();
    let b = m.balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let a = m.emissions(&b.c, b.s).a_atm;
    Ctx { p, m, b, a }
}

/// The Headline model of the exporter, with the registry flags of decision D7b.
fn ctx() -> &'static Ctx {
    static C: OnceLock<Ctx> = OnceLock::new();
    C.get_or_init(|| build(Inputs::headline().params().unwrap()))
}

/// The same model with definition B set to the membership of the equation S.1
/// table: reactor, IHX, licence and power island.
fn ctx_s1() -> &'static Ctx {
    static C: OnceLock<Ctx> = OnceLock::new();
    C.get_or_init(|| {
        let mut p = Inputs::headline().params().unwrap();
        p.set("SUBSIDY_ELIG_B_CAPTURE", 0.0).unwrap();
        p.set("SUBSIDY_ELIG_B_RECYCLE", 0.0).unwrap();
        p.set("SUBSIDY_ELIG_B_POWER_ISLAND", 1.0).unwrap();
        build(p)
    })
}

fn query(c: &Ctx, stage: Stage, life: f64) -> CostQuery {
    CostQuery::central(stage, life, c.m.n_linear, c.m.r_rated)
}

fn gq(share: f64, definition: Definition, timing: Timing) -> GrantQuery {
    GrantQuery {
        share,
        definition,
        timing,
    }
}

fn with(q: &CostQuery, g: GrantQuery) -> CostQuery {
    CostQuery {
        grant: Some(g),
        ..*q
    }
}

fn close(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} against {b}");
}

// ------------------------------------------------- closed forms from the derivation

fn crf_(i: f64, n: f64) -> f64 {
    i / (1.0 - (1.0 + i).powf(-n))
}

fn f_idc(i: f64, tc: f64) -> f64 {
    ((1.0 + i).powf(tc) - 1.0) / (i * tc)
}

fn f_dcf(i: f64, tc: f64) -> f64 {
    let w = tc.floor();
    let q = tc - w;
    let g = (1.0 + i).powf(w);
    (q * g + (g - 1.0) / i) / tc
}

fn h_close(i: f64, tc: f64, u: f64) -> f64 {
    ((1.0 + i).powf(tc * (1.0 - u)) - 1.0) / (i * tc)
}

/// Reactor overnight capital from the registry alone (equations L.5 to L.10).
fn cr_registry(c: &Ctx, stage: Stage) -> f64 {
    let g = |id: &str| c.p.get(id).unwrap();
    let foak = g("FOAK_CENTRAL");
    let lf10 = (1.0 - g("LEARNING")).powf(g("LEARNING_UNITS").log2());
    let mult = match stage {
        Stage::Foak => foak,
        Stage::Boak => 1.0,
        Stage::Oak10FromBoak => lf10,
        Stage::Oak10FromFoak => foak * lf10,
    };
    g("OCC_CENTRAL") * g("REACTOR_COST_REF_SIZE") * 1e3 / 1e6
        * g("F_2019USD_TO_2025SGD")
        * mult
        * (g("REACTOR_RATED_MWTH") / g("REACTOR_COST_REF_SIZE"))
}

/// Eligible capital other than the reactor, from the registry flags.
fn x_e(c: &Ctx, k: &CostItems, d: Definition) -> f64 {
    let set = match d {
        Definition::A => "A",
        Definition::B => "B",
    };
    let on = |item: &str| c.p.get(&format!("SUBSIDY_ELIG_{set}_{item}")).unwrap() == 1.0;
    let mut x = 0.0;
    if on("IHX") {
        x += k.ihx_capital;
    }
    if on("LICENCE") {
        x += k.licence_capital;
    }
    if on("POWER_ISLAND") {
        x += k.power_island_capital;
    }
    if on("CAPTURE") {
        x += k.capture_capital;
    }
    if on("RECYCLE") {
        x += k.recycle_capital;
    }
    x
}

/// Owner's present value of the reactor spend after a grant at close, by the
/// year-by-year account of S.3a, written out here.
fn pd_account(i: f64, tc: f64, c_r: f64, grant: f64) -> f64 {
    let w = tc.floor() as i32;
    let q = tc - f64::from(w);
    let mut buckets: Vec<(i32, f64)> = Vec::new();
    if q > 0.0 {
        buckets.push((-w, c_r * q / tc));
    }
    for k in 0..w {
        buckets.push((k - w + 1, c_r / tc));
    }
    let (mut spent, mut balance, mut prev) = (0.0, 0.0, None);
    for (date, b) in buckets {
        if let Some(p) = prev {
            balance *= (1.0 + i).powi(date - p);
        }
        let g = b.min((grant - spent).max(0.0));
        balance += b - g;
        spent += b;
        prev = Some(date);
    }
    balance * (1.0 + i).powi(-prev.unwrap())
}

/// Cost per tonne after a grant, by S.6a and S.6b, from the zero-subsidy cost.
struct Closed {
    i: f64,
    tc: f64,
    c_r: f64,
    x_e: f64,
    kappa: f64,
    lac5_0: f64,
    lacd_0: f64,
}

impl Closed {
    fn new(c: &Ctx, stage: Stage, life: f64, d: Definition, a: f64) -> Closed {
        let q = query(c, stage, life);
        let k = c.m.cost_items(&c.b, &q).unwrap();
        let (i, tc) = (
            c.p.get("DISCOUNT").unwrap(),
            c.p.get("CONSTRUCTION_YEARS").unwrap(),
        );
        Closed {
            i,
            tc,
            c_r: cr_registry(c, stage),
            x_e: x_e(c, &k, d),
            kappa: crf_(i, life) * 1e6 / a,
            lac5_0: c.m.ledger(&c.b, &q).unwrap().l5() * 1e6 / a,
            lacd_0: c.m.dcf_at(&c.b, &q, a).unwrap().lac,
        }
    }

    fn lac5(&self, s: f64, timing: Timing) -> f64 {
        let f = f_idc(self.i, self.tc);
        let e = self.c_r + self.x_e;
        match timing {
            Timing::Progressive => self.lac5_0 - self.kappa * s * (self.c_r * f + self.x_e),
            Timing::AtClose => {
                let g = s * e;
                let u = (g / self.c_r).min(1.0);
                self.lac5_0
                    - self.kappa
                        * (self.c_r * (f - h_close(self.i, self.tc, u)) + (g - self.c_r).max(0.0))
            }
        }
    }

    fn lacd(&self, s: f64, timing: Timing) -> f64 {
        let f = f_dcf(self.i, self.tc);
        let e = self.c_r + self.x_e;
        match timing {
            Timing::Progressive => self.lacd_0 - self.kappa * s * (self.c_r * f + self.x_e),
            Timing::AtClose => {
                let g = s * e;
                let pd = pd_account(self.i, self.tc, self.c_r, g.min(self.c_r));
                self.lacd_0 - self.kappa * (self.c_r * f - pd + (g - self.c_r).max(0.0))
            }
        }
    }
}

fn pv_of(d: &DcfResult, id: &str) -> f64 {
    d.lines.iter().filter(|l| l.id == id).map(|l| l.pv).sum()
}

fn split_csv(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let (mut cur, mut quoted) = (String::new(), false);
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match (ch, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => out.push(std::mem::take(&mut cur)),
            (c, _) => cur.push(c),
        }
    }
    out.push(cur);
    out
}

// -------------------------------------------------------------------- S.0.x

#[test]
fn eq_s_0_1_dcf_bucket_factor_equals_the_dated_buckets() {
    let c = ctx();
    let (i, tc) = (c.m.rate, c.m.idc_years);
    let pv: f64 = uniform_spend(1.0, tc)
        .unwrap()
        .iter()
        .map(|(y, v)| v * (1.0 + i).powi(-y))
        .sum();
    close(
        bucket_factor(i, tc).unwrap(),
        pv,
        1e-12,
        "S.0.1 against the buckets",
    );
    close(
        bucket_factor(i, tc).unwrap(),
        1.2002299997,
        1e-9,
        "S.0.1 worked number",
    );
    // A whole number of years reduces to the ledger factor.
    close(
        bucket_factor(i, 5.0).unwrap(),
        f_idc(i, 5.0),
        1e-13,
        "S.0.1 at an integer time",
    );
    assert!(bucket_factor(0.0, 5.5).is_err());
}

#[test]
fn eq_s_0_2_the_two_interest_factors_differ_by_the_closed_form_gap() {
    let c = ctx();
    let (i, tc) = (c.m.rate, c.m.idc_years);
    let (w, q) = (tc.floor(), tc - tc.floor());
    let df = (1.0 + i).powf(w) * (1.0 + q * i - (1.0 + i).powf(q)) / (i * tc);
    close(
        bucket_factor(i, tc).unwrap() - f_idc(i, tc),
        df,
        1e-13,
        "S.0.2 exact gap",
    );
    close(df, 0.0025697061, 1e-9, "S.0.2 worked number");
    assert!(df > 0.0);
    // Leading term of the series: 3.96 percent above the exact value.
    let series = (1.0 + i).powf(w) * q * (1.0 - q) * i / (2.0 * tc);
    let rel = series / df - 1.0;
    assert!((0.03..0.05).contains(&rel), "series is {rel} above");
}

#[test]
fn eq_s_0_3_replacement_factor_gives_the_replaced_item_gap() {
    let c = ctx();
    let (i, a) = (c.m.rate, c.a);
    let v = 1.0 / (1.0 + i);
    let (life, tu): (f64, f64) = (60.0, 25.0);
    let n = (life / tu).ceil();
    let rho = (0..n as i32)
        .map(|m| v.powf(f64::from(m) * tu))
        .sum::<f64>()
        - (n * tu - life) / tu * v.powf(life);
    close(rho, 1.1614136, 1e-7, "S.0.3 worked number");
    let q = query(c, Stage::Boak, life);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let d = c.m.dcf(&c.b, &q).unwrap();
    let mut sum = 0.0;
    for (id, cap) in [
        ("CAPTURE_CAP", k.capture_capital),
        ("IHX", k.ihx_capital),
        ("RECYCLE_TRAIN", k.recycle_capital),
    ] {
        close(pv_of(&d, id), cap * rho, 1e-9, id);
        let gap = cap * (rho * crf_(i, life) - crf_(i, tu)) * 1e6 / a;
        let term = d.gap_terms.iter().find(|t| t.id == id).unwrap();
        close(term.delta, gap, 1e-9, id);
        sum += gap;
    }
    close(sum, 0.065989, 1e-6, "S.0.3 summed gap, BOAK 60 y");
    // On a 25 y life nothing is bought again and the gap is zero.
    let d25 = c.m.dcf(&c.b, &query(c, Stage::Boak, 25.0)).unwrap();
    for id in ["CAPTURE_CAP", "IHX", "RECYCLE_TRAIN"] {
        let term = d25.gap_terms.iter().find(|t| t.id == id).unwrap();
        assert!(term.delta.abs() < 1e-9, "{id}");
    }
}

#[test]
fn eq_s_0_4_decommissioning_sinking_fund_equals_the_discounted_payment() {
    let c = ctx();
    let i = c.m.rate;
    for life in LIVES {
        let lhs = (1.0 + i).powf(-life) * crf_(i, life);
        let rhs = i / ((1.0 + i).powf(life) - 1.0);
        close(lhs, rhs, 1e-14, "S.0.4 identity");
        let d = c.m.dcf(&c.b, &query(c, Stage::Boak, life)).unwrap();
        let t = d.gap_terms.iter().find(|t| t.id == "DECOM").unwrap();
        assert!(t.delta.abs() < 1e-9, "DECOM gap {}", t.delta);
    }
}

#[test]
fn eq_s_0_5_closed_forms_reproduce_both_costs_per_tonne() {
    let c = ctx();
    let (i, tc, a) = (c.m.rate, c.m.idc_years, c.a);
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            let k = c.m.cost_items(&c.b, &q).unwrap();
            let y = -(k.ng_smr - k.ng_plant) * k.gas_s_m_per_mmscfd
                + k.capture_fom_y
                + k.capture_allin_y
                + k.ts_y
                + k.reactor_om_y
                + (k.steam_smr_mm - k.steam_plant_mm) * k.steam_s_m_per_mm
                - k.smr_power_y
                - k.export_revenue_y;
            let (crl, cr25) = (crf_(i, life), crf_(i, c.m.item_life));
            let d_c5 = y
                + k.recycle_fom_y
                + crl
                    * (k.reactor_capital * f_idc(i, tc)
                        + k.licence_capital
                        + k.power_island_capital)
                + cr25 * (k.capture_capital + k.ihx_capital + k.recycle_capital)
                + k.decom_total * i / ((1.0 + i).powf(life) - 1.0);
            let led = c.m.ledger(&c.b, &q).unwrap();
            close(d_c5, led.l5(), 1e-9, "S.0.5 ledger");
            let v = 1.0 / (1.0 + i);
            let alpha = (1..=life as i32).map(|t| v.powi(t)).sum::<f64>();
            let tu = c.m.item_life;
            let n = (life / tu).ceil();
            let rho = (0..n as i32)
                .map(|m| v.powf(f64::from(m) * tu))
                .sum::<f64>()
                - (n * tu - life) / tu * v.powf(life);
            let d_npv = alpha * (y + k.recycle_fom_y)
                + k.reactor_capital * f_dcf(i, tc)
                + rho * (k.capture_capital + k.ihx_capital + k.recycle_capital)
                + k.licence_capital
                + k.power_island_capital
                + k.decom_total * v.powf(life);
            let d = c.m.dcf(&c.b, &q).unwrap();
            close(d_npv * 1e6 / (a * alpha), d.lac, 1e-9, "S.0.5 DCF");
        }
    }
}

#[test]
fn eq_s_0_6_the_gap_between_the_methods_at_zero_subsidy_is_predicted() {
    let c = ctx();
    let (i, tc, a) = (c.m.rate, c.m.idc_years, c.a);
    let mut worst = 0.0f64;
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            let k = c.m.cost_items(&c.b, &q).unwrap();
            let v = 1.0 / (1.0 + i);
            let tu = c.m.item_life;
            let n = (life / tu).ceil();
            let rho = (0..n as i32)
                .map(|m| v.powf(f64::from(m) * tu))
                .sum::<f64>()
                - (n * tu - life) / tu * v.powf(life);
            let gap = 1e6 / a
                * (crf_(i, life) * k.reactor_capital * (f_dcf(i, tc) - f_idc(i, tc))
                    + (rho * crf_(i, life) - crf_(i, tu))
                        * (k.capture_capital + k.ihx_capital + k.recycle_capital));
            let d = c.m.dcf(&c.b, &q).unwrap();
            close(d.gap_abs, gap, 1e-9, "S.0.6 gap");
            worst = worst.max(d.gap_rel.abs());
        }
    }
    // The largest zero-subsidy gap is 0.204 percent of L5 (FOAK, 60 y).
    close(worst, 0.00204, 5e-5, "largest relative gap");
}

// ------------------------------------------------------ zero share changes nothing

#[test]
fn eq_s_9b_a_zero_share_changes_nothing_bit_for_bit() {
    let c = ctx();
    for stage in Stage::ALL {
        for life in LIVES {
            let q0 = query(c, stage, life);
            for d in Definition::ALL {
                for t in Timing::ALL {
                    let qz = with(&q0, gq(0.0, d, t));
                    assert_eq!(
                        c.m.ledger(&c.b, &q0).unwrap(),
                        c.m.ledger(&c.b, &qz).unwrap()
                    );
                    assert_eq!(c.m.dcf(&c.b, &q0).unwrap(), c.m.dcf(&c.b, &qz).unwrap());
                    assert_eq!(
                        c.m.cost_items(&c.b, &q0).unwrap(),
                        c.m.cost_items(&c.b, &qz).unwrap()
                    );
                    let ev = c.m.subsidy_eval(&c.b, &q0, gq(0.0, d, t), c.a).unwrap();
                    assert_eq!(ev.l5.lac_gross, ev.l5.lac_net);
                    assert_eq!(ev.dcf.lac_gross, ev.dcf.lac_net);
                    assert_eq!(ev.plan.grant, 0.0);
                    assert_eq!(ev.plan.private_capital, ev.plan.total_capital);
                }
            }
        }
    }
}

#[test]
fn eq_s_9b_zero_subsidy_reproduces_the_committed_ledger_and_dcf() {
    let c = ctx();
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../results/Economic_Ledger.csv"
    );
    let text = std::fs::read_to_string(path).unwrap();
    let rows: Vec<Vec<String>> = text.lines().skip(1).map(split_csv).collect();
    let mut checked = 0;
    for stage in Stage::ALL {
        for life in LIVES {
            let pick = |level: &str, line: &str| -> Vec<&Vec<String>> {
                rows.iter()
                    .filter(|r| {
                        r[0] == "B_REC_F"
                            && r[1] == stage.id()
                            && r[2] == format!("{life:.0}")
                            && r[3] == "1.0000"
                            && r[4] == "210.00"
                            && (level.is_empty() || r[5] == level)
                            && (line.is_empty() || r[6] == line)
                    })
                    .collect()
            };
            let l5 = pick("L5", "").last().unwrap()[12].parse::<f64>().unwrap();
            let lac = pick("DCF", "LAC_DCF")[0][18].parse::<f64>().unwrap();
            let q = query(c, stage, life);
            let led = c.m.ledger(&c.b, &q).unwrap();
            let d = c.m.dcf(&c.b, &q).unwrap();
            close(led.l5() * 1e6 / c.a, l5, 1e-6, "committed L5");
            close(d.lac, lac, 1e-6, "committed LAC_DCF");
            checked += 2;
        }
    }
    assert_eq!(checked, 16);
}

// ------------------------------------------------------------------------ S.1

#[test]
fn eq_s_1_eligible_capital_is_the_flagged_sum_and_b_contains_a() {
    let c = ctx();
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            let k = c.m.cost_items(&c.b, &q).unwrap();
            let pa =
                c.m.grant_plan(&k, &gq(0.5, Definition::A, Timing::Progressive))
                    .unwrap();
            let pb =
                c.m.grant_plan(&k, &gq(0.5, Definition::B, Timing::Progressive))
                    .unwrap();
            close(pa.eligible, k.reactor_capital, 0.0, "E_A is the reactor");
            close(
                pb.eligible,
                k.reactor_capital
                    + k.ihx_capital
                    + k.licence_capital
                    + k.capture_capital
                    + k.recycle_capital,
                1e-9,
                "E_B of decision D7b",
            );
            assert!(pb.eligible >= pa.eligible);
            assert!(pb.grant >= pa.grant);
            assert_eq!(pa.other_eligible, 0.0);
            close(
                pb.other_eligible,
                pb.eligible - pb.reactor_capital,
                1e-9,
                "X_E",
            );
            // The power island stays outside definition B of decision D7b.
            assert!(pb.eligible < pb.total_capital);
        }
    }
    // Every flag of A is also a flag of B.
    let (fa, fb) = (c.m.subsidy.eligible_a, c.m.subsidy.eligible_b);
    assert!(fa.reactor <= fb.reactor && fa.ihx <= fb.ihx && fa.licence <= fb.licence);
    assert!(fa.power_island <= fb.power_island && fa.capture <= fb.capture);
    assert!(fa.recycle <= fb.recycle);
}

#[test]
fn eq_s_1_worked_numbers_of_the_derivation_follow_from_its_membership() {
    let c = ctx_s1();
    let q = query(c, Stage::Boak, 25.0);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let pa =
        c.m.grant_plan(&k, &gq(0.5, Definition::A, Timing::Progressive))
            .unwrap();
    let pb =
        c.m.grant_plan(&k, &gq(0.5, Definition::B, Timing::Progressive))
            .unwrap();
    close(pa.eligible, 742.767, 1e-3, "E_A, BOAK 25 y");
    close(pb.eligible, 914.637, 1e-3, "E_B of the S.1 table");
    close(pb.other_eligible, 171.869, 1e-3, "X_E of the S.1 table");
    // The same inputs give 914.637 only with the power island in and the
    // capture plant and the recycle train out.
    assert!(!c.m.subsidy.eligible_b.capture && c.m.subsidy.eligible_b.power_island);
}

// ------------------------------------------------------------------------ S.2

#[test]
fn eq_s_2_private_capital_never_rises_with_the_share_and_the_grant_stays_within_eligible() {
    let c = ctx();
    for stage in Stage::ALL {
        let q = query(c, stage, 25.0);
        let k = c.m.cost_items(&c.b, &q).unwrap();
        for d in Definition::ALL {
            for t in Timing::ALL {
                let mut last: Option<(f64, f64)> = None;
                for n in 0..=100 {
                    let s = f64::from(n) / 100.0;
                    let p = c.m.grant_plan(&k, &gq(s, d, t)).unwrap();
                    assert!(p.grant <= p.eligible, "grant above eligible at {s}");
                    assert!(p.eligible <= p.total_capital);
                    close(p.private_capital, p.total_capital - p.grant, 1e-9, "S.2");
                    close(p.grant, s * p.eligible, 1e-9, "G = sE");
                    close(p.grant_reactor + p.grant_year0, p.grant, 1e-9, "split of G");
                    if let Some((_, prev)) = last {
                        assert!(p.private_capital <= prev, "private capital rose at {s}");
                    }
                    last = Some((s, p.private_capital));
                }
            }
        }
    }
    // BOAK 25 y at half: K_tot 1062.227, G 371.384 (A) and the private rest.
    let q = query(c, Stage::Boak, 25.0);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let p =
        c.m.grant_plan(&k, &gq(0.5, Definition::A, Timing::Progressive))
            .unwrap();
    close(p.total_capital, 1062.227, 1e-3, "K_tot");
    close(p.grant, 371.384, 1e-3, "G");
    close(p.private_capital, 690.844, 1e-3, "K_priv");
    // A share outside 0 to 1 is refused.
    assert!(c
        .m
        .grant_plan(&k, &gq(1.01, Definition::A, Timing::Progressive))
        .is_err());
    assert!(c
        .m
        .grant_plan(&k, &gq(-0.01, Definition::A, Timing::Progressive))
        .is_err());
    assert!(c
        .m
        .grant_plan(&k, &gq(f64::NAN, Definition::A, Timing::Progressive))
        .is_err());
}

#[test]
fn eq_s_2_the_subsidy_changes_no_physical_flow_and_no_operating_cost() {
    let c = ctx();
    for stage in Stage::ALL {
        for life in LIVES {
            let q0 = query(c, stage, life);
            let led0 = c.m.ledger(&c.b, &q0).unwrap();
            let d0 = c.m.dcf(&c.b, &q0).unwrap();
            for d in Definition::ALL {
                for t in Timing::ALL {
                    let qg = with(&q0, gq(0.6, d, t));
                    // The cost items, which hold every flow and every capital.
                    assert_eq!(
                        c.m.cost_items(&c.b, &q0).unwrap(),
                        c.m.cost_items(&c.b, &qg).unwrap()
                    );
                    // The gross ledger lines, which hold the operating cost.
                    let ledg = c.m.ledger(&c.b, &qg).unwrap();
                    assert_eq!(led0.lines(), ledg.lines());
                    assert_eq!(led0.ladder(), ledg.ladder());
                    assert!(led0.net.is_none() && ledg.net.is_some());
                    // The DCF: every line but the grant, and the tonnes.
                    let dg = c.m.dcf(&c.b, &qg).unwrap();
                    let keep = |r: &DcfResult| -> Vec<_> {
                        r.lines
                            .iter()
                            .filter(|l| l.id != "GRANT")
                            .cloned()
                            .collect()
                    };
                    assert_eq!(keep(&d0), keep(&dg));
                    assert_eq!(d0.annual_abatement_t, dg.annual_abatement_t);
                    assert_eq!(d0.annual_h2_t, dg.annual_h2_t);
                    assert_eq!(d0.npv_h2_t, dg.npv_h2_t);
                    assert_eq!(d0.npv_abated_t, dg.npv_abated_t);
                    assert_eq!(d0.npv_cost_smr_sm, dg.npv_cost_smr_sm);
                    assert_eq!(d0.lcoh_smr, dg.lcoh_smr);
                    // The NUS-H2 side moves by exactly the grant line.
                    close(
                        dg.npv_cost_nush2_sm - d0.npv_cost_nush2_sm,
                        pv_of(&dg, "GRANT"),
                        1e-9,
                        "NPV change is the GRANT line",
                    );
                    // The reported abatement is the same at every share.
                    let e = c.m.subsidy_eval(&c.b, &q0, gq(0.6, d, t), c.a).unwrap();
                    assert_eq!(e.abatement_t_y, c.a);
                }
            }
        }
    }
}

// ----------------------------------------------------------------- S.3a to S.3d

#[test]
fn eq_s_3a_a_grant_at_close_pays_the_earliest_buckets_and_leaves_the_owner_the_rest() {
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let buckets = uniform_spend(k.reactor_capital, c.m.idc_years).unwrap();
    let g = 0.5 * k.reactor_capital;
    let f = funded_balance(c.m.rate, &buckets, g);
    let want = [0.0, 0.0, 0.0, 101.286, 244.438, 399.042];
    assert_eq!(f.rows.len(), want.len());
    for (r, w) in f.rows.iter().zip(want) {
        close(r.balance, w, 5e-4, "funded balance");
        close(r.grant + r.private, r.bucket, 1e-12, "g_k + p_k = b_k");
    }
    // The grant pays years -5 to -3 whole and 33.76 of the bucket at year -2.
    for r in &f.rows[..3] {
        close(r.grant, r.bucket, 1e-12, "bucket paid whole");
    }
    close(f.rows[3].grant, 33.762, 5e-4, "partly paid bucket");
    close(
        f.rows.iter().map(|r| r.grant).sum::<f64>(),
        g,
        1e-9,
        "sum of g_k",
    );
    close(
        f.balance_at_0 - (k.reactor_capital - g),
        27.658,
        5e-4,
        "S.3a interest",
    );
    // The balance at year 0 is the present value of the private payments.
    let pv: f64 = f
        .rows
        .iter()
        .map(|r| r.private * (1.0 + c.m.rate).powi(-r.year))
        .sum();
    close(f.balance_at_0, pv, 1e-9, "balance against private PV");
    // A grant of zero leaves the gross value, a full one leaves nothing.
    let all: f64 = buckets
        .iter()
        .map(|(y, v)| v * (1.0 + c.m.rate).powi(-y))
        .sum();
    close(
        funded_balance(c.m.rate, &buckets, 0.0).balance_at_0,
        all,
        1e-9,
        "no grant",
    );
    close(
        funded_balance(c.m.rate, &buckets, k.reactor_capital).balance_at_0,
        0.0,
        1e-12,
        "full grant",
    );
}

#[test]
fn eq_s_3b_the_closed_form_equals_the_balance_recursion() {
    let c = ctx();
    let (i, tc) = (c.m.rate, c.m.idc_years);
    for stage in Stage::ALL {
        let c_r = cr_registry(c, stage);
        let buckets = uniform_spend(c_r, tc).unwrap();
        for n in 0..=200 {
            let g = c_r * f64::from(n) / 200.0;
            let loopv = funded_balance(i, &buckets, g).balance_at_0;
            let closed = private_pv_closed_form(i, tc, c_r, g);
            close(closed, loopv, 1e-9 * c_r, "S.3b against S.3a");
            close(
                closed,
                pd_account(i, tc, c_r, g),
                1e-9 * c_r,
                "S.3b against the account",
            );
        }
    }
    // The worked case: BOAK 25 y, half of the reactor.
    let c_r = cr_registry(c, Stage::Boak);
    close(
        private_pv_closed_form(i, tc, c_r, 0.5 * c_r),
        399.0417,
        5e-4,
        "S.3b number",
    );
    // Beyond the reactor the owner owes nothing.
    assert_eq!(private_pv_closed_form(i, tc, c_r, 1.2 * c_r), 0.0);
}

#[test]
fn eq_s_3c_the_ledger_form_at_close_and_its_limits() {
    let c = ctx();
    let (i, tc) = (c.m.rate, c.m.idc_years);
    close(
        private_factor_at_close(i, tc, 0.0),
        f_idc(i, tc),
        1e-14,
        "h(0) = f_IDC",
    );
    assert_eq!(private_factor_at_close(i, tc, 1.0), 0.0);
    close(
        private_factor_at_close(i, tc, 0.5),
        0.53569657,
        1e-8,
        "h(0.5)",
    );
    let q = query(c, Stage::Boak, 25.0);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let p =
        c.m.grant_plan(&k, &gq(0.5, Definition::A, Timing::AtClose))
            .unwrap();
    close(p.ledger_interest_private, 26.514, 5e-4, "IDC(2) at close");
    // Step 7: the smooth form dips below zero when the private window is
    // shorter than one year, with its minimum at u_min.
    let ln1 = (1.0 + i).ln();
    let u_min = 1.0 - (i / ln1).ln() / (tc * ln1);
    close(u_min, 0.90851, 1e-5, "u_min");
    let pm =
        c.m.grant_plan(&k, &gq(u_min, Definition::A, Timing::AtClose))
            .unwrap();
    close(
        pm.ledger_interest_private / k.reactor_capital,
        -0.0017490,
        1e-6,
        "IDC(2) minimum",
    );
    let neg = |s: f64| {
        c.m.grant_plan(&k, &gq(s, Definition::A, Timing::AtClose))
            .unwrap()
            .ledger_interest_private
    };
    assert!(neg(0.80) > 0.0 && neg(0.85) < 0.0);
}

#[test]
fn eq_s_3d_idle_cash_earning_the_rate_gives_the_lowest_break_even_share() {
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let s0 =
        c.m.idle_cash_bound_share(&c.b, &q, Definition::A, c.a, c.m.target_c)
            .unwrap();
    close(s0, 0.48829, 2e-5, "idle-cash bound, BOAK 25 y");
    // The bound fails the limit at a share of 1: the private value turns negative.
    let (i, tc) = (c.m.rate, c.m.idc_years);
    let f = f_idc(i, tc);
    assert!(f - (1.0 + i).powf(tc) < 0.0);
    close(
        f / (1.0 + i).powf(tc),
        0.7843,
        1e-4,
        "share at which the private value is zero",
    );
}

// ----------------------------------------------------------------- S.4a and S.4b

#[test]
fn eq_s_4a_a_progressive_grant_scales_every_private_payment() {
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    for d in Definition::ALL {
        let x = x_e(c, &k, d);
        for s in [0.25, 0.5, 0.75] {
            let p = c.m.grant_plan(&k, &gq(s, d, Timing::Progressive)).unwrap();
            let buckets = uniform_spend(k.reactor_capital, c.m.idc_years).unwrap();
            for ((y, b), (fy, fv)) in buckets.iter().zip(&p.dcf_flows) {
                assert_eq!(y, fy);
                let add = if *y == 0 { s * x } else { 0.0 };
                close(-fv, s * b + add, 1e-9, "grant flow s*b_k");
            }
            let fd = f_dcf(c.m.rate, c.m.idc_years);
            close(
                p.dcf_private_reactor_pv,
                (1.0 - s) * k.reactor_capital * fd,
                1e-9,
                "P_D progressive",
            );
            close(
                p.dcf_grant_pv,
                s * (k.reactor_capital * fd + x),
                1e-9,
                "PV of the GRANT line",
            );
            let d1 =
                c.m.dcf(&c.b, &with(&q, gq(s, d, Timing::Progressive)))
                    .unwrap();
            close(pv_of(&d1, "GRANT"), -p.dcf_grant_pv, 1e-9, "line PV");
        }
    }
    let p =
        c.m.grant_plan(&k, &gq(0.5, Definition::A, Timing::Progressive))
            .unwrap();
    close(p.dcf_private_reactor_pv, 445.746, 5e-4, "S.4a number");
}

#[test]
fn eq_s_4b_a_progressive_grant_scales_the_ledger_reactor_value() {
    let c = ctx();
    let (i, tc) = (c.m.rate, c.m.idc_years);
    let f = f_idc(i, tc);
    let q = query(c, Stage::Boak, 25.0);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let crl = crf_(i, 25.0);
    for d in Definition::ALL {
        let x = x_e(c, &k, d);
        for s in [0.25, 0.5, 0.75] {
            let p = c.m.grant_plan(&k, &gq(s, d, Timing::Progressive)).unwrap();
            close(
                p.ledger_interest_private,
                (1.0 - s) * k.reactor_capital * (f - 1.0),
                1e-9,
                "(1-s) C_R (f - 1)",
            );
            let led0 = c.m.ledger(&c.b, &q).unwrap().l5();
            let led =
                c.m.ledger(&c.b, &with(&q, gq(s, d, Timing::Progressive)))
                    .unwrap();
            close(
                led0 - led.l5_net(),
                crl * s * (k.reactor_capital * f + x),
                1e-9,
                "annual saving",
            );
        }
    }
    let p =
        c.m.grant_plan(&k, &gq(0.5, Definition::A, Timing::Progressive))
            .unwrap();
    close(p.ledger_interest_private, 73.408, 5e-4, "S.4b number");
}

// ----------------------------------------------------------------- S.5a and S.5b

#[test]
fn eq_s_5a_interest_is_charged_on_the_private_balance_and_the_saving_is_reported() {
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let prog =
        c.m.grant_plan(&k, &gq(0.5, Definition::A, Timing::Progressive))
            .unwrap();
    let close_ =
        c.m.grant_plan(&k, &gq(0.5, Definition::A, Timing::AtClose))
            .unwrap();
    close(prog.ledger_interest_gross, 146.816, 5e-4, "basis 1");
    close(
        prog.ledger_interest_private,
        73.408,
        5e-4,
        "basis 2, progressive",
    );
    close(
        prog.ledger_interest_saved,
        73.408,
        5e-4,
        "basis 3, progressive",
    );
    close(
        close_.ledger_interest_private,
        26.514,
        5e-4,
        "basis 2, at close",
    );
    close(
        close_.ledger_interest_saved,
        120.301,
        5e-4,
        "basis 3, at close",
    );
    for p in [&prog, &close_] {
        close(
            p.ledger_interest_saved,
            p.ledger_interest_gross - p.ledger_interest_private,
            1e-12,
            "IDC(3) = IDC(1) - IDC(2)",
        );
        close(
            p.dcf_interest_saved,
            p.dcf_interest_gross - p.dcf_interest_private,
            1e-12,
            "IDC(3) in the DCF",
        );
    }
    // The grant timing matters: at close the saving is larger.
    assert!(close_.ledger_interest_saved > prog.ledger_interest_saved);
    assert!(close_.dcf_interest_saved > prog.dcf_interest_saved);
    // The grant never reduces the DCF interest below zero.
    for n in 0..=100 {
        let p =
            c.m.grant_plan(
                &k,
                &gq(f64::from(n) / 100.0, Definition::B, Timing::AtClose),
            )
            .unwrap();
        assert!(p.dcf_interest_private >= -1e-9, "DCF interest below zero");
    }
}

#[test]
fn eq_s_5b_the_net_cost_is_the_gross_cost_less_the_grant_and_the_interest_it_saves() {
    let c = ctx();
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            let k = c.m.cost_items(&c.b, &q).unwrap();
            let crl = crf_(c.m.rate, life);
            for d in Definition::ALL {
                for t in Timing::ALL {
                    let p = c.m.grant_plan(&k, &gq(0.4, d, t)).unwrap();
                    let led = c.m.ledger(&c.b, &with(&q, gq(0.4, d, t))).unwrap();
                    let n = led.net.unwrap();
                    close(n.grant_credit, -p.grant * crl, 1e-12, "GRANT_CREDIT");
                    close(
                        n.idc_effect,
                        -p.ledger_interest_saved * crl,
                        1e-12,
                        "IDC_EFFECT",
                    );
                    let gross = c.m.ledger(&c.b, &q).unwrap().l5();
                    close(
                        led.l5_net(),
                        gross - crl * (p.grant + p.ledger_interest_saved),
                        1e-9,
                        "S.5b",
                    );
                    assert!(n.grant_credit < 0.0 && n.idc_effect < 0.0);
                }
            }
        }
    }
    // The size of the wrong ways named in S.5b, BOAK 25 y, half, definition A.
    let c0 = Closed::new(c, Stage::Boak, 25.0, Definition::A, c.a);
    let (f, s) = (f_idc(c0.i, c0.tc), 0.5);
    let w1 = c0.kappa * s * c0.c_r * (f - 1.0);
    close(w1, 11.896, 1e-3, "W1 and W2");
    close(w1 / c0.lac5_0, 0.0627, 5e-4, "W1 share of LAC5(0)");
    let w3 = c0.lac5(s, Timing::Progressive) - c0.lac5(s, Timing::AtClose);
    close(w3, 7.600, 1e-3, "W3");
    let w4 = c0.kappa * s * c0.c_r * f * ((1.0 + c0.i).powf(c0.tc) - 1.0);
    close(w4, 37.985, 1e-3, "W4");
    // The engine's at-close cost is none of the wrong values.
    let q = query(c, Stage::Boak, 25.0);
    let e =
        c.m.subsidy_eval(&c.b, &q, gq(s, Definition::A, Timing::AtClose), c.a)
            .unwrap();
    for wrong in [
        c0.lac5_0 - w1,
        c0.lac5(s, Timing::Progressive),
        c0.lac5_0 - w4,
    ] {
        assert!((e.l5.lac_net - wrong).abs() > 5.0);
    }
}

// ----------------------------------------------------------------- S.6a and S.6b

#[test]
fn eq_s_6a_ledger_cost_follows_the_closed_form_for_each_timing() {
    let c = ctx();
    for stage in Stage::ALL {
        for life in LIVES {
            for d in Definition::ALL {
                let cl = Closed::new(c, stage, life, d, c.a);
                let q = query(c, stage, life);
                for t in Timing::ALL {
                    for s in [0.0, 0.1, 0.25, 1.0 / 3.0, 0.5, 0.6, 0.75, 0.9, 1.0] {
                        let e = c.m.subsidy_eval(&c.b, &q, gq(s, d, t), c.a).unwrap();
                        close(e.l5.lac_net, cl.lac5(s, t), 1e-9, "S.6a");
                    }
                }
            }
        }
    }
    // BOAK 25 y, definition A: the table of S.6a.
    let q = query(c, Stage::Boak, 25.0);
    let want = [
        (0.25, 153.7081, 147.8023),
        (0.5, 117.6668, 110.0672),
        (0.75, 81.6254, 76.1213),
    ];
    for (s, prog, at_close) in want {
        let p =
            c.m.subsidy_eval(&c.b, &q, gq(s, Definition::A, Timing::Progressive), c.a)
                .unwrap();
        let k =
            c.m.subsidy_eval(&c.b, &q, gq(s, Definition::A, Timing::AtClose), c.a)
                .unwrap();
        close(p.l5.lac_net, prog, 5e-5, "S.6a table, progressive");
        close(k.l5.lac_net, at_close, 5e-5, "S.6a table, at close");
    }
}

#[test]
fn eq_s_6b_dcf_cost_follows_the_closed_form_for_each_timing() {
    let c = ctx();
    for stage in Stage::ALL {
        for life in LIVES {
            for d in Definition::ALL {
                let cl = Closed::new(c, stage, life, d, c.a);
                let q = query(c, stage, life);
                for t in Timing::ALL {
                    for s in [0.0, 0.1, 0.25, 1.0 / 3.0, 0.5, 0.6, 0.75, 0.9, 1.0] {
                        let e = c.m.subsidy_eval(&c.b, &q, gq(s, d, t), c.a).unwrap();
                        close(e.dcf.lac_net, cl.lacd(s, t), 1e-9, "S.6b");
                    }
                }
            }
        }
    }
    let q = query(c, Stage::Boak, 25.0);
    let want = [
        (0.25, 153.9401, 147.9264),
        (0.5, 117.8214, 110.2526),
        (0.75, 81.7028, 76.3338),
    ];
    for (s, prog, at_close) in want {
        let p =
            c.m.subsidy_eval(&c.b, &q, gq(s, Definition::A, Timing::Progressive), c.a)
                .unwrap();
        let k =
            c.m.subsidy_eval(&c.b, &q, gq(s, Definition::A, Timing::AtClose), c.a)
                .unwrap();
        close(p.dcf.lac_net, prog, 5e-5, "S.6b table, progressive");
        close(k.dcf.lac_net, at_close, 5e-5, "S.6b table, at close");
    }
}

#[test]
fn eq_s_6a_derivation_tables_for_definition_b_follow_from_its_membership() {
    let c = ctx_s1();
    let q = query(c, Stage::Boak, 25.0);
    let want = [
        // share, L5 progressive, L5 at close, DCF progressive, DCF at close
        (0.25, 146.7449, 138.7124, 146.9769, 138.8790),
        (0.5, 103.7403, 93.9107, 103.8949, 94.0087),
        (0.75, 60.7356, 54.5826, 60.8130, 54.7874),
    ];
    for (s, l5p, l5c, dp, dc) in want {
        let p =
            c.m.subsidy_eval(&c.b, &q, gq(s, Definition::B, Timing::Progressive), c.a)
                .unwrap();
        let k =
            c.m.subsidy_eval(&c.b, &q, gq(s, Definition::B, Timing::AtClose), c.a)
                .unwrap();
        close(p.l5.lac_net, l5p, 5e-5, "B L5 progressive");
        close(k.l5.lac_net, l5c, 5e-5, "B L5 at close");
        close(p.dcf.lac_net, dp, 5e-5, "B DCF progressive");
        close(k.dcf.lac_net, dc, 5e-5, "B DCF at close");
    }
}

// ------------------------------------------------------------------------ S.7x

#[test]
fn eq_s_7a_progressive_cost_is_affine_and_the_cost_at_close_bends() {
    let c = ctx();
    for stage in Stage::ALL {
        let q = query(c, stage, 25.0);
        for d in Definition::ALL {
            let cl = Closed::new(c, stage, 25.0, d, c.a);
            let f = |s: f64, t: Timing| c.m.subsidy_eval(&c.b, &q, gq(s, d, t), c.a).unwrap();
            let second = |t: Timing, pick: &dyn Fn(&nush2_plant::subsidy::SubsidyEval) -> f64| {
                pick(&f(0.25, t)) - 2.0 * pick(&f(0.5, t)) + pick(&f(0.75, t))
            };
            assert!(second(Timing::Progressive, &|e| e.l5.lac_net).abs() < 1e-9);
            assert!(second(Timing::Progressive, &|e| e.dcf.lac_net).abs() < 1e-9);
            let bend = second(Timing::AtClose, &|e| e.l5.lac_net);
            assert!(
                bend > if d == Definition::A { 0.5 } else { 0.0 },
                "bend {bend}"
            );
            let m5 = cl.kappa * (cl.c_r * f_idc(cl.i, cl.tc) + cl.x_e);
            let md = cl.kappa * (cl.c_r * f_dcf(cl.i, cl.tc) + cl.x_e);
            let e1 = f(1.0, Timing::Progressive);
            close(cl.lac5_0 - e1.l5.lac_net, m5, 1e-9, "slope M5");
            close(cl.lacd_0 - e1.dcf.lac_net, md, 1e-9, "slope MD");
        }
    }
    // BOAK 25 y, definition A: the slopes and the second difference of S.7a.
    let c0 = Closed::new(c, Stage::Boak, 25.0, Definition::A, c.a);
    close(
        c0.kappa * c0.c_r * f_idc(c0.i, c0.tc),
        144.16543,
        1e-4,
        "M5",
    );
    close(
        c0.kappa * c0.c_r * f_dcf(c0.i, c0.tc),
        144.47475,
        1e-4,
        "MD",
    );
    close(
        c0.lac5(0.25, Timing::AtClose) - 2.0 * c0.lac5(0.5, Timing::AtClose)
            + c0.lac5(0.75, Timing::AtClose),
        3.79,
        1e-2,
        "second difference at close",
    );
}

/// The share at which the cost curve of `eval` crosses `target`, bracketed on
/// the model's own output.
fn bisect_model(
    c: &Ctx,
    q: &CostQuery,
    d: Definition,
    t: Timing,
    dcf: bool,
    target: f64,
    a: f64,
) -> f64 {
    bisect_share(
        |s| {
            let e = c.m.subsidy_eval_at(&c.b, q, gq(s, d, t), a, target)?;
            Ok(if dcf { e.dcf.lac_net } else { e.l5.lac_net })
        },
        target,
        0.0,
        1.0,
        1e-12,
    )
    .unwrap()
}

fn share_of(r: &nush2_plant::subsidy::BreakEvenSolve) -> f64 {
    r.result.share().expect("a share in range")
}

#[test]
fn eq_s_7b_break_even_for_a_progressive_grant() {
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let be =
        c.m.breakeven(&c.b, &q, Definition::A, Timing::Progressive, c.a)
            .unwrap();
    close(share_of(&be.l5), 0.62255, 2e-5, "L5 break-even, BOAK 25 y");
    close(
        share_of(&be.dcf),
        0.62335,
        2e-5,
        "DCF break-even, BOAK 25 y",
    );
    assert_eq!(be.l5.route, SolveRoute::ClosedForm);
    assert_eq!(be.dcf.route, SolveRoute::ClosedForm);
    // The allowance form: (dC5(0) - T A 1e-6) / (CRF (C_R f + X_E)).
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let d_c5 = c.m.ledger(&c.b, &q).unwrap().l5();
    let (i, tc) = (c.m.rate, c.m.idc_years);
    let allow =
        (d_c5 - c.m.target_c * c.a * 1e-6) / (crf_(i, 25.0) * k.reactor_capital * f_idc(i, tc));
    close(allow, share_of(&be.l5), 1e-9, "S.7b allowance form");
    // The strict test: below the break-even fails, above passes.
    let at = |s: f64| {
        c.m.subsidy_eval(&c.b, &q, gq(s, Definition::A, Timing::Progressive), c.a)
            .unwrap()
    };
    let s = share_of(&be.l5);
    assert!(at(s + 1e-6).l5.lac_net < c.m.target_c);
    assert!(at(s - 1e-6).l5.lac_net > c.m.target_c);
    assert!(be.both.share().unwrap() >= share_of(&be.dcf));
}

#[test]
fn eq_s_7c_break_even_at_close_in_both_regimes_of_the_ledger() {
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let be =
        c.m.breakeven(&c.b, &q, Definition::A, Timing::AtClose, c.a)
            .unwrap();
    close(share_of(&be.l5), 0.57142, 2e-5, "L5 break-even at close");
    // For definition B with the S.1 membership the share scales by C_R / E.
    let c1 = ctx_s1();
    let be1 =
        c1.m.breakeven(&c1.b, &q, Definition::B, Timing::AtClose, c1.a)
            .unwrap();
    close(share_of(&be1.l5), 0.46405, 2e-5, "B at close, BOAK 25 y");
    // The reactor regime (H* >= 0) and the second regime (H* < 0): shrinking
    // the abatement raises the cost and pushes the root past the reactor.
    let k = c1.m.cost_items(&c1.b, &q).unwrap();
    let mut regimes = [false, false];
    for scale in [1.0, 0.8, 0.6, 0.5, 0.45, 0.4] {
        let a = c1.a * scale;
        let r =
            c1.m.breakeven(&c1.b, &q, Definition::B, Timing::AtClose, a)
                .unwrap();
        let Some(s) = r.l5.result.share() else {
            continue;
        };
        if matches!(r.l5.result, BreakEven::Unreachable { .. }) {
            continue;
        }
        let want = bisect_model(
            c1,
            &q,
            Definition::B,
            Timing::AtClose,
            false,
            c1.m.target_c,
            a,
        );
        close(s, want, 1e-10, "closed form against bisection");
        assert_eq!(r.l5.route, SolveRoute::ClosedForm, "scale {scale}");
        let past_reactor = s * (k.reactor_capital + x_e(c1, &k, Definition::B)) > k.reactor_capital;
        regimes[usize::from(past_reactor)] = true;
    }
    assert_eq!(regimes, [true, true], "both regimes of S.7c are exercised");
}

#[test]
fn eq_s_7d_the_exact_route_and_the_bisection_agree_and_the_orderings_hold() {
    // Break-even shares of the derivation table, legacy abatement: A on the
    // D7b registry, B on the S.1 membership. Columns: A progressive L5 and
    // DCF, A at close L5 and DCF, B progressive L5 and DCF, B at close L5 and
    // DCF, idle-cash bound (A, L5).
    let table: [(Stage, f64, [f64; 9]); 8] = [
        (
            Stage::Foak,
            25.0,
            [
                0.77138, 0.77187, 0.73126, 0.73301, 0.68382, 0.68442, 0.63403, 0.63555, 0.60502,
            ],
        ),
        (
            Stage::Foak,
            60.0,
            [
                0.71223, 0.71317, 0.66643, 0.66778, 0.63138, 0.63238, 0.57782, 0.57899, 0.55863,
            ],
        ),
        (
            Stage::Boak,
            25.0,
            [
                0.62255, 0.62335, 0.57142, 0.57302, 0.52174, 0.52260, 0.46405, 0.46535, 0.48829,
            ],
        ),
        (
            Stage::Boak,
            60.0,
            [
                0.52790, 0.52944, 0.47513, 0.47638, 0.44242, 0.44387, 0.38585, 0.38687, 0.41405,
            ],
        ),
        (
            Stage::Oak10FromBoak,
            25.0,
            [
                0.45622, 0.45738, 0.40473, 0.40611, 0.36036, 0.36144, 0.30694, 0.30799, 0.35783,
            ],
        ),
        (
            Stage::Oak10FromBoak,
            60.0,
            [
                0.32191, 0.32411, 0.27821, 0.27913, 0.25427, 0.25612, 0.21099, 0.21169, 0.25248,
            ],
        ),
        (
            Stage::Oak10FromFoak,
            25.0,
            [
                0.66743, 0.66814, 0.61849, 0.61911, 0.56872, 0.56951, 0.51206, 0.51257, 0.52349,
            ],
        ),
        (
            Stage::Oak10FromFoak,
            60.0,
            [
                0.58348, 0.58484, 0.53120, 0.53340, 0.49719, 0.49851, 0.43979, 0.44161, 0.45765,
            ],
        ),
    ];
    let (ca, cb) = (ctx(), ctx_s1());
    for (stage, life, w) in table {
        let q = query(ca, stage, life);
        let sets = [
            (ca, Definition::A, Timing::Progressive, w[0], w[1]),
            (ca, Definition::A, Timing::AtClose, w[2], w[3]),
            (cb, Definition::B, Timing::Progressive, w[4], w[5]),
            (cb, Definition::B, Timing::AtClose, w[6], w[7]),
        ];
        for (cx, d, t, want5, wantd) in sets {
            let be = cx.m.breakeven(&cx.b, &q, d, t, cx.a).unwrap();
            let (s5, sd) = (share_of(&be.l5), share_of(&be.dcf));
            close(s5, want5, 2e-5, "derivation table, L5");
            close(sd, wantd, 2e-5, "derivation table, DCF");
            // The exact route and the bisection on the model agree.
            close(
                s5,
                bisect_model(cx, &q, d, t, false, cx.m.target_c, cx.a),
                1e-10,
                "L5 bisection",
            );
            close(
                sd,
                bisect_model(cx, &q, d, t, true, cx.m.target_c, cx.a),
                1e-10,
                "DCF bisection",
            );
            assert_eq!(be.l5.route, SolveRoute::ClosedForm);
            assert_eq!(be.dcf.route, SolveRoute::ClosedForm);
            assert!(be.l5.residual.abs() <= 1e-9 && be.dcf.residual.abs() <= 1e-9);
            // The overall share is the larger one, and the DCF share is the larger here.
            close(be.both.share().unwrap(), s5.max(sd), 0.0, "both");
        }
        // Progressive above at close above the idle-cash bound.
        let idle =
            ca.m.idle_cash_bound_share(&ca.b, &q, Definition::A, ca.a, ca.m.target_c)
                .unwrap();
        close(idle, w[8], 2e-5, "idle-cash bound");
        for d in Definition::ALL {
            let cx = if d == Definition::A { ca } else { cb };
            let prog =
                cx.m.breakeven(&cx.b, &q, d, Timing::Progressive, cx.a)
                    .unwrap();
            let at = cx.m.breakeven(&cx.b, &q, d, Timing::AtClose, cx.a).unwrap();
            assert!(share_of(&prog.l5) > share_of(&at.l5));
            assert!(share_of(&prog.dcf) > share_of(&at.dcf));
        }
        assert!(
            idle < share_of(
                &ca.m
                    .breakeven(&ca.b, &q, Definition::A, Timing::AtClose, ca.a)
                    .unwrap()
                    .l5
            )
        );
    }
}

#[test]
fn eq_s_7e_cases_of_the_break_even_share_and_the_strict_target_flag() {
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let t = c.m.target_c;
    let prog = |s: f64, a: f64, target: f64| {
        c.m.subsidy_eval_at(
            &c.b,
            &q,
            gq(s, Definition::A, Timing::Progressive),
            a,
            target,
        )
        .unwrap()
    };
    // Twice the abatement: met without a grant, every grid share passes.
    let be =
        c.m.breakeven(&c.b, &q, Definition::A, Timing::Progressive, 2.0 * c.a)
            .unwrap();
    assert_eq!(be.l5.result, BreakEven::MetWithoutGrant);
    assert_eq!(be.both, BreakEven::MetWithoutGrant);
    assert_eq!(be.both.share(), Some(0.0));
    for s in c.m.subsidy.grid {
        assert!(prog(s, 2.0 * c.a, t).target_met);
    }
    // As it stands: 0.6225, and only the 75 percent share of the grid passes.
    let flags: Vec<bool> =
        c.m.subsidy
            .grid
            .iter()
            .map(|s| prog(*s, c.a, t).target_met)
            .collect();
    assert_eq!(flags, [false, false, false, true]);
    // FOAK 25 y: the break-even lies beyond the grid, so no grid share passes.
    let qf = query(c, Stage::Foak, 25.0);
    let bef =
        c.m.breakeven(&c.b, &qf, Definition::A, Timing::Progressive, c.a)
            .unwrap();
    let sf = share_of(&bef.l5);
    assert!(sf > 0.75 && sf < 1.0);
    for s in c.m.subsidy.grid {
        let e =
            c.m.subsidy_eval(&c.b, &qf, gq(s, Definition::A, Timing::Progressive), c.a)
                .unwrap();
        assert!(!e.target_met);
    }
    let e =
        c.m.subsidy_eval(
            &c.b,
            &qf,
            gq(sf + 1e-6, Definition::A, Timing::Progressive),
            c.a,
        )
        .unwrap();
    assert!(e.l5.target_met);
    // 0.4 times the abatement: unreachable, and the cost at the largest share is reported.
    let low =
        c.m.breakeven(&c.b, &q, Definition::A, Timing::Progressive, 0.4 * c.a)
            .unwrap();
    match low.l5.result {
        BreakEven::Unreachable { lac_at_max } => {
            let e = prog(1.0, 0.4 * c.a, t);
            close(lac_at_max, e.l5.lac_net, 1e-9, "cost at share 1");
            assert!(lac_at_max >= t);
        }
        other => panic!("expected unreachable, got {other:?}"),
    }
    assert!(low.both.share().is_none());
    // A cost exactly at the target fails the strict test. The break-even is
    // then 0 and every larger share passes.
    let lac0 = prog(0.0, c.a, t).l5.lac_net;
    let at_target = prog(0.0, c.a, lac0);
    assert!(
        !at_target.l5.target_met,
        "a cost equal to the target is not met"
    );
    assert!(
        prog(0.0, c.a, f64::from_bits(lac0.to_bits() + 1))
            .l5
            .target_met
    );
    let be0 =
        c.m.breakeven_at(&c.b, &q, Definition::A, Timing::Progressive, c.a, lac0)
            .unwrap();
    assert_eq!(be0.l5.result, BreakEven::Share(0.0));
    assert!(prog(1e-9, c.a, lac0).l5.target_met);
    // The overall flag needs both methods.
    let e = prog(0.5, c.a, t);
    assert_eq!(e.target_met, e.l5.target_met && e.dcf.target_met);
    let in_between = (e.l5.lac_net + e.dcf.lac_net) / 2.0;
    let split = prog(0.5, c.a, in_between);
    assert!(split.l5.target_met != split.dcf.target_met && !split.target_met);
    // The printed share is rounded up and passes.
    let s = share_of(
        &c.m.breakeven(&c.b, &q, Definition::A, Timing::Progressive, c.a)
            .unwrap()
            .l5,
    );
    let up = round_up_share(s, 6);
    assert!(up >= s && up - s < 1e-6);
    assert!(prog(up, c.a, t).l5.target_met);
    assert_eq!(round_up_share(0.5, 6), 0.5);
    assert_eq!(round_up_share(0.1234561, 6), 0.123457);
    // combine keeps the larger share and lets an unreachable result win.
    assert_eq!(
        combine(BreakEven::Share(0.3), BreakEven::Share(0.4)),
        BreakEven::Share(0.4)
    );
    assert_eq!(
        combine(BreakEven::MetWithoutGrant, BreakEven::Share(0.2)),
        BreakEven::Share(0.2)
    );
    assert!(matches!(
        combine(
            BreakEven::Share(0.9),
            BreakEven::Unreachable { lac_at_max: 120.0 }
        ),
        BreakEven::Unreachable { .. }
    ));
}

// -------------------------------------------------------------------- S.8 to S.9

#[test]
fn eq_s_8_reported_quantities_are_consistent() {
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let a = c.a;
    for d in Definition::ALL {
        for t in Timing::ALL {
            let e = c.m.subsidy_eval(&c.b, &q, gq(0.5, d, t), a).unwrap();
            close(
                e.l5.cost_net_sm_y,
                e.l5.lac_net * a * 1e-6,
                1e-9,
                "dC5(s) = LAC5(s) A",
            );
            close(
                e.dcf.cost_net_sm_y,
                e.dcf.lac_net * a * 1e-6,
                1e-9,
                "annual DCF cost",
            );
            close(e.l5.cost_gross_sm_y, 109.685004, 1e-5, "gross annual cost");
            let crl = crf_(c.m.rate, 25.0);
            close(
                e.l5.public_outlay_s_per_t,
                e.l5.state_pv_sm * crl * 1e6 / a,
                1e-9,
                "public outlay per tonne",
            );
            if t == Timing::Progressive {
                // A progressive grant is worth to the state what it saves the owner.
                close(
                    e.l5.public_outlay_s_per_t,
                    e.l5.lac_gross - e.l5.lac_net,
                    1e-9,
                    "progressive outlay equals the private saving",
                );
                close(
                    e.dcf.public_outlay_s_per_t,
                    e.dcf.lac_gross - e.dcf.lac_net,
                    1e-9,
                    "progressive DCF outlay equals the private saving",
                );
            } else {
                assert!(e.dcf.public_outlay_s_per_t > e.dcf.lac_gross - e.dcf.lac_net);
            }
        }
    }
    let e =
        c.m.subsidy_eval(&c.b, &q, gq(0.5, Definition::A, Timing::AtClose), a)
            .unwrap();
    let p = &e.plan;
    close(
        p.total_capital + p.ledger_interest_gross,
        1209.043,
        5e-4,
        "total with interest",
    );
    close(
        e.l5.cost_net_sm_y,
        63.625,
        5e-4,
        "A at close net annual cost",
    );
    // The state pays 567.092 for what is worth 492.450 to the owner: the carry on idle cash.
    close(p.dcf_state_pv, 567.092, 5e-4, "state-side PV at close");
    let kappa = crf_(c.m.rate, 25.0) * 1e6 / a;
    close(
        (e.dcf.lac_gross - e.dcf.lac_net) / kappa,
        492.450,
        5e-4,
        "owner-side PV at close",
    );
    let ep =
        c.m.subsidy_eval(&c.b, &q, gq(0.5, Definition::A, Timing::Progressive), a)
            .unwrap();
    close(
        ep.l5.cost_net_sm_y,
        68.017,
        5e-4,
        "A progressive net annual cost",
    );
    // Definition B on the S.1 membership: the derivation's capital and cost rows.
    let c1 = ctx_s1();
    let eb =
        c1.m.subsidy_eval(&c1.b, &q, gq(0.5, Definition::B, Timing::AtClose), c1.a)
            .unwrap();
    close(eb.plan.grant, 457.318, 5e-4, "G, definition B");
    close(
        eb.plan.private_capital,
        604.909,
        5e-4,
        "K_priv, definition B",
    );
    close(
        eb.l5.cost_net_sm_y,
        54.285,
        5e-4,
        "B at close net annual cost",
    );
    assert!(eb.l5.target_met && eb.dcf.target_met && eb.target_met);
    // Only the 50 percent share on B at close passes both tests.
    assert!(!ep.target_met && !e.target_met);
}

#[test]
fn eq_s_9a_the_cost_falls_strictly_with_the_share_and_bends_one_way() {
    let c = ctx();
    let slope_ratio = (1.0 + c.m.rate).ln() / c.m.rate;
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            for d in Definition::ALL {
                for t in Timing::ALL {
                    let vals: Vec<(f64, f64)> = (0..=200)
                        .map(|n| {
                            let s = f64::from(n) / 200.0;
                            let e = c.m.subsidy_eval(&c.b, &q, gq(s, d, t), c.a).unwrap();
                            (e.l5.lac_net, e.dcf.lac_net)
                        })
                        .collect();
                    let k = c.m.cost_items(&c.b, &q).unwrap();
                    let share_reactor =
                        k.reactor_capital / c.m.grant_plan(&k, &gq(1.0, d, t)).unwrap().eligible;
                    for (n, w) in vals.windows(2).enumerate() {
                        assert!(w[1].0 < w[0].0 && w[1].1 < w[0].1, "not strictly falling");
                        if n >= 1 {
                            let (a, b, cc) = (vals[n - 1], vals[n], vals[n + 1]);
                            let s_mid = n as f64 / 200.0;
                            let second5 = a.0 - 2.0 * b.0 + cc.0;
                            let secondd = a.1 - 2.0 * b.1 + cc.1;
                            // Convex everywhere except the ledger kink of definition B past the reactor.
                            let past = d == Definition::B
                                && t == Timing::AtClose
                                && (s_mid - share_reactor).abs() < 0.0075
                                && share_reactor < 1.0;
                            assert!(secondd >= -1e-9, "DCF not convex at {s_mid}");
                            if !past {
                                assert!(second5 >= -1e-9, "ledger not convex at {s_mid}");
                            }
                        }
                    }
                }
            }
        }
    }
    // The kink of the ledger at close for definition B, past the reactor: the
    // slope changes by the factor ln(1+i)/i (S.9a, step 3).
    let q = query(c, Stage::Boak, 25.0);
    let k = c.m.cost_items(&c.b, &q).unwrap();
    let e_b =
        c.m.grant_plan(&k, &gq(1.0, Definition::B, Timing::AtClose))
            .unwrap()
            .eligible;
    let sk = k.reactor_capital / e_b;
    let lac = |s: f64| {
        c.m.subsidy_eval(&c.b, &q, gq(s, Definition::B, Timing::AtClose), c.a)
            .unwrap()
            .l5
            .lac_net
    };
    let dl = 1e-7;
    let before = (lac(sk - dl) - lac(sk - 2.0 * dl)) / dl;
    let after = (lac(sk + 2.0 * dl) - lac(sk + dl)) / dl;
    close(before / after, slope_ratio, 1e-4, "slope ratio at the kink");
}

#[test]
fn eq_s_9b_limiting_cases_at_zero_and_full_share() {
    let c = ctx();
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            let led = c.m.ledger(&c.b, &q).unwrap();
            let k = c.m.cost_items(&c.b, &q).unwrap();
            let (i, tc) = (c.m.rate, c.m.idc_years);
            let kappa = crf_(i, life) * 1e6 / c.a;
            for t in Timing::ALL {
                let e0 =
                    c.m.subsidy_eval(&c.b, &q, gq(0.0, Definition::A, t), c.a)
                        .unwrap();
                assert_eq!(e0.l5.lac_net, e0.l5.lac_gross);
                // s = 1 on definition A removes the reactor capital and its interest.
                let e1 =
                    c.m.subsidy_eval(&c.b, &q, gq(1.0, Definition::A, t), c.a)
                        .unwrap();
                let removed = (led.reactor_capital + led.idc) * 1e6 / c.a;
                close(
                    e1.l5.lac_gross - e1.l5.lac_net,
                    removed,
                    1e-9,
                    "S.9b ledger, s = 1",
                );
                close(
                    e1.dcf.lac_gross - e1.dcf.lac_net,
                    kappa * k.reactor_capital * f_dcf(i, tc),
                    1e-9,
                    "S.9b DCF, s = 1",
                );
                assert_eq!(
                    e1.plan.private_capital,
                    e1.plan.total_capital - k.reactor_capital
                );
                // The private reactor value is nil.
                close(e1.plan.dcf_private_reactor_pv, 0.0, 1e-9, "P_D at s = 1");
                close(
                    e1.plan.ledger_interest_private,
                    0.0,
                    1e-9,
                    "IDC(2) at s = 1",
                );
                // Definition B at s = 1 also removes the other eligible first units.
                let eb =
                    c.m.subsidy_eval(&c.b, &q, gq(1.0, Definition::B, t), c.a)
                        .unwrap();
                let extra = eb.l5.lac_gross - eb.l5.lac_net - removed;
                close(extra, kappa * x_e(c, &k, Definition::B), 1e-9, "X_E credit");
            }
        }
    }
}

#[test]
fn eq_s_9c_the_grant_effect_by_four_routes_that_share_no_code_with_the_engine() {
    let c = ctx();
    let (i, tc) = (c.m.rate, c.m.idc_years);
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            let k = c.m.cost_items(&c.b, &q).unwrap();
            close(
                k.reactor_capital,
                cr_registry(c, stage),
                1e-9,
                "C_R from the registry",
            );
            let led = c.m.ledger(&c.b, &q).unwrap();
            let d0 = c.m.dcf(&c.b, &q).unwrap();
            let crl = crf_(i, life);
            for d in Definition::ALL {
                let cl = Closed::new(c, stage, life, d, c.a);
                for t in Timing::ALL {
                    for s in [0.25, 0.5, 0.75, 0.9] {
                        let e = c.m.subsidy_eval(&c.b, &q, gq(s, d, t), c.a).unwrap();
                        let (c_r, x) = (cl.c_r, cl.x_e);
                        let g = s * (c_r + x);
                        // Route 1, closed form.
                        close(e.l5.lac_net, cl.lac5(s, t), 1e-9, "route 1, ledger");
                        close(e.dcf.lac_net, cl.lacd(s, t), 1e-9, "route 1, DCF");
                        // Route 2, ledger lines rebuilt from the private capital.
                        let (g_r, g_x) = match t {
                            Timing::Progressive => (s * c_r, s * x),
                            Timing::AtClose => (g.min(c_r), (g - c_r).max(0.0)),
                        };
                        let private_value = match t {
                            Timing::Progressive => (1.0 - s) * c_r * f_idc(i, tc),
                            Timing::AtClose => c_r * h_close(i, tc, g_r / c_r),
                        };
                        let kept: f64 = led
                            .lines()
                            .iter()
                            .filter(|(id, _, _)| !matches!(*id, "REACTOR_CAP" | "IDC"))
                            .map(|(_, _, v)| *v)
                            .sum();
                        let rebuilt = kept + crl * private_value - crl * g_x;
                        close(rebuilt * 1e6 / c.a, e.l5.lac_net, 1e-9, "route 2");
                        // Route 3, dated cash flows with the grant flows added.
                        let mut flows: Vec<(i32, f64)> = Vec::new();
                        let buckets = uniform_spend(c_r, tc).unwrap();
                        match t {
                            Timing::Progressive => {
                                flows.extend(buckets.iter().map(|(y, b)| (*y, -s * b)));
                            }
                            Timing::AtClose => {
                                let mut spent = 0.0;
                                for (y, b) in &buckets {
                                    let paid = b.min((g_r - spent).max(0.0));
                                    flows.push((*y, -paid));
                                    spent += b;
                                }
                            }
                        }
                        flows.push((0, -g_x));
                        let grant_pv: f64 = flows.iter().map(|(y, v)| v * (1.0 + i).powi(-y)).sum();
                        let alpha = (1..=life as i32).map(|y| (1.0 + i).powi(-y)).sum::<f64>();
                        let lac_d = d0.lac + grant_pv * 1e6 / (c.a * alpha);
                        close(lac_d, e.dcf.lac_net, 1e-9, "route 3");
                        // Route 4, the owner's account year by year.
                        let pd = match t {
                            Timing::Progressive => (1.0 - s) * c_r * f_dcf(i, tc),
                            Timing::AtClose => pd_account(i, tc, c_r, g_r),
                        };
                        let removed = c_r * f_dcf(i, tc) - pd + g_x;
                        close(
                            d0.lac - removed * 1e6 / (c.a * alpha),
                            e.dcf.lac_net,
                            1e-9,
                            "route 4",
                        );
                    }
                }
            }
        }
    }
    // The printed check of S.9c: BOAK 25 y, definition A, half the reactor.
    let c0 = Closed::new(c, Stage::Boak, 25.0, Definition::A, c.a);
    close(
        c0.lac5(0.5, Timing::Progressive),
        117.6668,
        5e-5,
        "117.6668",
    );
    close(c0.lac5(0.5, Timing::AtClose), 110.0672, 5e-5, "110.0672");
    close(
        c0.lacd(0.5, Timing::Progressive),
        117.8214,
        5e-5,
        "117.8214",
    );
    close(c0.lacd(0.5, Timing::AtClose), 110.2526, 5e-5, "110.2526");
}

#[test]
fn eq_s_9c_the_gap_between_the_methods_equals_the_prediction_and_stays_below_the_limit() {
    let c = ctx();
    let mut worst = 0.0f64;
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            for d in Definition::ALL {
                let cl = Closed::new(c, stage, life, d, c.a);
                let gap0 = cl.lacd_0 - cl.lac5_0;
                for t in Timing::ALL {
                    for n in 0..=20 {
                        let s = f64::from(n) / 20.0;
                        let e = c.m.subsidy_eval(&c.b, &q, gq(s, d, t), c.a).unwrap();
                        let predicted = cl.lacd(s, t) - cl.lac5(s, t);
                        close(e.gap_s_per_t, predicted, 1e-9, "predicted gap");
                        // Progressive: the reactor term scales by (1 - s).
                        if t == Timing::Progressive {
                            let law = gap0
                                - s * cl.kappa * cl.c_r * (f_dcf(cl.i, cl.tc) - f_idc(cl.i, cl.tc));
                            close(e.gap_s_per_t, law, 1e-9, "gap law, progressive");
                        } else {
                            // At close the gap stays between zero and its zero-subsidy value
                            // when the lives carry no replaced units.
                            if life == 25.0 {
                                assert!(e.gap_s_per_t >= -1e-9 && e.gap_s_per_t <= gap0 + 1e-9);
                            }
                        }
                        // The relative limit is stated against a positive net cost, so
                        // it is tested up to the top of the share grid.
                        if s <= 0.75 {
                            assert!(e.l5.lac_net > 0.0);
                            assert!(e.gap_rel.abs() < 0.01, "gap {} at s = {s}", e.gap_rel);
                            assert!(e.gap_within(c.m.subsidy.gap_limit));
                            worst = worst.max(e.gap_rel.abs());
                        }
                    }
                }
            }
        }
    }
    assert!(worst < 0.01, "worst gap {worst}");
}

#[test]
fn eq_s_9c_gap_terms_still_sum_to_the_gap_with_a_grant() {
    let c = ctx();
    for stage in Stage::ALL {
        for life in LIVES {
            let q = query(c, stage, life);
            for d in Definition::ALL {
                for t in Timing::ALL {
                    for s in [0.0, 0.3, 0.8] {
                        let r = c.m.dcf(&c.b, &with(&q, gq(s, d, t))).unwrap();
                        let sum: f64 = r.gap_terms.iter().map(|g| g.delta).sum();
                        close(sum, r.gap_abs, 1e-9, "gap terms");
                        assert_eq!(r.gap_terms.len(), if s > 0.0 { 13 } else { 12 });
                        // The L5 inside the DCF result is the net L5.
                        let led = c.m.ledger(&c.b, &with(&q, gq(s, d, t))).unwrap();
                        close(
                            r.l5,
                            led.l5_net() * 1e6 / r.annual_abatement_t,
                            1e-9,
                            "net L5",
                        );
                    }
                }
            }
        }
    }
}

#[test]
fn eq_s_8_a_grant_leaves_the_rx_mult_lever_alone() {
    // Decision D5: the reactor stress lever stays what it was, and a grant
    // applies to the stressed reactor capital (convention 8).
    let c = ctx();
    let q = query(c, Stage::Boak, 25.0);
    let stressed = CostQuery { rx_mult: 0.8, ..q };
    let k1 = c.m.cost_items(&c.b, &q).unwrap();
    let k8 = c.m.cost_items(&c.b, &stressed).unwrap();
    close(
        k8.reactor_capital,
        0.8 * k1.reactor_capital,
        1e-9,
        "rx_mult",
    );
    let p =
        c.m.grant_plan(&k8, &gq(0.5, Definition::A, Timing::Progressive))
            .unwrap();
    close(
        p.grant,
        0.5 * k8.reactor_capital,
        1e-9,
        "grant on the stressed capital",
    );
    // The gross cost under the lever is unchanged by the presence of a zero grant.
    let qz = with(&stressed, gq(0.0, Definition::A, Timing::Progressive));
    assert_eq!(
        c.m.ledger(&c.b, &stressed).unwrap(),
        c.m.ledger(&c.b, &qz).unwrap()
    );
}

#[test]
fn eq_s_7e_a_break_even_set_is_a_pure_function_of_its_inputs() {
    let c = ctx();
    let q = query(c, Stage::Boak, 60.0);
    let a: BreakEvenSet =
        c.m.breakeven(&c.b, &q, Definition::B, Timing::AtClose, c.a)
            .unwrap();
    let b: BreakEvenSet =
        c.m.breakeven(&c.b, &q, Definition::B, Timing::AtClose, c.a)
            .unwrap();
    assert_eq!(a, b);
    assert!(c
        .m
        .breakeven(&c.b, &q, Definition::A, Timing::AtClose, 0.0)
        .is_err());
    assert!(c
        .m
        .subsidy_eval(&c.b, &q, gq(0.5, Definition::A, Timing::AtClose), -1.0)
        .is_err());
    assert!(c
        .m
        .subsidy_eval(&c.b, &q, gq(1.5, Definition::A, Timing::AtClose), c.a)
        .is_err());
    // A share that is out of range or not a number is never read as no grant.
    for bad in [-0.1, 1.5, f64::NAN] {
        let qb = with(&q, gq(bad, Definition::A, Timing::Progressive));
        assert!(c.m.ledger(&c.b, &qb).is_err(), "ledger accepted {bad}");
        assert!(c.m.dcf(&c.b, &qb).is_err(), "DCF accepted {bad}");
    }
    // Minus zero is zero.
    let qz = with(&q, gq(-0.0, Definition::B, Timing::AtClose));
    assert_eq!(
        c.m.ledger(&c.b, &qz).unwrap(),
        c.m.ledger(&c.b, &q).unwrap()
    );
}
