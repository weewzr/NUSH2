//! The IEAGHG 2017-02 Base Case reference plant: equations N.R.1 to N.R.12 and
//! N.R.15 of docs/derivation/03_reference_plant.md, one test per equation, and
//! the registry block that feeds them.
//!
//! Every expected value is written here from the printed report (PDF pages in
//! the comments) and worked out in the test, never copied from the engine.

use nush2_numerics::EngineError;
use nush2_plant::abatement::LedgerKind;
use nush2_plant::anchored::Model;
use nush2_plant::params::ParamSet;
use nush2_plant::refplant::{self, RefSp, RowClass, NSP_REF};
use std::sync::OnceLock;

fn model() -> &'static Model {
    static M: OnceLock<Model> = OnceLock::new();
    M.get_or_init(|| Model::embedded().unwrap())
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1e-300)
}

/// Heat and Material Balance, Base case, PDF p44 (streams 1 to 12) and p45
/// (13 to 15): id, T C, P MPa, kmol/h, kg/h.
const PRINTED: [(usize, f64, f64, f64, f64); 15] = [
    (1, 9.0, 7.00, 1696.6, 30563.0),
    (2, 128.0, 3.71, 1455.8, 26231.0),
    (3, 121.0, 0.50, 240.4, 4332.0),
    (4, 500.0, 3.39, 5514.0, 98874.0),
    (5, 320.0, 2.80, 8370.3, 101667.0),
    (6, 412.0, 2.77, 8370.3, 101667.0),
    (7, 35.0, 2.58, 6596.9, 69711.0),
    (8, 28.0, 0.13, 2106.3, 60658.0),
    (9, 136.0, 0.02, 8659.4, 257698.0),
    (10, 395.0, 4.23, 2556.0, 46053.0),
    (11, 15.0, 0.60, 5095.7, 91800.0),
    (12, 40.0, 2.50, 4461.5, 8994.0),
    (13, 40.0, 2.51, 29.1, 59.0),
    (14, 400.0, 4.29, 5290.1, 95301.0),
    (15, 177.0, 0.44, 30.0, 540.0),
];

/// Printed mole fractions in the order CO2, CO, H2, N2, O2, CH4, C2H6, C3H8,
/// nC4H10, nC5H12, H2O (PDF p44). Stream 9 prints CO as a note, read as 0.
const Y_PRINTED: [(usize, [f64; NSP_REF]); 7] = [
    (
        1,
        [
            0.0200, 0.0, 0.0, 0.0089, 0.0, 0.8900, 0.0700, 0.0100, 0.0010, 0.0001, 0.0,
        ],
    ),
    (
        4,
        [
            0.0053, 0.0, 0.0053, 0.0023, 0.0, 0.2350, 0.0185, 0.0026, 0.0003, 0.0, 0.7307,
        ],
    ),
    (
        5,
        [
            0.0492, 0.1156, 0.5171, 0.0015, 0.0, 0.0238, 0.0, 0.0, 0.0, 0.0, 0.2927,
        ],
    ),
    (
        6,
        [
            0.1283, 0.0366, 0.5961, 0.0015, 0.0, 0.0238, 0.0, 0.0, 0.0, 0.0, 0.2137,
        ],
    ),
    (
        7,
        [
            0.1627, 0.0464, 0.7563, 0.0020, 0.0, 0.0302, 0.0, 0.0, 0.0, 0.0, 0.0024,
        ],
    ),
    (
        8,
        [
            0.5095, 0.1454, 0.2369, 0.0062, 0.0, 0.0945, 0.0, 0.0, 0.0, 0.0, 0.0076,
        ],
    ),
    (
        9,
        [
            0.2123, 0.0, 0.0, 0.6083, 0.0102, 0.0, 0.0, 0.0, 0.0, 0.0, 0.1692,
        ],
    ),
];

// Carbon atoms per molecule of the natural gas: 0.02 + 0.89 + 2 x 0.07 +
// 3 x 0.01 + 4 x 0.001 + 5 x 0.0001.
const C_PER_NG: f64 = 0.02 + 0.89 + 2.0 * 0.07 + 3.0 * 0.01 + 4.0 * 0.001 + 5.0 * 0.0001;

// ------------------------------------------------------------ registry (WP0)

#[test]
fn eq_n_r_0_registry_reads_every_reference_entry() {
    let p = ParamSet::embedded().unwrap();
    let ids: Vec<&str> = p
        .entries()
        .filter(|e| e.id.starts_with("REF_"))
        .map(|e| e.id.as_str())
        .collect();
    let streams = ids
        .iter()
        .filter(|id| id.starts_with("REF_S") && id.as_bytes()[5].is_ascii_digit())
        .count();
    // 15 streams x (T, P, kmol/h, kg/h) and 43 printed nonzero fractions of
    // streams 1, 4, 5, 6, 7, 8 and 9 (7 + 8 + 6 + 6 + 6 + 6 + 4).
    assert_eq!(streams, 60 + 43, "{ids:?}");
    assert_eq!(ids.len(), 103 + 23, "{ids:?}");
    let (_, read) = Model::new_logged(&p).unwrap();
    for id in ids.iter().copied().chain([
        "SMR_ISLAND_POWER_ISLAND_TPC_EURM",
        "SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND",
        "SUBSIDY_SWEEP_STEP",
        "SUBSIDY_GAP_LIMIT_GROSS",
    ]) {
        assert!(read.contains(id), "{id} is never read");
    }
    // Every published reference entry names its PDF page; the switches name
    // their decision.
    for e in p.entries().filter(|e| e.id.starts_with("REF_")) {
        assert_eq!(e.group, "reference", "{}", e.id);
        match e.class.as_str() {
            "published" | "assumed" => assert!(e.locator.contains("PDF p"), "{}", e.id),
            "user_decision" => assert!(e.locator.starts_with("decision D"), "{}", e.id),
            other => panic!("{}: class {other}", e.id),
        }
    }
    // The reference ships switched off (critique C03), the power-island switch
    // ships off (decision D23).
    assert_eq!(p.get("REF_BASIS").unwrap(), 0.0);
    assert_eq!(
        p.get("SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND").unwrap(),
        0.0
    );
    let m = model();
    assert_eq!(m.refp.p.basis, refplant::RefBasis::Historical);
    assert!(!m.refp.p.island.nush2_excludes_power_island);
    // Sheet 85 of 112, PDF p222: 97.212 + 20.124 + 53.616 = 170.952 M EUR.
    assert_eq!(m.refp.p.island.power_island_tpc_eurm, 20.124);
    assert!((97.212 + 20.124 + 53.616_f64 - 170.952).abs() < 1e-9);
    assert_eq!(m.subsidy.sweep_step, 0.05);
    assert_eq!(m.subsidy.gap_limit_gross, 0.01);
    assert!((nush2_props::units::KCAL_KJ - 4.1868).abs() < 1e-15);
}

#[test]
fn eq_n_r_0_switches_and_steps_are_checked_when_read() {
    for (id, bad) in [
        ("REF_BASIS", 0.5),
        ("SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND", 2.0),
        ("SUBSIDY_SWEEP_STEP", 0.03),
        ("SUBSIDY_SWEEP_STEP", 0.0),
        ("SUBSIDY_GAP_LIMIT_GROSS", 0.0),
        ("REF_STEAM_NEIGHBOUR_T_H", 1.0),
        ("REF_S08_KMOL_H", -1.0),
        ("REF_S09_Y_CO2", 1.5),
    ] {
        let mut p = ParamSet::embedded().unwrap();
        p.set(id, bad).unwrap();
        assert!(
            matches!(Model::new(&p), Err(EngineError::Range { .. })),
            "{id} = {bad} was accepted"
        );
    }
    let mut p = ParamSet::embedded().unwrap();
    p.set("REF_BASIS", 1.0).unwrap();
    assert_eq!(
        Model::new(&p).unwrap().refp.p.basis,
        refplant::RefBasis::Reconstructed
    );
}

// --------------------------------------------------------------- N.R.1

#[test]
fn eq_n_r_1_published_table_rows_are_as_printed() {
    let rp = &model().refp;
    for (id, t, pr, n, m) in PRINTED {
        let s = rp.p.stream(id);
        assert_eq!((s.id, s.t_c, s.p_mpa, s.kmol_h, s.kg_h), (id, t, pr, n, m));
        let sum: f64 = s.y.iter().sum();
        assert!((sum - 1.0).abs() <= 1.0001e-4, "stream {id}: {sum}");
    }
    for (id, y) in Y_PRINTED {
        assert_eq!(rp.p.stream(id).y, y, "stream {id}");
    }
    assert_eq!(rp.p.stream(2).y, Y_PRINTED[0].1);
    assert_eq!(rp.p.stream(3).y, Y_PRINTED[0].1);
    for id in [10, 11, 14, 15] {
        assert_eq!(rp.p.stream(id).y[RefSp::H2o as usize], 1.0);
    }
    for id in [12, 13] {
        assert_eq!(rp.p.stream(id).y[RefSp::H2 as usize], 1.0);
    }
    // Carbon flows, kmol C/h.
    let b = rp.balance();
    let c = |id: usize| b.carbon[id - 1];
    assert!(rel(c(1), 1696.6 * C_PER_NG) < 1e-12);
    assert!(rel(c(2), 1455.8 * C_PER_NG) < 1e-12);
    assert!(rel(c(3), 240.4 * C_PER_NG) < 1e-12);
    assert!(rel(c(7), 6596.9 * (0.1627 + 0.0464 + 0.0302)) < 1e-12);
    assert!(rel(c(8), 2106.3 * (0.5095 + 0.1454 + 0.0945)) < 1e-12);
    assert!((c(1) - 1839.963).abs() < 1e-3 && (c(8) - 1578.461).abs() < 1e-3);
    // Stream 1 against streams 2 and 3: the masses add exactly, the moles
    // differ by 0.4 kmol/h in the printed digits.
    assert_eq!(30563.0 - 26231.0 - 4332.0, 0.0);
    let split = rp.check("REF_NG_STREAM_SPLIT_MASS").unwrap();
    assert_eq!(split.calculated, 0.0);
    assert!(split.blocking && split.passes());
}

// --------------------------------------------------------------- N.R.2

#[test]
fn eq_n_r_2_psa_balance_reproduces_stream_8() {
    let rp = &model().refp;
    let psa = &rp.balance().psa;
    let r = (4461.5 + 29.1) / (6596.9 * 0.7563);
    assert!(rel(psa.recovery, r) < 1e-12);
    assert!((psa.recovery - 0.900058).abs() < 1e-6);
    // Model tail gas minus stream 8, kmol/h, worked from the printed rows.
    let want = [
        6596.9 * 0.1627 - 2106.3 * 0.5095,
        6596.9 * 0.0464 - 2106.3 * 0.1454,
        6596.9 * 0.7563 - 4490.6 - 2106.3 * 0.2369,
        6596.9 * 0.0020 - 2106.3 * 0.0062,
        0.0,
        6596.9 * 0.0302 - 2106.3 * 0.0945,
        0.0,
        0.0,
        0.0,
        0.0,
        6596.9 * 0.0024 - 2106.3 * 0.0076,
    ];
    for (i, (got, w)) in psa.residual.iter().zip(want).enumerate() {
        assert!((got - w).abs() < 1e-9, "{i}");
    }
    assert!((psa.max_abs_residual - 0.347).abs() < 1e-3);
    assert!(psa.max_abs_residual <= refplant::TOL_PSA_SPECIES_KMOL_H);
    // Mass: 69,711 = 60,658 + 8,994 + 59.
    assert_eq!(psa.mass_closure_kg_h, 69711.0 - 60658.0 - 8994.0 - 59.0);
    assert_eq!(psa.mass_closure_kg_h, 0.0);
    // Carbon: C7 - C8 = 1578.638 - 1578.461.
    assert!((psa.element_residual[0] - (6596.9 * 0.2393 - 2106.3 * 0.7494)).abs() < 1e-9);
    // The test-only form at the derived recovery gives the same tail gas, and a
    // lower recovery sends more hydrogen to the tail gas.
    let t = rp.psa_tail(psa.recovery);
    for (a, b) in t.tail.iter().zip(psa.tail) {
        assert!((a - b).abs() < 1e-9);
    }
    assert!(rp.psa_tail(0.85).tail[RefSp::H2 as usize] > psa.tail[RefSp::H2 as usize]);
    // The printed range is 85 to 90 percent (Sheet 14, PDF p37); the derived
    // recovery lies above 0.90 by 5.8e-5 and inside the whole-percent reading.
    assert!(psa.recovery > 0.90 && psa.recovery < 0.905);
    assert!(rp.check("REF_PSA_RECOVERY_IN_RANGE").unwrap().passes());
}

// --------------------------------------------------------------- N.R.3

#[test]
fn eq_n_r_3_tail_gas_lhv_uses_the_engine_constants() {
    let f = &model().refp.balance().furnace;
    // H2 241.8, CO 283.0, CH4 802.3 kJ/mol on stream 8 (PDF p44).
    let q = 2106.3 * (0.2369 * 241.8 + 0.1454 * 283.0 + 0.0945 * 802.3) / 3600.0;
    assert!(rel(f.q_tail_mw, q) < 1e-12);
    assert!((f.q_tail_mw - 101.950).abs() < 1e-3);
    assert!(rel(f.q_tail_mj_kg, q * 3600.0 / 60658.0) < 1e-12);
    assert!((f.q_tail_mj_kg - 6.0506).abs() < 1e-4);
    assert!((f.q_tail_mj_nm3 - 7.7741).abs() < 1e-4);
}

// --------------------------------------------------------------- N.R.4

#[test]
fn eq_n_r_4_ng_fuel_heat_matches_the_utility_sheet_fuel() {
    let rp = &model().refp;
    let f = &rp.balance().furnace;
    // Stream 3 at the Table 2 LHV: 4,332 kg/h x 46.50 MJ/kg.
    assert!(rel(f.q_ng_supp_mw, 4332.0 * 46.50 / 3600.0) < 1e-12);
    // Utility sheet, PDF p48: 48.2 MMkcal/h.
    assert!(rel(f.q_utility_fuel_mw, 48.2 * 4.1868 / 3.6) < 1e-12);
    let c = rp.check("REF_UTILITY_FUEL_VS_STREAM_3").unwrap();
    assert!((c.residual - 0.0018).abs() < 1e-4, "{}", c.residual);
    assert!(c.passes() && !c.blocking);
}

// --------------------------------------------------------------- N.R.5

#[test]
fn eq_n_r_5_fired_duty_and_tail_gas_share() {
    let rp = &model().refp;
    let f = &rp.balance().furnace;
    assert!(rel(f.q_fired_mw, f.q_tail_mw + 4332.0 * 46.50 / 3600.0) < 1e-12);
    assert!((f.q_fired_mw - 157.905).abs() < 1e-3);
    assert!((f.tail_share - 0.6456).abs() < 1e-4);
    // 40.0 percent of the plant input (Table 2: 394.77 MW).
    assert!((f.q_fired_mw / 394.77 - 0.4000).abs() < 1e-3);
    // The test-only constant-duty form returns stream 3 at the published tail
    // gas and nothing once the tail gas alone covers the duty.
    assert!((rp.supp_ng_t_h(f.q_tail_mw) - 4.332).abs() < 1e-9);
    assert_eq!(rp.supp_ng_t_h(f.q_fired_mw + 1.0), 0.0);
}

// --------------------------------------------------------------- N.R.6

#[test]
fn eq_n_r_6_radiant_duty_is_an_indicator_only() {
    let f = &model().refp.balance().furnace;
    // 82.63 MMkcal/h, equipment list sheet 5 of 8, PDF p54.
    assert!(rel(f.q_rad_mw, 82.63 * 4.1868 / 3.6) < 1e-12);
    assert!((f.q_rad_mw - 96.099).abs() < 1e-3);
    assert!((f.radiant_fraction - 0.6086).abs() < 1e-4);
    // No efficiency is derived or stored.
    let p = ParamSet::embedded().unwrap();
    for id in ["REF_FURNACE_EFF", "REF_STEAM_DUTY"] {
        assert!(p.get(id).is_err(), "{id} must not exist");
    }
}

// --------------------------------------------------------------- N.R.7

#[test]
fn eq_n_r_7_carbon_closes_from_gas_to_stack() {
    let rp = &model().refp;
    let co2 = &rp.balance().co2;
    let stack = 8659.4 * 0.2123;
    assert!(rel(co2.stack_kmol_h, stack) < 1e-12);
    assert!(rel(co2.stack_t_h, stack * 44.0095 / 1000.0) < 1e-12);
    assert!((co2.stack_t_h - 80.9067).abs() < 1e-4);
    assert!(rel(co2.c_residual_rel, stack / (1696.6 * C_PER_NG) - 1.0) < 1e-9);
    assert!((co2.c_residual_rel + 0.000854).abs() < 1e-5);
    let c = rp.check("REF_CARBON_CLOSURE_NG_TO_STACK").unwrap();
    assert!(c.blocking && c.passes());
}

// --------------------------------------------------------------- N.R.8

#[test]
fn eq_n_r_8_co2_breakdown_adds_to_the_stack() {
    let co2 = &model().refp.balance().co2;
    let (a1, a2, a3) = (2106.3 * 0.5095, 2106.3 * 0.1454, 2106.3 * 0.0945);
    let (b1, b2) = (240.4 * 0.0200, 240.4 * (C_PER_NG - 0.0200));
    for (got, want) in [
        (co2.a1, a1),
        (co2.a2, a2),
        (co2.a3, a3),
        (co2.b1, b1),
        (co2.b2, b2),
    ] {
        assert!(rel(got, want) < 1e-12);
    }
    let rho = 8659.4 * 0.2123 / (a1 + a2 + a3 + b1 + b2);
    assert!(rel(co2.rho, rho) < 1e-12);
    assert!((co2.rho - 0.999574).abs() < 1e-6);
    // The rescaled parts add to the stack exactly; A is 85.8 percent of it.
    let stack_nm3 = 8659.4 * 0.2123 * 44.0095 / (4461.5 * 22.414);
    assert!(rel(co2.a_kg_nm3() + co2.b_kg_nm3(), stack_nm3) < 1e-12);
    assert!((co2.a_kg_nm3() / stack_nm3 - 0.8582).abs() < 1e-4);
    assert!((co2.a_kg_nm3() - 0.69438).abs() < 1e-5);
    // Critique C08: a CO2 wash ahead of the burners removes at most A1, which
    // is tighter than A; Case 2A captures 0.4556 kg/Nm3 (Table 2, PDF p16).
    assert!((co2.bound_co2_wash_kg_nm3() - 0.47209).abs() < 1e-5);
    assert!(co2.bound_co2_wash_kg_nm3() < co2.bound_tail_conversion_kg_nm3());
    assert!(0.4556 <= co2.bound_co2_wash_kg_nm3());
    assert!(rel(co2.bound_flue_gas_kg_nm3(), stack_nm3) < 1e-12);
}

// --------------------------------------------------------------- N.R.9

#[test]
fn eq_n_r_9_intensity_reproduces_table_2() {
    let rp = &model().refp;
    let b = rp.balance();
    let stack_kg_h = 8659.4 * 0.2123 * 44.0095;
    assert!(rel(b.intensity_kg_kg, stack_kg_h / 8994.0) < 1e-12);
    assert!((b.intensity_kg_kg - 8.995625).abs() < 1e-6);
    assert!(rel(b.intensity_kg_nm3, stack_kg_h / (4461.5 * 22.414)) < 1e-12);
    // Table 2: 0.8091 kg/Nm3, which is 8.995997 kg/kg at 8,994 kg/h.
    assert!((rel(b.intensity_kg_nm3, 0.8091)) < 5e-5);
    let per_kg = 0.8091 * 100000.0 / 8994.0;
    let r = b.intensity_kg_kg / per_kg - 1.0;
    assert!(r > -4.2e-5 && r < -4.0e-5, "{r}");
    for id in ["REF_STACK_CO2_VS_TABLE_2", "REF_INTENSITY_VS_TABLE_2"] {
        assert!(rp.check(id).unwrap().passes(), "{id}");
    }
    // The published value is the whole stack and is never increased by a
    // tail-gas term: the one central route is stream 9 over stream 12.
    assert!(b.intensity_kg_kg < per_kg);
}

// -------------------------------------------------------------- N.R.10

#[test]
fn eq_n_r_10_gas_energy_matches_table_2_and_converts_by_energy() {
    let rp = &model().refp;
    let g = &rp.balance().gas;
    assert!(rel(g.q_lhv_mw, 30563.0 * 46.50 / 3600.0) < 1e-12);
    assert!((g.q_lhv_mw - 394.772).abs() < 1e-3);
    assert!(rp.check("REF_TABLE2_ENERGY_IN").unwrap().passes());
    assert!(rp.check("REF_TABLE2_ENERGY_H2").unwrap().passes());
    // HHV 51.473 MJ/kg, PDF p255.
    assert!(rel(g.e_hhv_gj_h, 30.563 * 51.473) < 1e-12);
    assert!((g.q_hhv_mw - 436.99).abs() < 1e-2);
    // In the engine's gas unit: 1,044 Btu/scf x 1.05505585 GJ/MMBtu.
    let n = 30.563 * 51.473 * 24.0 / (1044.0 * 1.05505585);
    assert!(rel(g.n_ref_mmscfd, n) < 1e-12);
    assert!((g.n_ref_mmscfd - 34.2776).abs() < 1e-4);
    assert!(rp.check("REF_NG_LHV_ANNEX_VS_TABLE_2").unwrap().passes());
}

// -------------------------------------------------------------- N.R.11

#[test]
fn eq_n_r_11_power_balance_closes() {
    let rp = &model().refp;
    let sp = &rp.balance().steam_power;
    // Table 2 and utility sheet: 11.500 - 1.216 - (19 + 347) / 1000.
    assert!((sp.w_net_mwe - 9.918).abs() < 1e-9);
    let c = rp.check("REF_TABLE2_NET_POWER").unwrap();
    assert!(c.blocking && c.passes());
    assert!((sp.w_spec_kwh_t - 11500.0 / 46.053).abs() < 1e-9);
    assert!((sp.w_spec_kwh_t - 249.71).abs() < 1e-2);
    assert!((sp.w_spec_utility_kwh_t - 250.54).abs() < 1e-2);
    // Revenue 9.918 x 8,322 x 80 against 6,603,008 EUR/y (Table 4, PDF p18).
    assert!((sp.revenue_eur_y - 9.918 * 8322.0 * 80.0).abs() < 1e-6);
    assert!(rp.check("REF_POWER_REVENUE").unwrap().passes());
    // Stream 10 against the utility sheet, 46.053 against 45.9 t/h.
    let s10 = rp.check("REF_STEAM_STREAM_10_VS_UTILITY").unwrap();
    assert!((s10.residual - (46.053 / 45.9 - 1.0)).abs() < 1e-12 && s10.passes());
    assert_eq!(sp.steam_neighbour_t_h, 0.0);
    assert!(rp.check("REF_STEAM_TO_NEIGHBOUR_ZERO").unwrap().passes());
}

// -------------------------------------------------------------- N.R.12

#[test]
fn eq_n_r_12_scaling_is_linear_and_intensive_quantities_do_not_move() {
    let m = model();
    let rp = &m.refp;
    let one = rp.scaled(8994.0).unwrap();
    assert_eq!(one.x, 1.0);
    // Case B: 100,000 Nm3/h x 0.08988 kg/Nm3 = 8,988 kg/h.
    let b = rp.scaled(m.h2_b_kgh).unwrap();
    assert!(rel(b.x, 8988.0 / 8994.0) < 1e-12);
    assert!((b.x - 0.999333).abs() < 1e-6);
    assert_eq!(b.intensity_kg_kg, one.intensity_kg_kg);
    assert!(rel(b.gas_mmscfd_eq, b.x * one.gas_mmscfd_eq) < 1e-12);
    assert!((b.gas_mmscfd_eq - 34.2548).abs() < 1e-4);
    assert!(rel(b.stack_co2_t_h, b.x * one.stack_co2_t_h) < 1e-12);
    assert!(rel(b.power_buy_mwe, -b.x * 9.918) < 1e-9);
    assert_eq!(b.steam_neighbour_t_h, 0.0);
    let two = rp.scaled(2.0 * 8994.0).unwrap();
    assert!(rel(two.gas_mmscfd_eq, 2.0 * one.gas_mmscfd_eq) < 1e-12);
    for bad in [0.0, -1.0, f64::NAN] {
        assert!(rp.scaled(bad).is_err());
    }
}

#[test]
fn eq_n_r_12_stream_table_scales_published_rows_and_labels_derived_ones() {
    let m = model();
    let rp = &m.refp;
    let rows = rp.stream_table(m.h2_b_kgh).unwrap();
    assert_eq!(rows.len(), 18);
    let x = 8988.0 / 8994.0;
    for (row, (id, t, _, n, kg)) in rows.iter().zip(PRINTED) {
        assert_eq!(row.tag, format!("I{id:02}"));
        assert_eq!(row.class, RowClass::Published);
        assert_eq!(row.t_c, Some(t));
        assert!(rel(row.kmol_h, x * n) < 1e-12);
        assert!(rel(row.kg_h.unwrap(), x * kg) < 1e-12);
    }
    let derived = &rows[15..];
    assert!(derived
        .iter()
        .all(|r| r.class == RowClass::DerivedUnpublished && r.kg_h.is_none()));
    // At x = 1: dry air 6,648.5 kmol/h plus 62.7 kmol/h of humidity, from the
    // N balance with 21 percent O2 and the H balance of the furnace.
    let one = rp.stream_table(8994.0).unwrap();
    let n2_air: f64 = 8659.4 * 0.6083 - 240.4 * 0.0089 - 2106.3 * 0.0062;
    let dry = n2_air / 0.79;
    assert!((dry - 6648.5).abs() < 0.1);
    let h_fuel: f64 = 240.4 * (4.0 * 0.89 + 6.0 * 0.07 + 8.0 * 0.01 + 10.0 * 0.001 + 12.0 * 0.0001)
        + 2106.3 * (2.0 * 0.2369 + 4.0 * 0.0945 + 2.0 * 0.0076);
    let hum: f64 = 8659.4 * 0.1692 - h_fuel / 2.0;
    assert!((hum - 62.7).abs() < 0.1);
    assert!(rel(one[15].kmol_h, dry + hum) < 1e-12);
    // Second steam addition 155.17 kmol/h from the H balance of streams 4 and 5.
    let h4 = 5514.0
        * (2.0 * 0.0053
            + 4.0 * 0.2350
            + 6.0 * 0.0185
            + 8.0 * 0.0026
            + 10.0 * 0.0003
            + 2.0 * 0.7307);
    let h5 = 8370.3 * (2.0 * 0.5171 + 4.0 * 0.0238 + 2.0 * 0.2927);
    assert!(rel(one[16].kmol_h, (h5 - h4) / 2.0) < 1e-12);
    assert!((one[16].kmol_h - 155.17).abs() < 0.01);
    // Process condensate 1,772.9 kmol/h.
    assert!(rel(one[17].kmol_h, 8370.3 * 0.2137 - 6596.9 * 0.0024) < 1e-12);
    assert!((one[17].kmol_h - 1772.9).abs() < 0.1);
}

// ---------------------------------------------------------- blocking checks

#[test]
fn a_bad_registry_edit_is_refused_by_a_blocking_check() {
    for (id, v, check) in [
        ("REF_S09_Y_CO2", 0.2133, "REF_CARBON_CLOSURE_NG_TO_STACK"),
        ("REF_S08_Y_H2", 0.2375, "REF_PSA_SPECIES_MAX_RESIDUAL"),
        ("REF_S12_KG_H", 9000.0, "REF_PSA_MASS_CLOSURE"),
        ("REF_S01_KG_H", 30570.0, "REF_NG_STREAM_SPLIT_MASS"),
        ("REF_POWER_NET_MWE", 9.919, "REF_TABLE2_NET_POWER"),
        ("BASELINE_IEAGHG_KG_NM3", 0.8200, "REF_STACK_CO2_VS_TABLE_2"),
    ] {
        let mut p = ParamSet::embedded().unwrap();
        p.set(id, v).unwrap();
        match Model::new(&p) {
            Err(EngineError::Range { param, .. }) => assert_eq!(param, check, "{id}"),
            other => panic!("{id} = {v} gave {:?}", other.map(|_| ())),
        }
    }
    // Every blocking check passes on the published table.
    let rp = &model().refp;
    let blocking: Vec<_> = rp.balance().checks.iter().filter(|c| c.blocking).collect();
    assert_eq!(blocking.len(), 6);
    assert!(
        rp.balance().checks.iter().all(|c| c.passes()),
        "{:?}",
        rp.balance().checks
    );
}

// -------------------------------------------------------------- N.R.15

#[test]
fn eq_n_r_15_tail_gas_heat_is_claimed_once() {
    let m = model();
    let led = m.plant_ledger(LedgerKind::Rec, m.eta_mea).unwrap();
    let reg = refplant::heat_register(&led.run).unwrap();
    // The cryogenic unit conserves every species of S14 into S03, S21 and S15.
    assert!(
        reg.species_closure_max_rel < 1e-9,
        "{}",
        reg.species_closure_max_rel
    );
    assert!((reg.claim_sum_over_s14 - 1.0).abs() < 1e-9);
    assert!(rel(reg.claim_recycled_mw + reg.claim_burned_mw, reg.lhv_s14_mw) < 1e-9);
    // The only claim on the steam system is the burner duty, bounded by the
    // LHV of the cut it burns.
    assert!(reg.claim_steam_system_mw > 0.0);
    assert!(reg.claim_steam_system_mw <= reg.lhv_s15_mw);
    assert!(reg.lhv_s15_mw < reg.lhv_s14_mw);
}
