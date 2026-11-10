# Workstream 1: reference-plant engine

Authority: IEAGHG Report 2017-02, Techno-Economic Evaluation of SMR Based Standalone (Merchant) Hydrogen Plant with CCS, **Base Case** (unabated), not any CCS case. Read `reference/IEAGHG_REFERENCE_SPEC.md` (sections A to E) and `reference/IEAGHG_REFERENCE_SPEC_CRITIQUE.md` completely before writing code. The critique overrides the specification where they conflict. The report PDF is public; download it from the IEAGHG publications page and cite PDF page numbers. Do not commit the PDF.

## 1. Verified facts to build on (re-verify each against the PDF)

| Fact | Value | Source (PDF page) |
|---|---|---|
| Hydrogen product | 100,000 Nm3/h, 8.994 t/h | Table 2 (p16) |
| NG feedstock and furnace fuel | 26.231 t/h and 4.332 t/h, total 30.563 t/h | Table 2 (p16), streams 1 to 3 (p44) |
| NG LHV | 46.50 MJ/kg; energy input 394.77 MW; hydrogen energy 299.70 MW | Table 2 (p16) |
| Gross and net power | 11.500 MWe gross, 9.918 MWe exported | Table 2 (p16), utility sheet (p48) |
| Specific CO2 | 0.8091 kg/Nm3 H2, which is 8.996 kg/kg H2 (calculated) | Table 2 (p16) |
| Stack CO2 | stream 9: 8,659.4 kmol/h x 0.2123 x 44.0095 = 80.9067 t/h | stream table (p44) |
| PSA | feed stream 7, tail gas stream 8 (60,658 kg/h, 51 mol percent CO2), recovery about 90.0 percent (calculated, range 85 to 90 percent stated) | p44, p45, sheet 14 (p37) |
| Steam to neighbours | 0 (no steam export; internal turbine only) | block flow diagram (p34), utility sheet (p48) |
| Operating hours | 8,322 h/y | sheet 92 (p229) |

Answer already established: the published 0.8091 includes the burning of the tail gas and the CO2 that passes through it unburnt. 85.8 percent of the stack CO2 is feedstock carbon that left in the tail gas, 14.2 percent is supplementary gas. Not published and not to be invented: tail-gas LHV (derive with engine constants, 101.95 MW), combustion air flow, any furnace thermal efficiency.

## 2. Work packages (serial writers, shared files owned by one package at a time)

**WP0 foundation.** Commit the snapshot check (already `84dca45`). Add registry entries: 103 stream entries (`REF_S{nn}_{FIELD}`), the scalar entries of specification B.4, flags `REF_BASIS` (ship as 0), `SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND` (0), `SUBSIDY_SWEEP_STEP` (0.05), `SUBSIDY_GAP_LIMIT_GROSS` (0.01), constant `KCAL_KJ = 4.1868`. Add `engine/plant/src/refplant.rs` (structs and reader only). Never edit the text of an existing registry row (critique C01). Nothing numerical may change; legacy and snapshot gates must pass.

**WP1 reference core.** In `refplant.rs`: species and carbon flows, component-wise PSA balance `n_i,feed = n_i,product + n_i,tail`, tail-gas LHV, fired duty, supplementary gas, CO2 breakdown with no double counting, intensity, gas energy by energy (not volume), steam and power balance, scaling to plant hydrogen, blocking and non-blocking checks, stream-table rows labelled published, calculated or unavailable. Equations N.R.1 to N.R.17 as in the specification. `psa_tail(recovery)` and the constant-fired-duty form are test-only (critique C09). Tight CO2-wash bound (C08). Tests in `engine/plant/tests/m12_reference_plant.rs`, one per equation, expected values written in the test from the printed report table, not copied from the engine.

**WP2 subsidy core.** See `CARBON_AND_SUBSIDY.md`. Independent of WP1 files but shares `economics.rs`; run it after WP1.

**WP3 integration.** One accessor `baseline_kg_kg(Ieaghg)` returns the reconstructed value only on the independent basis with `REF_BASIS = 1` (critique C04); replace the direct reads of `Model.base_ieaghg` listed there. Wire into: baseline CO2 intensity; the unabated gas energy of the GAS line (energy basis); `LOST_STEAM` (reference steam to neighbour is 0 by publication); `BASE_POWER` (export revenue the reference earns and this plant does not); the K.18 system adjustment; the upstream credit; the N.9.2 decomposition. Keep the INL pair as labelled historical rows. Prove there is no double counting of power or steam against the plant's own power balance (pump and compressor lines, power island, reboiler heat); if a conflict appears, stop and record it as a blocker. Add the plant tail-gas heat register (S14 reject, S03 recycle, S21, S15, S16 stack) so that heat is claimed once. `Integrated_Mass_Balance.csv` and `Integrated_Energy_Balance.csv` stay byte-identical. The outage backup rows stay on the INL basis on purpose and say so (C07).

**WP4 exports.** New ninth table `results/Reference_Plant.csv` (`engine/plant/src/export_ref.rs`): published streams with ids, species and element balances with residuals, PSA balance, furnace and tail-gas register, CO2 breakdown, steam and power balance. Add the 14 validation rows listed below. Add CLI flag `--ref-basis reconstructed|historical`. Update scripts (`build_numbers.py`, `compare_results.py`, `derivations_tools.py`, `build_site.py`, `check_site.py`), both CI workflows, the CSV-count test (nine), and the `PUBLIC_MD` list for any new public Markdown. Only in this package flip the defaults (`REF_BASIS = 1`, default subsidy definition C) and regenerate `results/` (critique C03), then update committed-table tests and goldens that must move, each with a recorded reason. Never edit a golden merely to pass.

## 3. The 14 validation rows

`REF_TABLE2_ENERGY_IN` (1e-4), `REF_TABLE2_ENERGY_H2` (1e-4), `REF_TABLE2_NET_POWER` (1e-9), `REF_INTENSITY_VS_TABLE_2` (5e-4), `REF_CARBON_CLOSURE_NG_TO_STACK` (2e-3), `REF_PSA_SPECIES_MAX_RESIDUAL` (0.5 kmol/h absolute), `REF_PSA_RECOVERY_IN_RANGE` (store the rounding note; derived 0.900058 against the printed 85 to 90 percent), `REF_UTILITY_FUEL_VS_STREAM_3` (5e-3), `REF_STEAM_STREAM_10_VS_UTILITY` (5e-3), `REF_POWER_REVENUE` (1e-6), `REF_NG_STREAM_SPLIT_MASS` (exact), `REF_STEAM_TO_NEIGHBOUR_ZERO` (exact), `REF_LCOH_BRIDGE` (3e-2, independent first-order check against the report LCOH), and `IND_FLOOR_VS_IEAGHG` (existing row, re-pointed). New `REF_` rows must not start with a gate prefix listed in `engine/plant/src/gates.rs`, and no gate outcome may flip. If one flips, stop and report.

## 4. Acceptance criteria

- Every registry number cites a PDF page and table, or states it is calculated or assumed.
- Carbon from natural gas to stack closes within 0.2 percent; PSA species residuals within 0.5 kmol/h; stack CO2 equals 0.8091 kg/Nm3 within 0.1 percent; net power equals gross minus internal within 1e-9; PSA mass closes within 1 kg/h.
- With `REF_BASIS = 0`, every independent-basis table equals the pre-change snapshot; with the legacy basis, every table equals the legacy snapshot; both balance tables are byte-identical on both bases.
- Independent closed-form check of the Case B shifts agrees with the engine within 0.05 S$/t, otherwise stop and find the interaction.
- Clean-build note: after the first full rebuild, one cargo job at a time (see `TESTING_AND_VERIFICATION.md`, fork errors).

## 5. What this workstream must not do

Substitute the INL reference as the default plant. Store or derive a furnace efficiency. Change plant thermodynamics, the MEA or cryogenic models, `RHO_H2_NORMAL`, or the drift gate (which stays blocking on the legacy basis only).

## 6. Archive material

Use `process-data/session-dee70430/phase3b/ieaghg_streams.md` and `ieaghg_balances.py` in raystorage as the extraction record. Verify every value against the report PDF; port the checks into Rust tests that do not read the Python script. The recorded warning stands: 0.8091 kg/Nm3 already includes tail-gas combustion; never add it again.
