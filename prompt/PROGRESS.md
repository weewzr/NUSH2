# Progress record

Resume from this file. Update it at every milestone and before any pause. Keep entries factual: commands run, results, commit hashes.

## Starting state (recorded when this package was written, 2026-11-10)

Kept as written. Section "Milestone 1" below corrects it; `CURRENT_STATE_AND_BASELINE.md` section 7 lists each correction.

- Starting branch: `feature/independent-abatement-subsidy`, commit `84dca45` (working tree clean, no local changes).
- Package branch: `docs/prompt-completion-spec`, created from `84dca45`; contains only the `prompt/` folder on top.
- Ahead/behind of `84dca45` against `origin/main` (`b4817b3`): 17 ahead, 8 behind; merge base `8750b09`.
- Remote heads: `main` `b4817b3`; `feature/ui-ux` `83e4f6d` (merged, 0 ahead of main); `develop` `b30664b`; `fix/selftest-chrome-start` `f8d5a34`; `feat/astryx-nush2` `b4817b3`.
- PR 1, 2, 3: all merged. `be1e57f` stays reachable through PR 1.
- Rust `target/` folders deleted: first build is a full rebuild.
- Verification state: 412 workspace tests passing at `8dbfc6d` (verifier run); the coordinating session's own full re-run died on a fork error and must be redone.

## Current phase

Milestone 1 (repository audit and baseline lock) recorded below. Next: milestone 2, starting with WP0 of `ENGINE_REFERENCE_PLANT.md`.

## Milestones

| # | Milestone | Status | Commit | Pushed head |
|---|---|---|---|---|
| 1 | Repository audit and baseline lock | **partial**: recorded and pushed, but the milestone's gates are not met; the inherited baseline fails G5, the G0 snapshot regression, the G4 mass-balance byte comparison, `build_numbers.py` (and its `--self-test` and G1 zero-share check), the two SymPy scripts and one browser check (open findings F1 to F4 below); CI on Linux (pull request 4) fails the same default-basis comparison, so F1 is not a Windows-only effect. Stays partial until F1 to F4 are closed by their milestones | `9fc36ff`, `b37a76d`, `241e39e`, `486de50` (results), `78835a7` and `5d5a0b4` (verifier review), and the CI-record commit that follows `5d5a0b4` | see "Milestone 1" |
| 2 | Reference-plant equations and implementation | not started | | |
| 3 | Reference integration and ninth export | not started | | |
| 4 | Carbon and subsidy completion | not started | | |
| 5 | Economic reconciliation and regeneration | not started | | |
| 6 | Frontend reference PFD | not started | | |
| 7 | Scenario comparison and tour updates | not started | | |
| 8 | Derivation 03 and PDF | not started | | |
| 9 | Equation register and README | not started | | |
| 10 | Presentation and PFD correction | not started | | |
| 11 | Independent verification | not started | | |
| 12 | Final report and release candidate | not started | | |

## Milestone 1: repository audit and baseline lock (2026-11-24)

### True starting state

`git fetch origin`, then `git ls-remote --heads origin` and `git rev-list --left-right --count origin/main...origin/docs/prompt-completion-spec` (9 / 19), `git merge-base` (`8750b09`). Checkout clean (`git status --short --ignored` empty, no untracked or ignored files, no `target/`).

| Remote head | Commit |
|---|---|
| `main` | `cfef6d5` "Docs: restructure the README to the shared layout" (2026-11-19) |
| `docs/prompt-completion-spec` | `1a1e822` "docs(prompt): add the two source briefs" |
| `develop` | `b30664b` (contained in `main`) |
| `feature/ui-ux` | `83e4f6d` (contained in `main`) |
| `fix/selftest-chrome-start` | `f8d5a34` (contained in `main`) |
| `feat/astryx-nush2` | `b4817b3` (contained in `main`, 1 behind) |

`feature/independent-abatement-subsidy` and `backup/pre-scan-fix-84dca45-prompt` are not on the remote. `84dca45`, `8dbfc6d`, `0665b8e` and `9471f9f` are pre-rewrite hashes that exist neither on the remote nor in raystorage at `3a25c4a`. Mapping by message and content: `84dca45` is `212caf1`; the state called `8dbfc6d` matches `8863f2f` (inferred, see `CURRENT_STATE_AND_BASELINE.md` section 7).

### Branch

- `git switch -c feature/reference-plant-completion 1a1e822`.
- `git merge --no-ff origin/main`: one conflict, `README.md` (the only file changed on both sides since `8750b09`). Resolved by keeping main's restructured README and carrying over the development line's one change (eight results tables including the government co-funding scenarios), also in the run section. Merge commit `9fc36ff` (parents `1a1e822`, `cfef6d5`). The frontend files of PR 3 and `docs/UI_UX_ASSESSMENT.md` merged without conflict.
- `docs/prompt-completion-spec` and every other branch left untouched.

### Policy update (own commit `b37a76d`)

`ORIGINAL_BRIEF_3_MERGE_POLICY.md` records the owner's instruction of 2026-11-24. The operative merge prohibitions in `README.md`, `MASTER_PROMPT.md`, `AUTONOMOUS_EXECUTION.md`, `COMMIT_AND_DEPLOYMENT.md` and `TESTING_AND_VERIFICATION.md` are replaced by the release gate (`COMMIT_AND_DEPLOYMENT.md` section 5): the coordinating session merges only after gates G0 to G6 pass, the final verification report has no FAIL, the independent verifier has no open major finding and CI is green on the pull request, by merging `main` into the completion branch and fast-forward pushing it to `main`; never the web merge button, never force. Briefs 1 and 2 are unedited.

### Owner confirmations of 2026-11-24 (recorded here as the package requires)

- D21 (island grant mirrored on the reference), D22 (electricity S$0.20/kWh both ways) and D23 (IEAGHG power island kept inside the island on both sides) are confirmed as labelled assumptions, each with its sensitivity row.
- Definition C is primary, Definition B is a labelled legacy comparison, Definition A is a labelled secondary comparison with no default role. (The registry default today is still A, `SUBSIDY_DEFAULT_DEFINITION = 0`; the flip to C belongs to WP4.)
- The switch `SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND` removes capital only; the row text states the island's O&M share (critique C14).
- Legacy registry text stays unchanged; correct locators (for example the `DISCOUNT` page) go into the new `REF_` rows (critique C01).
- The guard that forbids web fonts stays.
- Unknown streams U1, U2, U4b, U6 stay unknown, and abatement keeps its upper-bound qualification (D16 confirmed).
- Replacing the live headline cost with the regenerated IEAGHG-reference result is accepted once the release gate passes.
- Commit attribution: the owner only. The session instructions give no attribution line, so commit messages and pull request bodies end without one.

### History scan substitute

The external pre-push history scan that this package names (about 5 minutes, outside the repository) is not available. The owner approved a recorded substitute, run before every push:

1. `python -I scripts/check_clean.py` (working tree: local paths, identifiers, withdrawn figures, em dashes, pointers).
2. `cargo test -p nush2_plant --test no_internal_pointers` (the Rust text guard).
3. `python -I scripts/scan_history.py` (new at this milestone): every commit message, author and committer address and every blob reachable from `HEAD`, with the rules of `check_clean.py` for local absolute paths, profile folders, matriculation numbers, group numbers, schedule, allocation and declaration text, plus personal e-mail addresses (anything other than the owner's GitHub noreply address and the two service addresses `noreply@github.com` and `noreply@anthropic.com`, both of which the recorded history needs; since the verifier review another account's `users.noreply.github.com` address is a hit, and `--self-test` proves it) and module-style course codes (reviewed technical identifiers such as AP1000 are listed in the script). Office files are opened and their XML scanned; PDF and other binaries get the path rules on raw bytes only, which is a known gap (compressed PDF streams are not decoded). `--self-test` proves each rule fires on a planted sample.

Known content of the pushed history that the scan accepts: 53 commit messages (all 34 of `main` and the 19 of `docs/prompt-completion-spec` beyond it) carry an assistant co-author trailer with a non-personal service address. They are pushed history, and rewriting them is not authorised. Commits made from 2026-11-24 on carry no such trailer.

### Toolchain

- `rustup toolchain install 1.96.0 --profile minimal --component rustfmt,clippy --target wasm32-unknown-unknown --no-self-update` (per-user rustup; default toolchain unchanged, still stable 1.100.0). In the repository `rust-toolchain.toml` selects `rustc 1.96.0 (ac68faa20 2026-05-25)`, `cargo 1.96.0`, `rustfmt 1.9.0`, `clippy 0.1.96`, host `x86_64-pc-windows-gnu`. Install time 35 s.
- Python 3.12.10 in a virtual environment outside the repository with SymPy 1.14.0; every script runs with `python -I`.
- Browser self-test: Microsoft Edge through `scripts/selftest.py --chrome <msedge.exe>`.
- Fork-error settings (`TESTING_AND_VERIFICATION.md` section 6): one cargo job at a time, `CARGO_BUILD_JOBS=2`, `cargo test -- --test-threads=2`, steps run sequentially by one script that logs each exit code and wall time; no browser open during cargo runs. No fork or resource error occurred.

### Clean rebuild and full gate on `241e39e` (2026-11-24, one script, sequential)

`cargo clean`, `_site/` and `build/` removed, then every step in order with `CARGO_BUILD_JOBS=2` and `--test-threads=2`. Wall times on this machine (Windows 11, `x86_64-pc-windows-gnu`).

| Step | Command | Result | Time |
|---|---|---|---|
| clean build | `cargo build --workspace --all-targets` | ok | 87 s |
| text guard | `python -I scripts/check_clean.py` | 275 files, 0 hits | 4 s |
| G6 format | `cargo fmt --all --check` | ok | 1 s |
| G6 lint | `cargo clippy --workspace --all-targets -- -D warnings` | ok | 21 s |
| G6 tests (dev) | `cargo test --workspace -- --test-threads=2` | 480 passed, 0 failed, 0 ignored (35 test binaries and doc-test sets) | 225 s |
| release CLI | `cargo build --release -p nush2_cli` | ok | 45 s |
| G0 | `engine/vv/data/pre_reference_results/` | present, 8 tables | |
| G2 | `nush2 export --basis legacy`, then `compare_results.py engine/vv/data/legacy_results <dir> --subset --names <7 tables> --allow-cell Validation_Results.csv:PARAMS_COMPLETE:conditions --allow-cell Validation_Results.csv:EXPORT_BYTE_IDENTICAL:conditions` (as in `ci.yml`) | PASS within the `compare_results.py` tolerance (as `ci.yml` runs it); **not byte-identical** on this platform: 110 cells differ as text (Economic_Ledger 12, Integrated_Mass_Balance 70, Sensitivity_Results 3, Validation_Results 25), all within tolerance; the largest mass-balance difference is 5e-6 kg/h, in run `FS_B_REC_F_T800`. The package defines G2 as bit-identical, so G2 is met only in the tolerance form (see also the G4 wording note) | 54 s |
| G5 | `nush2 export`, then `compare_results.py results <dir>` | **FAIL**, 12 cells (finding F1) | 50 s |
| G0 regression | `compare_results.py engine/vv/data/pre_reference_results <dir>` | **FAIL**, the same 12 cells (the snapshot equals `results/`) | |
| G4 (milestone 1 form) | regenerated balance tables against the committed ones, byte for byte | energy balance identical (legacy export, default export, `results/`, snapshot); mass balance **differs** in the same 12 cells (F1) | |
| G1 | `cargo test --workspace eq_s_9b eq_s_8` | 7 passed | 6 s |
| G1 | `subsidy_zero_share_equals_economic_ledger` in `build_numbers.py` | **not reached**: `build_numbers.py` stops earlier (finding F2) | |
| numbers | `python -I scripts/build_numbers.py`; `--self-test` | **FAIL** (F2); `--self-test` **FAIL** (exit 1): the 11 unit-map checks pass, then `build()` stops on F2, so the SENS token-unit checks are not reached. (Corrected at the verifier review; the earlier record said "self-test PASS", which was wrong.) | |
| Rust text guard | `cargo test -p nush2_plant --test no_internal_pointers` | 2 passed | 1 s |
| history scan | `python -I scripts/scan_history.py` | 56 commits, 635 objects, 0 hits | 5 s |
| SymPy | `python -I math/verify/sympy_carbon.py` | **108 of 115**, exit 1 (F3) | 2 s |
| SymPy | `python -I math/verify/sympy_subsidy.py` | **118 of 120**, exit 1, C19 and C15b (F3) | 3 s |
| release tests | `cargo test --workspace --release -- --test-threads=2` | 480 passed, 0 failed | 346 s |
| wasm | `build_wasm.py` with `GIT_SHA` set; `check_wasm.py` | 1,127,673 bytes, 0 imports, PASS | 37 s |
| G6 site build | `build_site.py --wasm ... --results results --commit <sha> --run-id local --test-log target/test.log --out _site` | ok | 2 s |
| G6 content guard | `check_site.py --site _site --repo .`; with `--prove` | PASS (34 site files, 269 repository files, 0 hits); proof 34 of 34 and 11 of 11 plants caught | 7 s |
| G6 browser self-test | `selftest.py --site _site --chrome <Edge> --expect-commit <sha>`, at 1440 x 900, 1440 x 900 reduced motion, 375 x 812 mobile, 1024 x 600 and 1024 x 768 | **FAIL** at every viewport: 35 passed, 1 skipped (`pfd_streams_bound`, by design), 1 failed (`headline_parity`, finding F4); 0 console errors, 31 requests, 0 to another host, 0 network problems. `tour_keyboard` passes at 1024 x 600. | 54 s each |
| G3, G3b | `REF_BASIS` | not applicable before milestone 3 (no reference basis exists) | |

No fork or resource error occurred. No tolerance, golden file, snapshot or results table was changed.

### Open findings of milestone 1 (inherited from the development line, none fixed here)

- **F1, G5 on this platform.** The default export differs from `results/Integrated_Mass_Balance.csv` in 12 cells of run `FS_B_REC_F_T800` (streams S07, S07A and S08, species h2, co, co2, h2o), by up to 9e-6 kmol/h (h2o; 8e-6 for h2, co and co2) on values near 4,358 kmol/h, beyond the tolerance of `compare_results.py` (one printed unit, absolute 1e-6, relative 1e-9). Every other run differs by at most about one printed unit, which the script documents as the expected difference between platforms; the same run on the legacy basis differs by up to 5 printed units and passes. The likely cause, inferred and not yet confirmed, is the build platform (this machine compiles for `x86_64-pc-windows-gnu`) amplified by the 800 C flowsheet solve. Supporting evidence: the engine sources are unchanged since `f71c2b5`, the mass balance table was last written at `cc03f19`, and the regeneration at `f71c2b5` left this table byte-identical on the machine that wrote it; the build that wrote `results/` is not identified. To be confirmed by CI on the pull request. No code defect is shown. CI (Linux) has never run on the development line, so whether CI reproduces `results/` is unknown. Next step: run `ci` and `site` on the pull request, then decide in milestone 3 (which regenerates the tables) how the T800 run is made reproducible, without loosening the tolerance.
  **Update after CI on pull request 4 (2026-11-24):** the Windows-build hypothesis is **refuted** in the form "Linux reproduces `results/`". On `ubuntu-24.04` with the pinned `rustc 1.96.0 (ac68faa20 2026-05-25)` both `ci` and `site` fail the default-basis comparison on 9 cells of the same run and streams (co, co2, h2o of S07, S07A, S08; h2 is within tolerance on Linux): co 160.935781, co2 1092.183245, h2o 3353.985299 against the committed 160.935777, 1092.183249, 3353.985294 (largest 5e-6 kmol/h). The Windows GNU build gives 160.935785, 1092.183241, 3353.985303 and h2 4358.199304 (committed 4358.199312). Each platform is deterministic (two Linux jobs on different runner images agree; two Windows runs agree), and Linux and Windows differ from each other by up to about 8e-6 kmol/h. So the T800 flowsheet result is platform-dependent at the 1e-6 level (a platform effect in that sense), but neither the CI platform nor this machine reproduces `results/`; the build that wrote the committed table is still not identified. Consequence: on this line CI cannot pass G5 with the committed `results/`. The decision on how the T800 run is made reproducible (or which platform's tables are authoritative) stays with milestone 3, which regenerates the tables; the tolerance stays as it is.
- **F2, `build_numbers.py` stops on the committed tables.** "results files disagree: RES_ABATEMENT_T_Y 557767.415516 against EM_B_REC_F_INL_9P21_DIRECT_ATM_ABATEMENT_T_Y 583851.592283; RES_BURNED_C_KMOL_H 99.205227 against MB_B_REC_F_N2_RICH_CUT_C_KMOL_H 93.924198" (the abatement values quoted are upper bounds on the modelled streams). The script runs clean on the tables of `8750b09`, `a9cd939`, `e768ee9` and `f0e8d84` and stops from `cc03f19` on. Its two cross-table identities take the headline abatement as the INL-baseline row and the burned carbon as the nitrogen-rich cut alone, which held on the legacy basis. Until it is updated, `build/numbers.json` cannot be built, so the README tokens, the derivation values, the G1 zero-share check of `build_numbers.py` and the SENS token-unit checks of `build_numbers.py --self-test` (which calls `build()` after its 11 unit-map checks) are not available; the self-test exits 1. Whether the burned-carbon identity or the exported row is wrong on the independent basis is not yet established. Belongs to milestones 4 and 5 (carbon completion, regeneration).
- **F3, the nine stale SymPy checks.** True counts 108 of 115 and 118 of 120 (the records said 115 and 120). Both scripts pass in full at `a9cd939`. Carbon: the seven checks named in derivation 01 compare legacy values with `results/`, which holds the independent basis since `cc03f19`; the values the script computes equal the legacy snapshot rows to the printed digits. Subsidy C15b: its two exact ratio conditions hold, and only the printed excess 37.985 S$/t fails, because it was computed at the legacy abatement (39.366876 S$/t now, inverse to the abatement). Subsidy C19: the rescaling factor 2 no longer reaches the case s* < 0 since `9565fe9` (needs more than 2.0318); the other four cases hold. No model defect shown. Repair with written decisions belongs to milestone 4 (`CARBON_AND_SUBSIDY.md` section 4). Records corrected in `docs/ABATEMENT_SUBSIDY_LOG.md` and derivations 01 and 02.
- **F4, browser `headline_parity`.** `kpi.abatement_system_t_y` and `kpi.abatement_lifecycle_t_y` of the wasm reply (507,347.36 and 601,999.94 t/y) differ from `data/reference.json` (521,654.03 and 616,306.61 t/y). The wasm values equal the `IEAGHG_9p00` rows of `results/Emissions_Ledger.csv` (the central reference on the independent basis, and the rows the wasm unit test compares with); `scripts/build_site.py` lines 101 to 104 still take the `INL_9p21` rows, and the page text in `frontend/views.js` line 385 still says the INL baseline gives the headline. Both are site-script and label work of milestone 7; fixing the reference alone would turn the check green while the page keeps the wrong label. These abatement values are upper bounds while U1, U2, U4b and U6 stay unknown.
- **G4 wording.** `TESTING_AND_VERIFICATION.md` G4 asks for byte-identical balance tables "on both bases". Across the carbon bases the mass balance differs by design (for example the `A_INL_SEL` rows), at the committed state too; the energy balance is identical. The log's "byte-identical" statements compare a table before and after a change. G4 is therefore read as "unchanged by the reference-basis switch" from milestone 3 on, and at milestone 1 as "regenerated equals committed".
- **README headline.** The README carries the legacy-basis headline and abatement figures from `main`; the committed tables on this branch are on the independent basis. Unchanged at milestone 1; regenerated from tokens in milestone 9 once F2 is resolved.
- **CI trigger.** `ci.yml` and `site.yml` run on pushes to `main` and `develop` and on pull requests to `main`; a push of the completion branch alone starts no CI. The release gate needs a pull request (or a manual dispatch). Draft pull request 4 (`feature/reference-plant-completion` into `main`) was opened on 2026-11-24 for this purpose; it is not for merging until the release gate passes.
- **Reference files.** `reference/IEAGHG_REFERENCE_SPEC.md` (175) and `IEAGHG_REFERENCE_SPEC_CRITIQUE.md` (39) hold 214 `<working folder>` substitutions; left unedited, to be treated as unreadable where damaged.

### Commits of milestone 1

`9fc36ff` (merge of `origin/main`), `b37a76d` (policy), `241e39e` (records and history scan), `486de50` (these results), `78835a7` (history-scan e-mail rule) and `5d5a0b4` (records fix) of the verifier review, and the commit that records the CI run of pull request 4 (the child of `5d5a0b4`). Pushed head: the tip of `origin/feature/reference-plant-completion`.

### Verifier review of milestone 1 (2026-11-24)

An independent verifier reviewed `1a1e822..486de50` (two major, five minor findings). Changes made in response, as new commits `78835a7` (scan rule) and `5d5a0b4` (records); all seven findings were checked against `5d5a0b4` and are addressed in the records:

- The `build_numbers.py --self-test` record was wrong (it said PASS; the run exits 1). Corrected in the gate table and in F2. Rerun on `486de50` with the fix commit's tree: 11 PASS lines after the F2 error, exit 1.
- Milestone 1 is marked partial in the milestone table: its gates are not met (F1 to F4). F2 to F4 belong to milestones 4, 5 and 7 and are not fixed here; no tolerance, golden file, snapshot or table was changed. CI on Linux needs a pull request; pull request 4 was opened and its result is recorded in the next section.
- F1: largest difference corrected to 9e-6 kmol/h (h2o), and its cause restated as an inference.
- G2: recorded as PASS within tolerance and not byte-identical (110 cells; rerun of the legacy export on this machine, 52 s, same counts, `compare_results.py` PASS).
- `scripts/scan_history.py`: the e-mail rule accepted any `users.noreply.github.com` address (through `check_site.py` `OK_MAIL`). It now accepts only the owner's noreply address and the two service addresses the history needs (removing either one gives hits: 54 for `noreply@anthropic.com`, 1 for `noreply@github.com`); two new self-test samples with another account's noreply address fire. `--self-test` 14 of 14 ok; scan of `HEAD` 0 hits.

### First CI run on Linux: draft pull request 4 (2026-11-24)

`gh pr create --draft --base main --head feature/reference-plant-completion` opened pull request 4 on head `5d5a0b4`. Both workflows ran on `ubuntu-24.04` with `rustc 1.96.0 (ac68faa20 2026-05-25)` and `cargo 1.96.0` from `rust-toolchain.toml` (Python 3.12.3).

| Workflow (run) | Job | Step | Result |
|---|---|---|---|
| `ci` (45392410977) | `rust` | toolchain, text guard (275 files, 0 hits), format, lint | pass |
| | | `cargo test --workspace` | pass: 480 passed, 0 failed, 0 ignored (35 result lines) |
| | | legacy basis against the snapshot (G2, as above) | pass within tolerance; not byte-identical (Economic_Ledger 12, Integrated_Mass_Balance 78, Sensitivity_Results 2, Validation_Results 25 cells unequal as text) |
| | | default basis against `results/` (G5) | **fail**: 9 cells, `FS_B_REC_F_T800` S07, S07A, S08, co, co2, h2o, largest 5e-6 kmol/h (F1) |
| | | job | **failure** |
| `site` (45392411287) | `build` | toolchain; `cargo test --workspace --release` | pass: 480 passed, 0 failed, 0 ignored |
| | | compile WebAssembly, `check_wasm.py` | pass: 1,127,673 bytes, 0 imports (same size as on Windows) |
| | | regenerate and compare results (G5) | **fail**: the same 9 cells with the same values as in `ci` (F1) |
| | | legacy basis, build site, content guard, both browser self-tests, Pages upload | **not run** (skipped after the failed step); F4 is therefore not yet observed on Linux |
| | `deploy` | | skipped (pull-request event) |

Reading: every failure is F1; no other defect of this branch appeared, so nothing was changed in code, workflows, tolerances or tables. F1 is platform-dependent (Linux and Windows give different, each deterministic values) but Linux does not reproduce the committed `results/` either; see the update under F1. Because `site` stops at G5, the site build, content guard and browser self-test (F4) do not run on CI until F1 is resolved in milestone 3.

The raystorage archive cleanup that `RAYSTORAGE_INTEGRATION.md` lists as a pending owner action (remove the browser-profile folders) is done: raystorage `main` moved from `3a25c4a` to `aeba052` on 2026-11-24. The package's pin stays `3a25c4a`, which is still in that history; the removed text extracts are read from history only and never copied here.

## Test and verification results

See milestone sections.

## Outstanding assumptions

D14 (cryogenic f and r, no source), D15 (shipping distance and unverified coordinates), D16 (unknown streams U1, U2, U4b, U6; upper bound confirmed), D21 (island mirrored), D22 (electricity S$0.20/kWh, no sourced tariff), D23 (power island kept in island); D21 to D23 confirmed as labelled assumptions on 2026-11-24. Dependence on INL Case 6 through the heat and power balance.

## Blockers

None blocking milestone 2. Candidate: no sourced Singapore electricity export price; unverified site coordinates.

## Next actions

1. Milestone 2: WP0 of `ENGINE_REFERENCE_PLANT.md` (registry entries, `refplant.rs` structs and reader; nothing numerical may change).
2. Re-derive every IEAGHG value from the report PDF with its page: the two files in `reference/` contain 214 `<working folder>` substitutions (175 in the specification, 39 in the critique), each of which also removed the character before a colon, so any value next to one is unreadable.
3. Open findings F1 to F4 of milestone 1 (above). F1: CI on pull request 4 shows that neither Linux nor Windows reproduces the committed T800 cells; decide in milestone 3 how the run is made reproducible, without loosening the tolerance. F2 and F3 in milestones 4 and 5, F4 in milestone 7. Rerun the milestone-1 gates once they are closed and record them; milestone 1 stays partial until then.

## Update 2026-11-10

- Package branch `docs/prompt-completion-spec` carries the package plus the raystorage integration files. Unpushed local history was rewritten once to pass the history scan (see `CURRENT_STATE_AND_BASELINE.md` section 6).
- raystorage pinned at `3a25c4a7`. Privacy findings recorded in `RAYSTORAGE_INTEGRATION.md`; owner action pending (remove browser-profile folders from raystorage).
