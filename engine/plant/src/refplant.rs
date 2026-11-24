//! IEAGHG 2017-02 Base Case reference plant (decision D18).
//!
//! The published heat and material balance of the unabated Base Case is read
//! from the registry (entries `REF_*`, PDF p44 and p45 of the report), and the
//! balances N.R.1 to N.R.12 and N.R.15 of
//! `docs/derivation/03_reference_plant.md` are reconstructed from it and checked
//! against the printed tables. Only published flows enter: the report gives no
//! furnace efficiency, no tail-gas heating value and no combustion air, so none
//! is stored, and the published 0.8091 kg/Nm3 is the whole stack CO2, which
//! already contains the combustion of the PSA tail gas.
//!
//! Nothing here feeds a cost, carbon or results value yet. `REF_BASIS` ships as
//! 0, and the switch point of the independent basis (N.R.13, N.R.14, N.R.16 and
//! N.R.17) is wired in the integration package.

use crate::flowsheet::FlowsheetRun;
use crate::params::Reader;
use nush2_numerics::{EngineError, Result};
use nush2_props::units::{KCAL_KJ, LHV_CH4, LHV_CO, LHV_H2, MMBTU_GJ, MW_CO2};
use nush2_units::stream::{Sp, NSP};

/// Number of species of the published stream table.
pub const NSP_REF: usize = 11;

/// Species of the published stream table, in its printed order.
#[repr(usize)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefSp {
    Co2,
    Co,
    H2,
    N2,
    O2,
    Ch4,
    C2h6,
    C3h8,
    Nc4h10,
    Nc5h12,
    H2o,
}

/// Registry spelling of each species, in `RefSp` order.
pub const SPECIES_ID: [&str; NSP_REF] = [
    "CO2", "CO", "H2", "N2", "O2", "CH4", "C2H6", "C3H8", "NC4H10", "NC5H12", "H2O",
];

/// Atoms of carbon, hydrogen, oxygen and nitrogen per molecule, in `RefSp` order.
pub const ATOMS: [[f64; 4]; NSP_REF] = [
    [1.0, 0.0, 2.0, 0.0],
    [1.0, 0.0, 1.0, 0.0],
    [0.0, 2.0, 0.0, 0.0],
    [0.0, 0.0, 0.0, 2.0],
    [0.0, 0.0, 2.0, 0.0],
    [1.0, 4.0, 0.0, 0.0],
    [2.0, 6.0, 0.0, 0.0],
    [3.0, 8.0, 0.0, 0.0],
    [4.0, 10.0, 0.0, 0.0],
    [5.0, 12.0, 0.0, 0.0],
    [0.0, 2.0, 1.0, 0.0],
];

/// Printed stream descriptions, ids 1 to 15 (PDF p44 and p45).
pub const STREAM_NAMES: [&str; 15] = [
    "Natural Gas From B.L.",
    "Natural Gas feedstock to Hydrogen Plant",
    "Natural Gas fuel to burners",
    "Purified Feedstock to Pre-reformer",
    "HTS Reactor Inlet",
    "HTS Reactor Outlet",
    "PSA inlet",
    "PSA Tail gas",
    "Flue gas to ATM",
    "HP Steam export",
    "Demi Water (make up) and steam turbine condensate",
    "Hydrogen to B.L",
    "H2 Recycle",
    "HP Steam to process",
    "LP Steam To Deareator",
];

/// The nonzero mole fractions the report prints for the streams that carry a
/// mixed composition. Every other printed fraction of these streams is 0.0000.
/// Stream 9 prints CO as a note (30 mg/Nm3 max), read as 0 here.
const PRINTED_Y: [(usize, &[RefSp]); 7] = {
    use RefSp::*;
    [
        (1, &[Co2, N2, Ch4, C2h6, C3h8, Nc4h10, Nc5h12]),
        (4, &[Co2, H2, N2, Ch4, C2h6, C3h8, Nc4h10, H2o]),
        (5, &[Co2, Co, H2, N2, Ch4, H2o]),
        (6, &[Co2, Co, H2, N2, Ch4, H2o]),
        (7, &[Co2, Co, H2, N2, Ch4, H2o]),
        (8, &[Co2, Co, H2, N2, Ch4, H2o]),
        (9, &[Co2, N2, O2, H2o]),
    ]
};

// Tolerances of the checks. Each is an assumption (A) set at about twice the
// band that the rounding of the printed digits allows; the band is named.

/// Carbon from natural gas to stack, relative. Rounding band of the four-decimal
/// fractions and one-decimal flows: about 1.8 kmol C/h, 0.10 percent.
pub const TOL_CARBON_CLOSURE_REL: f64 = 2e-3;
/// PSA species, model tail gas against stream 8, kmol/h. Band about 0.44 kmol/h.
pub const TOL_PSA_SPECIES_KMOL_H: f64 = 0.5;
/// Stack CO2 against the 0.8091 kg/Nm3 of Table 2 (blocking), relative. Band of
/// the four-decimal CO2 fraction of stream 9: 0.024 percent.
pub const TOL_STACK_VS_TABLE2_REL: f64 = 1e-3;
/// Intensity against Table 2 (reported row), relative.
pub const TOL_INTENSITY_VS_TABLE2_REL: f64 = 5e-4;
/// Net power equals gross minus internal loads, relative (an identity).
pub const TOL_NET_POWER_REL: f64 = 1e-9;
/// PSA mass closure m7 = m8 + m12 + m13, kg/h (integer printed masses).
pub const TOL_PSA_MASS_KG_H: f64 = 1.0;
/// Natural gas split m1 = m2 + m3, kg/h (integer printed masses).
pub const TOL_NG_SPLIT_KG_H: f64 = 1.0;
/// Table 2 energy input and energy in product, relative (two printed decimals).
pub const TOL_TABLE2_ENERGY_REL: f64 = 1e-4;
/// Utility-sheet fuel against stream 3, relative (one printed decimal of 48.2).
pub const TOL_UTILITY_FUEL_REL: f64 = 5e-3;
/// Stream 10 against the utility-sheet steam to the power island, relative.
pub const TOL_STEAM_UTILITY_REL: f64 = 5e-3;
/// Report revenue against W_net x hours x price, relative (printed to 1 EUR).
pub const TOL_POWER_REVENUE_REL: f64 = 1e-6;
/// Annex LHV against Table 2 LHV: half a printed unit of 46.50, relative.
pub const TOL_LHV_ANNEX_REL: f64 = 0.005 / 46.50;
/// Furnace oxygen balance with the assumed air (non-blocking), relative.
pub const TOL_FURNACE_O_REL: f64 = 5e-3;
/// Half of one printed percent: "85 to 90 %" admits a recovery below 90.5 %.
pub const RECOVERY_ROUNDING: f64 = 0.005;

/// O2 and N2 shares of dry combustion air. Assumed (A): the report prints no
/// air flow or composition. Used only by the derived row I16.
pub const AIR_O2: f64 = 0.21;
pub const AIR_N2: f64 = 0.79;

/// One published stream of the Base Case.
#[derive(Debug, Clone, PartialEq)]
pub struct RefStream {
    pub id: usize,
    pub name: &'static str,
    pub t_c: f64,
    /// MPa as printed; gauge or absolute is not marked.
    pub p_mpa: f64,
    pub kmol_h: f64,
    pub kg_h: f64,
    /// Mole fractions, wet, in `RefSp` order.
    pub y: [f64; NSP_REF],
}

impl RefStream {
    /// Species flows, kmol/h. eq: N.R.1
    pub fn species(&self) -> [f64; NSP_REF] {
        let mut n = [0.0; NSP_REF];
        for (i, v) in n.iter_mut().enumerate() {
            *v = self.kmol_h * self.y[i];
        }
        n
    }
}

/// Element flows (C, H, O, N) of species flows, kmol atoms/h. eq: N.R.1
pub fn elements(n: &[f64; NSP_REF]) -> [f64; 4] {
    let mut e = [0.0; 4];
    for (i, ni) in n.iter().enumerate() {
        for (k, ek) in e.iter_mut().enumerate() {
            *ek += ni * ATOMS[i][k];
        }
    }
    e
}

/// Carbon flow of species flows, kmol C/h. eq: N.R.1
pub fn carbon_of(n: &[f64; NSP_REF]) -> f64 {
    elements(n)[0]
}

/// Lower heating value of species flows in kmol/h with the engine constants
/// (H2, CO and CH4 only, the combustibles of the tail gas), MW. eq: N.R.3
pub fn lhv_mw(n: &[f64; NSP_REF]) -> f64 {
    (n[RefSp::H2 as usize] * LHV_H2
        + n[RefSp::Co as usize] * LHV_CO
        + n[RefSp::Ch4 as usize] * LHV_CH4)
        / 3600.0
}

/// Power balance inputs, Table 2 (PDF p16) and the utility sheet (PDF p48).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RefPower {
    pub gross_mwe: f64,
    pub h2_plant_mwe: f64,
    pub aux_island_kw: f64,
    pub aux_bop_kw: f64,
    pub net_published_mwe: f64,
}

/// Which reference the independent basis uses (`REF_BASIS`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefBasis {
    /// 0: the independent basis of the frozen snapshot.
    Historical,
    /// 1: the reconstructed IEAGHG Base Case.
    Reconstructed,
}

/// Power-island share of the SMR island (decision D23).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IslandSplit {
    /// Total plant cost of the IEAGHG power island, M EUR (Sheet 85, PDF p222).
    pub power_island_tpc_eurm: f64,
    /// Remove that share from the NUS-H2 side only (capital only).
    pub nush2_excludes_power_island: bool,
}

/// Everything read from the registry.
#[derive(Debug, Clone, PartialEq)]
pub struct RefPlantParams {
    /// Streams 1 to 15, index id - 1.
    pub streams: Vec<RefStream>,
    pub h2_nm3_h: f64,
    pub nm3_kmol: f64,
    pub ng_lhv_mj_kg: f64,
    pub ng_lhv_annex_mj_kg: f64,
    pub ng_hhv_mj_kg: f64,
    pub h2_lhv_mj_kg: f64,
    pub energy_in_mw: f64,
    pub energy_h2_mw: f64,
    pub power: RefPower,
    pub radiant_mmkcal_h: f64,
    pub utility_fuel_mmkcal_h: f64,
    pub steam_neighbour_t_h: f64,
    pub steam_utility_t_h: f64,
    pub export_price_ratio: f64,
    pub price_eur_mwh: f64,
    pub hours: f64,
    pub revenue_eur_y: f64,
    /// Printed recovery range (low, high) and its central value.
    pub psa_recovery_range: (f64, f64),
    pub psa_recovery_central: f64,
    /// Published 0.8091 kg/Nm3 (`BASELINE_IEAGHG_KG_NM3`, Table 2).
    pub co2_kg_nm3_published: f64,
    /// Engine gas heating value, Btu/scf (`NG_HHV`), the unit of the GAS line.
    pub ng_hhv_btu_scf: f64,
    pub basis: RefBasis,
    pub island: IslandSplit,
}

fn range(param: &str, msg: String) -> EngineError {
    EngineError::Range {
        param: param.into(),
        msg,
    }
}

fn read_flag(r: &mut Reader<'_>, id: &str) -> Result<bool> {
    let v = r.get(id)?;
    if v == 0.0 {
        Ok(false)
    } else if v == 1.0 {
        Ok(true)
    } else {
        Err(range(id, format!("{v} is not a switch; use 0 or 1")))
    }
}

fn positive(r: &mut Reader<'_>, id: &str) -> Result<f64> {
    let v = r.get(id)?;
    if !(v > 0.0 && v.is_finite()) {
        return Err(range(id, format!("{v} must be positive")));
    }
    Ok(v)
}

impl RefPlantParams {
    /// Read and check the reference entries. A value outside its physical range
    /// is refused, never replaced.
    pub fn read(r: &mut Reader<'_>) -> Result<Self> {
        let mut streams: Vec<RefStream> = Vec::with_capacity(15);
        for id in 1..=15usize {
            let f = |field: &str| format!("REF_S{id:02}_{field}");
            let t_c = r.get(&f("T_C"))?;
            let p_mpa = r.get(&f("P_MPA"))?;
            let kmol_h = positive(r, &f("KMOL_H"))?;
            let kg_h = positive(r, &f("KG_H"))?;
            let mut y = [0.0; NSP_REF];
            match id {
                2 | 3 => y = streams[0].y,
                10 | 11 | 14 | 15 => y[RefSp::H2o as usize] = 1.0,
                12 | 13 => y[RefSp::H2 as usize] = 1.0,
                _ => {
                    let (_, printed) = PRINTED_Y
                        .iter()
                        .find(|(k, _)| *k == id)
                        .ok_or_else(|| range(&f("Y"), "no printed composition".into()))?;
                    for sp in printed.iter() {
                        let key = f(&format!("Y_{}", SPECIES_ID[*sp as usize]));
                        let v = r.get(&key)?;
                        if !(v > 0.0 && v <= 1.0) {
                            return Err(range(&key, format!("{v} must lie in (0, 1]")));
                        }
                        y[*sp as usize] = v;
                    }
                    let sum: f64 = y.iter().sum();
                    if (sum - 1.0).abs() > 1e-3 {
                        return Err(range(&f("Y"), format!("fractions add to {sum}")));
                    }
                }
            }
            streams.push(RefStream {
                id,
                name: STREAM_NAMES[id - 1],
                t_c,
                p_mpa,
                kmol_h,
                kg_h,
                y,
            });
        }
        let steam_neighbour_t_h = r.get("REF_STEAM_NEIGHBOUR_T_H")?;
        if steam_neighbour_t_h != 0.0 {
            return Err(range(
                "REF_STEAM_NEIGHBOUR_T_H",
                format!(
                    "{steam_neighbour_t_h} is not implemented: the Base Case exports no steam \
                     across the battery limit (PDF p34, p48)"
                ),
            ));
        }
        let central = r.get("REF_PSA_H2_RECOVERY_RANGE")?;
        let (low, high) = r
            .band("REF_PSA_H2_RECOVERY_RANGE")
            .ok_or_else(|| range("REF_PSA_H2_RECOVERY_RANGE", "needs low and high".into()))?;
        if !(0.0 < low && low <= central && central <= high && high <= 1.0) {
            return Err(range(
                "REF_PSA_H2_RECOVERY_RANGE",
                format!("{low} <= {central} <= {high} must lie in (0, 1]"),
            ));
        }
        let basis = if read_flag(r, "REF_BASIS")? {
            RefBasis::Reconstructed
        } else {
            RefBasis::Historical
        };

        Ok(Self {
            streams,
            h2_nm3_h: positive(r, "REF_H2_NM3_H")?,
            nm3_kmol: positive(r, "REF_NM3_KMOL")?,
            ng_lhv_mj_kg: positive(r, "REF_NG_LHV_MJ_KG")?,
            ng_lhv_annex_mj_kg: positive(r, "REF_NG_LHV_ANNEX_MJ_KG")?,
            ng_hhv_mj_kg: positive(r, "REF_NG_HHV_MJ_KG")?,
            h2_lhv_mj_kg: positive(r, "REF_H2_LHV_MJ_KG")?,
            energy_in_mw: positive(r, "REF_ENERGY_IN_MW")?,
            energy_h2_mw: positive(r, "REF_ENERGY_H2_MW")?,
            power: RefPower {
                gross_mwe: positive(r, "REF_POWER_GROSS_MWE")?,
                h2_plant_mwe: positive(r, "REF_POWER_H2_PLANT_MWE")?,
                aux_island_kw: positive(r, "REF_POWER_AUX_ISLAND_KW")?,
                aux_bop_kw: positive(r, "REF_POWER_AUX_BOP_KW")?,
                net_published_mwe: r.get("REF_POWER_NET_MWE")?,
            },
            radiant_mmkcal_h: positive(r, "REF_RADIANT_MMKCAL_H")?,
            utility_fuel_mmkcal_h: positive(r, "REF_UTILITY_FUEL_MMKCAL_H")?,
            steam_neighbour_t_h,
            steam_utility_t_h: positive(r, "REF_STEAM_UTILITY_T_H")?,
            export_price_ratio: positive(r, "REF_POWER_EXPORT_PRICE_RATIO")?,
            price_eur_mwh: positive(r, "REF_POWER_PRICE_EUR_MWH")?,
            hours: positive(r, "REF_OPERATING_HOURS")?,
            revenue_eur_y: positive(r, "REF_POWER_REVENUE_EUR_Y")?,
            psa_recovery_range: (low, high),
            psa_recovery_central: central,
            co2_kg_nm3_published: positive(r, "BASELINE_IEAGHG_KG_NM3")?,
            ng_hhv_btu_scf: positive(r, "NG_HHV")?,
            basis,
            island: IslandSplit {
                power_island_tpc_eurm: positive(r, "SMR_ISLAND_POWER_ISLAND_TPC_EURM")?,
                nush2_excludes_power_island: read_flag(
                    r,
                    "SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND",
                )?,
            },
        })
    }

    /// Published stream by id (1 to 15).
    pub fn stream(&self, id: usize) -> &RefStream {
        &self.streams[id - 1]
    }
}

/// One check of the reconstruction against a printed value.
#[derive(Debug, Clone, PartialEq)]
pub struct RefCheck {
    pub id: &'static str,
    pub label: &'static str,
    pub published: f64,
    pub calculated: f64,
    /// calculated / published - 1, or calculated - published when `absolute`.
    pub residual: f64,
    pub tol: f64,
    pub absolute: bool,
    pub blocking: bool,
}

impl RefCheck {
    fn rel(
        id: &'static str,
        label: &'static str,
        published: f64,
        calculated: f64,
        tol: f64,
        blocking: bool,
    ) -> Self {
        Self {
            id,
            label,
            published,
            calculated,
            residual: calculated / published - 1.0,
            tol,
            absolute: false,
            blocking,
        }
    }

    fn abs(
        id: &'static str,
        label: &'static str,
        published: f64,
        calculated: f64,
        tol: f64,
        blocking: bool,
    ) -> Self {
        Self {
            id,
            label,
            published,
            calculated,
            residual: calculated - published,
            tol,
            absolute: true,
            blocking,
        }
    }

    pub fn passes(&self) -> bool {
        self.residual.abs() <= self.tol
    }
}

/// Component-wise PSA balance (N.R.2) at the published scale.
#[derive(Debug, Clone, PartialEq)]
pub struct PsaBalance {
    /// H2 recovery R = (n12 + n13) / (n7 y7,H2), product plus recycle.
    pub recovery: f64,
    /// Model tail gas, kmol/h: every impurity rejected, the H2 not recovered.
    pub tail: [f64; NSP_REF],
    /// Model tail gas minus published stream 8, kmol/h.
    pub residual: [f64; NSP_REF],
    pub max_abs_residual: f64,
    /// m7 - m8 - m12 - m13, kg/h.
    pub mass_closure_kg_h: f64,
    /// Elements C, H, O, N of stream 7 minus streams 8, 12 and 13, kmol/h.
    pub element_residual: [f64; 4],
}

/// Furnace heat release (N.R.3 to N.R.6). No efficiency is derived.
#[derive(Debug, Clone, PartialEq)]
pub struct FurnaceBalance {
    pub q_tail_mw: f64,
    pub q_tail_mj_kmol: f64,
    pub q_tail_mj_kg: f64,
    pub q_tail_mj_nm3: f64,
    pub q_ng_supp_mw: f64,
    pub q_fired_mw: f64,
    pub tail_share: f64,
    pub q_rad_mw: f64,
    /// Radiant duty over fired duty, an indicator only.
    pub radiant_fraction: f64,
    /// Utility-sheet fuel of the hydrogen plant, MW.
    pub q_utility_fuel_mw: f64,
}

/// Stack CO2 and its breakdown by origin (N.R.7, N.R.8), kmol/h unless named.
#[derive(Debug, Clone, PartialEq)]
pub struct Co2Breakdown {
    pub stack_kmol_h: f64,
    pub stack_t_h: f64,
    /// Carbon from natural gas (stream 1) to stack, stack / C1 - 1.
    pub c_residual_rel: f64,
    /// A1 CO2 of the tail gas, passes the burners unburnt.
    pub a1: f64,
    /// A2 CO of the tail gas, burnt.
    pub a2: f64,
    /// A3 CH4 of the tail gas, burnt.
    pub a3: f64,
    /// B1 CO2 of the supplementary natural gas.
    pub b1: f64,
    /// B2 hydrocarbon carbon of the supplementary natural gas, burnt.
    pub b2: f64,
    /// Stack over the sum of the parts, applied to every part so they add up.
    pub rho: f64,
    /// H2 product, Nm3/h, from stream 12 and the normal volume.
    pub h2_nm3_h: f64,
}

impl Co2Breakdown {
    fn kg_nm3(&self, kmol_h: f64) -> f64 {
        self.rho * kmol_h * MW_CO2 / self.h2_nm3_h
    }

    /// Feedstock carbon that leaves in the tail gas (A = A1 + A2 + A3), kg/Nm3.
    pub fn a_kg_nm3(&self) -> f64 {
        self.kg_nm3(self.a1 + self.a2 + self.a3)
    }

    /// Supplementary natural gas (B = B1 + B2), kg/Nm3.
    pub fn b_kg_nm3(&self) -> f64 {
        self.kg_nm3(self.b1 + self.b2)
    }

    /// Largest removal of a step that washes CO2 already in the tail gas,
    /// upstream of the burners: A1 only (critique C08), kg/Nm3.
    pub fn bound_co2_wash_kg_nm3(&self) -> f64 {
        self.kg_nm3(self.a1)
    }

    /// Largest removal of a step that also converts the CO and CH4 of the tail
    /// gas before combustion: A, kg/Nm3.
    pub fn bound_tail_conversion_kg_nm3(&self) -> f64 {
        self.a_kg_nm3()
    }

    /// A step on the flue gas sees the whole stack, A + B, kg/Nm3.
    pub fn bound_flue_gas_kg_nm3(&self) -> f64 {
        self.a_kg_nm3() + self.b_kg_nm3()
    }
}

/// Natural gas energy (N.R.10): converted by energy, the volume is a cross-check.
#[derive(Debug, Clone, PartialEq)]
pub struct GasEnergy {
    pub q_lhv_mw: f64,
    pub e_hhv_gj_h: f64,
    pub q_hhv_mw: f64,
    /// The same energy in the engine's gas unit at x = 1, MMSCFD.
    pub n_ref_mmscfd: f64,
}

/// Steam and power (N.R.11).
#[derive(Debug, Clone, PartialEq)]
pub struct SteamPower {
    pub w_aux_mwe: f64,
    pub w_net_mwe: f64,
    /// Gross power per t of turbine steam, kWh/t, with stream 10.
    pub w_spec_kwh_t: f64,
    /// The same with the utility-sheet 45.9 t/h.
    pub w_spec_utility_kwh_t: f64,
    pub revenue_eur_y: f64,
    pub steam_neighbour_t_h: f64,
}

/// The reconstructed Base Case at the published scale.
#[derive(Debug, Clone, PartialEq)]
pub struct RefBalance {
    /// Species flows of streams 1 to 15, kmol/h (index id - 1).
    pub species: Vec<[f64; NSP_REF]>,
    /// Carbon flows of streams 1 to 15, kmol C/h.
    pub carbon: Vec<f64>,
    pub psa: PsaBalance,
    pub furnace: FurnaceBalance,
    pub co2: Co2Breakdown,
    /// Stack CO2 per kg H2 product (N.R.9), kg/kg.
    pub intensity_kg_kg: f64,
    /// Stack CO2 per Nm3 H2 product (N.R.9), kg/Nm3.
    pub intensity_kg_nm3: f64,
    pub gas: GasEnergy,
    pub steam_power: SteamPower,
    pub checks: Vec<RefCheck>,
}

/// Reference quantities scaled to a plant hydrogen output (N.R.12).
#[derive(Debug, Clone, PartialEq)]
pub struct RefScaled {
    pub x: f64,
    pub intensity_kg_kg: f64,
    pub gas_mmscfd_eq: f64,
    pub stack_co2_t_h: f64,
    pub steam_neighbour_t_h: f64,
    /// Power bought from the grid, MWe; negative is a net export.
    pub power_buy_mwe: f64,
}

/// Class of a row of the reference stream table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowClass {
    /// Printed in the report (scaled by x when x is not 1).
    Published,
    /// Derived from published rows and an assumption; never enters a cost or
    /// carbon equation.
    DerivedUnpublished,
}

/// One row of the reference stream table (tags I01 to I18).
#[derive(Debug, Clone, PartialEq)]
pub struct RefStreamRow {
    pub tag: String,
    pub name: &'static str,
    pub t_c: Option<f64>,
    pub p_mpa: Option<f64>,
    pub kmol_h: f64,
    pub kg_h: Option<f64>,
    pub y: [f64; NSP_REF],
    pub class: RowClass,
    pub pdf_page: Option<u32>,
    pub note: &'static str,
}

/// A tail gas the PSA would give at another recovery with the published feed.
#[derive(Debug, Clone, PartialEq)]
pub struct PsaTail {
    pub recovery: f64,
    pub tail: [f64; NSP_REF],
    pub h2_out_kmol_h: f64,
}

/// The reference plant: registry inputs and the balance they give.
#[derive(Debug, Clone, PartialEq)]
pub struct RefPlant {
    pub p: RefPlantParams,
    pub bal: RefBalance,
}

impl RefPlant {
    /// Build and run the balance once. A blocking check that fails refuses the
    /// build, which protects the engine against a bad registry edit.
    pub fn new(p: RefPlantParams) -> Result<Self> {
        let bal = balance(&p);
        if let Some(c) = bal.checks.iter().find(|c| c.blocking && !c.passes()) {
            return Err(range(
                c.id,
                format!(
                    "{}: calculated {} against published {}, residual {:e} beyond {:e}",
                    c.label, c.calculated, c.published, c.residual, c.tol
                ),
            ));
        }
        Ok(Self { p, bal })
    }

    /// The balance at the published scale.
    pub fn balance(&self) -> &RefBalance {
        &self.bal
    }

    /// Check by id.
    pub fn check(&self, id: &str) -> Option<&RefCheck> {
        self.bal.checks.iter().find(|c| c.id == id)
    }

    /// PSA tail gas at another H2 recovery with the published feed held fixed.
    ///
    /// Test only (critique C09): with the feed fixed, a change of R changes the
    /// hydrogen product, so the 100,000 Nm3/h basis moves. Never wire it to a
    /// sensitivity group or to the headline; a recovery sensitivity must scale
    /// the whole plant to constant hydrogen first.
    ///
    /// eq: N.R.2
    pub fn psa_tail(&self, recovery: f64) -> PsaTail {
        let feed = self.bal.species[6];
        let mut tail = feed;
        let h2 = RefSp::H2 as usize;
        tail[h2] = (1.0 - recovery) * feed[h2];
        PsaTail {
            recovery,
            tail,
            h2_out_kmol_h: recovery * feed[h2],
        }
    }

    /// Supplementary natural gas, t/h, that keeps the published fired duty for a
    /// changed tail-gas heat.
    ///
    /// Test only (critique C09): the constant fired duty per kg of H2 is an
    /// assumption (A), and the reformer duty and the steam raised both change
    /// with the tail gas. Never wire it to a sensitivity group or the headline.
    ///
    /// eq: N.R.5
    pub fn supp_ng_t_h(&self, q_tail_mw: f64) -> f64 {
        (self.bal.furnace.q_fired_mw - q_tail_mw).max(0.0) * 3600.0 / self.p.ng_lhv_mj_kg / 1000.0
    }

    /// Scale factor x of a plant hydrogen output, kg/h. eq: N.R.12
    pub fn scale(&self, h2_kg_h: f64) -> Result<f64> {
        if !(h2_kg_h > 0.0 && h2_kg_h.is_finite()) {
            return Err(range("h2_kg_h", format!("{h2_kg_h} must be positive")));
        }
        Ok(h2_kg_h / self.p.stream(12).kg_h)
    }

    /// Extensive reference quantities scaled to a plant hydrogen output, kg/h;
    /// the intensive ones do not move. eq: N.R.12
    pub fn scaled(&self, h2_kg_h: f64) -> Result<RefScaled> {
        let x = self.scale(h2_kg_h)?;
        Ok(RefScaled {
            x,
            intensity_kg_kg: self.bal.intensity_kg_kg,
            gas_mmscfd_eq: x * self.bal.gas.n_ref_mmscfd,
            stack_co2_t_h: x * self.bal.co2.stack_t_h,
            steam_neighbour_t_h: x * self.p.steam_neighbour_t_h,
            power_buy_mwe: -x * self.bal.steam_power.w_net_mwe,
        })
    }

    /// The reference stream table at a plant hydrogen output, kg/h: the
    /// published rows I01 to I15 times x, and three derived rows I16 to I18 that
    /// never enter a cost or carbon equation.
    pub fn stream_table(&self, h2_kg_h: f64) -> Result<Vec<RefStreamRow>> {
        let x = self.scale(h2_kg_h)?;
        let mut rows: Vec<RefStreamRow> = self
            .p
            .streams
            .iter()
            .map(|s| RefStreamRow {
                tag: format!("I{:02}", s.id),
                name: s.name,
                t_c: Some(s.t_c),
                p_mpa: Some(s.p_mpa),
                kmol_h: x * s.kmol_h,
                kg_h: Some(x * s.kg_h),
                y: s.y,
                class: RowClass::Published,
                pdf_page: Some(if s.id <= 12 { 44 } else { 45 }),
                note: match s.id {
                    2 | 3 => "composition printed equal to stream 1",
                    9 => "CO printed as a limit, 30 mg/Nm3 max, read as 0",
                    12 | 13 => "printed 0.9999+ hydrogen, taken as pure hydrogen (A)",
                    14 => "19.9 t/h above the reformer steam implied by the H and O balances; unresolved in the report",
                    _ => "",
                },
            })
            .collect();
        for d in derived_rows(&self.bal) {
            rows.push(RefStreamRow {
                kmol_h: x * d.kmol_h,
                ..d
            });
        }
        Ok(rows)
    }
}

fn derived_rows(b: &RefBalance) -> Vec<RefStreamRow> {
    let s = |id: usize| b.species[id - 1];
    let (n2, h2o, o2) = (RefSp::N2 as usize, RefSp::H2o as usize, RefSp::O2 as usize);
    // I16: combustion air. Nitrogen balance over the furnace with an assumed
    // 21 percent O2 in dry air; the humidity follows from the hydrogen balance.
    let air_n2 = s(9)[n2] - s(3)[n2] - s(8)[n2];
    let dry = air_n2 / AIR_N2;
    let h_fuel = elements(&s(3))[1] + elements(&s(8))[1];
    let humidity = s(9)[h2o] - h_fuel / 2.0;
    let total = dry + humidity;
    let mut y_air = [0.0; NSP_REF];
    y_air[o2] = AIR_O2 * dry / total;
    y_air[n2] = AIR_N2 * dry / total;
    y_air[h2o] = humidity / total;
    // I17: second steam addition ahead of the reformer, from the hydrogen
    // balance of streams 4 and 5 (the oxygen balance gives a lower value).
    let steam2 = (elements(&s(5))[1] - elements(&s(4))[1]) / 2.0;
    // I18: process condensate removed ahead of the PSA.
    let cond = s(6)[h2o] - s(7)[h2o];
    let mut water = [0.0; NSP_REF];
    water[h2o] = 1.0;
    let row = |tag: &str, name, kmol_h, y, note| RefStreamRow {
        tag: tag.into(),
        name,
        t_c: None,
        p_mpa: None,
        kmol_h,
        kg_h: None,
        y,
        class: RowClass::DerivedUnpublished,
        pdf_page: None,
        note,
    };
    vec![
        row(
            "I16",
            "Combustion air (derived)",
            total,
            y_air,
            "N balance with 21 percent O2 in dry air (A); humidity from the H balance",
        ),
        row(
            "I17",
            "Second steam addition to the reformer (derived)",
            steam2,
            water,
            "H balance of streams 4 and 5",
        ),
        row(
            "I18",
            "Process condensate ahead of the PSA (derived)",
            cond,
            water,
            "water of stream 6 minus water of stream 7",
        ),
    ]
}

/// The reconstruction N.R.1 to N.R.11 and its checks.
#[allow(clippy::needless_range_loop)]
fn balance(p: &RefPlantParams) -> RefBalance {
    let species: Vec<[f64; NSP_REF]> = p.streams.iter().map(RefStream::species).collect();
    let carbon: Vec<f64> = species.iter().map(carbon_of).collect();
    let s = |id: usize| species[id - 1];
    let st = |id: usize| p.stream(id);
    let h2 = RefSp::H2 as usize;

    // N.R.2: component-wise PSA balance.
    let feed = s(7);
    let product = st(12).kmol_h + st(13).kmol_h;
    let recovery = product / feed[h2];
    let mut tail = feed;
    tail[h2] = (1.0 - recovery) * feed[h2];
    let mut residual = [0.0; NSP_REF];
    let mut max_abs_residual: f64 = 0.0;
    for i in 0..NSP_REF {
        residual[i] = tail[i] - s(8)[i];
        max_abs_residual = max_abs_residual.max(residual[i].abs());
    }
    let mass_closure_kg_h = st(7).kg_h - st(8).kg_h - st(12).kg_h - st(13).kg_h;
    let (e7, e8, e12, e13) = (
        elements(&s(7)),
        elements(&s(8)),
        elements(&s(12)),
        elements(&s(13)),
    );
    let mut element_residual = [0.0; 4];
    for k in 0..4 {
        element_residual[k] = e7[k] - e8[k] - e12[k] - e13[k];
    }
    let psa = PsaBalance {
        recovery,
        tail,
        residual,
        max_abs_residual,
        mass_closure_kg_h,
        element_residual,
    };

    // N.R.3 to N.R.6: furnace heat release.
    let q_tail_mw = lhv_mw(&s(8));
    let q_ng_supp_mw = st(3).kg_h * p.ng_lhv_mj_kg / 3600.0;
    let q_fired_mw = q_tail_mw + q_ng_supp_mw;
    let mmkcal_to_mw = |v: f64| v * 1e6 * KCAL_KJ / 3.6e6;
    let q_rad_mw = mmkcal_to_mw(p.radiant_mmkcal_h);
    let furnace = FurnaceBalance {
        q_tail_mw,
        q_tail_mj_kmol: q_tail_mw * 3600.0 / st(8).kmol_h,
        q_tail_mj_kg: q_tail_mw * 3600.0 / st(8).kg_h,
        q_tail_mj_nm3: q_tail_mw * 3600.0 / (st(8).kmol_h * p.nm3_kmol),
        q_ng_supp_mw,
        q_fired_mw,
        tail_share: q_tail_mw / q_fired_mw,
        q_rad_mw,
        radiant_fraction: q_rad_mw / q_fired_mw,
        q_utility_fuel_mw: mmkcal_to_mw(p.utility_fuel_mmkcal_h),
    };

    // N.R.7 and N.R.8: stack CO2 and its origin.
    let co2 = RefSp::Co2 as usize;
    let stack_kmol_h = s(9)[co2];
    let stack_t_h = stack_kmol_h * MW_CO2 / 1000.0;
    let (a1, a2, a3) = (
        s(8)[co2],
        s(8)[RefSp::Co as usize],
        s(8)[RefSp::Ch4 as usize],
    );
    let b1 = s(3)[co2];
    let b2 = carbon[2] - b1;
    let h2_nm3_h = st(12).kmol_h * p.nm3_kmol;
    let co2b = Co2Breakdown {
        stack_kmol_h,
        stack_t_h,
        c_residual_rel: stack_kmol_h / carbon[0] - 1.0,
        a1,
        a2,
        a3,
        b1,
        b2,
        rho: stack_kmol_h / (a1 + a2 + a3 + b1 + b2),
        h2_nm3_h,
    };

    // N.R.9: the central unabated intensity, one route.
    let intensity_kg_kg = stack_t_h * 1000.0 / st(12).kg_h;
    let intensity_kg_nm3 = stack_t_h * 1000.0 / h2_nm3_h;

    // N.R.10: gas energy by energy.
    let e_hhv_gj_h = st(1).kg_h * p.ng_hhv_mj_kg / 1000.0;
    let gas = GasEnergy {
        q_lhv_mw: st(1).kg_h * p.ng_lhv_mj_kg / 3600.0,
        e_hhv_gj_h,
        q_hhv_mw: e_hhv_gj_h / 3.6,
        n_ref_mmscfd: e_hhv_gj_h * 24.0 / (p.ng_hhv_btu_scf * MMBTU_GJ),
    };

    // N.R.11: power balance.
    let w = &p.power;
    let w_aux_mwe = (w.aux_island_kw + w.aux_bop_kw) / 1000.0;
    let w_net_mwe = w.gross_mwe - w.h2_plant_mwe - w_aux_mwe;
    let steam_power = SteamPower {
        w_aux_mwe,
        w_net_mwe,
        w_spec_kwh_t: w.gross_mwe * 1000.0 / (st(10).kg_h / 1000.0),
        w_spec_utility_kwh_t: w.gross_mwe * 1000.0 / p.steam_utility_t_h,
        revenue_eur_y: w_net_mwe * p.hours * p.price_eur_mwh,
        steam_neighbour_t_h: p.steam_neighbour_t_h,
    };

    // Furnace oxygen balance with the derived air (non-blocking, assumed air).
    let (n2i, h2oi) = (RefSp::N2 as usize, RefSp::H2o as usize);
    let air_dry = (s(9)[n2i] - s(3)[n2i] - s(8)[n2i]) / AIR_N2;
    let humidity = s(9)[h2oi] - (elements(&s(3))[1] + elements(&s(8))[1]) / 2.0;
    let o_in = 2.0 * AIR_O2 * air_dry + humidity + elements(&s(3))[2] + elements(&s(8))[2];
    let o_out = elements(&s(9))[2];

    let (low, high) = p.psa_recovery_range;
    let in_range = if recovery >= low && recovery < high + RECOVERY_ROUNDING {
        0.0
    } else {
        1.0
    };
    let checks = vec![
        RefCheck::rel(
            "REF_CARBON_CLOSURE_NG_TO_STACK",
            "carbon of stream 1 against the CO2 of stream 9",
            carbon[0],
            stack_kmol_h,
            TOL_CARBON_CLOSURE_REL,
            true,
        ),
        RefCheck::abs(
            "REF_PSA_SPECIES_MAX_RESIDUAL",
            "largest species residual of the model tail gas against stream 8, kmol/h",
            0.0,
            max_abs_residual,
            TOL_PSA_SPECIES_KMOL_H,
            true,
        ),
        RefCheck::rel(
            "REF_STACK_CO2_VS_TABLE_2",
            "stack CO2 per Nm3 of H2 against Table 2",
            p.co2_kg_nm3_published,
            intensity_kg_nm3,
            TOL_STACK_VS_TABLE2_REL,
            true,
        ),
        RefCheck::rel(
            "REF_INTENSITY_VS_TABLE_2",
            "stack CO2 per kg of H2 against Table 2 per kg (0.8091 x 100,000 / 8,994)",
            p.co2_kg_nm3_published * p.h2_nm3_h / st(12).kg_h,
            intensity_kg_kg,
            TOL_INTENSITY_VS_TABLE2_REL,
            false,
        ),
        RefCheck::rel(
            "REF_TABLE2_NET_POWER",
            "gross power minus internal loads against the export power of Table 2",
            w.net_published_mwe,
            w_net_mwe,
            TOL_NET_POWER_REL,
            true,
        ),
        RefCheck::abs(
            "REF_PSA_MASS_CLOSURE",
            "PSA mass, stream 7 minus streams 8, 12 and 13, kg/h",
            0.0,
            mass_closure_kg_h,
            TOL_PSA_MASS_KG_H,
            true,
        ),
        RefCheck::abs(
            "REF_NG_STREAM_SPLIT_MASS",
            "natural gas mass, stream 1 minus streams 2 and 3, kg/h",
            0.0,
            st(1).kg_h - st(2).kg_h - st(3).kg_h,
            TOL_NG_SPLIT_KG_H,
            true,
        ),
        RefCheck::rel(
            "REF_TABLE2_ENERGY_IN",
            "stream 1 times the Table 2 LHV against the total energy input",
            p.energy_in_mw,
            gas.q_lhv_mw,
            TOL_TABLE2_ENERGY_REL,
            false,
        ),
        RefCheck::rel(
            "REF_TABLE2_ENERGY_H2",
            "stream 12 times the H2 LHV against the energy in product",
            p.energy_h2_mw,
            st(12).kg_h * p.h2_lhv_mj_kg / 3600.0,
            TOL_TABLE2_ENERGY_REL,
            false,
        ),
        RefCheck::rel(
            "REF_UTILITY_FUEL_VS_STREAM_3",
            "utility-sheet fuel against stream 3 times the Table 2 LHV",
            q_ng_supp_mw,
            furnace.q_utility_fuel_mw,
            TOL_UTILITY_FUEL_REL,
            false,
        ),
        RefCheck::rel(
            "REF_STEAM_STREAM_10_VS_UTILITY",
            "stream 10 against the utility-sheet HP steam to the power island",
            p.steam_utility_t_h,
            st(10).kg_h / 1000.0,
            TOL_STEAM_UTILITY_REL,
            false,
        ),
        RefCheck::rel(
            "REF_POWER_REVENUE",
            "net power times hours times price against the Table 4 revenue",
            p.revenue_eur_y,
            steam_power.revenue_eur_y,
            TOL_POWER_REVENUE_REL,
            false,
        ),
        RefCheck::rel(
            "REF_NG_LHV_ANNEX_VS_TABLE_2",
            "specification LHV against the Table 2 LHV",
            p.ng_lhv_mj_kg,
            p.ng_lhv_annex_mj_kg,
            TOL_LHV_ANNEX_REL,
            false,
        ),
        RefCheck::abs(
            "REF_PSA_RECOVERY_IN_RANGE",
            "derived recovery outside the printed 85 to 90 percent, read to the whole percent (1 = outside)",
            0.0,
            in_range,
            0.0,
            false,
        ),
        RefCheck::abs(
            "REF_STEAM_TO_NEIGHBOUR_ZERO",
            "steam crossing the battery limit, t/h",
            0.0,
            p.steam_neighbour_t_h,
            0.0,
            false,
        ),
        RefCheck::rel(
            "REF_FURNACE_O_BALANCE",
            "furnace oxygen, flue gas against air, fuel and tail gas, with the assumed air",
            o_in,
            o_out,
            TOL_FURNACE_O_REL,
            false,
        ),
    ];

    RefBalance {
        species,
        carbon,
        psa,
        furnace,
        co2: co2b,
        intensity_kg_kg,
        intensity_kg_nm3,
        gas,
        steam_power,
        checks,
    }
}

/// Where the LHV of the plant's own PSA reject S14 goes (N.R.15). The plant has
/// no furnace: every MW of S14 is recycled as reformer feed (S03) or as PSA feed
/// (S21), or burned in N2-BURN (S15), and the only claim on the steam system is
/// Q_N2_BURN, bounded by the LHV of S15. MW unless named.
#[derive(Debug, Clone, PartialEq)]
pub struct HeatRegister {
    pub lhv_s14_mw: f64,
    pub lhv_s03_mw: f64,
    pub lhv_s21_mw: f64,
    pub lhv_s15_mw: f64,
    pub claim_recycled_mw: f64,
    pub claim_burned_mw: f64,
    pub claim_steam_system_mw: f64,
    /// Largest relative difference of a species flow, S14 against S03 + S21 + S15.
    pub species_closure_max_rel: f64,
    /// (S03 + S21 + S15) LHV over S14 LHV.
    pub claim_sum_over_s14: f64,
}

fn plant_lhv_mw(n: &[f64; NSP]) -> f64 {
    // mol/s times kJ/mol is kW.
    (n[Sp::H2 as usize] * LHV_H2 + n[Sp::Co as usize] * LHV_CO + n[Sp::Ch4 as usize] * LHV_CH4)
        / 1000.0
}

/// The tail-gas heat register of a plant run with the cryogenic recycle.
///
/// eq: N.R.15
#[allow(clippy::needless_range_loop)]
pub fn heat_register(run: &FlowsheetRun) -> Result<HeatRegister> {
    let n14 = run.gas("S14")?.n;
    let n03 = run.gas("S03")?.n;
    let n21 = run.gas("S21")?.n;
    let n15 = run.gas("S15")?.n;
    let mut worst: f64 = 0.0;
    for i in 0..NSP {
        let out = n03[i] + n21[i] + n15[i];
        if n14[i] > 0.0 {
            worst = worst.max((out - n14[i]).abs() / n14[i]);
        } else if out != 0.0 {
            worst = f64::INFINITY;
        }
    }
    let q_burn = run
        .duties
        .iter()
        .find(|d| d.tag == "Q_N2_BURN")
        .ok_or_else(|| range("Q_N2_BURN", format!("no N2-BURN duty in run {}", run.id)))?;
    let (l14, l03, l21, l15) = (
        plant_lhv_mw(&n14),
        plant_lhv_mw(&n03),
        plant_lhv_mw(&n21),
        plant_lhv_mw(&n15),
    );
    Ok(HeatRegister {
        lhv_s14_mw: l14,
        lhv_s03_mw: l03,
        lhv_s21_mw: l21,
        lhv_s15_mw: l15,
        claim_recycled_mw: l03 + l21,
        claim_burned_mw: l15,
        claim_steam_system_mw: -q_burn.w_into_process / 1e6,
        species_closure_max_rel: worst,
        claim_sum_over_s14: (l03 + l21 + l15) / l14,
    })
}
