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
//! This file holds the inputs read from the registry and the grant query. The
//! grant plan, the break-even solve and the reported quantities follow in the
//! same module.

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::anchored::Model;
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
    fn eq_s_3d_and_s_5a_only_the_implemented_conventions_are_accepted() {
        let mut p = ParamSet::embedded().unwrap();
        p.set("SUBSIDY_GRANT_CASH_RATE", 0.08).unwrap();
        assert!(Model::new(&p).is_err());
        let mut p = ParamSet::embedded().unwrap();
        p.set("SUBSIDY_IDC_BASIS", 1.0).unwrap();
        assert!(Model::new(&p).is_err());
    }
}
