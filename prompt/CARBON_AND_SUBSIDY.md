# Workstream 2: carbon abatement and subsidy

## 1. Decision log to write into docs/ABATEMENT_SUBSIDY_LOG.md

`docs/ABATEMENT_SUBSIDY_LOG.md` currently ends at D12. Add a section "Owner decisions of 6 and 7 November" with D13 to D23 below, rewritten in the log's style. Preserve the history: D2 was an earlier default (generic IEAGHG intensity) that a verifier flagged as awaiting the owner's acceptance; the owner has since accepted the IEAGHG-based specification, which settles it. Do not invent decisions that are not listed here. Items marked default were applied by the previous session and still await the owner's explicit confirmation.

| Id | Substance | Status |
|---|---|---|
| D13 | Cost and carbon share one conventional reference. On the independent basis both come from the IEAGHG 2017-02 base case (gas 14.212 MJ/Nm3 H2 and CO2 0.8091 kg/Nm3, one table). The legacy basis keeps INL for both. | Owner answer 6 Nov |
| D14 | Cryogenic carbon recovery f = 0.752178 and hydrogen return r = 0.921630 stay labelled design assumptions, calibrated to INL's published pair (INL Case 6, Case 5 stream tables, TEV-961 assumptions). No source supports them for the NUS-H2 unit. Bands: f 0.7299 to 0.7718, r plus or minus 0.05. Context references only: IEAGHG 2017-02 Case 2B, Air Liquide Cryocap H2 material. | Owner answer 6 Nov |
| D15 | Shipping distance 1,000 km central, sweep 200, 600, 1,000, 1,400, 2,000 km. Candidate sites named: Kasawari CCS (Sarawak, Block SK316, about 200 km offshore of Bintulu) and Sunda Asri (offshore West Java). Coordinates not verified; distances are assumptions. Policy basis: the Singapore and Indonesia letter of intent of February 2024 and Indonesian Presidential Regulation 14 of 2024 (30 percent of storage capacity open to imported CO2). No Singapore to Malaysia agreement found. | Owner answer 6 Nov |
| D16 | Unknown streams U1 (MEA make-up and degradation), U2 (CO2 vent between absorber product and pipeline), U4b (leakage after storage), U6 (start-up, shutdown, flaring, fugitive methane). Closure verdict stays Undetermined. Abatement is an upper bound on modelled streams; any pass is conditional. Never enter an unknown as zero. | Owner question 6 Nov, defaulted |
| D17 | A committed snapshot of about 1.2 MB of legacy tables is accepted. | Owner answer 6 Nov |
| D18 | IEAGHG 2017-02 Base Case is the authoritative, reconstructed conventional reference. The plant-built floor (about 7.85 kg/kg) and INL become labelled historical or comparison rows. | Owner answer 7 Nov |
| D19 | Lost-steam and replacement base-power costs stay in the primary result, recalculated from IEAGHG's steam and electricity balances; INL value kept as historical comparison; PSA tail gas explicit, never double counted. | Owner answer 7 Nov |
| D20 | Definition C: uniform share of total integrated project CAPEX, 0 to 100 percent, hypothetical risk-sharing policy; definition B demoted to labelled legacy comparison. | Owner answer 7 Nov |
| D21 | Island treatment: the grant on the SMR island is mirrored on the reference so it cancels from the abatement cost (like for like); the literal project-side reading is a labelled sensitivity ("includes a windfall on shared equipment"). | Default, awaits confirmation |
| D22 | Electricity S$0.20/kWh for export and import, ratio 1.0 as the report uses one price. Report-equivalent S$0.186/kWh as a sensitivity. No sourced Singapore export tariff exists in the repository. | Default, awaits confirmation |
| D23 | The power island (20.124 M EUR TPC) stays inside the island on both sides; a switch `SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND` and a sensitivity row report the effect. | Default, awaits confirmation |

Also record the finding that decision records D13 to D17 existed only in session notes and are transcribed here. Locate the originals first (the working notes of the previous session are not in the repository; the verified wording above is the transcription).

## 2. Definition C (work package WP2)

Full design in `reference/IEAGHG_REFERENCE_SPEC.md` section D with critique items C02 and C05. Requirements:

- `S_gov = f_sub x C_CAPEX,total`, f from 0 to 1, sweep nodes `k x 0.05` generated from integer k (21 nodes; the existing 0, 25, 50, 75 are nodes). Rows: A and B keep the four-point matrix; C uses the sweep (720 rows expected; 1,072 in total).
- Base lines (D.2): reactor, IHX, licence, incremental power island, capture plant (TCR basis), recycle train (TCR-equivalent), SMR island (reported on the integrated total, mirrored on the reference so it cancels from the cost). Excluded: interest during construction (financing result, never granted twice), decommissioning, replacement units (labelled sensitivity only), fixed O&M and consumables, working capital (inside TCR lines).
- Report separately: unsubsidised abatement cost; subsidised private cost; total government expenditure (nominal and year-0 value); private investment with and without interest during construction; break-even share for the strict S$100/t target by L5, DCF and both, per stage (FOAK, BOAK, tenth-of-a-kind from BOAK and from FOAK) and life (25 y, 60 y). Report honestly when no share up to 100 percent reaches the target.
- Safeguards, each with a named test: the subsidy changes no physical struct, operating cost or abatement; no double counting of grant, interest, depreciation (the model is real and pre-tax; write down why no depreciation base exists) or capital recovery; grant at close and progressive both supported; strict inequality at exactly S$100/t is not met.
- Do not restate the L5 against DCF gap limit against gross cost for A and B (critique C02). Keep `gap_rel`, `gap_within`, `SUBSIDY_GAP_LIMIT` (decision D11) unchanged; add `gap_share_of_gross_l5` and `SUBSIDY_GAP_LIMIT_GROSS` as separate new checks for definition C.
- Every output and document labels the policy hypothetical.

## 3. Missing U6 unit test

Finding 4 of the verifier: `eq_n_6_1_verdicts_are_never_closed_while_a_stream_is_unknown` asserts only that U1_IN and U2 are unknown; U1_OUT, U4B or U6 can be set to a known zero with no cargo test failing (a mutation that sets U6 to zero was caught only by the exported-table comparison). Add a test in `engine/plant/tests/m11_carbon.rs` that calls the carbon account directly and asserts, for each of U1_IN, U1_OUT, U2, U4B and U6: its status is Unknown, setting it to a known zero is not accepted by the closure report (verdict stays Undetermined for any single unknown), a limiting case where every other stream balances still gives Undetermined, and the headline abatement function labels the result an upper bound naming the unknown ids. Also test that the modelled-streams residual is computed without the unknown entries and that a known non-zero value for U6 changes the abatement in the right direction (unknown air outlets can only lower abatement). Confirm by a mutation (set U6 to a known zero) that the new test fails.

## 4. The nine stale SymPy checks

`math/verify/sympy_carbon.py` has 7 checks that compare legacy values with `results/` and fail because `results/` now holds the independent basis. `math/verify/sympy_subsidy.py` has checks C19 and C15b that are stale. The derivation text still states "115 of 115 checks pass". For each check decide, with written reasoning in `docs/ABATEMENT_SUBSIDY_LOG.md`: repair it (point the fixture at the legacy snapshot `engine/vv/data/legacy_results` when it tests the legacy basis, at current results when it tests the independent basis); or replace it by an equivalent symbolic or numerical assertion; or remove it with a stated reason. Never weaken a tolerance. Keep the check count and strength at least equal. Update the derivation's verification paragraph to the true count.

## 5. Nine minor verifier findings (resolve each, record against the original)

| # | Where | Finding | Resolution task |
|---|---|---|---|
| 1 | `abatement.rs` plant_balance; registry text of CO2_CAPTURED_C6, NG_CASE6, ETA_SELEXOL, PHI_RECYCLE | Cost still depends on INL Case 6 through the heat and power balance; the four legacy entries are not labelled as feeding the independent cost. | Label them in the new `REF_`/documentation rows without editing existing registry text (use the new-rows route of critique C01); document the dependency in `ECONOMIC_ASSUMPTIONS.md`. |
| 2 | `FLOOR_L5_*`, `FLOOR_LAC_DCF_*` rows | Floor cost keeps INL's LOST_STEAM and BASE_POWER lines; a reference with no firing gas cannot raise that export steam, so the floor cost is overstated; no row combines the floor with the system lines off. | Re-derive the floor lines from the reconstructed reference, or add the combined row; label. |
| 3 | `export.rs` lifecycle views for ids CHO_9p35 and INL_9p21 | Upstream gas credit of the comparison baselines uses the central reference gas, so rule 1 of N.7 holds only for the central row. | Compute each comparison row's upstream credit from its own reference gas; test. |
| 4 | `m11_carbon.rs` eq_n_6_1 | Covered by the U6 test above. | Done when the new test exists and the mutation fails it. |
| 5 | `Final_Design_Basis.csv` RES_ABATEMENT_T_Y and related rows; derivation N.5.2, N.9.4 | Headline abatement rows do not say upper bound with respect to the unknown streams although the derivation promises it. | Add the upper-bound wording and the unknown ids to the row text and every display of the figure. |
| 6 | `parameters.toml` TS_PURGE_FRAC | The stated arithmetic does not reproduce the value, and a CO2e-derived number enters a CO2 headline quantity (decision D4). | Re-derive from a published source in CO2 units, or relabel as assumed with the correct arithmetic; do not change a legacy value silently. |
| 7 | `Final_Design_Basis.csv` source column | 34 `carbon_independent` rows (including RES_ABATEMENT_T_Y, RES_ATM_T_Y) and 97 `gated_T05` cost rows carry the stale source "engine anchored port"; RES_ATM_T_Y keeps the label "Atm t y". | Correct provenance and labels in the exporter; regenerate. |
| 8 | `sympy_carbon.py` and derivation 01 verification text | Script exits 1; text says 115 of 115 pass. | Covered by section 4. |
| 9 | Validation rows `IND_*`; equation-id comments | Two of the six new passes are calibration tautologies (disclosed); equation ids N.1.3, N.5.1, N.6.2, N.8.1, N.8.3 appear in test names but in no code comment; N.10.1 to N.10.3 share one test. | Add the id comments; split the shared test into one per equation; relabel tautological rows so they cannot read as independent evidence. |

## 6. Upper-bound qualification

While U1, U2, U4b and U6 are unknown the abatement figure is an upper bound on the modelled streams. Every place that prints it (results rows, README, site KPI, tour, PDF, final report) says so and names the four streams. A pass against S$100/t that depends on it is printed as conditional. Never convert it into an asserted value and never invent a value for an unknown stream. Sourced bounds may be added only when a primary source is read in full.

## 7. Archive material

The original wording of D13 to D17 is at the end of `derivations/session-dee70430/phase3/carbon_plan.md`; the nine minor findings, the stale SymPy logs and a mutation harness are under `validation/session-dee70430/phase3/` (see `RAYSTORAGE_FILE_INDEX.md`). Do not invent missing decisions: D18 to D23 exist only in this package.
