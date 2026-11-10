# Current state and baseline

Facts below were read from Git on 2026-11-10 and must be re-verified at the start of the session (`git fetch`, then the commands in section 1). Git history and repository contents override this summary.

## 1. Git baseline (verify, do not assume)

| Ref | Head at inspection | Relation |
|---|---|---|
| `origin/main` | `b4817b3` | 8 commits ahead of `8750b09`, all frontend accessibility and documentation (PR 3, merged). |
| `origin/feature/ui-ux` | `83e4f6d` | Fully contained in `main` (0 commits ahead). Merged by PR 1. Preserve it. |
| `origin/develop` | `b30664b` | Ancestor of the development branch. |
| `origin/fix/selftest-chrome-start` | `f8d5a34` | Ancestor of the development branch. Merged by PR 2. |
| `origin/feat/astryx-nush2` | `b4817b3` | Equals `main`. Merged by PR 3. |
| `feature/independent-abatement-subsidy` | `84dca45` | The development line. 17 commits ahead of merge base `8750b09`, 8 behind `main`. |

- Commit `84dca45` ("Freeze the eight result tables before the reference-plant work") is on `feature/independent-abatement-subsidy` and is the last engineering commit. It copies the eight committed result tables to `engine/vv/data/pre_reference_results/` as the regression snapshot for the reference-plant work.
- PR 1 merged `feature/ui-ux` into `main` (merge commit `be1e57f3`, GitHub-stamped). The commit exposure of `be1e57f` through that pull request page cannot be removed by the session. Only GitHub Support can remove it. Never claim otherwise.
- The primary checkout's local `main` may be stale (it once pointed at `0d4a860`). Always compare with `origin/main`.
- Record in `PROGRESS.md`: starting branch, starting commit, ahead/behind counts, `git status --short`, ignored and untracked files, and remote heads.
- Rust `target/` folders were deleted on purpose. Expect a clean rebuild of the whole workspace (several minutes). Separate ordinary build time from genuine errors.
- The development line has not been pushed before this package. Pushing the package branch publishes its 17 commits; the history scan hook takes about 5 minutes.

First milestone after the baseline lock: merge `origin/main` into the completion branch with an ordinary merge commit (no rebase, no force-push). Expect conflicts only in the frontend files that PR 3 touched (`frontend/pfd.js`, `pfd.css`, `app.js`, `index.html`, `selftest.js`) and in `docs/UI_UX_ASSESSMENT.md` and `README.md`.

## 2. What exists

- Rust workspace (`engine/`): crates `plant`, `props`, `mea`, `numerics`, `units`, `vv`, `wasm`, `cli`. Registry in `engine/data/parameters.toml` with provenance and `PARAMS_COMPLETE` validation.
- Two carbon bases: the legacy INL-anchored ledger (`CARBON_BASIS = 0`, bit-identical to `engine/vv/data/legacy_results`) and the independent plant-own basis (default), which runs the Level 3 flowsheet on the hydrogen target.
- Two cost methods: the annual incremental ledger L0 to L5 (`economics.rs`) and the discounted cash flow (`dcf.rs`), on the same assumptions.
- Subsidy model (`subsidy.rs`): definitions A and B, progressive or at-close grant, share grid 0, 25, 50, 75 percent, closed-form break-even with a residual check. Eighth table `Subsidy_Scenarios.csv` (352 rows).
- Results (`results/`): eight CSV tables written by the exporter and compared by `scripts/compare_results.py`.
- Frontend (`frontend/`): vanilla JavaScript and SVG, WebAssembly engine reply, interactive PFD with unit panels, guided process tour, accessible equipment and stream list (from PR 3), browser self-test (`scripts/selftest.py`).
- Documents: `docs/ABATEMENT_SUBSIDY_LOG.md`, `_REVIEW.md`, `_REFERENCES.md`, `docs/FINAL_DESIGN_DECISIONS.md`, `docs/VALIDATION_MATRIX.md`, derivations `01` and `02`, `math/` (LaTeX derivation book, equation register, SymPy scripts), `references/`.
- Guards: `scripts/check_clean.py`, the Rust test `no_internal_pointers`, `scripts/check_site.py` (withdrawn figures), CI workflows `ci.yml` and `site.yml`, and a local pre-push history scan.
- Test count at `8dbfc6d`: 412 workspace tests passing, formatting and clippy clean (`-D warnings`). Re-run to confirm.

## 3. Verified findings so far

- Subsidy build: independent verifier judged it sound.
- Carbon build: sound after two fix rounds. One major finding remained, a decision (the central reference is the published IEAGHG intensity). The owner has since accepted the IEAGHG-based specification, so it is settled (see `CARBON_AND_SUBSIDY.md`).
- Nine minor verifier findings remain open (listed individually in `CARBON_AND_SUBSIDY.md`).
- Headline figures at `8dbfc6d` (central reference is the published IEAGHG intensity, INL steam and power lines): L5 about S$203/t for BOAK 25 y. These belong to the previous reference and move after the work in this package. Do not quote them as final.

## 4. The reference-plant specification

Not yet implemented beyond the snapshot commit. The verified specification (sections A to F) and a completeness critique are in `reference/IEAGHG_REFERENCE_SPEC.md` and `reference/IEAGHG_REFERENCE_SPEC_CRITIQUE.md`. Decision ids in the specification (D13, D14) map to D18 to D20 in this package (see `CARBON_AND_SUBSIDY.md`).

## 5. Outstanding task inventory

Engine: reference module, registry (103 stream entries, scalar entries), PSA tail-gas balance, steam and power lines, integration, ninth table, 14 `REF_` rows. Carbon and subsidy: definition C, decision log, U6 test, nine stale SymPy checks, nine minor findings. Economics: regenerate and reconcile. Frontend: reference PFD, scenario page, labels, tour, site scripts, two tour defects. Documents: derivation 03, PDF, equation-to-Rust table, register, README, validation matrix, presentation fixes. Verification: full independent run and the 17-item report. Housekeeping: see `COMMIT_AND_DEPLOYMENT.md`.

## 6. Update of 2026-11-10 (history scan)

The local history scan blocked the first push because one phrase in the subsidy derivation (in two early commits) and one commit message matched a scheduling rule. The 18 unpushed local commits were rewritten once, before any push, to reword that phrase; their hashes changed. The backup branch `backup/pre-scan-fix-84dca45-prompt` and the old branch head keep the original hashes locally. Commit `84dca45` named elsewhere in this package now has a different hash on the pushed branch: find it with `git log --grep='Freeze the eight result tables'`. Verify by message, not by hash.

The public archive weewzr/raystorage (commit `3a25c4a7`) is a complementary source of historical evidence (see `RAYSTORAGE_INTEGRATION.md`).
