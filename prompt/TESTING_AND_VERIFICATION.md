# Workstream 6: testing and independent verification

No tolerance is weakened. No golden file is updated merely to pass. A failed literature benchmark stays failed (the MEA column gate and the flowsheet carbon gate). Compilation is not verification.

## 1. Gates (run per package; all at the end)

| Gate | Check |
|---|---|
| G0 | Frozen snapshot `engine/vv/data/pre_reference_results/` exists (`84dca45`). |
| G1 | Zero subsidy: `eq_s_9b_*`, `eq_s_8_*`, and `subsidy_zero_share_equals_economic_ledger` in `build_numbers.py`, for every definition including C. |
| G2 | Legacy basis: `cargo run --release -p nush2_cli -- export --out <dir> --basis legacy`, then `python scripts/compare_results.py engine/vv/data/legacy_results <dir> --subset` with only the documented allowed cells. Bit-identical. |
| G3 | `REF_BASIS = 0` (or `--ref-basis historical`) reproduces the pre-change snapshot except documented new rows and columns. |
| G3b | Gate outcomes do not flip (`gates.rs` `RECORDED`). If one flips, stop and report. |
| G4 | `Integrated_Mass_Balance.csv` and `Integrated_Energy_Balance.csv` byte-identical on both bases. |
| G5 | Default basis against committed `results/` after regeneration. |
| G6 | `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `python scripts/check_clean.py`; `python scripts/check_site.py`; the site build; the browser self-test. |

## 2. Engineering tests to exist

Reference-plant mass, energy, element and PSA species balances; steam and power balances; boundary accounting; carbon closure (Undetermined while unknown streams exist, Closed when all known); zero-stream behaviour including the explicit U6 test (see `CARBON_AND_SUBSIDY.md`); integration with abatement, GAS, lost steam, base power; subsidy cases and closed-form break-even residuals; economic ledger against DCF within the stated gap; scenario reproducibility (same inputs, same outputs, byte for byte); a mutation set per package that each fails at least one test. Where useful add generated tests (see `GITHUB_REFERENCES.md`: proptest, Hypothesis) for conservation and monotonicity properties; they supplement and never replace the named equation tests.

## 3. Export tests

The nine tables including `Reference_Plant.csv`; all 14 reference validation rows (`ENGINE_REFERENCE_PLANT.md` section 3); regenerated exports compared with the final engine. The CSV-count test says nine.

## 4. SymPy

Resolve the nine stale checks (see `CARBON_AND_SUBSIDY.md` section 4) with equal or greater strength. Add the checks for derivation 03. Run with the project's local environment (`python -I`).

## 5. Browser tests

Run `scripts/selftest.py` at 1440 x 900, 375 x 812, 1024 x 600 and 1024 x 768, each with and without reduced motion where the script allows. Cover: guided tour, keyboard controls, scenario comparison, reference PFD, unit panels, live-reply fields, the S$100/t line, mobile layout, no console errors, no failed requests, no external requests. Chrome start flakiness was fixed in PR 2; if a start still fails, retry with a fresh profile and record it, but do not hide a real failure.

## 6. Fork and resource errors

The earlier full check died partway with "fork: Resource temporarily unavailable" in Git Bash on Windows (cargo test spawning many processes while other jobs ran). Investigate the cause, do not suppress it. Run one cargo job at a time, lower test concurrency (`cargo test -- --test-threads=2`, `CARGO_BUILD_JOBS=2`), keep browsers closed during test runs, and run checks sequentially in one script that logs each exit code. Document the exact settings used in `PROGRESS.md`.

## 7. Independent verification run

After the final implementation, run an independent verifier (a separate agent with no write access except temporary mutations restored by `git checkout`) that reads the diff, re-runs every gate, recomputes at least five headline numbers in its own Python from the IEAGHG PDF and committed CSVs, and tries at least five mutations per package. Blocking or major findings send the package back to a fixer, then a fresh verifier (up to two rounds, then record as BLOCKED with the reason).

## 8. Final 17-item report

Write `docs/FINAL_VERIFICATION_REPORT.md`. Each item has: requirement, test or evidence, actual result, PASS, FAIL or BLOCKED, file or commit reference, remaining qualification. Never mark PASS because the code appears to implement it; only ran-and-passed evidence counts.

1. Repository audit and baseline lock (branch heads, ahead/behind, history preserved).
2. IEAGHG parameter table traceable to page and table.
3. Reference-plant element, mass and energy balances with residuals.
4. PSA tail-gas balance and no double counting of its combustion.
5. Steam and power balance, and the recalculated LOST_STEAM and BASE_POWER lines.
6. Integration into abatement, GAS, K.18, upstream credit and decomposition.
7. Ninth table `Reference_Plant.csv` and the 14 validation rows.
8. Carbon closure, unknown streams, the U6 test, upper-bound labelling everywhere.
9. Definition C, the 0 to 100 percent sweep, double-counting safeguards, break-even per case.
10. Economic reconciliation: regenerated headlines, S$100/t verdict per case, old versus new.
11. Legacy bit-identity and snapshot regression (G2, G3, G4).
12. The nine SymPy checks and the nine minor findings individually resolved.
13. Reference PFD and scenario comparison page driven by the live reply.
14. Browser tests at four viewports, reduced motion, keyboard, no console errors, no external requests.
15. Derivation 03, the PDF build, the equation-to-Rust table with verified line numbers.
16. README and documents generated from results; text guards; withdrawn figures; no privacy-flagged material in Git.
17. Independent verification run, fork-error investigation, the pull request open and unmerged.

Also list unresolved assumptions (D14, D15, D16, D21 to D23) and limitations in the report's opening paragraph.

## 9. Archive material

Historical verification logs in raystorage show what passed on older code. They are not evidence for the final code. Rerun everything and record new results. Mutation scripts edit source files; restore and confirm a clean tree.
