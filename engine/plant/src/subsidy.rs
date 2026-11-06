//! Government co-funding of the capital of the incremental project.
//!
//! The grant is a share `s` of the eligible first-installation capital, paid
//! either pro rata with the spend (progressive) or in one sum at financial
//! close. It changes how the owner finances the capital and nothing else: the
//! annual abatement, the operating lines and every physical flow stay as they
//! are. Interest during construction is never granted. It is charged on the
//! private balance outstanding, and the part the grant saves is reported on its
//! own line.
//!
//! The specification is `docs/derivation/02_cost_and_subsidy.md`; equation ids
//! (S.1 to S.9c) are quoted at each function. The annual net abatement is an
//! input of every evaluation here, so the carbon basis can change without a
//! change to this module.
//!
//! The module holds the inputs read from the registry, the grant query, the
//! grant plan with its ledger and DCF forms, the reported quantities of S.8
//! and the break-even solve. `economics.rs` and `dcf.rs` call `grant_plan` when
//! a `CostQuery` carries a grant with a non-zero share, and take the
//! unchanged code path otherwise.

use crate::anchored::{crf, Balance, Model};
use crate::dcf::uniform_spend;
use crate::economics::{CostItems, CostQuery};
use crate::params::Reader;
use nush2_numerics::{EngineError, Result};
use serde::Serialize;

/// Which capital items the grant covers.
///
/// eq: S.1
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Definition {
    /// The reactor overnight capital alone.
    A,
    /// The reactor, the IHX, the capture plant, the recycle train and the licence
    /// (decision D7b). The flags in `parameters.toml` decide the membership.
    B,
}

impl Definition {
    pub const ALL: [Definition; 2] = [Definition::A, Definition::B];

    pub fn id(self) -> &'static str {
        match self {
            Definition::A => "A",
            Definition::B => "B",
        }
    }
}

/// When the grant arrives.
///
/// eq: S.3a
/// eq: S.4a
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum Timing {
    /// Pro rata with the eligible spend, on the date of each instalment.
    Progressive,
    /// The whole grant at financial close, spent first (convention 3).
    AtClose,
}

impl Timing {
    pub const ALL: [Timing; 2] = [Timing::Progressive, Timing::AtClose];

    pub fn id(self) -> &'static str {
        match self {
            Timing::Progressive => "PROGRESSIVE",
            Timing::AtClose => "AT_CLOSE",
        }
    }
}

/// One grant: a share of the eligible capital, the eligible set and the timing.
///
/// eq: S.2
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrantQuery {
    pub share: f64,
    pub definition: Definition,
    pub timing: Timing,
}

/// Eligibility flags of the six capital items of `CostItems`.
///
/// eq: S.1
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Eligibility {
    pub reactor: bool,
    pub ihx: bool,
    pub licence: bool,
    pub power_island: bool,
    pub capture: bool,
    pub recycle: bool,
}

/// A registry entry that must be exactly 0 or 1.
fn read_switch(r: &mut Reader<'_>, id: &str) -> Result<bool> {
    let v = r.get(id)?;
    if v == 0.0 {
        Ok(false)
    } else if v == 1.0 {
        Ok(true)
    } else {
        Err(EngineError::Range {
            param: id.into(),
            msg: format!("{v} is not a switch; use 0 or 1"),
        })
    }
}

impl Eligibility {
    fn read(r: &mut Reader<'_>, set: &str) -> Result<Self> {
        let mut flag = |item: &str| read_switch(r, &format!("SUBSIDY_ELIG_{set}_{item}"));
        let e = Self {
            reactor: flag("REACTOR")?,
            ihx: flag("IHX")?,
            licence: flag("LICENCE")?,
            power_island: flag("POWER_ISLAND")?,
            capture: flag("CAPTURE")?,
            recycle: flag("RECYCLE")?,
        };
        if !e.reactor {
            return Err(EngineError::Range {
                param: format!("SUBSIDY_ELIG_{set}_REACTOR"),
                msg: "the reactor is eligible in both definitions: equations S.3c and S.6a \
                      take the reactor spend as the first thing a grant pays"
                    .into(),
            });
        }
        Ok(e)
    }
}

/// Inputs of the co-funding model, read from the registry.
///
/// eq: S.1
/// eq: S.7e
#[derive(Debug, Clone, PartialEq)]
pub struct SubsidyParams {
    /// Share grid of the scenario table, in order.
    pub grid: [f64; 4],
    pub eligible_a: Eligibility,
    pub eligible_b: Eligibility,
    pub default_definition: Definition,
    pub default_timing: Timing,
    /// Largest share that can be granted.
    pub share_max: f64,
    /// Bracket width at which the bisection fallback stops.
    pub solver_tol: f64,
    /// Largest accepted distance of a closed-form break-even cost from the
    /// target, S$/t.
    pub residual_tol: f64,
    /// Largest accepted gap between the two cost methods at a subsidised point,
    /// as a share of the ledger cost.
    pub gap_limit: f64,
}

impl SubsidyParams {
    /// Read and check the inputs. A value the model does not implement is
    /// refused, never silently replaced.
    pub fn read(r: &mut Reader<'_>) -> Result<Self> {
        let range = |param: &str, msg: String| EngineError::Range {
            param: param.into(),
            msg,
        };
        let share_max = r.get("SUBSIDY_SHARE_MAX")?;
        if !(share_max > 0.0 && share_max <= 1.0) {
            return Err(range(
                "SUBSIDY_SHARE_MAX",
                format!("{share_max} must lie in (0, 1]"),
            ));
        }
        let mut grid = [0.0; 4];
        for (k, slot) in grid.iter_mut().enumerate() {
            let id = format!("SUBSIDY_SHARE_GRID_{}", k + 1);
            let v = r.get(&id)?;
            if !(0.0..=share_max).contains(&v) {
                return Err(range(&id, format!("{v} lies outside 0 to {share_max}")));
            }
            *slot = v;
        }
        let default_definition = if read_switch(r, "SUBSIDY_DEFAULT_DEFINITION")? {
            Definition::B
        } else {
            Definition::A
        };
        let default_timing = if read_switch(r, "SUBSIDY_DEFAULT_TIMING")? {
            Timing::AtClose
        } else {
            Timing::Progressive
        };
        let solver_tol = r.get("SUBSIDY_SOLVER_TOL")?;
        if !(solver_tol > 0.0 && solver_tol < 1e-3) {
            return Err(range(
                "SUBSIDY_SOLVER_TOL",
                format!("{solver_tol} must lie in (0, 1e-3)"),
            ));
        }
        let residual_tol = r.get("SUBSIDY_RESIDUAL_TOL")?;
        if !(residual_tol > 0.0 && residual_tol.is_finite()) {
            return Err(range(
                "SUBSIDY_RESIDUAL_TOL",
                format!("{residual_tol} must be positive"),
            ));
        }
        let cash = r.get("SUBSIDY_GRANT_CASH_RATE")?;
        if cash != 0.0 {
            return Err(range(
                "SUBSIDY_GRANT_CASH_RATE",
                format!(
                    "{cash} is not implemented: grant cash earns nothing while it waits \
                     (convention 3); equation S.3d is kept as a bound only"
                ),
            ));
        }
        let basis = r.get("SUBSIDY_IDC_BASIS")?;
        if basis != 2.0 {
            return Err(range(
                "SUBSIDY_IDC_BASIS",
                format!(
                    "{basis} is not implemented: interest is charged on the private balance \
                     (basis 2 of equation S.5a)"
                ),
            ));
        }
        let gap_limit = r.get("SUBSIDY_GAP_LIMIT")?;
        if !(gap_limit > 0.0 && gap_limit <= 1.0) {
            return Err(range(
                "SUBSIDY_GAP_LIMIT",
                format!("{gap_limit} must lie in (0, 1]"),
            ));
        }
        Ok(Self {
            grid,
            eligible_a: Eligibility::read(r, "A")?,
            eligible_b: Eligibility::read(r, "B")?,
            default_definition,
            default_timing,
            share_max,
            solver_tol,
            residual_tol,
            gap_limit,
        })
    }

    /// Flags of one eligible set.
    pub fn eligibility(&self, d: Definition) -> Eligibility {
        match d {
            Definition::A => self.eligible_a,
            Definition::B => self.eligible_b,
        }
    }
}

// ------------------------------------------------------------ closed forms

/// Present-value factor of a payment dated `year`, negative before year 0.
fn growth(rate: f64, year: i32) -> f64 {
    (1.0 + rate).powi(-year)
}

/// DCF bucket factor: the year-0 value of the reactor spend per unit of
/// overnight capital when the spend sits in the year-end buckets of
/// `dcf::uniform_spend`, a part year first.
///
/// eq: S.0.1
pub fn bucket_factor(rate: f64, years: f64) -> Result<f64> {
    if !(rate > 0.0 && years > 0.0) {
        return Err(EngineError::Range {
            param: "bucket_factor".into(),
            msg: format!("rate {rate} and construction time {years} must both be positive"),
        });
    }
    let whole = years.floor();
    let part = years - whole;
    let g = (1.0 + rate).powf(whole);
    Ok((part * g + (g - 1.0) / rate) / years)
}

/// The ledger's private present-value factor of a grant paid at close, per unit
/// of overnight capital, when the grant covers the fraction `u` of the reactor
/// spend.
///
/// eq: S.3c
pub fn private_factor_at_close(rate: f64, years: f64, u: f64) -> f64 {
    ((1.0 + rate).powf(years * (1.0 - u)) - 1.0) / (rate * years)
}

/// One bucket of the funded-balance recursion of S.3a.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct FundedRow {
    pub year: i32,
    /// Gross reactor spend of the bucket, S$M.
    pub bucket: f64,
    /// Paid by the grant, S$M.
    pub grant: f64,
    /// Paid by the owner, S$M.
    pub private: f64,
    /// Owner's funded balance after the bucket: cash paid in plus the interest
    /// accrued on it, S$M.
    pub balance: f64,
}

/// The owner's account when a grant of `grant` S$M arrives at financial close.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Funded {
    pub rows: Vec<FundedRow>,
    /// Balance carried to year 0: the present value of the private payments.
    pub balance_at_0: f64,
}

/// Apply a grant paid at close to the dated reactor buckets, earliest first,
/// and run the owner's balance forward. Idle grant cash earns nothing.
///
/// eq: S.3a
pub fn funded_balance(rate: f64, buckets: &[(i32, f64)], grant: f64) -> Funded {
    let mut rows = Vec::with_capacity(buckets.len());
    let mut spent = 0.0;
    let mut balance = 0.0;
    let mut last_year: Option<i32> = None;
    for &(year, bucket) in buckets {
        if let Some(y) = last_year {
            balance *= (1.0 + rate).powi(year - y);
        }
        let paid_by_grant = bucket.min((grant - spent).max(0.0));
        let private = bucket - paid_by_grant;
        balance += private;
        spent += bucket;
        last_year = Some(year);
        rows.push(FundedRow {
            year,
            bucket,
            grant: paid_by_grant,
            private,
            balance,
        });
    }
    let balance_at_0 = match last_year {
        Some(y) => balance * (1.0 + rate).powi(-y),
        None => 0.0,
    };
    Funded { rows, balance_at_0 }
}

/// Closed form of the owner's present value of the reactor spend after a grant
/// of `grant` S$M at close, without a loop.
///
/// eq: S.3b
pub fn private_pv_closed_form(rate: f64, years: f64, reactor: f64, grant: f64) -> f64 {
    let whole = years.floor();
    let part = years - whole;
    let gamma = grant * years / reactor;
    if gamma >= years {
        return 0.0;
    }
    let (n_r, kept) = if gamma <= part {
        (whole, part - gamma)
    } else {
        let j = (gamma - part).floor().min(whole - 1.0);
        (whole - 1.0 - j, 1.0 - (gamma - part - j))
    };
    let g = (1.0 + rate).powf(n_r);
    reactor / years * (kept * g + (g - 1.0) / rate)
}

// ------------------------------------------------------------- grant plan

/// A grant worked out against one set of capital items. Amounts are S$M at
/// year 0 unless stated. The ledger side and the DCF side differ only in how
/// interest during construction is measured (S.0.2).
///
/// eq: S.8
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GrantPlan {
    pub share: f64,
    pub definition: Definition,
    pub timing: Timing,
    /// Reactor overnight capital, `CostItems::reactor_capital`.
    pub reactor_capital: f64,
    /// Eligible capital other than the reactor.
    pub other_eligible: f64,
    /// Eligible capital, S.1.
    pub eligible: f64,
    /// The grant, `share * eligible`, S.2.
    pub grant: f64,
    /// First-unit capital of the incremental project, S.8.
    pub total_capital: f64,
    /// `total_capital - grant`.
    pub private_capital: f64,
    /// Part of the grant that pays reactor spend.
    pub grant_reactor: f64,
    /// Part of the grant that pays items bought at year 0.
    pub grant_year0: f64,
    /// Ledger interest factor `f_IDC(t_c)`.
    pub ledger_factor: f64,
    /// DCF bucket factor, the gross reactor value per unit of capital.
    pub dcf_factor: f64,
    /// Interest during construction on the gross spend, ledger form.
    pub ledger_interest_gross: f64,
    /// Interest on the private balance, ledger form.
    pub ledger_interest_private: f64,
    /// Interest the grant saves, ledger form.
    pub ledger_interest_saved: f64,
    /// Year-0 value of the grant to the state, ledger form.
    pub ledger_state_pv: f64,
    pub dcf_interest_gross: f64,
    pub dcf_interest_private: f64,
    pub dcf_interest_saved: f64,
    /// Present value of the private reactor payments in the DCF.
    pub dcf_private_reactor_pv: f64,
    /// Present value of the GRANT line of the DCF, as a positive amount.
    pub dcf_grant_pv: f64,
    pub dcf_state_pv: f64,
    /// Negative flows of the GRANT line at whole years.
    pub dcf_flows: Vec<(i32, f64)>,
}

impl Model {
    /// Work out the grant against the capital items of one query.
    ///
    /// eq: S.1
    /// eq: S.2
    /// eq: S.3a
    /// eq: S.3c
    /// eq: S.4a
    /// eq: S.4b
    /// eq: S.5a
    /// eq: S.8
    pub fn grant_plan(&self, k: &CostItems, g: &GrantQuery) -> Result<GrantPlan> {
        let sp = &self.subsidy;
        if !(0.0..=sp.share_max).contains(&g.share) {
            return Err(EngineError::Range {
                param: "SUBSIDY_SHARE".into(),
                msg: format!("{} lies outside 0 to {}", g.share, sp.share_max),
            });
        }
        let c_r = k.reactor_capital;
        if !(c_r > 0.0 && c_r.is_finite()) {
            return Err(EngineError::Range {
                param: "reactor_capital".into(),
                msg: format!("{c_r} must be positive"),
            });
        }
        let el = sp.eligibility(g.definition);
        let other_eligible: f64 = [
            (el.ihx, k.ihx_capital),
            (el.licence, k.licence_capital),
            (el.power_island, k.power_island_capital),
            (el.capture, k.capture_capital),
            (el.recycle, k.recycle_capital),
        ]
        .iter()
        .filter(|(on, _)| *on)
        .map(|(_, v)| *v)
        .sum();
        let eligible = c_r + other_eligible;
        let s = g.share;
        let grant = s * eligible;
        let total_capital = c_r
            + k.capture_capital
            + k.ihx_capital
            + k.licence_capital
            + k.power_island_capital
            + k.recycle_capital;
        let rate = self.rate;
        let tc = self.idc_years;
        let f_idc = self.idc_factor();
        let buckets = uniform_spend(c_r, tc)?;
        let gross_pv: f64 = buckets.iter().map(|(y, v)| v * growth(rate, *y)).sum();
        let ledger_interest_gross = c_r * (f_idc - 1.0);
        let dcf_interest_gross = gross_pv - c_r;
        let (grant_reactor, grant_year0) = match g.timing {
            Timing::Progressive => (s * c_r, s * other_eligible),
            Timing::AtClose => (grant.min(c_r), (grant - c_r).max(0.0)),
        };
        // Ledger side: S.4b for a progressive grant, S.3c at close.
        let (ledger_interest_private, ledger_state_pv) = match g.timing {
            Timing::Progressive => {
                let private = (1.0 - s) * ledger_interest_gross;
                (private, grant + (ledger_interest_gross - private))
            }
            Timing::AtClose => {
                let u = grant_reactor / c_r;
                let h = private_factor_at_close(rate, tc, u);
                (c_r * (h - (1.0 - u)), grant * (1.0 + rate).powf(tc))
            }
        };
        // DCF side: S.4a for a progressive grant, S.3a at close.
        let (mut flows, dcf_private_reactor_pv) = match g.timing {
            Timing::Progressive => (
                buckets
                    .iter()
                    .map(|(y, b)| (*y, -s * b))
                    .collect::<Vec<_>>(),
                buckets
                    .iter()
                    .map(|(y, b)| (1.0 - s) * b * growth(rate, *y))
                    .sum::<f64>(),
            ),
            Timing::AtClose => {
                let f = funded_balance(rate, &buckets, grant_reactor);
                (
                    f.rows.iter().map(|r| (r.year, -r.grant)).collect(),
                    f.balance_at_0,
                )
            }
        };
        if grant_year0 > 0.0 {
            match flows.last_mut() {
                Some(last) if last.0 == 0 => last.1 -= grant_year0,
                _ => flows.push((0, -grant_year0)),
            }
        }
        let dcf_grant_pv: f64 = flows.iter().map(|(y, v)| -v * growth(rate, *y)).sum();
        let dcf_interest_private = dcf_private_reactor_pv - (c_r - grant_reactor);
        let dcf_state_pv = match g.timing {
            Timing::Progressive => dcf_grant_pv,
            Timing::AtClose => grant * (1.0 + rate).powf(tc),
        };
        Ok(GrantPlan {
            share: s,
            definition: g.definition,
            timing: g.timing,
            reactor_capital: c_r,
            other_eligible,
            eligible,
            grant,
            total_capital,
            private_capital: total_capital - grant,
            grant_reactor,
            grant_year0,
            ledger_factor: f_idc,
            dcf_factor: gross_pv / c_r,
            ledger_interest_gross,
            ledger_interest_private,
            ledger_interest_saved: ledger_interest_gross - ledger_interest_private,
            ledger_state_pv,
            dcf_interest_gross,
            dcf_interest_private,
            dcf_interest_saved: dcf_interest_gross - dcf_interest_private,
            dcf_private_reactor_pv,
            dcf_grant_pv,
            dcf_state_pv,
            dcf_flows: flows,
        })
    }
}

// ------------------------------------------------------- reported quantities

/// The quantities of S.8 for one method at one grant.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct MethodEval {
    /// Annual incremental cost with the grant ignored, S$M/y. For the DCF this
    /// is the levelised cost per tonne times the abatement.
    pub cost_gross_sm_y: f64,
    /// The same net of the grant.
    pub cost_net_sm_y: f64,
    pub lac_gross: f64,
    pub lac_net: f64,
    pub interest_gross_sm: f64,
    pub interest_private_sm: f64,
    pub interest_saved_sm: f64,
    /// Year-0 value of the grant to the state, S$M.
    pub state_pv_sm: f64,
    /// The state's outlay levelised over the abated tonnes, S$/t.
    pub public_outlay_s_per_t: f64,
    /// Strict: the net cost is below the target.
    pub target_met: bool,
}

/// Both methods at one grant, with the plan behind them.
///
/// eq: S.8
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SubsidyEval {
    pub plan: GrantPlan,
    pub abatement_t_y: f64,
    pub life: f64,
    pub target: f64,
    pub l5: MethodEval,
    pub dcf: MethodEval,
    /// Both methods meet the target (decision D3).
    pub target_met: bool,
    /// DCF less L5 at the net cost, S$/t.
    pub gap_s_per_t: f64,
    /// That gap over the L5 net cost.
    pub gap_rel: f64,
}

impl SubsidyEval {
    /// The ledger-to-cash-flow gap stays within the registry limit (decision D11).
    pub fn gap_within(&self, limit: f64) -> bool {
        self.gap_rel.abs() <= limit
    }
}

/// Where a break-even share stands against the target.
///
/// eq: S.7e
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum BreakEven {
    /// The cost with no grant is already below the target. The share is 0.
    MetWithoutGrant,
    /// The infimum of the shares that meet the target. The cost at this share
    /// equals the target, so only larger shares pass.
    Share(f64),
    /// Even the largest share leaves the cost at or above the target.
    Unreachable { lac_at_max: f64 },
}

impl BreakEven {
    /// The share to print, `None` when no share in range meets the target.
    pub fn share(&self) -> Option<f64> {
        match self {
            BreakEven::MetWithoutGrant => Some(0.0),
            BreakEven::Share(s) => Some(*s),
            BreakEven::Unreachable { .. } => None,
        }
    }
}

/// How a break-even share was found.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum SolveRoute {
    /// No root was needed: met without a grant, or unreachable.
    NotNeeded,
    /// The closed form of S.7b to S.7d passed the residual check.
    ClosedForm,
    /// The closed form failed the residual check and the bracketed bisection
    /// of S.7d ran.
    Bisection,
}

/// A break-even share with the check that stands behind it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BreakEvenSolve {
    pub result: BreakEven,
    pub route: SolveRoute,
    /// Cost at the returned share less the target, S$/t. Within the registry
    /// tolerance on the closed-form route.
    pub residual: f64,
}

/// The break-even shares of one scenario, per method and for both together.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BreakEvenSet {
    pub l5: BreakEvenSolve,
    pub dcf: BreakEvenSolve,
    /// Both methods meet the target: the larger share.
    pub both: BreakEven,
}

/// The larger of two break-even results, or the unreachable one.
///
/// eq: S.7e
pub fn combine(a: BreakEven, b: BreakEven) -> BreakEven {
    use BreakEven::{MetWithoutGrant, Share, Unreachable};
    match (a, b) {
        (Unreachable { lac_at_max: x }, Unreachable { lac_at_max: y }) => Unreachable {
            lac_at_max: x.max(y),
        },
        (Unreachable { lac_at_max: x }, _) | (_, Unreachable { lac_at_max: x }) => {
            Unreachable { lac_at_max: x }
        }
        (MetWithoutGrant, MetWithoutGrant) => MetWithoutGrant,
        (MetWithoutGrant, Share(s)) | (Share(s), MetWithoutGrant) => Share(s),
        (Share(x), Share(y)) => Share(x.max(y)),
    }
}

/// Round a share up in the last of `decimals` printed digits, so the printed
/// share lies on the passing side of the infimum.
///
/// eq: S.7e
pub fn round_up_share(s: f64, decimals: i32) -> f64 {
    let scale = 10f64.powi(decimals);
    (s * scale).ceil() / scale
}

/// Bracketed bisection for the share at which a falling cost crosses the
/// target. `lac(lo) >= target > lac(hi)` must hold. Returns the upper end of
/// the final bracket, which always passes the strict test.
///
/// eq: S.7d
pub fn bisect_share<F>(mut lac: F, target: f64, lo: f64, hi: f64, tol: f64) -> Result<f64>
where
    F: FnMut(f64) -> Result<f64>,
{
    let (mut lo, mut hi) = (lo, hi);
    if !(lac(lo)? >= target && lac(hi)? < target) {
        return Err(EngineError::Range {
            param: "bisect_share".into(),
            msg: format!("the target {target} is not bracketed by shares {lo} and {hi}"),
        });
    }
    while hi - lo > tol {
        let mid = 0.5 * (lo + hi);
        if lac(mid)? < target {
            hi = mid;
        } else {
            lo = mid;
        }
    }
    Ok(hi)
}

/// What the closed forms of S.7b to S.7d need about one scenario.
struct Curve {
    rate: f64,
    years: f64,
    /// Levelising factor, S$/t per S$M.
    kappa: f64,
    plan: GrantPlan,
    target: f64,
}

impl Curve {
    /// Break-even share of the ledger cost. Negative when the target is already
    /// met.
    ///
    /// eq: S.7b
    /// eq: S.7c
    fn ledger(&self, lac0: f64, timing: Timing) -> f64 {
        let p = &self.plan;
        match timing {
            Timing::Progressive => {
                let slope = self.kappa * (p.reactor_capital * p.ledger_factor + p.other_eligible);
                (lac0 - self.target) / slope
            }
            Timing::AtClose => {
                let d0 = (lac0 - self.target) / self.kappa;
                let h_star = p.ledger_factor - d0 / p.reactor_capital;
                if h_star >= 0.0 {
                    let ln1 = (1.0 + self.rate).ln();
                    let u_star =
                        1.0 - (1.0 + self.rate * self.years * h_star).ln() / (self.years * ln1);
                    u_star * p.reactor_capital / p.eligible
                } else {
                    (d0 - p.reactor_capital * p.ledger_factor + p.reactor_capital) / p.eligible
                }
            }
        }
    }

    /// Cost of the DCF at close as a function of the share, by S.6b with the
    /// closed form of S.3b.
    ///
    /// eq: S.6b
    fn dcf_close(&self, lac0: f64, s: f64) -> f64 {
        let p = &self.plan;
        let grant = s * p.eligible;
        let reactor = grant.min(p.reactor_capital);
        let pd = private_pv_closed_form(self.rate, self.years, p.reactor_capital, reactor);
        let removed = p.reactor_capital * p.dcf_factor - pd + (grant - p.reactor_capital).max(0.0);
        lac0 - self.kappa * removed
    }

    /// Break-even share of the DCF cost. Progressive: S.7b. At close: the exact
    /// route of S.7d, a straight line between the shares at which the grant
    /// clears a bucket.
    ///
    /// eq: S.7b
    /// eq: S.7d
    fn dcf(&self, lac0: f64, timing: Timing, share_max: f64) -> Result<f64> {
        let p = &self.plan;
        match timing {
            Timing::Progressive => {
                let slope = self.kappa * (p.reactor_capital * p.dcf_factor + p.other_eligible);
                Ok((lac0 - self.target) / slope)
            }
            Timing::AtClose => {
                let buckets = uniform_spend(p.reactor_capital, self.years)?;
                let mut knots = vec![0.0];
                let mut cum = 0.0;
                for (_, b) in &buckets {
                    cum += b;
                    knots.push(cum / p.eligible);
                }
                knots.push(share_max);
                knots.retain(|s| *s <= share_max);
                knots.sort_by(f64::total_cmp);
                knots.dedup_by(|a, b| (*a - *b).abs() < 1e-15);
                let mut prev = (knots[0], self.dcf_close(lac0, knots[0]));
                if prev.1 < self.target {
                    return Ok(-1.0);
                }
                for &s in &knots[1..] {
                    let v = self.dcf_close(lac0, s);
                    if v < self.target {
                        return Ok(prev.0 + (prev.1 - self.target) / (prev.1 - v) * (s - prev.0));
                    }
                    prev = (s, v);
                }
                Ok(f64::INFINITY)
            }
        }
    }
}

impl Model {
    /// Evaluate both methods at one grant against the registry target.
    pub fn subsidy_eval(
        &self,
        b: &Balance,
        q: &CostQuery,
        grant: GrantQuery,
        a: f64,
    ) -> Result<SubsidyEval> {
        self.subsidy_eval_at(b, q, grant, a, self.target_c)
    }

    /// Evaluate both methods at one grant. `a` is the annual net abatement,
    /// t/y, and `target` the cost to undercut, S$/t. The `grant` field of `q`
    /// is replaced.
    ///
    /// eq: S.6a
    /// eq: S.6b
    /// eq: S.7e
    /// eq: S.8
    pub fn subsidy_eval_at(
        &self,
        b: &Balance,
        q: &CostQuery,
        grant: GrantQuery,
        a: f64,
        target: f64,
    ) -> Result<SubsidyEval> {
        if !(a > 0.0 && a.is_finite()) {
            return Err(EngineError::Range {
                param: "abatement".into(),
                msg: format!("{a} t/y must be positive"),
            });
        }
        let q0 = CostQuery { grant: None, ..*q };
        let qg = CostQuery {
            grant: Some(grant),
            ..*q
        };
        let k = self.cost_items(b, &q0)?;
        let plan = self.grant_plan(&k, &grant)?;
        let led0 = self.ledger(b, &q0)?;
        let ledg = self.ledger(b, &qg)?;
        let d0 = self.dcf_at(b, &q0, a)?;
        let dg = self.dcf_at(b, &qg, a)?;
        let per_t = 1e6 / a;
        let outlay = crf(self.rate, q.life)? * per_t;
        let lac5_gross = led0.l5() * per_t;
        let lac5_net = ledg.l5_net() * per_t;
        let l5 = MethodEval {
            cost_gross_sm_y: led0.l5(),
            cost_net_sm_y: ledg.l5_net(),
            lac_gross: lac5_gross,
            lac_net: lac5_net,
            interest_gross_sm: plan.ledger_interest_gross,
            interest_private_sm: plan.ledger_interest_private,
            interest_saved_sm: plan.ledger_interest_saved,
            state_pv_sm: plan.ledger_state_pv,
            public_outlay_s_per_t: plan.ledger_state_pv * outlay,
            target_met: lac5_net < target,
        };
        let dcf = MethodEval {
            cost_gross_sm_y: d0.lac * a * 1e-6,
            cost_net_sm_y: dg.lac * a * 1e-6,
            lac_gross: d0.lac,
            lac_net: dg.lac,
            interest_gross_sm: plan.dcf_interest_gross,
            interest_private_sm: plan.dcf_interest_private,
            interest_saved_sm: plan.dcf_interest_saved,
            state_pv_sm: plan.dcf_state_pv,
            public_outlay_s_per_t: plan.dcf_state_pv * outlay,
            target_met: dg.lac < target,
        };
        Ok(SubsidyEval {
            plan,
            abatement_t_y: a,
            life: q.life,
            target,
            target_met: l5.target_met && dcf.target_met,
            gap_s_per_t: dcf.lac_net - l5.lac_net,
            gap_rel: (dcf.lac_net - l5.lac_net) / l5.lac_net,
            l5,
            dcf,
        })
    }

    /// The break-even shares of one scenario against the registry target.
    pub fn breakeven(
        &self,
        b: &Balance,
        q: &CostQuery,
        definition: Definition,
        timing: Timing,
        a: f64,
    ) -> Result<BreakEvenSet> {
        self.breakeven_at(b, q, definition, timing, a, self.target_c)
    }

    /// Solve the continuous break-even share for the ledger and for the DCF.
    ///
    /// The closed forms of S.7b to S.7d give the share. Each is checked against
    /// the cost the model itself returns at that share, and a result more than
    /// `SUBSIDY_RESIDUAL_TOL` from the target is replaced by the bracketed
    /// bisection of S.7d. A cost already below the target at no grant returns
    /// `MetWithoutGrant`. A cost at or above the target at the largest share
    /// returns `Unreachable`.
    ///
    /// eq: S.7b
    /// eq: S.7c
    /// eq: S.7d
    /// eq: S.7e
    pub fn breakeven_at(
        &self,
        b: &Balance,
        q: &CostQuery,
        definition: Definition,
        timing: Timing,
        a: f64,
        target: f64,
    ) -> Result<BreakEvenSet> {
        if !(a > 0.0 && a.is_finite()) {
            return Err(EngineError::Range {
                param: "abatement".into(),
                msg: format!("{a} t/y must be positive"),
            });
        }
        let q0 = CostQuery { grant: None, ..*q };
        let k = self.cost_items(b, &q0)?;
        let plan = self.grant_plan(
            &k,
            &GrantQuery {
                share: 0.0,
                definition,
                timing,
            },
        )?;
        let per_t = 1e6 / a;
        let kappa = crf(self.rate, q.life)? * per_t;
        let lac5_0 = self.ledger(b, &q0)?.l5() * per_t;
        let lacd_0 = self.dcf_at(b, &q0, a)?.lac;
        let curve = Curve {
            rate: self.rate,
            years: self.idc_years,
            kappa,
            plan,
            target,
        };
        let with = |s: f64| CostQuery {
            grant: Some(GrantQuery {
                share: s,
                definition,
                timing,
            }),
            ..q0
        };
        let lac5 = |s: f64| -> Result<f64> { Ok(self.ledger(b, &with(s))?.l5_net() * per_t) };
        let lacd = |s: f64| -> Result<f64> { Ok(self.dcf_at(b, &with(s), a)?.lac) };
        let l5 = self.solve_share(target, lac5, curve.ledger(lac5_0, timing))?;
        let dcf = self.solve_share(
            target,
            lacd,
            curve.dcf(lacd_0, timing, self.subsidy.share_max)?,
        )?;
        Ok(BreakEvenSet {
            both: combine(l5.result, dcf.result),
            l5,
            dcf,
        })
    }

    /// Classify one cost curve against the target and find its break-even
    /// share: the closed form `guess` if the model agrees with it, the
    /// bisection otherwise.
    fn solve_share<F>(&self, target: f64, mut lac: F, guess: f64) -> Result<BreakEvenSolve>
    where
        F: FnMut(f64) -> Result<f64>,
    {
        let max = self.subsidy.share_max;
        let at0 = lac(0.0)?;
        if at0 < target {
            return Ok(BreakEvenSolve {
                result: BreakEven::MetWithoutGrant,
                route: SolveRoute::NotNeeded,
                residual: at0 - target,
            });
        }
        let at_max = lac(max)?;
        if at_max >= target {
            return Ok(BreakEvenSolve {
                result: BreakEven::Unreachable { lac_at_max: at_max },
                route: SolveRoute::NotNeeded,
                residual: at_max - target,
            });
        }
        if guess.is_finite() && (0.0..=max).contains(&guess) {
            let residual = lac(guess)? - target;
            if residual.abs() <= self.subsidy.residual_tol {
                return Ok(BreakEvenSolve {
                    result: BreakEven::Share(guess),
                    route: SolveRoute::ClosedForm,
                    residual,
                });
            }
        }
        let s = bisect_share(&mut lac, target, 0.0, max, self.subsidy.solver_tol)?;
        Ok(BreakEvenSolve {
            result: BreakEven::Share(s),
            route: SolveRoute::Bisection,
            residual: lac(s)? - target,
        })
    }

    /// The share at which the ledger cost meets the target if idle grant cash
    /// earned the discount rate. It is a lower bound that the model does not
    /// use, because it fails the limit at a share of 1 (S.3d, S.9b). It can lie
    /// outside 0 to 1.
    ///
    /// eq: S.3d
    pub fn idle_cash_bound_share(
        &self,
        b: &Balance,
        q: &CostQuery,
        definition: Definition,
        a: f64,
        target: f64,
    ) -> Result<f64> {
        let q0 = CostQuery { grant: None, ..*q };
        let k = self.cost_items(b, &q0)?;
        let plan = self.grant_plan(
            &k,
            &GrantQuery {
                share: 0.0,
                definition,
                timing: Timing::AtClose,
            },
        )?;
        let per_t = 1e6 / a;
        let kappa = crf(self.rate, q.life)? * per_t;
        let lac5_0 = self.ledger(b, &q0)?.l5() * per_t;
        let slope = kappa * plan.eligible * (1.0 + self.rate).powf(self.idc_years);
        Ok((lac5_0 - target) / slope)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::params::ParamSet;

    #[test]
    fn eq_s_1_registry_membership_follows_decision_d7b() {
        let m = Model::embedded().unwrap();
        let a = m.subsidy.eligibility(Definition::A);
        assert_eq!(
            a,
            Eligibility {
                reactor: true,
                ihx: false,
                licence: false,
                power_island: false,
                capture: false,
                recycle: false,
            }
        );
        let b = m.subsidy.eligibility(Definition::B);
        assert_eq!(
            b,
            Eligibility {
                reactor: true,
                ihx: true,
                licence: true,
                power_island: false,
                capture: true,
                recycle: true,
            }
        );
    }

    #[test]
    fn eq_s_1_a_flag_that_is_not_zero_or_one_is_refused() {
        let mut p = ParamSet::embedded().unwrap();
        p.set("SUBSIDY_ELIG_B_IHX", 0.5).unwrap();
        assert!(Model::new(&p).is_err());
        let mut p = ParamSet::embedded().unwrap();
        p.set("SUBSIDY_ELIG_A_REACTOR", 0.0).unwrap();
        assert!(Model::new(&p).is_err());
    }

    #[test]
    fn eq_s_2_the_grid_defaults_and_limits_are_read_and_checked() {
        let m = Model::embedded().unwrap();
        assert_eq!(m.subsidy.grid, [0.0, 0.25, 0.5, 0.75]);
        assert_eq!(m.subsidy.default_definition, Definition::A);
        assert_eq!(m.subsidy.default_timing, Timing::Progressive);
        assert_eq!(m.subsidy.share_max, 1.0);
        let mut p = ParamSet::embedded().unwrap();
        p.set("SUBSIDY_SHARE_GRID_4", 1.2).unwrap();
        assert!(Model::new(&p).is_err());
    }

    #[test]
    fn eq_s_7d_a_closed_form_that_fails_the_residual_check_falls_back_to_bisection() {
        let m = Model::embedded().unwrap();
        // A straight cost curve of 150 - 100 s crosses the target 100 at 0.5.
        let line = |s: f64| -> Result<f64> { Ok(150.0 - 100.0 * s) };
        let good = m.solve_share(100.0, line, 0.5).unwrap();
        assert_eq!(good.route, SolveRoute::ClosedForm);
        assert_eq!(good.result, BreakEven::Share(0.5));
        // A wrong guess is caught by the residual and replaced by the bisection,
        // which returns the upper end of the final bracket: it passes.
        for guess in [0.9, 0.2, -1.0, f64::INFINITY, f64::NAN] {
            let r = m.solve_share(100.0, line, guess).unwrap();
            assert_eq!(r.route, SolveRoute::Bisection, "guess {guess}");
            let s = r.result.share().unwrap();
            assert!((s - 0.5).abs() <= m.subsidy.solver_tol, "{s}");
            assert!(line(s).unwrap() < 100.0 && r.residual < 0.0);
        }
        // No root needed: met without a grant, or out of reach.
        let met = m.solve_share(100.0, |s| Ok(90.0 - 10.0 * s), 0.0).unwrap();
        assert_eq!(met.result, BreakEven::MetWithoutGrant);
        assert_eq!(met.route, SolveRoute::NotNeeded);
        let out = m.solve_share(100.0, |s| Ok(150.0 - 10.0 * s), 5.0).unwrap();
        assert_eq!(out.result, BreakEven::Unreachable { lac_at_max: 140.0 });
        // A cost equal to the target at no grant is not met: the infimum is 0.
        let tie = m.solve_share(100.0, |s| Ok(100.0 - 10.0 * s), 0.0).unwrap();
        assert_eq!(tie.result, BreakEven::Share(0.0));
    }

    #[test]
    fn eq_s_3d_and_s_5a_only_the_implemented_conventions_are_accepted() {
        let mut p = ParamSet::embedded().unwrap();
        p.set("SUBSIDY_GRANT_CASH_RATE", 0.08).unwrap();
        assert!(Model::new(&p).is_err());
        let mut p = ParamSet::embedded().unwrap();
        p.set("SUBSIDY_IDC_BASIS", 1.0).unwrap();
        assert!(Model::new(&p).is_err());
    }
}
