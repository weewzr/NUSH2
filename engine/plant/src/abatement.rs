//! Independent carbon basis. The CO2 captured and emitted by the plant come from
//! its own stream ledger (the Level 3 flowsheet solved on the hydrogen target),
//! and the unabated reference is a documented generic intensity times the annual
//! hydrogen. Equations N.1 to N.10 of `docs/derivation/01_carbon_and_abatement.md`
//! are quoted at each function.
//!
//! The anchored functions of `anchored.rs` keep their code and their results
//! (the legacy basis, decision D1). `Model::plant_balance` returns the same
//! `Balance` with the carbon lines and the captured tonnage replaced by the
//! plant's ledger when `CARBON_BASIS` is 1. The heat and power rows of that
//! balance stay on the anchored carbon (a stated limit, see the reconciliation
//! rows of the exporter).

use crate::anchored::{Balance, BalanceOpts, Carbon, CarbonSrc, Case, Model, Scenario};
use crate::carbon::{closure_report, AccountReport, CarbonLedger};
use crate::emissions::Emissions;
use crate::flowsheet::{
    self, ng_carbon_per_mol, CaptureSel, FeedBasis, FlowsheetRun, FlowsheetSpec, TearStart,
    NG_MOL_FRACTIONS,
};
use crate::params::Reader;
use nush2_numerics::{EngineError, Result};
use nush2_props::units::{kmol_h_to_t_h, mmscfd_to_kmol_h, MW_CO2};
use nush2_units::stream::{Sp, NSP};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock, PoisonError};

/// Which carbon ledger the abatement figures use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CarbonBasis {
    /// Ledger anchored to the INL Case 5 and Case 6 values, INL baseline.
    Legacy,
    /// The plant's own stream ledger, generic baseline intensity.
    Independent,
}

impl CarbonBasis {
    pub fn label(self) -> &'static str {
        match self {
            CarbonBasis::Legacy => "legacy",
            CarbonBasis::Independent => "independent",
        }
    }
}

/// Published unabated reference intensities (decision D2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BaselineKind {
    Ieaghg,
    Inl,
    Cho,
}

impl BaselineKind {
    /// Row id the emissions ledger prints.
    pub fn id(self) -> &'static str {
        match self {
            BaselineKind::Ieaghg => "IEAGHG_9p00",
            BaselineKind::Inl => "INL_9p21",
            BaselineKind::Cho => "CHO_9p35",
        }
    }

    pub const ALL: [BaselineKind; 3] = [BaselineKind::Ieaghg, BaselineKind::Inl, BaselineKind::Cho];
}

/// Inputs of the transport and storage emissions (equation N.9.4, decision D7c).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TsInputs {
    /// Ship fuel, g per t of CO2 and km.
    pub ship_g_t_km: f64,
    /// Energy content of the ship fuel, MWh/t.
    pub ship_fuel_mwh_t: f64,
    /// CO2 per energy of the ship fuel, t/TJ.
    pub ship_fuel_ef: f64,
    pub dist_km: f64,
    /// Conditioning electricity, kWh/t.
    pub cond_kwh_t: f64,
    /// Injection pumping electricity, kWh/t.
    pub inj_kwh_t: f64,
    /// Terminal operating emissions, t/t.
    pub terminal: f64,
    /// Purge, cool-down and methane slip, t/t.
    pub purge: f64,
    /// Grid emission factor, t/MWh.
    pub grid_ef: f64,
}

/// Emissions per tonne of CO2 delivered at S20, t/t, by source.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct TsParts {
    pub ship: f64,
    pub conditioning: f64,
    pub injection: f64,
    pub terminal: f64,
    pub purge: f64,
}

impl TsParts {
    /// The burden tau_TS of equation N.9.4.
    pub fn total(&self) -> f64 {
        self.ship + self.conditioning + self.injection + self.terminal + self.purge
    }

    pub fn lines(&self) -> [(&'static str, f64); 5] {
        [
            ("TS_SHIP", self.ship),
            ("TS_CONDITIONING", self.conditioning),
            ("TS_INJECTION", self.injection),
            ("TS_TERMINAL", self.terminal),
            ("TS_PURGE", self.purge),
        ]
    }
}

/// Inputs of the independent carbon basis, read once from the registry.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CarbonParams {
    pub basis: CarbonBasis,
    pub baseline: BaselineKind,
    /// Cryogenic carbon recovery f, a design input (decision D8).
    pub cryo_f: f64,
    /// Cryogenic hydrogen return r, a design input (decision D8).
    pub cryo_r: f64,
    /// Share v of the condensate CO2 that reaches the air.
    pub cond_vent: f64,
    /// Closure tolerance tau_C.
    pub closure_tol: f64,
    pub ts: TsInputs,
}

fn switch(r: &mut Reader<'_>, id: &str, n: u32) -> Result<u32> {
    let v = r.get(id)?;
    let k = v.round();
    if !(v.is_finite() && (v - k).abs() < 1e-12 && k >= 0.0 && k < f64::from(n)) {
        return Err(EngineError::Range {
            param: id.into(),
            msg: format!("{v} must be one of 0 to {}", n - 1),
        });
    }
    Ok(k as u32)
}

fn unit_interval(r: &mut Reader<'_>, id: &str) -> Result<f64> {
    let v = r.get(id)?;
    if !(0.0..=1.0).contains(&v) {
        return Err(EngineError::Range {
            param: id.into(),
            msg: format!("{v} must lie in [0, 1]"),
        });
    }
    Ok(v)
}

fn non_negative(r: &mut Reader<'_>, id: &str) -> Result<f64> {
    let v = r.get(id)?;
    if !(v >= 0.0 && v.is_finite()) {
        return Err(EngineError::Range {
            param: id.into(),
            msg: format!("{v} must be zero or positive"),
        });
    }
    Ok(v)
}

impl CarbonParams {
    /// Read and check the inputs. A value outside its range is refused and never
    /// clamped (test 26 of the derivation).
    pub fn read(r: &mut Reader<'_>) -> Result<Self> {
        let basis = match switch(r, "CARBON_BASIS", 2)? {
            0 => CarbonBasis::Legacy,
            _ => CarbonBasis::Independent,
        };
        let baseline = match switch(r, "CARBON_BASELINE", 3)? {
            0 => BaselineKind::Ieaghg,
            1 => BaselineKind::Inl,
            _ => BaselineKind::Cho,
        };
        let cryo_f = unit_interval(r, "CRYO_C_RECOVERY")?;
        let cryo_r = unit_interval(r, "CRYO_H2_RETURN")?;
        let cond_vent = unit_interval(r, "COND_CO2_VENT")?;
        let closure_tol = r.get("CARBON_CLOSURE_TOL")?;
        if !(closure_tol > 0.0 && closure_tol <= 1.0e-6) {
            return Err(EngineError::Range {
                param: "CARBON_CLOSURE_TOL".into(),
                msg: format!("{closure_tol} must lie in (0, 1e-6]"),
            });
        }
        let ship_fuel_mwh_t = r.get("TS_SHIP_FUEL_MWH_T")?;
        if ship_fuel_mwh_t <= 0.0 {
            return Err(EngineError::Range {
                param: "TS_SHIP_FUEL_MWH_T".into(),
                msg: format!("{ship_fuel_mwh_t} must be positive"),
            });
        }
        let ts = TsInputs {
            ship_g_t_km: non_negative(r, "TS_SHIP_G_T_KM")?,
            ship_fuel_mwh_t,
            ship_fuel_ef: non_negative(r, "TS_SHIP_FUEL_EF")?,
            dist_km: non_negative(r, "TS_SHIP_DIST_KM")?,
            cond_kwh_t: non_negative(r, "TS_COND_KWH_T")?,
            inj_kwh_t: non_negative(r, "TS_INJ_KWH_T")?,
            terminal: non_negative(r, "TS_TERMINAL_FRAC")?,
            purge: non_negative(r, "TS_PURGE_FRAC")?,
            grid_ef: non_negative(r, "TS_GRID_EF")?,
        };
        Ok(Self {
            basis,
            baseline,
            cryo_f,
            cryo_r,
            cond_vent,
            closure_tol,
            ts,
        })
    }

    pub fn independent(&self) -> bool {
        self.basis == CarbonBasis::Independent
    }
}

/// Run of the stream ledger.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
pub enum LedgerKind {
    /// MEA capture with the cryogenic recycle (REC_F).
    Rec,
    /// INL's Selexol train with the cryogenic recycle, at the hydrogen target.
    InlSel,
    /// MEA capture and no cryogenic unit: f = 0, r = 0, all reject burned.
    NoRecycle,
    /// Unabated reference: no capture, f = 0, r = 0 (floor of equation N.7).
    Reference,
}

impl LedgerKind {
    pub fn id(self) -> &'static str {
        match self {
            LedgerKind::Rec => "LEDGER_REC",
            LedgerKind::InlSel => "LEDGER_INL_SEL",
            LedgerKind::NoRecycle => "LEDGER_NO_RECYCLE",
            LedgerKind::Reference => "LEDGER_REFERENCE",
        }
    }
}

/// One flowsheet run on the Case A hydrogen target, with its carbon lines.
#[derive(Debug, Clone)]
pub struct PlantLedger {
    pub kind: LedgerKind,
    /// Per-pass capture of the run (Selexol's for `InlSel`, 0 for `Reference`).
    pub eta: f64,
    pub run: FlowsheetRun,
}

impl PlantLedger {
    pub fn lines(&self) -> &CarbonLedger {
        &self.run.carbon
    }

    /// Fresh gas, MMSCFD at the Case A hydrogen.
    pub fn ng_mmscfd(&self) -> f64 {
        self.run.ng_mmscfd
    }

    /// Fresh gas, kmol/h.
    pub fn ng_kmol_h(&self) -> f64 {
        mmscfd_to_kmol_h(self.run.ng_mmscfd)
    }

    fn stream_carbon(&self, tag: &str, sp: Sp) -> f64 {
        self.run
            .gas(tag)
            .map(|s| s.n[sp as usize] * 3.6)
            .unwrap_or(f64::NAN)
    }

    /// Carbon shares of the tube carbon at the absorber: CO2 (phi), CH4 (psi),
    /// CO (omega) and the condensate CO2 (kappa), equation N.2.2.
    pub fn shares(&self) -> Shares {
        let l = self.lines();
        Shares {
            phi: l.co2_abs / l.tube,
            psi: self.stream_carbon("S11", Sp::Ch4) / l.tube,
            omega: self.stream_carbon("S11", Sp::Co) / l.tube,
            kappa: l.condensate / l.tube,
        }
    }

    /// H2 in the treated gas S12 and in the product S13, kmol/h.
    pub fn h2_s12_s13(&self) -> (f64, f64) {
        let h = |tag: &str| {
            self.run
                .gas(tag)
                .map(|s| s.n[Sp::H2 as usize] * 3.6)
                .unwrap_or(f64::NAN)
        };
        (h("S12"), h("S13"))
    }
}

/// Carbon shares of the tube carbon (equation N.2.2).
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Shares {
    pub phi: f64,
    pub psi: f64,
    pub omega: f64,
    pub kappa: f64,
}

type LedgerKey = (u8, [u64; 7]);

fn ledger_cache() -> &'static Mutex<BTreeMap<LedgerKey, Arc<PlantLedger>>> {
    static CACHE: OnceLock<Mutex<BTreeMap<LedgerKey, Arc<PlantLedger>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(BTreeMap::new()))
}

/// Cache size beyond which the oldest keys are dropped.
const CACHE_LIMIT: usize = 256;

/// The closed forms of the derivation. Each checks the flowsheet and none
/// replaces it; the shares psi, omega, kappa and phi are numerical outputs of the
/// reformer, shift and flash models.
pub mod closed_form {
    use super::*;

    /// Hydrogen potential of a species vector, kmol H2 per kmol basis (N.1.2):
    /// twice the carbon, less the oxygen, plus half the hydrogen.
    ///
    /// eq: N.1.2
    pub fn hydrogen_potential(n: &[f64; NSP]) -> Result<f64> {
        let e = nush2_props::mixture::elements(n)?;
        Ok(2.0 * e[0] - e[2] + e[1] / 2.0)
    }

    /// Carbon per kmol of the lumped gas and its hydrogen potential.
    ///
    /// eq: N.1.1
    pub fn gas_constants() -> Result<(f64, f64)> {
        let mut n = [0.0; NSP];
        for (sp, y) in NG_MOL_FRACTIONS {
            n[sp as usize] += y;
        }
        Ok((ng_carbon_per_mol(), hydrogen_potential(&n)?))
    }

    /// Net PSA recovery with the cryogenic hydrogen return closed.
    ///
    /// eq: N.1.4
    pub fn xi_eff(xi: f64, r: f64) -> f64 {
        xi / (1.0 - r * (1.0 - xi))
    }

    /// Loop denominator D.
    ///
    /// eq: N.2.1
    pub fn loop_denominator(f: f64, kappa: f64, eta: f64, phi: f64) -> f64 {
        1.0 - f * (1.0 - kappa - eta * phi)
    }

    /// Tube carbon T from the fresh carbon, the recycle fraction and the product
    /// carbon.
    ///
    /// eq: N.2.1
    pub fn tube_carbon(fc: f64, f: f64, p: f64, d: f64) -> f64 {
        (fc - f * p) / d
    }

    /// Carbon of every modelled stream from the tube carbon.
    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct Fates {
        pub g: f64,
        pub r: f64,
        pub b: f64,
        pub k: f64,
        pub c: f64,
        pub slip_co2: f64,
    }

    /// eq: N.2.2
    /// eq: N.3.1
    pub fn fates(t: f64, f: f64, p: f64, s: &Shares, eta: f64) -> Fates {
        let c = eta * s.phi * t;
        let g = (1.0 - s.kappa - eta * s.phi) * t - p;
        Fates {
            g,
            r: f * g,
            b: (1.0 - f) * g,
            k: s.kappa * t,
            c,
            slip_co2: (1.0 - eta) * s.phi * t,
        }
    }

    /// Plant capture C / F_C.
    ///
    /// eq: N.3.2
    pub fn plant_capture(eta: f64, phi: f64, f: f64, p: f64, fc: f64, d: f64) -> f64 {
        eta * phi * (1.0 - f * p / fc) / d
    }

    /// Hydrogen deficit coefficient chi.
    pub fn chi(f: f64, s: &Shares, d: f64) -> f64 {
        (1.0 - f) * (4.0 * s.psi + s.omega) / d
    }

    /// Gas rate for a hydrogen target, kmol/h.
    ///
    /// eq: N.1.5
    pub fn gas_rate_from_h2(
        n_h2: f64,
        xi_eff: f64,
        f: f64,
        pi: f64,
        chi: f64,
        z_ng: f64,
        cbar: f64,
    ) -> f64 {
        n_h2 * (1.0 / xi_eff + f * pi * (4.0 - chi)) / (z_ng - chi * cbar)
    }

    /// Recycle fraction read back from the streams.
    ///
    /// eq: N.4.1
    pub fn recycle_read_back(t: f64, fc: f64, g: f64) -> f64 {
        (t - fc) / g
    }

    /// Nitrogen-cut diagnostics: carbon per kmol of purged N2 and the recycle
    /// fraction that follows from the N2 balance.
    ///
    /// eq: N.4.2
    pub fn n2_cut(b: f64, n2_feed: f64, g: f64) -> (f64, f64) {
        let beta = b / n2_feed;
        (beta, 1.0 - beta * n2_feed / g)
    }

    /// Stack carbon at a fixed fresh carbon, and the recycle fraction that gives
    /// a chosen stack carbon.
    ///
    /// eq: N.4.4
    pub fn stack_vs_f(f: f64, m: f64, fc: f64, p: f64) -> f64 {
        (1.0 - f) * (m * fc - p) / (1.0 - m * f)
    }

    /// eq: N.4.4
    pub fn f_for_stack(b: f64, m: f64, fc: f64, p: f64) -> f64 {
        let b0 = m * fc - p;
        (b0 - b) / (b0 - m * b)
    }

    /// Stack carbon at a fixed hydrogen output, with the gas rate of N.1.5 at
    /// every f. The shares are held at the values they have in the run.
    ///
    /// eq: N.4.4
    pub fn stack_at_fixed_h2(
        f: f64,
        s: &Shares,
        eta: f64,
        xi: f64,
        r: f64,
        pi: f64,
        n_h2: f64,
    ) -> Result<f64> {
        let (cbar, z_ng) = gas_constants()?;
        let d = loop_denominator(f, s.kappa, eta, s.phi);
        let n_ng = gas_rate_from_h2(n_h2, xi_eff(xi, r), f, pi, chi(f, s, d), z_ng, cbar);
        let m = 1.0 - s.kappa - eta * s.phi;
        Ok(stack_vs_f(f, m, cbar * n_ng, pi * n_h2))
    }

    /// Derivative of the legacy recycle fraction with respect to the carbon
    /// split phi at which INL's numbers are read (T6 and G6 at Case A scale).
    ///
    /// eq: N.4.3
    pub fn d_f_inl_d_phi(t6: f64, phi: f64, fc6: f64, c6: f64, p_a: f64) -> f64 {
        let g6 = t6 - c6 - p_a;
        -(t6 / phi) * (fc6 - c6 - p_a) / (g6 * g6)
    }

    /// Extra gas the furnace of the unabated reference burns after its reject,
    /// kmol/h. The central result does not use it: the furnace efficiency and the
    /// steam duty have no source in the repository, so the reference is taken
    /// without firing gas and the result is a floor.
    ///
    /// eq: N.7.2
    pub fn firing_gas(q_f: f64, eta_f: f64, q_rej: f64, lhv_ng: f64) -> f64 {
        (q_f / eta_f - q_rej).max(0.0) / lhv_ng
    }

    /// Lost steam and avoided grid power of the system view, t/y: steam in
    /// MMBtu/h, power in MW.
    ///
    /// eq: N.7.4
    #[allow(clippy::too_many_arguments)]
    pub fn system_terms(
        lost_mmbtu_h: f64,
        power_mw: f64,
        hours: f64,
        e_ng: f64,
        eta_b: f64,
        ef_grid: f64,
    ) -> (f64, f64) {
        let lost = lost_mmbtu_h * nush2_props::units::MMBTU_GJ * hours * e_ng / eta_b / 1000.0;
        (lost, power_mw * hours * ef_grid)
    }

    /// Lifecycle view from the net abatement: the upstream gas saved at CO2 and
    /// CH4 per heat, with the CH4 entering only through its warming potential, less
    /// the nuclear lifecycle emission. A potential of zero is the CO2 view.
    ///
    /// eq: N.9.5
    pub fn lifecycle(
        a_ts: f64,
        dg_gj: f64,
        e_up_co2: f64,
        gwp: f64,
        e_up_ch4: f64,
        e_nuc: f64,
    ) -> f64 {
        a_ts + dg_gj * (e_up_co2 + gwp * e_up_ch4) - e_nuc
    }

    /// Burden per tonne delivered, the sum of the sources.
    ///
    /// eq: N.9.4
    pub fn ts_ratio(p: &TsParts) -> f64 {
        p.total()
    }

    /// Annual tonnes per kmol/h of carbon: M_CO2 t_h / 1000.
    ///
    /// eq: N.8.2
    pub fn tonnes_per_kmol_h(hours: f64) -> f64 {
        MW_CO2 * hours / 1000.0
    }
}

/// Share of the direct abatement by origin against the central reference
/// (equation N.9.2), t/y.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Decomposition {
    /// Captured CO2: k C.
    pub captured: f64,
    /// Carbon of the gas the plant does not buy: k (cbar n_ref - cbar n_NG).
    pub avoided_gas: f64,
    /// Condensate term: -(1 - v) k (K' - K).
    pub condensate: f64,
    /// Sum of the three terms.
    pub total: f64,
    /// Gas of the unabated reference implied by the central baseline, MMSCFD at
    /// the case scale.
    pub n_ref_mmscfd: f64,
    /// Gas of the plant, MMSCFD at the case scale.
    pub n_plant_mmscfd: f64,
}

/// One reference intensity and the abatement against it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct BaselineRow {
    pub id: &'static str,
    pub e_ref_t_y: f64,
    pub kg_kg_ref: f64,
    pub a_direct: f64,
    pub a_net: f64,
    pub a_h2_direct: f64,
    pub a_h2_net: f64,
    pub reduction: f64,
}

/// The plant-built unabated reference of equations N.7.1 and N.7.3 without
/// firing gas, as a cost comparison. It is a floor: the furnace of a real
/// reference burns extra gas (open issue 2 of the derivation), so its gas and
/// its emission are the lowest the plant's own balance supports. The central
/// result stays on the generic reference intensity (decision D2); this is the
/// other end of the range, printed beside it.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct FloorReference {
    /// Process gas of the reference at this plant's hydrogen, MMSCFD at the
    /// case scale.
    pub gas_mmscfd: f64,
    /// Net abatement against it, after transport and storage, t/y.
    pub a_net: f64,
    /// Its emission intensity, kg CO2 per kg H2.
    pub kg_kg: f64,
}

/// Annual quantities of the independent basis for one balance (N.5 to N.10).
#[derive(Debug, Clone, Serialize)]
pub struct Abatement {
    pub src: CarbonSrc,
    pub h2_t_y: f64,
    pub captured_t_y: f64,
    pub stack_t_y: f64,
    pub cond_vented_t_y: f64,
    /// Stack plus vented condensate: the modelled air outlets.
    pub residual_direct_t_y: f64,
    pub ts_emission_t_y: f64,
    pub ts: TsParts,
    pub tau_ts: f64,
    pub central: &'static str,
    /// IEAGHG, INL and Cho, then the plant-built floor when the plant ledger is
    /// the source.
    pub rows: Vec<BaselineRow>,
    pub intensity_kg_kg: f64,
    pub closure: AccountReport,
    pub decomposition: Option<Decomposition>,
    /// Gas of the unabated reference implied by the plant-built floor, MMSCFD.
    pub floor_gas_mmscfd: Option<f64>,
}

impl Model {
    /// Emissions per tonne delivered at S20, by source (equation N.9.4).
    ///
    /// eq: N.9.4
    pub fn ts_parts(&self) -> TsParts {
        let t = &self.carbon_params.ts;
        let kg_co2_per_kg_fuel = t.ship_fuel_ef * t.ship_fuel_mwh_t * 3.6 / 1000.0;
        TsParts {
            ship: t.ship_g_t_km * t.dist_km / 1000.0 * kg_co2_per_kg_fuel / 1000.0,
            conditioning: t.cond_kwh_t / 1000.0 * t.grid_ef,
            injection: t.inj_kwh_t / 1000.0 * t.grid_ef,
            terminal: t.terminal,
            purge: t.purge,
        }
    }

    /// Reference intensity of a published baseline, kg CO2 per kg H2.
    pub fn baseline_kg_kg(&self, kind: BaselineKind) -> f64 {
        match kind {
            BaselineKind::Ieaghg => self.base_ieaghg,
            BaselineKind::Inl => self.base_inl,
            BaselineKind::Cho => self.base_cho,
        }
    }

    /// Spec of a ledger run on the Case A hydrogen target. The cryogenic inputs
    /// are the registry design inputs, the solvent loading is fixed (the carbon of
    /// S19 equals the capture at convergence either way).
    pub fn ledger_spec(&self, kind: LedgerKind, eta: f64) -> FlowsheetSpec {
        let p = &self.carbon_params;
        let (scn, capture, f, r) = match kind {
            LedgerKind::Rec => (Scenario::RecF, CaptureSel::Eta(eta), p.cryo_f, p.cryo_r),
            LedgerKind::InlSel => (Scenario::InlSel, CaptureSel::Model, p.cryo_f, p.cryo_r),
            LedgerKind::NoRecycle => (Scenario::RecF, CaptureSel::Eta(eta), 0.0, 0.0),
            LedgerKind::Reference => (Scenario::RecF, CaptureSel::Off, 0.0, 0.0),
        };
        let mut spec =
            FlowsheetSpec::new(self, Case::A, scn, FeedBasis::Hydrogen(self.h2_a_kgh), r);
        spec.carbon_recycle = f;
        spec.h2_return = r;
        spec.capture = capture;
        spec.solvent_tear = false;
        spec
    }

    /// Stream ledger of one run, from the cache when the same inputs ran before.
    /// A run that does not converge is an error and is never replaced by the
    /// legacy ledger.
    ///
    /// eq: N.2.2
    pub fn plant_ledger(&self, kind: LedgerKind, eta: f64) -> Result<Arc<PlantLedger>> {
        let eta = match kind {
            LedgerKind::InlSel => self.eta_sel,
            LedgerKind::Reference => 0.0,
            _ => eta,
        };
        let spec = self.ledger_spec(kind, eta);
        let key: LedgerKey = (
            kind as u8,
            [
                eta.to_bits(),
                spec.carbon_recycle.to_bits(),
                spec.h2_return.to_bits(),
                spec.psa_recovery.to_bits(),
                spec.ch4_in_product.to_bits(),
                self.h2_a_kgh.to_bits(),
                self.eta_sel.to_bits(),
            ],
        );
        if let Some(hit) = ledger_cache()
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .get(&key)
        {
            return Ok(hit.clone());
        }
        let start = match kind {
            LedgerKind::NoRecycle | LedgerKind::Reference => TearStart::Zero,
            _ => TearStart::Anchored,
        };
        let run = flowsheet::solve(self, &spec, kind.id(), start)?;
        let led = Arc::new(PlantLedger { kind, eta, run });
        let mut cache = ledger_cache()
            .lock()
            .unwrap_or_else(PoisonError::into_inner);
        if cache.len() >= CACHE_LIMIT {
            cache.clear();
        }
        cache.insert(key, led.clone());
        Ok(led)
    }

    /// The unabated reference run (equation N.7.1 at its floor).
    pub fn reference_ledger(&self) -> Result<Arc<PlantLedger>> {
        self.plant_ledger(LedgerKind::Reference, 0.0)
    }

    /// Plant carbon lines at Case A scale for a scenario, or `None` when the
    /// scenario has no plant ledger (REC_FIX is pinned to INL's stack by
    /// definition and stays on the anchored basis).
    ///
    /// eq: N.2.2
    pub fn plant_carbon(&self, scn: Scenario, eta: f64, base: &Carbon) -> Result<Option<Carbon>> {
        let (led, k, ng) = match scn {
            Scenario::RecFix => return Ok(None),
            Scenario::InlSel => {
                let l = self.plant_ledger(LedgerKind::InlSel, eta)?;
                let ng = l.ng_mmscfd();
                (l, 1.0, ng)
            }
            Scenario::RecF => {
                let l = self.plant_ledger(LedgerKind::Rec, eta)?;
                let ng = l.ng_mmscfd();
                (l, 1.0, ng)
            }
            Scenario::CombH | Scenario::Export => {
                let l = self.plant_ledger(LedgerKind::NoRecycle, eta)?;
                let ng = l.ng_mmscfd();
                (l, 1.0, ng)
            }
            Scenario::CombN => {
                // Gas held at the recycle plant's own rate; by homogeneity the
                // hydrogen and every line scale with the gas ratio.
                let rec = self.plant_ledger(LedgerKind::Rec, eta)?;
                let nr = self.plant_ledger(LedgerKind::NoRecycle, eta)?;
                (
                    nr.clone(),
                    rec.ng_mmscfd() / nr.ng_mmscfd(),
                    rec.ng_mmscfd(),
                )
            }
        };
        let a = *led.lines();
        let v = self.carbon_params.cond_vent;
        let vented = v * a.condensate * k;
        let mut c = *base;
        c.src = CarbonSrc::Plant;
        c.ng = ng;
        c.h2 = self.h2_a_mmscfd * k;
        c.h_scale = k;
        c.f = a.feed * k;
        c.t = a.tube * k;
        c.co2_abs = a.co2_abs * k;
        c.cap = a.captured * k;
        c.slip = a.slip * k;
        c.g = a.tail_gas * k;
        c.r = a.recycled * k;
        c.p = a.product * k;
        c.cond = a.condensate * k;
        c.unconv = (a.tube - a.co2_abs - a.condensate) * k;
        if scn == Scenario::Export {
            c.burn = vented;
            c.exported = a.burned * k;
        } else {
            c.burn = a.burned * k + vented;
            c.exported = 0.0;
        }
        c.closure = (a.feed - a.captured - a.burned - a.product - a.condensate) * k;
        Ok(Some(c))
    }

    /// Heat and power balance with the carbon lines of the selected basis. On
    /// the legacy basis, or for a scenario without a plant ledger, it is the
    /// anchored balance unchanged.
    pub fn plant_balance(&self, scn: Scenario, o: &BalanceOpts) -> Result<Balance> {
        let mut b = self.balance(scn, o)?;
        if self.carbon_params.independent() {
            if let Some(c) = self.plant_carbon(scn, o.eta_c, &b.c)? {
                b.cap_t = kmol_h_to_t_h(c.cap) * o.s;
                b.c = c;
            }
        }
        Ok(b)
    }

    /// Central balance of a scenario at scale `s` on the selected basis.
    pub fn plant_balance_at(&self, scn: Scenario, s: f64) -> Result<Balance> {
        self.plant_balance(scn, &self.opts(s))
    }

    /// Scalar summary on the selected basis, keyed as `case_summary` is, with the
    /// independent quantities added when the plant ledger is the source.
    ///
    /// eq: N.9.1
    /// eq: N.10.1
    pub fn plant_summary(&self, scn: Scenario, s: f64) -> Result<BTreeMap<&'static str, f64>> {
        let b = self.plant_balance(scn, &self.opts(s))?;
        let mut k = self.case_summary_from(scn, s, &b)?;
        if b.c.src == CarbonSrc::Plant {
            let ab = self.abatement(&b, s)?;
            let central = ab.rows.iter().find(|r| r.id == ab.central);
            k.insert("captured_t_y", ab.captured_t_y);
            k.insert("stack_t_y", ab.stack_t_y);
            k.insert("cond_vented_t_y", ab.cond_vented_t_y);
            k.insert("ts_emission_t_y", ab.ts_emission_t_y);
            k.insert("tau_ts", ab.tau_ts);
            if let Some(r) = central {
                k.insert("A_dir_t_y", r.a_direct);
                k.insert("A_net_t_y", r.a_net);
                k.insert("A_net_per_kg", r.a_h2_net);
                k.insert("A_dir_per_kg", r.a_h2_direct);
                k.insert("baseline_central_t_y", r.e_ref_t_y);
                // The headline keys of the legacy summary now describe the
                // central reference: net abatement, its reduction and its
                // intensity (decision D9).
                k.insert("A_t_y", r.a_net);
                k.insert("baseline_t_y", r.e_ref_t_y);
                k.insert("baseline_kg_kg", r.kg_kg_ref);
                k.insert("reduction", r.reduction);
                k.insert(
                    "never_generated_t_y",
                    r.e_ref_t_y - ab.captured_t_y - ab.residual_direct_t_y,
                );
            }
            if b.c.f > 0.0 {
                k.insert("plant_capture", b.c.cap / b.c.f);
            }
            k.insert("intensity_kg_kg", ab.intensity_kg_kg);
            if let Some(rel) = ab.closure.relative_modelled {
                k.insert("closure_rel", rel);
            }
        }
        Ok(k)
    }

    /// Carbon of all the gas that the unabated reference plant takes in to make
    /// this plant's hydrogen, kmol C/h in the units of the `Carbon` lines of the
    /// balance (per unit of the case scale). The reference emission is
    /// `k (cbar n_ref - P - (1 - v) K')` (N.7.3), so the carbon follows from the
    /// central intensity: `E_ref / k + P + (1 - v) K'`, with the product carbon
    /// `P` of the plant and the condensate CO2 `K'` of the reference run.
    ///
    /// eq: N.7.3
    pub fn reference_carbon(&self, c: &Carbon, s: f64) -> Result<f64> {
        let e = self.emissions(c, s);
        let rf = self.reference_ledger()?;
        let k = self.tpy(1.0, s);
        let v = self.carbon_params.cond_vent;
        let k_ref = rf.lines().condensate * c.h_scale;
        Ok(e.baseline_central / k + c.p + (1.0 - v) * k_ref)
    }

    /// Natural gas of the unabated reference plant at this plant's hydrogen
    /// output, MMSCFD. This is the one number behind the GAS line of the cost
    /// ledger, the saved upstream gas of the lifecycle view and the avoided-gas
    /// term of the decomposition (rule 1 of the boundary table of N.7). On the
    /// independent basis it is the gas that the central intensity implies; on the
    /// anchored basis it is the INL conventional-plant gas scaled to the plant.
    ///
    /// eq: N.7.3
    /// eq: N.9.2
    pub fn reference_gas_mmscfd(&self, c: &Carbon, s: f64) -> Result<f64> {
        if c.src != CarbonSrc::Plant {
            let h_s = s * c.h_scale;
            return Ok(self.ng_conv * h_s);
        }
        let cbar = ng_carbon_per_mol();
        let mmscfd_per_kmol_h = 1.0 / mmscfd_to_kmol_h(1.0);
        Ok(self.reference_carbon(c, s)? / cbar * mmscfd_per_kmol_h * s)
    }

    /// The plant-built reference at this balance's hydrogen, for the cost
    /// comparison under it: the gas to price (`CostQuery::ng_smr`) and the net
    /// abatement to divide by. `None` on the anchored carbon, which has no
    /// plant ledger.
    ///
    /// eq: N.7.1
    /// eq: N.7.3
    pub fn floor_reference(&self, b: &Balance, s: f64) -> Result<Option<FloorReference>> {
        if b.c.src != CarbonSrc::Plant {
            return Ok(None);
        }
        let ab = self.abatement(b, s)?;
        let row = ab.rows.iter().find(|r| r.id == "PLANT_FLOOR");
        Ok(match (row, ab.floor_gas_mmscfd) {
            (Some(r), Some(gas)) => Some(FloorReference {
                gas_mmscfd: gas,
                a_net: r.a_net,
                kg_kg: r.kg_kg_ref,
            }),
            _ => None,
        })
    }

    /// Annual quantities of the balance: captured, stack and condensate CO2,
    /// transport and storage emissions, direct and net abatement against each
    /// reference, intensities and the closure report.
    ///
    /// eq: N.5.2
    /// eq: N.7.3
    /// eq: N.9.1
    /// eq: N.9.2
    /// eq: N.9.3
    /// eq: N.9.4
    /// eq: N.10.1
    /// eq: N.10.2
    /// eq: N.10.3
    /// eq: N.10.4
    pub fn abatement(&self, b: &Balance, s: f64) -> Result<Abatement> {
        let c = &b.c;
        let e: Emissions = self.emissions(c, s);
        let h2_t_y = e.h2_t_y;
        let k = self.tpy(1.0, s);
        let ts = self.ts_parts();
        let tau = if c.src == CarbonSrc::Plant {
            ts.total()
        } else {
            0.0
        };
        let v = self.carbon_params.cond_vent;
        let cond_vented = if c.src == CarbonSrc::Plant {
            v * c.cond * k
        } else {
            0.0
        };
        let stack = e.atm - cond_vented;
        let mut rows = Vec::new();
        let mk = |id: &'static str, e_ref: f64| -> BaselineRow {
            let a_direct = e_ref - e.atm;
            let a_net = a_direct - e.ts_emission;
            BaselineRow {
                id,
                e_ref_t_y: e_ref,
                kg_kg_ref: e_ref / h2_t_y,
                a_direct,
                a_net,
                a_h2_direct: a_direct / h2_t_y,
                a_h2_net: a_net / h2_t_y,
                reduction: a_direct / e_ref,
            }
        };
        for kind in BaselineKind::ALL {
            let e_ref = match kind {
                BaselineKind::Inl => e.baseline,
                other => self.baseline_kg_kg(other) * h2_t_y,
            };
            rows.push(mk(kind.id(), e_ref));
        }
        let mut floor_gas = None;
        let mut decomposition = None;
        if c.src == CarbonSrc::Plant {
            let rf = self.reference_ledger()?;
            let l = rf.lines();
            let hs = c.h_scale;
            let cbar = ng_carbon_per_mol();
            let mmscfd_per_kmol_h = 1.0 / mmscfd_to_kmol_h(1.0);
            // Reference at the same hydrogen: every carbon atom of the gas leaves
            // as CO2 except the product methane and the part of the condensate CO2
            // that stays in the water (N.7.3 with no firing gas, a floor).
            let ref_floor_carbon = (l.feed - l.product - (1.0 - v) * l.condensate) * hs;
            rows.push(mk("PLANT_FLOOR", k * ref_floor_carbon));
            floor_gas = Some(rf.ng_mmscfd() * hs * s);
            // Decomposition against the central reference (N.9.2): the reference
            // carbon is E_ref / k + P + (1 - v) K', and the gas it implies follows.
            // The same two functions give the gas of the GAS line of the ledger.
            let k_ref = l.condensate * hs;
            let ref_carbon = self.reference_carbon(c, s)?;
            let captured = k * c.cap;
            let avoided_gas = k * (ref_carbon - c.f);
            let condensate = -(1.0 - v) * k * (k_ref - c.cond);
            decomposition = Some(Decomposition {
                captured,
                avoided_gas,
                condensate,
                total: captured + avoided_gas + condensate,
                n_ref_mmscfd: self.reference_gas_mmscfd(c, s)?,
                n_plant_mmscfd: c.f / cbar * mmscfd_per_kmol_h * s,
            });
        }
        let central_id = if c.src == CarbonSrc::Plant {
            self.carbon_params.baseline.id()
        } else {
            "INL_9p21"
        };
        Ok(Abatement {
            src: c.src,
            h2_t_y,
            captured_t_y: e.captured,
            stack_t_y: stack,
            cond_vented_t_y: cond_vented,
            residual_direct_t_y: e.atm,
            ts_emission_t_y: e.ts_emission,
            ts,
            tau_ts: tau,
            central: central_id,
            rows,
            intensity_kg_kg: e.atm / h2_t_y,
            closure: c_account(self, c),
            decomposition,
            floor_gas_mmscfd: floor_gas,
        })
    }
}

/// Closure report of a balance's carbon lines. The legacy lines carry no
/// condensate stream, so only the plant lines get the full account.
fn c_account(m: &Model, c: &Carbon) -> AccountReport {
    let led = CarbonLedger {
        feed: c.f,
        tube: c.t,
        co2_abs: c.co2_abs,
        captured: c.cap,
        slip: c.slip,
        tail_gas: c.g,
        recycled: c.r,
        burned: c.burn + c.exported - m.carbon_params.cond_vent * c.cond,
        product: c.p,
        condensate: c.cond,
        n2_cut: 0.0,
    };
    closure_report(
        &led.account(m.carbon_params.cond_vent),
        m.carbon_params.closure_tol,
    )
}
