# Progress log: independent CO2 abatement and government co-funding

Branch `feature/independent-abatement-subsidy`, cut from `main` at 8750b09. Each phase adds an entry that records what was done, what is outstanding, what failed, which files changed and which tests ran. A task is marked complete only when its evidence exists.

## Phase 1: audit (complete)

Completed:

- Seven read-only audits mapped the carbon balance, the provenance of external values, the L0 to L5 ledger, the discounted cash flow, the parameters, the exporter and test chain, and the browser simulator. The record is `docs/ABATEMENT_SUBSIDY_REVIEW.md`.
- The headline abatement of Case B rests on INL numbers. The unabated reference (TEV-953, 3,205 st/d scaled to this plant) supplies 93.8 percent of it, and the residual emission comes from a ledger anchored to INL Case 5 and Case 6 values. Air Liquide supplies only the 100,000 Nm3/h capacity. This is the copied result that the rebuild replaces.
- The existing co-funding is a reactor-capital multiplier aimed at S$96/t. It is not reused as the grant model.
- The L5 ledger and the DCF share one cost item set, so their close agreement (0.47 percent of L5 at every one of 72 points) cannot expose an error that both methods share. The new checks therefore include an independent closed-form calculation of the grant effect.

Outstanding from Phase 1: the baseline engine run on `main` (format, tests, export, comparison with `results/`). Its result is recorded below.

Failures: none in the audit.

Limits: the auditors did not run the engine. The source PDFs of INL, IEAGHG and Cho are not in the repository, so no quoted value was checked against its source page. That check belongs to Phase 2.

Files modified: `docs/ABATEMENT_SUBSIDY_REVIEW.md` (new), `docs/ABATEMENT_SUBSIDY_LOG.md` (new).

Tests run: none by the audit.

### Baseline run on main

Run on the branch before any change, at the `main` commit 8750b09, with the toolchain pinned by `rust-toolchain.toml`:

- `cargo fmt --all --check`: passed.
- `cargo test --workspace`: 368 passed, 0 failed, 0 ignored.
- `nush2 export` followed by `scripts/compare_results.py results <regenerated>`: all seven tables agree, 0 cells outside tolerance.
- Validation table: 650 rows, of which 325 pass, 32 fail, 251 are reported and 42 are not run. The 32 failures are literature benchmarks and stay recorded as failed.
- `cargo clippy` and the browser self test were not rerun in Phase 1.

This run is the zero-subsidy regression reference. The legacy basis with zero subsidy must reproduce these tables exactly.

## Decisions (defaults chosen at the start; each can be changed)

- **D1 Abatement basis.** The new default basis is computed from the plant's own carbon balance, starting from the hydrogen output and the natural gas composition. The INL-anchored ledger stays in the engine as a selectable legacy basis and as a validation reference. With the legacy basis and zero subsidy the engine must reproduce `results/` at 8750b09 cell for cell.
- **D2 Unabated reference (revised after Phase 2).** The central baseline is a documented generic steam methane reforming intensity multiplied by the annual hydrogen output: IEAGHG 2017-02 Case 1, 8.996 kg CO2 per kg H2. It belongs to the plant whose capital cost the ledger already uses for the SMR island, and it is the lowest of the three published intensities, so it gives the lowest abatement and cannot favour the target. The rule was fixed before any new result was computed. INL (9.21) and Cho (9.35) stay as comparison rows, the target verdict is reported under each, and the intensity built from the plant's own feed is kept as a floor check. Status after fix round 2 (below): the owner has not yet accepted D2 in this form. Wherever the headline is shown it now says generic reference intensity, and the cost and break-even shares under the plant-built floor are printed beside it.
- **D3 Target test.** The target is met only when the cost is strictly below S$100/t. The break-even subsidy share is the infimum at which the cost equals S$100/t, and a share passes only when it lies above it. L5 and DCF are tested separately, and the overall verdict requires both to pass.
- **D4 Gas basis.** Headline quantities are kg CO2. Any CO2e quantity appears only in the lifecycle view, with its GWP horizon and source stated beside it.
- **D5 Existing outputs.** The S$96/t co-funding sweep, the MATURE_60Y scenario and the 40 y life stay in the existing tables. The new scenario matrix covers 25 y and 60 y.
- **D6 Output table.** The matrix is written to a new eighth table, `Subsidy_Scenarios.csv`. Every hard-coded list of the seven tables is updated in the same change.
- **D7 Grant conventions (settled in Phase 2).** Financial close sits at the start of construction, year minus t_c, in both methods, and zero subsidy changes nothing in either. The default timing is progressive, pro rata with the eligible spend, and the grant paid at close is reported beside it under the rule that the grant is spent first and idle grant cash earns nothing. Interest during construction is never an eligible cost and is charged on the private balance outstanding. Replacement units are never eligible, and a one-off grant on an item that is replaced is annualised over the project life. The share grid is 0, 25, 50 and 75 percent. The 75 percent case is labelled a stress case, because no plant-capital precedent was found and Singapore has no capital grant scheme for nuclear or nuclear-heated hydrogen.
- **D7b Eligible items.** Definition A is the reactor overnight capital. Definition B is the reactor, the IHX, the capture plant, the recycle train and the licence, which follows the eligible-cost lists of the EU Innovation Fund and the UK Net Zero Hydrogen Fund. Excluded: the power island, decommissioning, transport, storage, interest during construction, replacement units and the SMR island that both plants carry. Membership is set by flags in `parameters.toml`, so it can change without code changes.
- **D7c Transport and storage.** The flat ratio of 0.025 t per t, which has no verified source, is replaced by physical inputs: conditioning power, ship fuel per 1000 km, terminal operations, boil-off and leakage. The shipping distance has no source in the repository and enters as a labelled assumption with a sensitivity band. The old ratio stays as a cross-check.
- **D8 Cryogenic design inputs.** The carbon recovery f and the hydrogen return r of the cryogenic unit cannot be fixed by atom balances. They stay as design inputs (assumed, calibrated to INL's published pair) and are reported with a sensitivity band. Abatement then follows from the atom balances given them. These two numbers carry INL information into the result, as the capture efficiency of 0.90 carries a design value.
- **D9 Headline pairing.** The cost per tonne uses net abatement after transport and storage on the direct boundary. The system boundary, which adds the carbon of lost steam and base power, is reported beside it, and the report says so wherever the two verdicts differ.
- **D10 Benchmarks.** Failed literature benchmarks stay failed (the MEA column gate and the flowsheet carbon gate), and no tolerance is weakened. The blocking drift check at 578,000 t/y and the screening goldens belong to the legacy basis. On the independent basis they become reported validation rows.
- **D11 L5 against DCF.** The existing 10 percent limit cannot see a grant, because the largest zero-subsidy gap is 0.204 percent of L5. Subsidised points are tested against the gap predicted by the derivation and against a 1 percent limit.
- **D12 Legacy regression.** The export gets a `--basis legacy|independent` switch with independent as the default. In CI, the legacy output at zero subsidy is compared with a committed snapshot of the tables at 8750b09.

## Phase 2: independent derivation (complete)

Completed:

- Two derivations with every equation worked step by step: `docs/derivation/01_carbon_and_abatement.md` (33 equations, N.1.1 to N.10.4) and `docs/derivation/02_cost_and_subsidy.md` (27 equation blocks, S.0.1 to S.9c). The lines for the Rust implementation and the validation test wait for Phases 3 and 4.
- SymPy scripts `math/verify/sympy_carbon.py` (115 of 115 checks pass) and `math/verify/sympy_subsidy.py` (120 of 120 pass). Both were rerun after the verifiers' edits.
- Two independent verifiers re-derived the key relations. Carbon closes unit by unit on all six Level 3 runs (worst residual 1.2e-9). The zero-subsidy L5 and DCF are reproduced for FOAK, BOAK and both tenth-of-a-kind routes at 25 y and 60 y, with a largest difference of 3.8e-6 S$/t, which is the rounding of the CSV.
- 63 references in `docs/ABATEMENT_SUBSIDY_REFERENCES.md`: 56 fetched and read, 5 seen only as search snippets and 2 not verified. Entries that were not read in full support no parameter.

Findings that matter for Phase 3:

- The cost is exactly affine in the share for a progressive grant, so the break-even share has a closed form. A grant paid at close is convex, with a closed-form root on the ledger side and a piecewise-linear root on the DCF side.
- The two interest during construction forms differ by a factor 0.0026 (the DCF half-year bucket earns simple interest where the ledger compounds a fractional exponent). The derivation keeps both so that zero subsidy reproduces today's results.
- L5 contains LOST_STEAM and BASE_POWER, but the direct abatement leaves their carbon out. On the legacy row the system abatement is 8.72 percent lower, and the cost per tonne 9.6 percent higher.
- No source was found for upstream gas intensity of pipeline imports, for an electricity price at the plant basis, or for the regulatory permanence wording. Singapore's carbon tax (S$45 in 2026 and 2027, an aim of S$50 to 80 by 2030) is below the S$100/t target and serves only as a comparator.

Still open after the verifiers: bounds for dissolved gases in the condensate and for carry-over in the product, one set of identity checks in the carbon script that carry no independent content, some symbols missing from the Reference Frame of the subsidy derivation, and a tautological check in the subsidy script.

Files modified: `docs/derivation/` (2 new), `math/verify/` (2 new), `docs/ABATEMENT_SUBSIDY_REFERENCES.md` (new), `docs/ABATEMENT_SUBSIDY_REVIEW.md` (renamed from the Phase 1 record, because the repository text guard rejects file names that end in the word audit).

Tests run: both SymPy scripts, and `scripts/check_clean.py` over the repository (0 hits).

## Phase 3: backend implementation, independent carbon basis

This entry covers the carbon track. The co-funding model is recorded by its own commits. The branch head at the start of this work was 9471f9f, which already held the co-funding model, so the legacy snapshot comparison skips the rows and the table that track added.

Completed:

- The plant's own stream ledger now drives the default. `engine/plant/src/abatement.rs` runs the Level 3 flowsheet on the hydrogen target, with the cryogenic carbon recovery f and hydrogen return r read from the registry as flagged design inputs (decision D8), and reads captured CO2, stack, condensate CO2, product methane and the gas rate from it. Closure carries the unmodelled streams U1, U2, U4b and U6 as unknown, never zero, so the whole-account verdict is undetermined and the verdict on the modelled streams is closed at a relative residual of 4.3e-12 against the tolerance of 1e-9.
- The central unabated reference is the IEAGHG 2017-02 Case 1 intensity times the annual hydrogen (decision D2). INL, Cho and a plant-built floor (the same flowsheet without capture, recycle or firing gas) are comparison rows.
- Transport and storage emissions come from physical inputs (decision D7c): a 1000 km ship at 6.77 g fuel per t per km, conditioning and injection electricity at the 2020 grid factor, terminal operations and purge. The sum is 0.03242 t per t against the flat 0.025, which stays as a cross-check row. The shipping distance is a labelled assumption with a band of 200 to 2000 km.
- The legacy basis is selectable and bit-identical: `CARBON_BASIS` in the registry, `--basis legacy|independent` on the command line (default independent) and a carbon basis control in the wasm reply. A snapshot of the seven tables at main 8750b09 sits in `engine/vv/data/legacy_results/`, `scripts/compare_results.py` gained `--subset`, `--names` and `--allow-cell`, and both workflows export with `--basis legacy` and compare with the snapshot.
- Tests: `engine/plant/tests/m11_carbon.rs` checks every equation N.1.1 to N.10.4 by id plus closure, non-negative streams, one hydrogen output for plant and reference, boundaries and the legacy equality. The existing tests that encode copied values keep passing on the legacy functions. Where a test compared the exporter with the committed tables or cost sweeps it now uses the plant basis, and the closed forms of the co-funding tests keep their literal anchors on the legacy basis.

Outstanding from this track:

- N.7.2 is a function and a test, and the central result does not call it: the furnace efficiency and the steam duty of the reference have no source, so every figure against the plant-built reference is a floor.
- N.9.5 was first carried as the CO2 view with the CO2e term at its zero default, on the belief that no value of a warming potential existed in the repository. Fix round 1 below replaces this: the lifecycle views are CO2e views with the horizon and source of each term stated.
- The heat and power balance keeps the anchored captured CO2 (359,777 t per year), while abatement, the transport and storage tonnage and the capture capital use the plant's 363,277 t per year, 0.97 percent more. A validation row prints the reboiler duty that the plant's own capture would imply.
- The GAS line of the cost ledger first kept INL's 52.5 MMSCFD reference gas, while the central intensity implies 35.09 MMSCFD at Case B against 35.87 for INL scaled. Fix round 1 below replaces this: the ledger prices the implied gas.
- The site scripts (`build_numbers.py`, `build_site.py`, `check_site.py`) and the frontend labels still describe the INL headline. They belong to Phases 5 and 6.

Tests run on the final tree of this track: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed, `cargo test --workspace` passed 462 tests with none failed, the default export agrees with `results/` in all eight tables, and the export with `--basis legacy` agrees with the snapshot of main 8750b09 in the seven tables it holds (650 validation rows, of which 325 pass, 32 fail, 251 are reported and 42 are not run, as at the baseline). The default export has 668 validation rows: 331 pass, 32 fail, 263 reported and 42 not run.

Failures: none new. The 32 failed literature benchmarks stay failed. Two of the new validation rows are reported comparisons with the copied INL values (net abatement 557,767 t per year against 578,052, and the 578,000 band of the drift check) and carry no limit.

## Phase 3, fix round 1: carbon track (findings of the independent verifier)

Two major findings of the verifier were fixed on the existing build. Nothing else was changed.

Finding 1, the cost per tonne still depended on a copied INL plant result. The L0 GAS line, the gas lines of the discounted cash flow, the saved upstream gas of the lifecycle view and the gas keys of the case summary priced the gas saved against INL's conventional-plant gas (`NG_CONV_NOCC`, 52.5 MMSCFD scaled to the plant), while the abatement reference is the central intensity. The two used unlike reference plants.

- `Model::reference_gas_mmscfd` and `Model::reference_carbon` (equations N.7.3 and N.9.2) now give the one reference gas of the independent basis: the carbon identity of N.7.3 solved for the gas that the central intensity implies. The ledger, the DCF, the lifecycle credit, the case summary and the decomposition all read it, so rule 1 of the boundary table of N.7 holds and a test compares the gas line with the decomposition to 1e-12. On the legacy basis the function returns INL's gas with the same floating-point expression as before.
- INL's 52.5 MMSCFD is kept as `NG_CONV_NOCC` for the legacy basis and as a comparison input of the cost (`CostQuery::ng_smr`). Three reported validation rows (`IND_GAS_LINE_UNDER_INL_GAS`, `IND_L5_UNDER_INL_GAS`, `IND_LAC_DCF_UNDER_INL_GAS`) print the cost under INL's gas beside the default. A test shows that the implied gas reproduces INL's gas within 0.5 percent when it is fed INL's own intensity.
- Effect on B_REC_F, BOAK, 25 y, n = 1.0: the implied reference gas is 35.092267 MMSCFD (INL scaled: 35.872251). The L0 GAS line moves from -65.478036 to -61.479836 S$M/y, L5 from 196.012113 to 203.180333 S$/t (+7.168220) and LAC_DCF from 196.332684 to 203.500904 S$/t. The abatement is unchanged at 557,767.415516 t/y. The break-even share of the progressive grant (both methods) moves from 0.643381 to 0.691256 for definition A and from 0.480129 to 0.515856 for definition B. Seventeen grid points of `Subsidy_Scenarios.csv` change `target_met` from true to false.
- The LOST_STEAM and BASE_POWER lines still use INL's conventional-plant steam and power (`EXPORT_STEAM_CONV`, `E_CONV_NOCC`). The verifier did not raise them and no source exists for a plant-built reference duty (`REF_STEAM_DUTY`); they stay as they were and are listed as open.

Finding 2, a CO2e quantity sat among CO2 rows. `EF_UPSTREAM` is 11.5 kg CO2e per GJ at a 100-year potential of 30 (IEA 2026, PDF pp. 4 and 6) and `EF_NUCLEAR` is a CO2e range of UNECE 2021 read through a secondary summary, with no horizon stated. Both enter the lifecycle views, whose rows carried the status `carbon_independent`, the source text of the CO2 ledger and no horizon.

- The fields of `Lifecycle` are now `up_credit_co2e` and `nuclear_co2e`. On the independent basis every row of `lifecycle_burdens` and `lifecycle_upstream` in `Emissions_Ledger.csv` says CO2e in its line text, names the horizon (`GWP100 = 30`) for the upstream term and says that the horizon of the nuclear range is not stated, and the source text names both factors. The transport and storage burden row says CO2. The legacy rows keep their committed wording.
- The registry gains `EF_UPSTREAM_GWP100` (30, published, IEA 2026 p. 6) as the label of `EF_UPSTREAM`; the unit and text of `EF_UPSTREAM` itself are untouched, so the legacy snapshot comparison still agrees. The wasm label of `abatement_lifecycle_t_y` says CO2e total.
- The split of `EF_UPSTREAM` into a CO2 part and a CH4 part, which N.9.5 asks for, was not done. The sources read state the intensity only as one CO2e total, and any split would need an invented share. The aggregate enters the CO2 slot of the closed form with the methane term at zero; a test shows that the exported lifecycle view equals `closed_form::lifecycle` on that factor. The split stays an open item (open issue 6 of the derivation).
- The guard test `no_co2e_quantity_appears_without_a_label_in_any_view` now walks every `lifecycle_*` view of every run on the plant ledger (126 rows, whatever status word a preset gave them) and checks every other view for the CO2e word. Applied to the committed table from before the fix it flags all 90 lifecycle rows that carry a CO2e term and no label.

Cells of `results/` that changed, and why:

- `Economic_Ledger.csv`: the L0 GAS value, low and high cells (88 cells) and every cumulative cell from L0 upward (1,232 cells in each of the two cumulative columns), the `dcf_value` cells (232) and one status cell (`LAC_L5_GAP_REL`, B_REC_F, 10OAK_P47, 40 y, from `gated_T05` to `anchored_port`, because the gap between the Headline and Anchored presets fell below the printed digits).
- `Emissions_Ledger.csv`: the line text of the 126 lifecycle rows, the source text of the 90 of them that carry a CO2e term (the 36 transport and storage rows are CO2 and keep their source), and the values of the 36 rows `UPSTREAM_CREDIT` and `ABATEMENT` of `lifecycle_upstream` with the matching per-kg cells (the saved upstream gas follows the implied reference gas).
- `Final_Design_Basis.csv`: 58 result rows (`AC_L3_*`, `AC_L5_*`, `LAC_DCF_*`, `LCOH_SMR_*`, the target-excess rows, `RES_NG_UNABATED_MMSCFD` and `RES_GAS_SAVING_INL`) and one new input row, `EF_UPSTREAM_GWP100`.
- `Sensitivity_Results.csv`: 619 result cells, all through the gas line and the L5 and DCF denominators of the cost sweeps.
- `Subsidy_Scenarios.csv`: every cost, interest, share and verdict cell that depends on the cost, including the 32 break-even shares.
- `Validation_Results.csv`: three new reported rows, a new explanation for `IND_REF_GAS_IMPLIED_VS_INL` and the registry entry count in `PARAMS_COMPLETE`.
- `Integrated_Mass_Balance.csv` and `Integrated_Energy_Balance.csv` are byte-identical.

Test fixture: `ten_oak_at_25_gas_reads_one_verdict` in the wasm tests needs a point that sits below the target. The old fixture (stage 10OAK_P46, gas at the control maximum of S$25/GJ) now reads 102.9 and 103.2 S$/t, so the fixture adds transport and storage at S$40/t (89.9 and 90.1 S$/t). No tolerance was changed.

Tests run on the final tree of this fix round: `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` passed, `cargo test --workspace` passed 468 tests with none failed, the default export agrees with `results/` in all eight tables, and the export with `--basis legacy` agrees with the snapshot of main 8750b09 in the seven tables it holds (650 validation rows: 325 pass, 32 fail, 251 reported, 42 not run, as at the baseline). The default export has 671 validation rows: 331 pass, 32 fail, 266 reported, 42 not run. `scripts/check_clean.py` scanned 245 files with 0 hits.

Outstanding after this round:

- LOST_STEAM and BASE_POWER still price INL's conventional-plant steam and power. They are the same kind of copied value as the gas was. No source exists for a plant-built reference duty, so they stay and are listed for the owner to decide.
- The CO2 and CH4 split of `EF_UPSTREAM` waits for a source that states it; the heating-value basis of the factor is not stated either.
- `math/verify/sympy_subsidy.py`: check C19 (a case with the abatement doubled, which expected a negative break-even share at the old cost level) now finds a small positive share, because the corrected L5 is above S$100/t at that rescaling; its fixture factor needs a larger value. Check C15b carries the literal 37.985 S$/t, which belongs to the legacy abatement and had already stopped matching when the independent abatement became the default. `math/verify/sympy_carbon.py` has seven checks that compare legacy values with `results/`, which now holds the independent basis; they should read `engine/vv/data/legacy_results/`. These scripts are outside this fix and were not changed.
- The frontend and site scripts still describe the INL headline and the lifecycle KPI without the CO2e wording (Phases 5 and 6).

Failures: none new. The 32 failed literature benchmarks stay failed.

## Phase 3, fix round 2: carbon track (findings of the second independent verification)

Three major findings were fixed on the existing build. Nothing else was changed: the central abatement, the L0 to L5 lines, the discounted cash flow and every other table value are as they were.

Finding 1, the reference side of the headline is a generic intensity (decision D2). The verifier offered two routes: obtain the owner's acceptance of D2 and carry the wording generic reference intensity, or make the plant-built reference the centre. The second route needs a furnace efficiency and a steam duty for the reference (`REF_FURNACE_EFF`, `REF_STEAM_DUTY`), for which the repository holds no source, so it was not taken. D2 is a recorded decision and the task fixes it as the centre, so it stays. The owner's acceptance of D2 in this form is not on record and is listed below as open.

- Wording: the cost rows of `Final_Design_Basis.csv` (`AC_L5_*`, `AC_L3_*`, `LAC_DCF_*`), the three headline abatement rows, the central rows of `Emissions_Ledger.csv`, the GAS line of `Economic_Ledger.csv`, the source text of every row of `Subsidy_Scenarios.csv` and the labels of the five headline indicators of the wasm reply say generic reference intensity. The legacy export keeps its committed text.
- Beside the headline: `Model::floor_reference` gives the plant-built reference (the floor of N.7.1 and N.7.3, no firing gas) as a cost comparison, its gas for `CostQuery::ng_smr` and its net abatement. `Final_Design_Basis.csv` gains `FLOOR_L5_*`, `FLOOR_LAC_DCF_*` and `FLOOR_BREAKEVEN_{A,B}_{L5,DCF,BOTH}_*` for the four stages at 25 y and 60 y (default timing), and `Validation_Results.csv` gains the reported rows `IND_L5_UNDER_PLANT_FLOOR`, `IND_LAC_DCF_UNDER_PLANT_FLOOR`, `IND_BREAKEVEN_A_UNDER_PLANT_FLOOR` and `IND_BREAKEVEN_B_UNDER_PLANT_FLOOR`.
- Values for B_REC_F, BOAK, 25 y, n = 1.0, as the engine prints them in `Final_Design_Basis.csv`: L5 203.180333 S$/t against 283.076719 under the floor, LAC_DCF 203.500904 against 283.448388, the net abatement 481,083.692586 t/y against 557,767.415516 (reference intensity 7.850175 against 8.995997 kg/kg). The break-even share of definition B (progressive, both methods) is 0.515856 against 0.788615 under the floor. Definition A (0.691256 at the centre) has no break-even share under the floor: even the largest share leaves the cost at or above S$100/t, at every stage and both lives. The verifier's hand figure of 283.1 S$/t for L5 under the floor is confirmed.

Finding 2, LOST_STEAM and BASE_POWER price INL's conventional plant. `EXPORT_STEAM_CONV` and `E_CONV_NOCC` stay as published INL values (TEV-953 Table 2), and no source exists for a plant-built reference duty, so the lines are labelled and reported with and without.

- `CostQuery::system_lines` (default true, the ladder as committed) switches the two lines off in `Model::cost_items`: the reference then exports what the plant exports and buys no power. The ledger and the cash flow read the same items, so both drop the same two lines.
- Labels: on the independent export the source text of the two registry inputs in `Final_Design_Basis.csv` gains the role comparison input, and the line text of LOST_STEAM and BASE_POWER in `Economic_Ledger.csv` says INL conventional plant, comparison input. The registry entries themselves are untouched, because their text belongs to the legacy snapshot; the class and status words (published, primary) keep describing where the numbers come from.
- Beside the headline: `NOSYS_L5_*`, `NOSYS_LAC_DCF_*`, `SYSLINES_L5_*` and `SYSLINES_LAC_DCF_*` in `Final_Design_Basis.csv` and the reported rows `IND_L5_WITHOUT_SYSTEM_LINES` and `IND_LAC_DCF_WITHOUT_SYSTEM_LINES`. Values for the same case: L5 without the two lines 187.361277 S$/t, LAC_DCF 187.681848 S$/t, so the two lines add 15.819055 S$/t to each method (7.8 percent of L5). The verifier's 15.82 S$/t is confirmed.

Finding 3, the BASELINE sensitivity group shifted the abatement alone. `solve` in `engine/plant/src/sens.rs` now selects the baseline of a BASELINE point through `CARBON_BASELINE` on the plant ledger, so the reference gas of the cost follows the baseline (rule 1 of N.7); the old offset of the abatement stays for the anchored carbon, where it keeps the committed legacy values. The `base_value` column prints the central intensity of the basis (8.995997 kg/kg), where it printed INL's 9.209770.

- Effect, B_REC_F, 25 y: AC_L5_25 for the INL point moves from 198.099114 to 190.645146 S$/t and for the Cho point from 194.901764 to 182.757397; AC_L3_25 from 145.501046 to 138.047078 and from 143.152637 to 131.008270; LAC_DCF_25 from 198.411668 to 190.957700 and from 195.209273 to 183.064906. The 60 y metrics move the same way. The IEAGHG point is the central run and does not move. The three points now equal the three points of the CARBON_BASELINE group metric by metric.

Cells of `results/` that changed, and why:

- `Economic_Ledger.csv`: 272 `line` texts (the GAS line of the runs on the plant ledger, and LOST_STEAM and BASE_POWER of every run). No value changed.
- `Emissions_Ledger.csv`: 24 `line` texts (the four central lines of each of the six runs). No value changed.
- `Final_Design_Basis.csv`: 39 `parameter` texts (the 36 cost rows and three headline abatement rows), 2 `source` texts (the two comparison inputs) and 96 new result rows (`FLOOR_*`, `NOSYS_*`, `SYSLINES_*`). No existing value changed.
- `Sensitivity_Results.csv`: 30 `base_value` cells of the BASELINE group, and 12 `result` and 12 `delta` cells (six cost metrics for the INL point and for the Cho point). Finding 3.
- `Subsidy_Scenarios.csv`: 352 `source` texts. No value changed.
- `Validation_Results.csv`: six new reported rows. No existing row changed.
- `Integrated_Mass_Balance.csv` and `Integrated_Energy_Balance.csv` are byte-identical.

New tests: `rule_1_of_n_7_holds_at_every_sensitivity_point_that_changes_the_baseline` and `legacy_baseline_points_keep_the_committed_offsets` (in `sens.rs`), `rule_1_of_n_7_holds_for_every_central_baseline`, `the_baseline_sensitivity_group_equals_the_carbon_baseline_group`, `the_two_system_lines_vanish_together_when_they_are_switched_off`, `the_system_lines_are_on_by_default_and_unchanged_on_the_legacy_basis`, `the_plant_built_floor_is_one_reference_for_gas_and_abatement`, `the_exported_floor_and_system_line_rows_are_the_engine_values`, `the_exported_floor_break_even_shares_meet_the_target_strictly` and `the_headline_rows_say_generic_reference_intensity_and_the_inputs_say_comparison` (in `m11_carbon.rs`), `the_floor_and_system_line_rows_are_reported_beside_the_independent_headline` (in `benchmark_rows.rs`) and `headline_indicator_labels_name_the_generic_reference_intensity` (wasm). No tolerance was changed.

Outstanding after this round:

- The owner has to accept decision D2 in its present form, or give sources for `REF_FURNACE_EFF` and `REF_STEAM_DUTY` so that the plant-built reference can become the centre. Until then the floor rows are the range that must be read with the headline: at the floor the cost without a grant is above S$100/t at every stage and life, and definition A cannot reach the target with any share.
- LOST_STEAM and BASE_POWER still price INL's conventional plant. They are labelled and reported with and without, and the pairing of the cost level with the carbon view (open issue 3 of the derivation) is the owner's decision.
- The frontend and the site scripts still describe the INL headline and do not yet show the floor rows (Phases 5 and 6).

## Phase status

| Phase | Status |
|---|---|
| 1 Audit | complete |
| 2 Independent derivation | complete |
| 3 Backend implementation | carbon track and co-funding track committed; site scripts and frontend labels wait for Phases 5 and 6 |
| 4 Validation | not started |
| 5 Results export | not started |
| 6 Frontend | not started |
| 7 Documentation | not started |
| 8 Final verification | not started |
