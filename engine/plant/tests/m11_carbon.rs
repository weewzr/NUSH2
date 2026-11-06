//! The independent carbon basis: every equation N.1.1 to N.10.4 of
//! docs/derivation/01_carbon_and_abatement.md checked by id against the
//! flowsheet it describes, plus the properties the basis must keep: carbon
//! closure, non-negative streams, one hydrogen output for plant and reference,
//! consistent boundaries and a legacy basis that stays bit-identical.

use nush2_plant::abatement::closed_form as cf;
use nush2_plant::abatement::{BaselineKind, LedgerKind};
use nush2_plant::anchored::{CarbonSrc, Case, Model, Scenario};
use nush2_plant::carbon::{closure_report, Entry, Status, Verdict};
use nush2_plant::flowsheet::{self, ng_carbon_per_mol, FeedBasis, FlowsheetSpec, TearStart};
use nush2_plant::params::ParamSet;
use nush2_props::units::{kmol_h_to_t_h, mmscfd_to_kmol_h, st_d_to_kmol_h, MW_CO2, MW_H2};
use nush2_units::stream::{Sp, NSP};
use std::sync::{Arc, OnceLock};

fn model() -> &'static Model {
    static M: OnceLock<Model> = OnceLock::new();
    M.get_or_init(|| Model::embedded().unwrap())
}

fn legacy_model() -> Model {
    let mut p = ParamSet::embedded().unwrap();
    p.set("CARBON_BASIS", 0.0).unwrap();
    Model::new(&p).unwrap()
}

fn rec() -> Arc<nush2_plant::abatement::PlantLedger> {
    let m = model();
    m.plant_ledger(LedgerKind::Rec, m.eta_mea).unwrap()
}

fn rel(a: f64, b: f64) -> f64 {
    (a - b).abs() / b.abs().max(1e-300)
}

fn kmol_h(run: &flowsheet::FlowsheetRun, tag: &str, sp: Sp) -> f64 {
    run.gas(tag).unwrap().n[sp as usize] * 3.6
}

// ---------------------------------------------------------------- N.1: feed

#[test]
fn eq_n_1_1_carbon_per_kmol_and_feed_carbon() {
    let (cbar, _) = cf::gas_constants().unwrap();
    assert!((cbar - 1.05379).abs() < 1e-6, "{cbar}");
    let l = rec();
    let n_ng = mmscfd_to_kmol_h(l.ng_mmscfd());
    assert!(rel(l.lines().feed, cbar * n_ng) < 1e-12);
}

#[test]
fn eq_n_1_2_hydrogen_potential_table_and_conservation() {
    let z = |sp: Sp| {
        let mut n = [0.0; NSP];
        n[sp as usize] = 1.0;
        cf::hydrogen_potential(&n).unwrap()
    };
    for (sp, want) in [
        (Sp::H2, 1.0),
        (Sp::Ch4, 4.0),
        (Sp::Co, 1.0),
        (Sp::Co2, 0.0),
        (Sp::H2o, 0.0),
        (Sp::N2, 0.0),
        (Sp::C2h6, 7.0),
        (Sp::C3h8, 10.0),
        (Sp::Nc4h10, 13.0),
        (Sp::Nc5h12, 16.0),
    ] {
        assert_eq!(z(sp), want, "{sp:?}");
    }
    let (_, z_ng) = cf::gas_constants().unwrap();
    assert!((z_ng - 4.13939).abs() < 1e-5, "{z_ng}");
    // Z flows through the reformer and the shift train unchanged.
    let l = rec();
    let run = &l.run;
    let zf = |tag: &str| cf::hydrogen_potential(&run.gas(tag).unwrap().n).unwrap();
    for (a, b) in [("S04", "S05"), ("S05", "S09")] {
        assert!(rel(zf(a), zf(b)) < 1e-9, "{a} against {b}");
    }
}

#[test]
fn eq_n_1_3_hydrogen_at_the_absorber() {
    let l = rec();
    let run = &l.run;
    let z_in = (cf::hydrogen_potential(&run.gas("S01").unwrap().n).unwrap()
        + 4.0 * run.gas("S03").unwrap().n[Sp::Ch4 as usize]
        + run.gas("S03").unwrap().n[Sp::Co as usize])
        * 3.6;
    let h2_12 = z_in - 4.0 * kmol_h(run, "S09", Sp::Ch4) - kmol_h(run, "S09", Sp::Co);
    assert!(rel(h2_12, kmol_h(run, "S12", Sp::H2)) < 1e-9);
}

#[test]
fn eq_n_1_4_psa_loop_with_the_hydrogen_return() {
    let m = model();
    let l = rec();
    let (h2_12, h2_13) = l.h2_s12_s13();
    let xi_eff = cf::xi_eff(m.psa_rec, m.carbon_params.cryo_r);
    assert!(rel(h2_13 / h2_12, xi_eff) < 1e-9);
    assert!((cf::xi_eff(m.psa_rec, 0.0) - m.psa_rec).abs() < 1e-15);
    assert!((cf::xi_eff(m.psa_rec, 1.0) - 1.0).abs() < 1e-15);
    // Product carbon P = pi n_H2.
    let pi = m.ch4_in_h2 / (1.0 - m.ch4_in_h2);
    assert!(rel(l.lines().product, pi * h2_13) < 1e-9);
}

fn gas_rate_closed_form(l: &nush2_plant::abatement::PlantLedger, m: &Model) -> (f64, f64) {
    let f = m.carbon_params.cryo_f;
    let r = m.carbon_params.cryo_r;
    let s = l.shares();
    let (cbar, z_ng) = cf::gas_constants().unwrap();
    let d = cf::loop_denominator(f, s.kappa, l.eta, s.phi);
    let chi = cf::chi(f, &s, d);
    let pi = m.ch4_in_h2 / (1.0 - m.ch4_in_h2);
    let (_, h2_13) = l.h2_s12_s13();
    let n = cf::gas_rate_from_h2(h2_13, cf::xi_eff(m.psa_rec, r), f, pi, chi, z_ng, cbar);
    (n, mmscfd_to_kmol_h(l.ng_mmscfd()))
}

#[test]
fn eq_n_1_5_gas_rate_from_the_hydrogen_target() {
    let m = model();
    let l = rec();
    let (closed, flow) = gas_rate_closed_form(&l, m);
    assert!(rel(closed, flow) < 1e-9, "{closed} against {flow}");
    // Lower bound with complete conversion.
    let (_, z_ng) = cf::gas_constants().unwrap();
    let (_, h2_13) = l.h2_s12_s13();
    let xi_eff = cf::xi_eff(m.psa_rec, m.carbon_params.cryo_r);
    assert!(flow >= h2_13 / (xi_eff * z_ng));
    // Homogeneity: the gas per unit hydrogen is the same at Case B.
    let spec = m.ledger_spec(LedgerKind::Rec, m.eta_mea);
    let mut b = spec.clone();
    b.case = Case::B;
    b.basis = FeedBasis::Hydrogen(m.h2_b_kgh);
    let run_b = flowsheet::solve(m, &b, "HOMOG", TearStart::Anchored).unwrap();
    let a = l.run.ng_mmscfd / m.h2_a_kgh;
    let bq = run_b.ng_mmscfd / m.h2_b_kgh;
    assert!(rel(a, bq) < 1e-9, "{a} against {bq}");
    assert!(
        rel(
            run_b.carbon.captured / m.h2_b_kgh,
            l.lines().captured / m.h2_a_kgh
        ) < 1e-9
    );
}

// ----------------------------------------------------------- N.2 and N.3: loop

#[test]
fn eq_n_2_1_tube_carbon_closed_form_for_three_tear_starts() {
    let m = model();
    let spec = m.ledger_spec(LedgerKind::Rec, m.eta_mea);
    let l = rec();
    let s = l.shares();
    let f = m.carbon_params.cryo_f;
    let (_, h2_13) = l.h2_s12_s13();
    let pi = m.ch4_in_h2 / (1.0 - m.ch4_in_h2);
    let d = cf::loop_denominator(f, s.kappa, l.eta, s.phi);
    let t_closed = cf::tube_carbon(l.lines().feed, f, pi * h2_13, d);
    for start in [TearStart::Zero, TearStart::Anchored, TearStart::Double] {
        let run = flowsheet::solve(m, &spec, "THREE_STARTS", start).unwrap();
        assert!(rel(t_closed, run.carbon.tube) < 1e-9, "{start:?}");
    }
    assert!(d > 0.0 && d < 1.0);
}

#[test]
fn eq_n_2_2_stream_vector_of_the_modelled_streams() {
    let m = model();
    let l = rec();
    let a = l.lines();
    let s = l.shares();
    let f = m.carbon_params.cryo_f;
    let fate = cf::fates(a.tube, f, a.product, &s, l.eta);
    assert!(rel(fate.g, a.tail_gas) < 1e-9);
    assert!(rel(fate.r, a.recycled) < 1e-9);
    assert!(rel(fate.b, a.burned) < 1e-9);
    assert!(rel(fate.k, a.condensate) < 1e-9);
    assert!(rel(fate.c, a.captured) < 1e-9);
    assert!(rel(fate.slip_co2, a.slip) < 1e-9);
    assert!((s.phi + s.psi + s.omega + s.kappa - 1.0).abs() < 1e-12);
}

#[test]
fn eq_n_3_1_captured_co2_is_the_specified_capture_times_the_absorber_co2() {
    let m = model();
    let l = rec();
    let run = &l.run;
    let c = kmol_h(run, "S20", Sp::Co2);
    assert!(rel(c, m.eta_mea * kmol_h(run, "S11", Sp::Co2)) < 1e-9);
    assert!(rel(kmol_h(run, "S19", Sp::Co2), c) < 1e-9);
}

#[test]
fn eq_n_3_2_plant_capture_stays_below_one() {
    let m = model();
    let l = rec();
    let a = l.lines();
    let s = l.shares();
    let f = m.carbon_params.cryo_f;
    let d = cf::loop_denominator(f, s.kappa, l.eta, s.phi);
    let eta_pl = cf::plant_capture(l.eta, s.phi, f, a.product, a.feed, d);
    assert!(rel(eta_pl, a.captured / a.feed) < 1e-9);
    let d1 = cf::loop_denominator(1.0, s.kappa, l.eta, s.phi);
    assert!(cf::plant_capture(l.eta, s.phi, 1.0, a.product, a.feed, d1) < 1.0);
}

// ------------------------------------------------------------- N.4: recycle

#[test]
fn eq_n_4_1_recycle_fraction_read_back_from_the_streams() {
    let m = model();
    let a = *rec().lines();
    let f = cf::recycle_read_back(a.tube, a.feed, a.tail_gas);
    assert!((f - m.carbon_params.cryo_f).abs() < 1e-9);
    assert!(rel(a.recycled / a.tail_gas, m.carbon_params.cryo_f) < 1e-9);
    // The registry input equals the J.7 value that INL's numbers imply.
    assert!((flowsheet::recycle_fraction(m) - m.carbon_params.cryo_f).abs() < 1e-9);
}

#[test]
fn eq_n_4_2_nitrogen_balance_of_the_cut() {
    let l = rec();
    let a = l.lines();
    let y_n2 = flowsheet::NG_MOL_FRACTIONS
        .iter()
        .find(|(sp, _)| *sp == Sp::N2)
        .map(|(_, y)| *y)
        .unwrap();
    let n_ng = mmscfd_to_kmol_h(l.ng_mmscfd());
    assert!(rel(a.n2_cut, y_n2 * n_ng) < 1e-12);
    let (beta, f) = cf::n2_cut(a.burned, a.n2_cut, a.tail_gas);
    assert!((f - model().carbon_params.cryo_f).abs() < 1e-9);
    assert!(beta > 0.0);
    let y_cut = a.n2_cut / (l.run.gas("S15").unwrap().total() * 3.6);
    assert!(y_cut > 0.0 && y_cut < 0.2, "{y_cut}");
}

#[test]
fn eq_n_4_3_inl_back_calculation_is_fragile() {
    let m = model();
    let cap6 = st_d_to_kmol_h(m.co2_cap_c6);
    let f_of = |phi: f64| {
        let c = m.carbon(Scenario::InlSel, phi, m.eta_mea);
        c.r / c.g
    };
    let t6 = |phi: f64| (cap6 / m.eta_sel) / phi;
    let phi = m.phi_rec;
    let analytic = cf::d_f_inl_d_phi(t6(phi), phi, m.f_c6, cap6, m.p_c);
    let h = 1e-6;
    let numeric = (f_of(phi + h) - f_of(phi - h)) / (2.0 * h);
    assert!(
        rel(analytic, numeric) < 1e-5,
        "{analytic} against {numeric}"
    );
    // Evaluated at the flowsheet's own CO2 share the same formula gives 0.67.
    assert!((f_of(0.8226) - 0.6687).abs() < 1e-3, "{}", f_of(0.8226));
}

#[test]
fn eq_n_4_4_stack_carbon_curve() {
    let m = model();
    let l = rec();
    let a = *l.lines();
    let s = l.shares();
    let f = m.carbon_params.cryo_f;
    let mm = 1.0 - s.kappa - l.eta * s.phi;
    assert!(rel(cf::stack_vs_f(f, mm, a.feed, a.product), a.burned) < 1e-9);
    let b0 = mm * a.feed - a.product;
    assert!(rel(cf::stack_vs_f(0.0, mm, a.feed, a.product), b0) < 1e-12);
    assert!(cf::stack_vs_f(1.0, mm, a.feed, a.product).abs() < 1e-12);
    assert!((cf::f_for_stack(a.burned, mm, a.feed, a.product) - f).abs() < 1e-9);
    // Fixed hydrogen output: equals the stack at the run's own f.
    let (_, h2_13) = l.h2_s12_s13();
    let pi = m.ch4_in_h2 / (1.0 - m.ch4_in_h2);
    let bh =
        cf::stack_at_fixed_h2(f, &s, l.eta, m.psa_rec, m.carbon_params.cryo_r, pi, h2_13).unwrap();
    assert!(rel(bh, a.burned) < 1e-9, "{bh} against {}", a.burned);
    // More recycle leaves less stack carbon.
    let b_hi = cf::stack_at_fixed_h2(0.9, &s, l.eta, m.psa_rec, m.carbon_params.cryo_r, pi, h2_13)
        .unwrap();
    assert!(b_hi < bh);
}

// -------------------------------------------------------- N.5: stack and air

#[test]
fn eq_n_5_1_complete_combustion_of_the_cut() {
    let l = rec();
    let run = &l.run;
    assert!(rel(kmol_h(run, "S16", Sp::Co2), l.lines().burned) < 1e-9);
    let z15 = cf::hydrogen_potential(&run.gas("S15").unwrap().n).unwrap();
    assert!(rel(run.air_o2_mol_s, z15 / 2.0) < 1e-9);
}

#[test]
fn eq_n_5_2_direct_emission_is_the_stack_plus_the_vented_condensate() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let a = *rec().lines();
    let k = cf::tonnes_per_kmol_h(m.hours) * m.s_b;
    let want = k * (a.burned + m.carbon_params.cond_vent * a.condensate);
    let e = m.emissions(&b.c, m.s_b);
    assert!(rel(e.atm, want) < 1e-9);
    assert!(rel(e.cond_vented, k * m.carbon_params.cond_vent * a.condensate) < 1e-9);
    // v = 0 against v = 1 changes the emission by exactly k K.
    let mut p0 = ParamSet::embedded().unwrap();
    p0.set("COND_CO2_VENT", 0.0).unwrap();
    let m0 = Model::new(&p0).unwrap();
    let b0 = m0.plant_balance(Scenario::RecF, &m0.opts(m0.s_b)).unwrap();
    let e0 = m0.emissions(&b0.c, m0.s_b);
    assert!((e.atm - e0.atm - k * a.condensate).abs() < 1e-6);
}

// ------------------------------------------------------------ N.6: closure

fn entries(v: f64) -> Vec<Entry> {
    rec().lines().account(v)
}

#[test]
fn eq_n_6_1_verdicts_are_never_closed_while_a_stream_is_unknown() {
    let tol = model().carbon_params.closure_tol;
    let rep = closure_report(&entries(1.0), tol);
    assert_eq!(rep.on_modelled, Verdict::Closed);
    assert_eq!(rep.overall, Verdict::Undetermined);
    assert!(rep.unknown.contains(&"U2") && rep.unknown.contains(&"U1_IN"));
    assert!(rep.out_of_boundary.contains(&"U5"));
    // Drop the product stream: open.
    let mut e = entries(1.0);
    e.retain(|x| x.id != "PRODUCT");
    assert_eq!(closure_report(&e, tol).on_modelled, Verdict::Open);
    // Every stream known: closed, and open after a relative loss of 1e-6.
    let mut known: Vec<Entry> = entries(1.0)
        .into_iter()
        .filter(|x| x.status == Status::Modelled || x.status == Status::Specified)
        .collect();
    assert_eq!(closure_report(&known, tol).overall, Verdict::Closed);
    for x in known.iter_mut().filter(|x| x.id == "CAPTURED") {
        x.kmol_c_h = x.kmol_c_h.map(|v| v * (1.0 - 1e-6));
    }
    assert_eq!(closure_report(&known, tol).overall, Verdict::Open);
}

#[test]
fn eq_n_6_1_tolerance_and_the_exported_residual() {
    let m = model();
    assert_eq!(m.carbon_params.closure_tol, flowsheet::ELEMENT_TOL);
    let a = rec();
    let l = a.lines();
    let res = l.feed - l.captured - l.burned - l.product - l.condensate;
    assert!(res.abs() / l.feed < m.carbon_params.closure_tol);
    // The smallest modelled stream, the product, is far above the tolerance.
    assert!(l.product / l.feed > 1e6 * m.carbon_params.closure_tol);
}

#[test]
fn eq_n_6_2_the_tear_check_against_the_closed_forms() {
    let m = model();
    let l = rec();
    let s = l.shares();
    let f = m.carbon_params.cryo_f;
    let a = l.lines();
    let d = cf::loop_denominator(f, s.kappa, l.eta, s.phi);
    let t_closed = cf::tube_carbon(a.feed, f, a.product, d);
    assert!(((t_closed - a.tube) / a.tube).abs() <= m.carbon_params.closure_tol);
    let (n_closed, n_flow) = gas_rate_closed_form(&l, m);
    assert!(((n_closed - n_flow) / n_flow).abs() <= m.carbon_params.closure_tol);
}

// ---------------------------------------------------------- N.7: reference

#[test]
fn eq_n_7_1_reference_process_gas_and_its_limits() {
    let m = model();
    let rf = m.reference_ledger().unwrap();
    let (h2_12, h2_13) = rf.h2_s12_s13();
    // No cryogenic unit: xi_eff equals xi, carbon closes.
    assert!(rel(h2_13 / h2_12, m.psa_rec) < 1e-9);
    let a = rf.lines();
    assert!((a.feed - a.captured - a.burned - a.product - a.condensate).abs() / a.feed < 1e-9);
    assert_eq!(a.captured, 0.0);
    assert_eq!(a.recycled, 0.0);
    let (_, z_ng) = cf::gas_constants().unwrap();
    let n_proc = mmscfd_to_kmol_h(rf.ng_mmscfd());
    assert!(n_proc >= h2_13 / (m.psa_rec * z_ng));
}

#[test]
fn eq_n_7_2_firing_gas_is_not_negative_and_needs_no_invented_efficiency() {
    // The central result uses the reference without firing gas, a floor, because
    // the furnace efficiency and the steam duty have no source. The balance form
    // is checked for its limits.
    let lhv = 826.7;
    assert_eq!(cf::firing_gas(100.0, 0.9, 500.0, lhv), 0.0);
    let n = cf::firing_gas(1000.0, 0.8, 100.0, lhv);
    assert!(n > 0.0);
    assert!(((n * lhv + 100.0) * 0.8 - 1000.0).abs() < 1e-9);
}

#[test]
fn eq_n_7_3_reference_identity_on_the_inl_gas_flows() {
    let m = model();
    let k = st_d_to_kmol_h(1.0);
    for (gas, want) in [(m.ng_conv, m.co2_conv), (m.ng_norec, m.co2_htgr_nocc)] {
        let c = mmscfd_to_kmol_h(gas) * ng_carbon_per_mol() - m.p_c;
        assert!(rel(c / k, want) < 0.005, "{gas} MMSCFD: {} st/d", c / k);
    }
}

#[test]
fn eq_n_7_floors_lie_below_every_published_reference() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let ab = m.abatement(&b, m.s_b).unwrap();
    let floor = ab.rows.iter().find(|r| r.id == "PLANT_FLOOR").unwrap();
    for kind in BaselineKind::ALL {
        let r = ab.rows.iter().find(|r| r.id == kind.id()).unwrap();
        assert!(floor.kg_kg_ref < r.kg_kg_ref, "{}", kind.id());
    }
    // The chemistry floor of K.5: M_CO2 / (4 M_H2).
    assert!(floor.kg_kg_ref > MW_CO2 / (4.0 * MW_H2));
}

#[test]
fn eq_n_7_4_system_terms_are_added_once() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let sys = m.system_adjust(&b, m.s_b);
    let h_s = m.s_b * b.c.h_scale;
    let lost_mm =
        m.export_conv * h_s - b.export_kept - nush2_props::units::mw_to_mmbtu_h(b.tg_surplus);
    let st = |lost_mm, power_mw| {
        cf::system_terms(
            lost_mm,
            power_mw,
            m.hours,
            m.ef_ng_lhv,
            m.system_boiler_eff,
            m.ef_grid,
        )
    };
    let (lost, power) = st(lost_mm, m.e_conv * h_s);
    assert!((sys - (-lost + power)).abs() < 1e-6 * sys.abs().max(1.0));
    // Equal steam export gives no lost-steam term.
    assert_eq!(st(0.0, 0.0), (0.0, 0.0));
}

// ----------------------------------------------------------- N.8: conversion

#[test]
fn eq_n_8_1_operating_hours() {
    let m = model();
    assert!((m.hours - 8760.0 * 0.85).abs() < 1e-9);
}

#[test]
fn eq_n_8_2_annual_mass_of_a_carbon_flow() {
    let m = model();
    assert!((cf::tonnes_per_kmol_h(m.hours) - 327.695).abs() < 1e-3);
    assert!(rel(m.tpy(1.0, 1.0), cf::tonnes_per_kmol_h(m.hours)) < 1e-12);
    // Linear in the availability.
    let mut p = ParamSet::embedded().unwrap();
    p.set("AVAILABILITY", 0.5).unwrap();
    let m2 = Model::new(&p).unwrap();
    assert!(rel(m2.tpy(1.0, 1.0) / m.tpy(1.0, 1.0), 0.5 / 0.85) < 1e-12);
}

#[test]
fn eq_n_8_3_annual_hydrogen_and_the_unit_conversions() {
    let m = model();
    assert!((st_d_to_kmol_h(1927.0) - 1655.083).abs() < 1e-3);
    assert!((mmscfd_to_kmol_h(1.0) - 49.803).abs() < 1e-3);
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let e = m.emissions(&b.c, m.s_b);
    assert!((e.h2_t_y - 66924.648).abs() < 1e-3, "{}", e.h2_t_y);
}

// ------------------------------------------------------- N.9: abatement

#[test]
fn eq_n_9_1_hand_ledger_of_the_direct_abatement() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let e = m.emissions(&b.c, m.s_b);
    let a = *rec().lines();
    let k = cf::tonnes_per_kmol_h(m.hours) * m.s_b;
    let e_ref = m.base_ieaghg * e.h2_t_y;
    let e_plant = k * (a.burned + a.condensate);
    assert!(rel(e.a_direct, e_ref - e_plant) < 1e-12);
    assert!(rel(e.baseline_central, e_ref) < 1e-12);
    assert_eq!(e.baseline_id, "IEAGHG_9p00");
}

#[test]
fn eq_n_9_2_decomposition_of_the_direct_abatement() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let ab = m.abatement(&b, m.s_b).unwrap();
    let d = ab.decomposition.unwrap();
    let central = ab.rows.iter().find(|r| r.id == ab.central).unwrap();
    // Identity of the carbon account: holds to the closure residual.
    assert!(
        rel(d.total, central.a_direct) < 1e-9,
        "{} {}",
        d.total,
        central.a_direct
    );
    assert!(d.captured > 0.0 && d.avoided_gas > 0.0);
    assert!(d.n_ref_mmscfd > d.n_plant_mmscfd);
    // Each MMSCFD of reference gas moves the abatement by 17.2 kt/y at Case A
    // scale, 11.8 kt/y at Case B.
    let per_mmscfd = mmscfd_to_kmol_h(1.0) * ng_carbon_per_mol() * cf::tonnes_per_kmol_h(m.hours);
    assert!((per_mmscfd - 17.2e3).abs() < 0.1e3, "{per_mmscfd}");
}

#[test]
fn eq_n_9_3_capture_based_abatement() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let ab = m.abatement(&b, m.s_b).unwrap();
    let k = cf::tonnes_per_kmol_h(m.hours) * m.s_b;
    assert!(rel(ab.captured_t_y, k * rec().lines().captured) < 1e-12);
    assert!(
        (ab.captured_t_y - 363_277.0).abs() < 1.0,
        "{}",
        ab.captured_t_y
    );
    assert!((ab.stack_t_y - 30_778.0).abs() < 1.0, "{}", ab.stack_t_y);
    assert!(
        (ab.cond_vented_t_y - 1_731.0).abs() < 1.0,
        "{}",
        ab.cond_vented_t_y
    );
}

#[test]
fn eq_n_9_4_net_abatement_after_transport_and_storage() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let e = m.emissions(&b.c, m.s_b);
    let parts = m.ts_parts();
    assert!(rel(cf::ts_ratio(&parts), parts.total()) < 1e-15);
    assert!(rel(e.a_net, e.a_direct - parts.total() * e.captured) < 1e-12);
    // A burden in the published range: the whole Northern Lights chain is 0.026
    // and the 1000 km ship alone 0.021.
    assert!(
        parts.total() > 0.009 && parts.total() < 0.06,
        "{}",
        parts.total()
    );
    // Zero inputs give the direct abatement.
    let mut p = ParamSet::embedded().unwrap();
    for id in [
        "TS_SHIP_DIST_KM",
        "TS_COND_KWH_T",
        "TS_INJ_KWH_T",
        "TS_TERMINAL_FRAC",
        "TS_PURGE_FRAC",
    ] {
        p.set(id, 0.0).unwrap();
    }
    let m0 = Model::new(&p).unwrap();
    let b0 = m0.plant_balance(Scenario::RecF, &m0.opts(m0.s_b)).unwrap();
    let e0 = m0.emissions(&b0.c, m0.s_b);
    assert_eq!(m0.ts_parts().total(), 0.0);
    assert_eq!(e0.a_net, e0.a_direct);
}

#[test]
fn eq_n_9_5_lifecycle_view_counts_transport_and_storage_once() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let e = m.emissions(&b.c, m.s_b);
    let lc = m.lifecycle_terms(&b, m.s_b, b.r).unwrap();
    // The burden of the lifecycle view is the same tonnes the net abatement took off.
    assert!(rel(lc.ts_burden, e.ts_emission) < 1e-12);
    // Lifecycle view from the direct abatement, not from the net one.
    let from_direct = e.a_direct - lc.ts_burden - lc.nuclear_co2e;
    let from_net = e.a_net - lc.nuclear_co2e;
    assert!(rel(from_direct, from_net) < 1e-12);
    // The CO2e view exceeds the CO2 view by exactly GWP e_up,CH4 dG.
    let (a_ts, dg, e_co2, e_ch4, e_nuc) = (1.0e5, 4.0e6, 8.0e-3, 3.5e-3, 100.0);
    let view = |gwp: f64| cf::lifecycle(a_ts, dg, e_co2, gwp, e_ch4, e_nuc);
    assert!((view(30.0) - view(0.0) - 30.0 * e_ch4 * dg).abs() < 1e-9);
}

// ------------------------------------------------------------- N.10: metrics

#[test]
fn eq_n_10_intensities_and_the_reduction() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let ab = m.abatement(&b, m.s_b).unwrap();
    let e = m.emissions(&b.c, m.s_b);
    let central = ab.rows.iter().find(|r| r.id == ab.central).unwrap();
    // N.10.1
    assert!(rel(central.a_h2_net, central.a_net / e.h2_t_y) < 1e-15);
    // N.10.2
    assert!(rel(central.reduction, 1.0 - e.atm / central.e_ref_t_y) < 1e-12);
    // N.10.3 and the direct identity a = e_ref - ci.
    let ci = ab.intensity_kg_kg;
    assert!(rel(ci, e.atm / e.h2_t_y) < 1e-15);
    assert!(rel(central.a_h2_direct, central.kg_kg_ref - ci) < 1e-12);
    assert!(central.kg_kg_ref >= MW_CO2 / (4.0 * MW_H2));
    assert!(ci < central.kg_kg_ref);
}

#[test]
fn eq_n_10_4_reported_residual_carries_its_tolerance_and_statuses() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let ab = m.abatement(&b, m.s_b).unwrap();
    let c = &ab.closure;
    assert_eq!(c.tol, m.carbon_params.closure_tol);
    assert!(c.relative_modelled.unwrap().abs() < c.tol);
    assert_eq!(c.overall, Verdict::Undetermined);
    assert!(
        c.relative.is_none(),
        "an unknown stream leaves the residual undefined"
    );
}

// ------------------------------------------------- properties of the basis

#[test]
fn ledger_streams_are_non_negative_and_close_for_every_kind() {
    let m = model();
    for kind in [
        LedgerKind::Rec,
        LedgerKind::InlSel,
        LedgerKind::NoRecycle,
        LedgerKind::Reference,
    ] {
        let l = m.plant_ledger(kind, m.eta_mea).unwrap();
        let a = l.lines();
        for (name, v) in [
            ("feed", a.feed),
            ("tube", a.tube),
            ("co2_abs", a.co2_abs),
            ("captured", a.captured),
            ("slip", a.slip),
            ("tail_gas", a.tail_gas),
            ("recycled", a.recycled),
            ("burned", a.burned),
            ("product", a.product),
            ("condensate", a.condensate),
            ("n2_cut", a.n2_cut),
        ] {
            assert!(v >= 0.0 && v.is_finite(), "{kind:?} {name} {v}");
        }
        let res = a.feed - a.captured - a.burned - a.product - a.condensate;
        assert!(res.abs() / a.feed < 1e-9, "{kind:?}: {res}");
    }
}

#[test]
fn plant_and_reference_make_the_same_hydrogen() {
    let m = model();
    let rec = rec();
    let rf = m.reference_ledger().unwrap();
    assert!(rel(rec.run.h2_product_kg_h, m.h2_a_kgh) < 1e-9);
    assert!(rel(rf.run.h2_product_kg_h, m.h2_a_kgh) < 1e-9);
    // One product carbon on both sides, so it cancels in the abatement.
    assert!(rel(rec.lines().product, rf.lines().product) < 1e-9);
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let e = m.emissions(&b.c, m.s_b);
    assert!(rel(e.h2_t_y, m.h2_b_kgh * m.hours / 1000.0) < 1e-9);
}

#[test]
fn ledger_run_equals_the_suite_run() {
    let m = model();
    let s = flowsheet::suite(m).unwrap();
    let suite_run = s.run("FS_A_REC_F").unwrap();
    let l = rec();
    let a = l.lines();
    let b = &suite_run.carbon;
    for (x, y) in [
        (a.feed, b.feed),
        (a.tube, b.tube),
        (a.captured, b.captured),
        (a.burned, b.burned),
        (a.condensate, b.condensate),
    ] {
        assert!(rel(x, y) < 1e-9, "{x} against {y}");
    }
}

#[test]
fn ledger_cache_is_deterministic() {
    let m = model();
    let a = m.plant_ledger(LedgerKind::Rec, m.eta_mea).unwrap();
    let b = m.plant_ledger(LedgerKind::Rec, m.eta_mea).unwrap();
    assert!(Arc::ptr_eq(&a, &b));
    let other = m.plant_ledger(LedgerKind::Rec, 0.85).unwrap();
    assert!(other.lines().captured < a.lines().captured);
}

#[test]
fn a_run_that_cannot_solve_is_an_error_and_never_the_legacy_ledger() {
    let m = model();
    assert!(m.plant_ledger(LedgerKind::Rec, 1.5).is_err());
    assert!(m.plant_ledger(LedgerKind::Rec, 0.0).is_err());
}

#[test]
fn range_guards_refuse_and_never_clamp() {
    for (id, bad) in [
        ("CRYO_C_RECOVERY", 1.2),
        ("CRYO_C_RECOVERY", -0.1),
        ("CRYO_H2_RETURN", 1.01),
        ("COND_CO2_VENT", 2.0),
        ("CARBON_BASIS", 2.0),
        ("CARBON_BASELINE", 3.0),
        ("CARBON_BASELINE", 0.5),
        ("CARBON_CLOSURE_TOL", 1e-3),
        ("ETA_MEA", 0.0),
        ("ETA_MEA", 1.2),
    ] {
        let mut p = ParamSet::embedded().unwrap();
        p.set(id, bad).unwrap();
        assert!(Model::new(&p).is_err(), "{id} = {bad} was accepted");
    }
}

#[test]
fn cryogenic_defaults_equal_their_calibration() {
    let m = model();
    let cl = flowsheet::close_h2_return(m).unwrap();
    assert!((m.carbon_params.cryo_f - flowsheet::recycle_fraction(m)).abs() < 1e-9);
    assert!((m.carbon_params.cryo_r - cl.h2_return).abs() < 1e-9);
    // The closure reproduces INL's pair with the registry inputs.
    let spec = m.ledger_spec(LedgerKind::InlSel, m.eta_sel);
    let mut s = spec;
    s.basis = FeedBasis::NaturalGas(m.ng_c6);
    let run = flowsheet::solve(m, &s, "INL_PAIR", TearStart::Anchored).unwrap();
    assert!(rel(run.h2_product_kg_h, m.h2_a_kgh) < 1e-9);
}

// --------------------------------------------- scenarios and the two bases

#[test]
fn comb_h_export_and_comb_n_ledgers() {
    let m = model();
    let s = m.s_b;
    let h = m.plant_balance(Scenario::CombH, &m.opts(s)).unwrap();
    let x = m.plant_balance(Scenario::Export, &m.opts(s)).unwrap();
    let n = m.plant_balance(Scenario::CombN, &m.opts(s)).unwrap();
    for b in [&h, &x, &n] {
        assert_eq!(b.c.src, CarbonSrc::Plant);
        assert!(b.c.closure.abs() / b.c.f < 1e-9, "{:?}", b.c.scn);
    }
    // The reject is burned on site in COMB_H and leaves the boundary in EXPORT; the
    // atmosphere sees the same CO2 either way.
    assert!(h.c.exported == 0.0 && x.c.exported > 0.0);
    let (eh, ex) = (m.emissions(&h.c, s), m.emissions(&x.c, s));
    assert!(rel(eh.atm, ex.atm) < 1e-12);
    assert!(ex.site < eh.site);
    // COMB_N holds the recycle plant's own gas and makes less hydrogen.
    let rec_b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    assert!(rel(n.c.ng, rec_b.c.ng) < 1e-9);
    assert!(n.c.h_scale < 1.0);
    // REC_FIX is pinned to INL's stack by definition and stays anchored.
    let fix = m.plant_balance(Scenario::RecFix, &m.opts(s)).unwrap();
    assert_eq!(fix.c.src, CarbonSrc::Anchored);
}

#[test]
fn the_legacy_basis_is_bit_identical_to_the_anchored_functions() {
    let m = legacy_model();
    assert!(!m.carbon_params.independent());
    for scn in [
        Scenario::InlSel,
        Scenario::RecF,
        Scenario::RecFix,
        Scenario::CombH,
        Scenario::CombN,
        Scenario::Export,
    ] {
        for s in [1.0, m.s_b] {
            let a = m.balance(scn, &m.opts(s)).unwrap();
            let b = m.plant_balance(scn, &m.opts(s)).unwrap();
            assert_eq!(a.c, b.c, "{scn:?}");
            assert_eq!(a.cap_t.to_bits(), b.cap_t.to_bits());
            let e = m.emissions(&a.c, s);
            assert_eq!(e.a_net.to_bits(), e.a_atm.to_bits());
            assert_eq!(e.ts_emission, 0.0);
            assert_eq!(a.c.src, CarbonSrc::Anchored);
        }
    }
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let e = m.emissions(&b.c, m.s_b);
    assert!((e.a_atm - 578051.6117).abs() < 1e-3, "{}", e.a_atm);
    assert!((e.captured - 359777.3529).abs() < 1e-3);
    assert!((e.atm - 38309.0114).abs() < 1e-3);
}

#[test]
fn the_cost_denominator_is_the_net_abatement() {
    let m = model();
    let b = m.plant_balance(Scenario::RecF, &m.opts(m.s_b)).unwrap();
    let e = m.emissions(&b.c, b.s);
    let p = nush2_plant::cost_point(
        m,
        &b,
        nush2_plant::economics::Stage::Boak,
        25.0,
        m.n_linear,
        m.r_rated,
    )
    .unwrap();
    assert!(rel(p.ac_l5 * e.a_net / 1e6, p.ladder_sm_y[5]) < 1e-12);
    // The legacy basis divides by the same quantity it always did.
    let ml = legacy_model();
    let bl = ml.balance(Scenario::RecF, &ml.opts(ml.s_b)).unwrap();
    let el = ml.emissions(&bl.c, bl.s);
    assert_eq!(el.a_net.to_bits(), el.a_atm.to_bits());
}

#[test]
fn the_energy_table_does_not_move_with_the_basis() {
    let new = include_str!("../../../results/Integrated_Energy_Balance.csv");
    let old = include_str!("../../vv/data/legacy_results/Integrated_Energy_Balance.csv");
    assert_eq!(new, old);
}

#[test]
fn only_the_ledger_rows_of_the_plant_carry_the_independent_status() {
    let p = ParamSet::embedded().unwrap();
    let t = nush2_plant::export::tables(&p).unwrap();
    let mut plant_rows = 0;
    for line in t.emissions_ledger.lines().skip(1) {
        let cells: Vec<&str> = line.split(',').collect();
        let (run, status) = (cells[0], *cells.last().unwrap());
        if run == "B_REC_FIX" {
            // Anchored by definition: never the independent word.
            assert_ne!(status, nush2_plant::export::CARBON_STATUS, "{line}");
        }
        if run == "B_REC_F" && cells[2] == "independent_carbon" {
            assert_eq!(status, nush2_plant::export::CARBON_STATUS, "{line}");
            plant_rows += 1;
        }
    }
    assert!(plant_rows > 20, "{plant_rows}");
    // Decision D4: headline quantities are kg of CO2, and no CO2e word enters
    // the independent view.
    assert!(
        !t.emissions_ledger
            .lines()
            .filter(|l| l.contains(",independent_carbon,"))
            .any(|l| l.contains("CO2e")),
        "a CO2e quantity in the independent view"
    );
    // Every line the owner asked for is present on the headline run.
    for id in [
        "CAPTURED",
        "RESIDUAL_DIRECT",
        "TS_EMISSIONS",
        "ABATEMENT_NET",
        "ABATEMENT_DIRECT",
        "REDUCTION",
        "INTENSITY",
        "CLOSURE_REL",
        "CLOSURE_TOL",
        "DECOMP_CAPTURED",
    ] {
        let key = "B_REC_F,".to_string();
        assert!(
            t.emissions_ledger
                .lines()
                .any(|l| l.starts_with(&key) && l.contains(&format!(",independent_carbon,{id},"))),
            "{id}"
        );
    }
    let _ = kmol_h_to_t_h(1.0);
}

#[test]
fn flowsheet_spec_defaults_follow_the_basis() {
    let m = model();
    let spec = FlowsheetSpec::new(
        m,
        Case::B,
        Scenario::RecF,
        FeedBasis::Hydrogen(m.h2_b_kgh),
        0.5,
    );
    assert_eq!(spec.carbon_recycle, m.carbon_params.cryo_f);
    let l = legacy_model();
    let spec = FlowsheetSpec::new(
        &l,
        Case::B,
        Scenario::RecF,
        FeedBasis::Hydrogen(l.h2_b_kgh),
        0.5,
    );
    assert_eq!(spec.carbon_recycle, flowsheet::recycle_fraction(&l));
}

// ------------------------------------------------ the legacy snapshot

fn split_cells(line: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quoted = false;
    let mut chars = line.chars().peekable();
    while let Some(ch) = chars.next() {
        match (ch, quoted) {
            ('"', true) if chars.peek() == Some(&'"') => {
                cur.push('"');
                chars.next();
            }
            ('"', _) => quoted = !quoted,
            (',', false) => out.push(std::mem::take(&mut cur)),
            _ => cur.push(ch),
        }
    }
    out.push(cur);
    out
}

fn same_cell(a: &str, b: &str) -> bool {
    if a == b {
        return true;
    }
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(x), Ok(y)) => (x - y).abs() <= 1e-6_f64.max(1e-9 * x.abs().max(y.abs())),
        _ => false,
    }
}

/// With the legacy basis the six plant tables contain every row of the snapshot
/// of main 8750b09 with the same cells. Rows that later work added to the tables
/// (the co-funding inputs and the independent carbon inputs) are not compared.
#[test]
fn legacy_export_matches_the_snapshot() {
    let mut p = ParamSet::embedded().unwrap();
    p.set("CARBON_BASIS", 0.0).unwrap();
    let t = nush2_plant::export::tables(&p).unwrap();
    let tables: [(&str, &str, &str, usize); 6] = [
        (
            "Economic_Ledger",
            &t.economic_ledger,
            include_str!("../../vv/data/legacy_results/Economic_Ledger.csv"),
            7,
        ),
        (
            "Emissions_Ledger",
            &t.emissions_ledger,
            include_str!("../../vv/data/legacy_results/Emissions_Ledger.csv"),
            4,
        ),
        (
            "Final_Design_Basis",
            &t.final_design_basis,
            include_str!("../../vv/data/legacy_results/Final_Design_Basis.csv"),
            1,
        ),
        (
            "Integrated_Energy_Balance",
            &t.integrated_energy_balance,
            include_str!("../../vv/data/legacy_results/Integrated_Energy_Balance.csv"),
            2,
        ),
        (
            "Integrated_Mass_Balance",
            &t.integrated_mass_balance,
            include_str!("../../vv/data/legacy_results/Integrated_Mass_Balance.csv"),
            2,
        ),
        (
            "Sensitivity_Results",
            &t.sensitivity_results,
            include_str!("../../vv/data/legacy_results/Sensitivity_Results.csv"),
            7,
        ),
    ];
    for (name, now, snapshot, width) in tables {
        let key = |row: &[String],
                   n: usize,
                   seen: &mut std::collections::BTreeMap<Vec<String>, usize>| {
            let k: Vec<String> = row[..width].to_vec();
            let occ = seen.entry(k.clone()).or_insert(0);
            let out = (k, *occ);
            *occ += 1;
            let _ = n;
            out
        };
        let mut seen = std::collections::BTreeMap::new();
        let mut index = std::collections::BTreeMap::new();
        for line in now.lines().skip(1) {
            let row = split_cells(line);
            index.insert(key(&row, width, &mut seen), row);
        }
        let head_now = split_cells(now.lines().next().unwrap());
        let head_old = split_cells(snapshot.lines().next().unwrap());
        assert_eq!(head_now, head_old, "{name}: header");
        let mut seen = std::collections::BTreeMap::new();
        let mut compared = 0;
        for line in snapshot.lines().skip(1) {
            let row = split_cells(line);
            let k = key(&row, width, &mut seen);
            let got = index
                .get(&k)
                .unwrap_or_else(|| panic!("{name}: row {k:?} missing"));
            assert_eq!(got.len(), row.len(), "{name} {k:?}");
            for (i, (a, b)) in row.iter().zip(got).enumerate() {
                assert!(same_cell(a, b), "{name} {k:?} column {i}: {a} against {b}");
                compared += 1;
            }
        }
        assert!(compared > 1000, "{name}: {compared} cells");
    }
}

// ------------------------------------- one reference gas for cost and carbon

fn boak_query(m: &Model) -> nush2_plant::economics::CostQuery {
    nush2_plant::economics::CostQuery::central(
        nush2_plant::economics::Stage::Boak,
        25.0,
        m.n_linear,
        m.r_rated,
    )
}

/// Rule 1 of the boundary table of N.7: the GAS line of the cost ledger, the
/// saved upstream gas of the lifecycle view and the avoided-gas term of N.9.2
/// read one pair of gas flows, the reference gas implied by the central
/// intensity and the plant's own gas (derivation test 19).
#[test]
fn rule_1_of_n_7_the_gas_line_the_decomposition_and_the_lifecycle_read_one_gas_difference() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let ab = m.abatement(&b, s).unwrap();
    let d = ab.decomposition.unwrap();
    let q = boak_query(m);
    let k = m.cost_items(&b, &q).unwrap();
    let led = m.ledger(&b, &q).unwrap();

    // The reference gas of the cost items is the gas of the decomposition.
    assert!(
        rel(k.ng_smr, d.n_ref_mmscfd) < 1e-12,
        "{} {}",
        k.ng_smr,
        d.n_ref_mmscfd
    );
    assert!(rel(k.ng_plant, d.n_plant_mmscfd) < 1e-9);
    // GAS line over the price is the gas difference, to 1e-12 of the line.
    let diff = -led.gas / k.gas_s_m_per_mmscfd;
    assert!(rel(diff, d.n_ref_mmscfd - k.ng_plant) < 1e-12);
    // The avoided-gas carbon of N.9.2 is the carbon of the same difference.
    let t_y_per_mmscfd =
        mmscfd_to_kmol_h(1.0) * ng_carbon_per_mol() * cf::tonnes_per_kmol_h(m.hours);
    let from_gas = (d.n_ref_mmscfd - d.n_plant_mmscfd) * t_y_per_mmscfd;
    assert!(
        rel(d.avoided_gas, from_gas) < 1e-9,
        "{} {}",
        d.avoided_gas,
        from_gas
    );
    // The saved upstream gas of the lifecycle view is priced on it too.
    let lc = m.lifecycle_terms(&b, s, b.r).unwrap();
    let gj = m.ng_hhv * nush2_props::units::MMBTU_GJ * m.days;
    assert!(rel(lc.up_credit_co2e, diff * gj * m.ef_up / 1000.0) < 1e-12);
    // And the case summary prints the same unabated gas.
    let k_sum = m.plant_summary(Scenario::RecF, s).unwrap();
    assert!(rel(k_sum["ng_unabated_mmscfd"], d.n_ref_mmscfd) < 1e-12);
    assert!(rel(k_sum["ng_saved_mmscfd"], diff) < 1e-12);
    assert!(rel(k_sum["gas_saving_SM_y"], -led.gas) < 1e-9);
}

/// INL's 52.5 MMSCFD is a comparison and no longer a model input: pricing it
/// changes the gas line by the gas difference times the price and nothing else,
/// and the plant's own reference is the lower one.
#[test]
fn inls_reference_gas_is_a_comparison_with_a_reported_cost_effect() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let q = boak_query(m);
    let inl = m.ng_conv * s * b.c.h_scale;
    let q_inl = nush2_plant::economics::CostQuery {
        ng_smr: Some(inl),
        ..q
    };
    let k = m.cost_items(&b, &q).unwrap();
    let k_inl = m.cost_items(&b, &q_inl).unwrap();
    let (led, led_inl) = (m.ledger(&b, &q).unwrap(), m.ledger(&b, &q_inl).unwrap());
    // Only the gas of the unabated plant differs.
    assert_eq!(k_inl.ng_smr.to_bits(), inl.to_bits());
    assert!(k.ng_smr < k_inl.ng_smr, "{} {}", k.ng_smr, k_inl.ng_smr);
    let shift = -(inl - k.ng_smr) * k.gas_s_m_per_mmscfd;
    assert!(rel(led_inl.gas - led.gas, shift) < 1e-12);
    assert_eq!(
        led_inl.reactor_capital.to_bits(),
        led.reactor_capital.to_bits()
    );
    assert_eq!(led_inl.lost_steam.to_bits(), led.lost_steam.to_bits());
    // The default cost is higher than the cost under INL's gas, and so is the DCF.
    let a = m.emissions(&b.c, s).a_net;
    let (l5, l5_inl) = (led.l5() * 1e6 / a, led_inl.l5() * 1e6 / a);
    assert!(l5 > l5_inl, "{l5} {l5_inl}");
    assert!(m.dcf(&b, &q).unwrap().lac > m.dcf(&b, &q_inl).unwrap().lac);
    // The shift of L5 is the gas shift over the net abatement, exactly.
    assert!(rel(l5 - l5_inl, -shift * 1e6 / a) < 1e-9);
}

/// Fed with INL's own intensity the implied gas reproduces INL's gas: the
/// carbon identity of N.7.3 closes on the published pair within 0.5 percent
/// (derivation test 16), so the cost does not depend on the copied gas by a
/// different route.
#[test]
fn the_implied_gas_reproduces_inls_gas_when_given_inls_intensity() {
    let mut p = ParamSet::embedded().unwrap();
    p.set("CARBON_BASELINE", 1.0).unwrap();
    let m = Model::new(&p).unwrap();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let n = m.reference_gas_mmscfd(&b.c, s).unwrap();
    let inl = m.ng_conv * s * b.c.h_scale;
    assert!(rel(n, inl) < 0.005, "{n} against {inl}");
    // The cost follows that baseline: its GAS line prices the same gas.
    let k = m.cost_items(&b, &boak_query(&m)).unwrap();
    assert!(rel(k.ng_smr, n) < 1e-15);
}

/// The anchored basis keeps INL's gas bit for bit in the cost items, the
/// summary and the lifecycle terms.
#[test]
fn the_legacy_basis_keeps_inls_reference_gas_bit_for_bit() {
    let m = legacy_model();
    let s = m.s_b;
    let b = m.balance(Scenario::RecF, &m.opts(s)).unwrap();
    let k = m.cost_items(&b, &boak_query(&m)).unwrap();
    assert_eq!(
        k.ng_smr.to_bits(),
        (m.ng_conv * (s * b.c.h_scale)).to_bits()
    );
    assert_eq!(
        m.reference_gas_mmscfd(&b.c, s).unwrap().to_bits(),
        k.ng_smr.to_bits()
    );
    let lc = m.lifecycle_terms(&b, s, b.r).unwrap();
    let gj = m.ng_hhv * nush2_props::units::MMBTU_GJ * m.days;
    assert_eq!(
        lc.up_credit_co2e.to_bits(),
        ((m.ng_conv * (s * b.c.h_scale) - b.c.ng * s) * gj * m.ef_up / 1000.0).to_bits()
    );
    let sum = m.case_summary(Scenario::RecF, s).unwrap();
    assert_eq!(
        sum["ng_unabated_mmscfd"].to_bits(),
        (m.ng_conv * s).to_bits()
    );
    assert_eq!(
        sum["ng_saved_mmscfd"].to_bits(),
        ((m.ng_conv - m.ng_c6) * s).to_bits()
    );
}

// --------------------------------------- the CO2e terms are always labelled

fn default_tables() -> &'static nush2_plant::export::Tables {
    static T: OnceLock<nush2_plant::export::Tables> = OnceLock::new();
    T.get_or_init(|| nush2_plant::export::tables(&ParamSet::embedded().unwrap()).unwrap())
}

/// Decision D4: no CO2e quantity sits under a CO2 label. In the emissions table
/// the lifecycle views are the only place where the CO2e upstream factor and the
/// CO2e nuclear factor enter, and every row of those views that carries one says
/// CO2e, names the warming-potential horizon and the source. No other view
/// mentions CO2e. The check runs over every lifecycle view of every run.
#[test]
fn no_co2e_quantity_appears_without_a_label_in_any_view() {
    let m = model();
    let t = default_tables();
    let horizon = format!("GWP100 = {}", m.gwp_up);
    let (mut lifecycle_rows, mut other_rows) = (0, 0);
    for line in t.emissions_ledger.lines().skip(1) {
        let c = split_cells(line);
        let (run, view, id, name, source) = (&c[0], &c[2], &c[3], &c[4], &c[10]);
        // B_REC_FIX is anchored by definition and keeps the legacy wording. Every
        // other run is on the plant ledger, whatever status word a preset gave
        // the row (`carbon_independent` or a `gated_*` word).
        if run == "B_REC_FIX" {
            continue;
        }
        if view.starts_with("lifecycle_") {
            lifecycle_rows += 1;
            if id == "TS_BURDEN" {
                assert!(
                    !name.contains("CO2e") && name.contains("CO2"),
                    "{run} {line}"
                );
                assert!(!source.contains("CO2e"), "{run} {line}");
                continue;
            }
            assert!(name.contains("CO2e"), "unlabelled CO2e row: {line}");
            assert!(
                source.contains("CO2e") && source.contains(&horizon),
                "{line}"
            );
            match id.as_str() {
                "UPSTREAM_CREDIT" | "ABATEMENT" if view == "lifecycle_upstream" => {
                    assert!(
                        name.contains(&horizon),
                        "no horizon on the upstream term: {line}"
                    );
                }
                "NUCLEAR_LC" => assert!(name.contains("horizon not stated"), "{line}"),
                _ => {}
            }
        } else {
            other_rows += 1;
            assert!(!name.contains("CO2e"), "CO2e in a CO2 view: {line}");
            assert!(!source.contains("CO2e"), "CO2e in a CO2 view: {line}");
        }
    }
    // Six runs on the plant ledger, three references, seven lifecycle lines.
    assert_eq!(lifecycle_rows, 6 * 3 * 7, "{lifecycle_rows}");
    assert!(other_rows > 100, "{other_rows}");
    // The legacy table keeps the wording of the committed tables; the snapshot
    // test below compares its rows cell by cell.
}

/// The lifecycle view of the exporter is equation N.9.5 on the aggregate
/// factor: the sources read state the upstream intensity only as a CO2e total
/// at one horizon, so the aggregate enters in the CO2 slot with the methane term
/// at zero. No CO2/CH4 split is invented.
#[test]
fn eq_n_9_5_the_exported_lifecycle_is_the_closed_form_on_the_aggregate_factor() {
    let m = model();
    let s = m.s_b;
    let b = m.plant_balance(Scenario::RecF, &m.opts(s)).unwrap();
    let e = m.emissions(&b.c, s);
    let lc = m.lifecycle_terms(&b, s, b.r).unwrap();
    let n_ref = m.reference_gas_mmscfd(&b.c, s).unwrap();
    let dg = (n_ref - b.c.ng * s) * m.ng_hhv * nush2_props::units::MMBTU_GJ * m.days;
    let from_form = cf::lifecycle(e.a_net, dg, m.ef_up / 1000.0, 0.0, 0.0, lc.nuclear_co2e);
    let from_terms = e.a_direct + lc.up_credit_co2e - lc.nuclear_co2e - lc.ts_burden;
    assert!(
        rel(from_form, from_terms) < 1e-12,
        "{from_form} {from_terms}"
    );
    // The warming potential is a label of the factor: 100 years, 30 (IEA 2026).
    assert_eq!(m.gwp_up, 30.0);
}
