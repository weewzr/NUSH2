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
- **D2 Unabated reference (revised after Phase 2).** The central baseline is a documented generic steam methane reforming intensity multiplied by the annual hydrogen output: IEAGHG 2017-02 Case 1, 8.996 kg CO2 per kg H2. It belongs to the plant whose capital cost the ledger already uses for the SMR island, and it is the lowest of the three published intensities, so it gives the lowest abatement and cannot favour the target. The rule was fixed before any new result was computed. INL (9.21) and Cho (9.35) stay as comparison rows, the target verdict is reported under each, and the intensity built from the plant's own feed is kept as a floor check.
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

## Phase status

| Phase | Status |
|---|---|
| 1 Audit | complete |
| 2 Independent derivation | complete |
| 3 Backend implementation | not started |
| 4 Validation | not started |
| 5 Results export | not started |
| 6 Frontend | not started |
| 7 Documentation | not started |
| 8 Final verification | not started |
