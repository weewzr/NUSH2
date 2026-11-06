//! Carbon ledger read from flowsheet streams.
//!
//! Rewritten from an earlier PSA carbon closure for the new stream
//! set, which adds C4 and C5 and tracks N2. The rule carried over is that a
//! missing stream is unknown: a closure with any stream absent returns `None`,
//! and no missing carbon is ever set to zero.

use nush2_units::stream::{Sp, Stream, NSP};
use serde::Serialize;

/// Carbon atoms per molecule, by `Sp` index.
pub const CARBON_ATOMS: [f64; NSP] = [0.0, 1.0, 1.0, 1.0, 0.0, 0.0, 2.0, 3.0, 4.0, 5.0];

/// Carbon flow of a species array, in the unit of the array.
pub fn carbon_of(n: &[f64; NSP]) -> f64 {
    n.iter().zip(CARBON_ATOMS).map(|(v, c)| v * c).sum()
}

/// Carbon flow of a stream, kmol C/h.
pub fn carbon_kmol_h(s: &Stream) -> f64 {
    carbon_of(&s.n) * 3.6
}

/// Nitrogen flow of a stream as N2, kmol/h. Carried so the ledger shows where the
/// feed nitrogen leaves, since the cryogenic cut is its only outlet.
pub fn n2_kmol_h(s: &Stream) -> f64 {
    s.n[Sp::N2 as usize] * 3.6
}

/// Elemental closure over carbon flows, kmol C/h.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CarbonClosure {
    pub input_kmol_c_h: f64,
    pub output_kmol_c_h: f64,
    pub residual_kmol_c_h: f64,
    pub relative_residual: f64,
}

/// Close carbon over the given flows, kmol C/h. `None` when any flow is unknown,
/// when either side is empty, or when the sums are not finite.
///
/// eq: J.11
pub fn close_carbon(inputs: &[Option<f64>], outputs: &[Option<f64>]) -> Option<CarbonClosure> {
    if inputs.is_empty() || outputs.is_empty() {
        return None;
    }
    let mut inp = 0.0;
    for v in inputs {
        inp += (*v)?;
    }
    let mut out = 0.0;
    for v in outputs {
        out += (*v)?;
    }
    if !(inp.is_finite() && out.is_finite()) || inp <= 0.0 || out < 0.0 {
        return None;
    }
    Some(CarbonClosure {
        input_kmol_c_h: inp,
        output_kmol_c_h: out,
        residual_kmol_c_h: inp - out,
        relative_residual: (inp - out) / inp,
    })
}

/// Carbon flows of one flowsheet run, kmol C/h, named as in the anchored ledger
/// (cards J.8 to J.11) so the two can be compared line by line. `condensate` has
/// no anchored counterpart: the screening balance leaves dissolved CO2 out.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct CarbonLedger {
    /// Fresh natural gas, S01.
    pub feed: f64,
    /// Reformer feed after the recycle joins, S04.
    pub tube: f64,
    /// CO2 in the absorber feed, S11.
    pub co2_abs: f64,
    /// CO2 to storage, S20.
    pub captured: f64,
    /// CO2 left in the treated gas, S12.
    pub slip: f64,
    /// PSA reject, S14.
    pub tail_gas: f64,
    /// Recycle to the reformer, S03.
    pub recycled: f64,
    /// Nitrogen-rich cut to the burner, S15.
    pub burned: f64,
    /// Methane in the H2 product, S13.
    pub product: f64,
    /// CO2 dissolved in the process condensate, S10.
    pub condensate: f64,
    /// Feed nitrogen leaving in the cut, kmol N2/h.
    pub n2_cut: f64,
}

impl CarbonLedger {
    /// Carbon closure over the plant: fresh feed in; storage, stack, product and
    /// condensate out.
    ///
    /// eq: J.11
    pub fn closure(&self) -> Option<CarbonClosure> {
        close_carbon(
            &[Some(self.feed)],
            &[
                Some(self.captured),
                Some(self.burned),
                Some(self.product),
                Some(self.condensate),
            ],
        )
    }

    /// The carbon account of the plant: every stream the flowsheet models, the
    /// ones fixed by a specification, and the register of streams it does not
    /// model (U1 to U9 of the derivation), each with its status. `v` is the
    /// share of the condensate CO2 that reaches the air.
    ///
    /// eq: N.6.1
    pub fn account(&self, v: f64) -> Vec<Entry> {
        let known = |id, what, inlet, x: f64, to_air| Entry {
            id,
            what,
            inlet,
            kmol_c_h: Some(x),
            status: Status::Modelled,
            to_air,
        };
        let unknown = |id, what, inlet, to_air| Entry {
            id,
            what,
            inlet,
            kmol_c_h: None,
            status: Status::Unknown,
            to_air,
        };
        let specified = |id, what| Entry {
            id,
            what,
            inlet: false,
            kmol_c_h: Some(0.0),
            status: Status::Specified,
            to_air: false,
        };
        vec![
            known("FEED", "fresh natural gas, S01", true, self.feed, false),
            unknown("U1_IN", "MEA make-up carbon, inlet", true, false),
            known(
                "CAPTURED",
                "CO2 to storage, S20",
                false,
                self.captured,
                false,
            ),
            known(
                "STACK",
                "nitrogen-rich cut burned, S16",
                false,
                self.burned,
                true,
            ),
            known(
                "PRODUCT",
                "methane in the hydrogen product, S13",
                false,
                self.product,
                false,
            ),
            known(
                "CONDENSATE",
                "CO2 dissolved in the condensate, S10",
                false,
                self.condensate,
                v > 0.0,
            ),
            unknown("U1_OUT", "MEA degradation carbon, outlets", false, true),
            unknown("U2", "CO2 vented between S19 and S20", false, true),
            unknown("U4B", "leakage after S20", false, true),
            Entry {
                id: "U5",
                what: "fuel for steam and grid electricity",
                inlet: false,
                kmol_c_h: None,
                status: Status::OutOfBoundary,
                to_air: true,
            },
            unknown("U6", "start-up, flaring and fugitive methane", false, true),
            specified(
                "U7",
                "carry-over in the PSA product, zero by the 99.9 / 0.1 mol% product",
            ),
            specified(
                "U8",
                "gases dissolved in the condensate, zero in the flash model",
            ),
        ]
    }

    /// Recycled share of the tail-gas carbon, the quantity INL's split fixes.
    ///
    /// eq: J.10
    pub fn recycle_share(&self) -> f64 {
        self.recycled / self.tail_gas
    }

    /// Per-pass capture and plant capture, printed together.
    ///
    /// eq: J.33
    /// eq: K.14
    pub fn capture_pair(&self) -> (f64, f64) {
        (self.captured / self.co2_abs, self.captured / self.feed)
    }
}

/// Knowledge status of one carbon stream in the account (equation N.6.1).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Status {
    /// Computed by the flowsheet.
    Modelled,
    /// Fixed by a stated specification, with its reason.
    Specified,
    /// Present in the plant and not modelled. The flow is absent, never zero.
    Unknown,
    /// Outside the plant envelope by definition.
    OutOfBoundary,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Modelled => "modelled",
            Status::Specified => "specified",
            Status::Unknown => "unknown",
            Status::OutOfBoundary => "out_of_boundary",
        }
    }
}

/// One line of the carbon account. `kmol_c_h` is `None` for a stream whose size
/// the repository does not know.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub struct Entry {
    pub id: &'static str,
    pub what: &'static str,
    pub inlet: bool,
    pub kmol_c_h: Option<f64>,
    pub status: Status,
    /// True when the stream, or part of it, can reach the atmosphere.
    pub to_air: bool,
}

/// Verdict of a closure test.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Verdict {
    Closed,
    Open,
    Undetermined,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Closed => "closed",
            Verdict::Open => "open",
            Verdict::Undetermined => "undetermined",
        }
    }
}

/// Closure report of the account (equations N.6.1 and N.10.4).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AccountReport {
    /// Absolute residual over every entry, `None` while a stream is unknown.
    pub residual: Option<f64>,
    pub relative: Option<f64>,
    pub tol: f64,
    /// Residual over the modelled and specified entries only.
    pub residual_modelled: Option<f64>,
    pub relative_modelled: Option<f64>,
    /// Closed or open on the modelled and specified entries.
    pub on_modelled: Verdict,
    /// Never closed while a stream is unknown.
    pub overall: Verdict,
    pub unknown: Vec<&'static str>,
    pub out_of_boundary: Vec<&'static str>,
}

fn verdict(c: Option<CarbonClosure>, tol: f64) -> Verdict {
    match c {
        None => Verdict::Undetermined,
        Some(c) if c.relative_residual.abs() <= tol => Verdict::Closed,
        Some(_) => Verdict::Open,
    }
}

/// Close the account at tolerance `tol`. A stream with unknown status makes the
/// overall verdict undetermined, and a missing flow is never entered as zero.
///
/// eq: N.6.1
/// eq: N.10.4
pub fn closure_report(entries: &[Entry], tol: f64) -> AccountReport {
    let counted = |e: &&Entry| e.status != Status::OutOfBoundary;
    let known = |e: &&Entry| matches!(e.status, Status::Modelled | Status::Specified);
    let flows = |inlet: bool, keep: &dyn Fn(&&Entry) -> bool| -> Vec<Option<f64>> {
        entries
            .iter()
            .filter(|e| e.inlet == inlet)
            .filter(|e| keep(e))
            .map(|e| e.kmol_c_h)
            .collect()
    };
    let all = close_carbon(&flows(true, &counted), &flows(false, &counted));
    let modelled = close_carbon(&flows(true, &known), &flows(false, &known));
    let unknown: Vec<&'static str> = entries
        .iter()
        .filter(|e| e.status == Status::Unknown)
        .map(|e| e.id)
        .collect();
    let overall = if unknown.is_empty() {
        verdict(all, tol)
    } else {
        Verdict::Undetermined
    };
    AccountReport {
        residual: all.map(|c| c.residual_kmol_c_h),
        relative: all.map(|c| c.relative_residual),
        tol,
        residual_modelled: modelled.map(|c| c.residual_kmol_c_h),
        relative_modelled: modelled.map(|c| c.relative_residual),
        on_modelled: verdict(modelled, tol),
        overall,
        unknown,
        out_of_boundary: entries
            .iter()
            .filter(|e| e.status == Status::OutOfBoundary)
            .map(|e| e.id)
            .collect(),
    }
}

/// Ratio of two reported CO2 streams. Plant capture comes from
/// [`CarbonLedger::capture_pair`] instead.
pub fn reported_co2_stream_ratio(captured: f64, emitted: f64) -> Option<f64> {
    if !(captured.is_finite() && emitted.is_finite())
        || captured < 0.0
        || emitted < 0.0
        || captured + emitted <= 0.0
    {
        return None;
    }
    Some(captured / (captured + emitted))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eq_j_11_closed_synthetic_case() {
        let r = close_carbon(&[Some(100.0)], &[Some(60.0), Some(40.0)]).unwrap();
        assert!(r.residual_kmol_c_h.abs() < 1e-12);
    }

    #[test]
    fn eq_j_11_unclosed_case_keeps_its_residual() {
        let r = close_carbon(&[Some(100.0)], &[Some(90.0)]).unwrap();
        assert!((r.residual_kmol_c_h - 10.0).abs() < 1e-12);
        assert!((r.relative_residual - 0.1).abs() < 1e-12);
    }

    #[test]
    fn eq_j_11_missing_tail_gas_is_never_zero() {
        assert!(close_carbon(&[Some(100.0)], &[Some(90.0), None]).is_none());
        assert!(close_carbon(&[], &[Some(1.0)]).is_none());
        assert!(close_carbon(&[Some(f64::NAN)], &[Some(1.0)]).is_none());
    }

    #[test]
    fn carbon_counts_every_carbon_species() {
        let mut n = [0.0; NSP];
        n[Sp::Ch4 as usize] = 1.0;
        n[Sp::Co as usize] = 1.0;
        n[Sp::Co2 as usize] = 1.0;
        n[Sp::C2h6 as usize] = 1.0;
        n[Sp::C3h8 as usize] = 1.0;
        n[Sp::Nc4h10 as usize] = 1.0;
        n[Sp::Nc5h12 as usize] = 1.0;
        n[Sp::H2 as usize] = 5.0;
        n[Sp::H2o as usize] = 5.0;
        n[Sp::N2 as usize] = 5.0;
        assert_eq!(carbon_of(&n), 17.0);
        let s = Stream::gas("X", 300.0, 1.0e5, n);
        let el = s.elements().unwrap();
        assert!((carbon_of(&s.n) - el[0]).abs() < 1e-12);
    }

    #[test]
    fn reported_ratio_is_a_stream_ratio() {
        let r = reported_co2_stream_ratio(1927.0, 142.0).unwrap();
        assert!((r - 1927.0 / 2069.0).abs() < 1e-12);
        assert!(reported_co2_stream_ratio(-1.0, 1.0).is_none());
    }
}
