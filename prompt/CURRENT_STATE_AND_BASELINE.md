# Current state and baseline

The package was written from Git on 2026-11-10. Section 1 was re-verified and corrected on 2026-11-24 at milestone 1 (`git fetch`, `git ls-remote --heads origin`, `git rev-list --left-right --count`); section 7 lists what changed against the 2026-11-10 record. Git history and repository contents override this summary.

## 1. Git baseline (verified 2026-11-24)

| Ref | Head | Relation |
|---|---|---|
| `origin/main` | `cfef6d5` | README restructure of 2026-11-19 on top of `b4817b3` (PR 3, merged). 9 commits ahead of merge base `8750b09`. |
| `origin/feature/ui-ux` | `83e4f6d` | Fully contained in `main`. Merged by PR 1. Preserve it. |
| `origin/develop` | `b30664b` | Contained in `main` and an ancestor of the development line. |
| `origin/fix/selftest-chrome-start` | `f8d5a34` | Contained in `main`. Merged by PR 2. |
| `origin/feat/astryx-nush2` | `b4817b3` | Contained in `main` (1 behind). Merged by PR 3. |
| `origin/docs/prompt-completion-spec` | `1a1e822` | The development line (17 engineering commits `1fe5a93` to `212caf1`) plus the two package commits `6b5d0d9` and `1a1e822`. 19 ahead of `main`, 9 behind, merge base `8750b09`. Left untouched as a record. |
| `origin/feature/reference-plant-completion` | see `PROGRESS.md` | The completion branch, created at `1a1e822` on 2026-11-24 with `origin/main` merged in by the ordinary merge commit `9fc36ff`. |

- `feature/independent-abatement-subsidy` was never pushed. It existed only in the checkout that wrote this package, and its content is reachable on the remote only through `docs/prompt-completion-spec`.
- The snapshot commit "Freeze the eight result tables before the reference-plant work", recorded here on 2026-11-10 as `84dca45`, is `212caf1` on the remote (section 6 explains the rewrite). It copies the eight committed result tables to `engine/vv/data/pre_reference_results/` and is the last engineering commit.
- PR 1 merged `feature/ui-ux` into `main` (merge commit `be1e57f3`, GitHub-stamped). The commit exposure of `be1e57f` through that pull request page cannot be removed by the session. Only GitHub Support can remove it. Never claim otherwise. `be1e57f` is no longer an ancestor of `main`: main was force-pushed once afterwards, on 2026-11-06, to the redone merge `fbfe631` (parents `0d4a860` and `83e4f6d`).
- The primary checkout's local `main` may be stale (it once pointed at `0d4a860`). Always compare with `origin/main`.
- Record in `PROGRESS.md`: starting branch, starting commit, ahead/behind counts, `git status --short`, ignored and untracked files, and remote heads.
- Rust `target/` folders were deleted on purpose. Expect a clean rebuild of the whole workspace (several minutes). Separate ordinary build time from genuine errors.
- The development line has not been pushed before this package. Pushing the package branch publishes its 17 commits; the history scan hook takes about 5 minutes.

First milestone after the baseline lock: merge `origin/main` into the completion branch with an ordinary merge commit (no rebase, no force-push). Done on 2026-11-24 as `9fc36ff`. The 2026-11-10 forecast of conflicts in the frontend files of PR 3 and in `docs/UI_UX_ASSESSMENT.md` was wrong, because the development line touches none of them: `README.md` was the only file changed on both sides, and its one conflict (the results row, seven tables against eight) was resolved by keeping main's layout with the development line's eight tables.

## 2. What exists

- Rust workspace (`engine/`): crates `plant`, `props`, `mea`, `numerics`, `units`, `vv`, `wasm`, `cli`. Registry in `engine/data/parameters.toml` with provenance and `PARAMS_COMPLETE` validation.
- Two carbon bases: the legacy INL-anchored ledger (`CARBON_BASIS = 0`, bit-identical to `engine/vv/data/legacy_results`) and the independent plant-own basis (default), which runs the Level 3 flowsheet on the hydrogen target.
- Two cost methods: the annual incremental ledger L0 to L5 (`economics.rs`) and the discounted cash flow (`dcf.rs`), on the same assumptions.
- Subsidy model (`subsidy.rs`): definitions A and B, progressive or at-close grant, share grid 0, 25, 50, 75 percent, closed-form break-even with a residual check. Eighth table `Subsidy_Scenarios.csv` (352 rows).
- Results (`results/`): eight CSV tables written by the exporter and compared by `scripts/compare_results.py`.
- Frontend (`frontend/`): vanilla JavaScript and SVG, WebAssembly engine reply, interactive PFD with unit panels, guided process tour, accessible equipment and stream list (from PR 3), browser self-test (`scripts/selftest.py`).
- Documents: `docs/ABATEMENT_SUBSIDY_LOG.md`, `_REVIEW.md`, `_REFERENCES.md`, `docs/FINAL_DESIGN_DECISIONS.md`, `docs/VALIDATION_MATRIX.md`, derivations `01` and `02`, `math/` (LaTeX derivation book, equation register, SymPy scripts), `references/`.
- Guards: `scripts/check_clean.py`, the Rust test `no_internal_pointers`, `scripts/check_site.py` (withdrawn figures), CI workflows `ci.yml` and `site.yml`, and the history scan. The external pre-push scan of 2026-11-10 is not available; its recorded substitute is `scripts/scan_history.py` (see `PROGRESS.md`).
- Test count: 480 workspace tests, all passing, formatting and clippy clean (`-D warnings`), measured on 2026-11-24 at milestone 1 with Rust 1.96.0 (`PROGRESS.md`). The 2026-11-10 record said "412 at `8dbfc6d`". 412 is the number of `#[test]` functions in the co-funding commits `f0e8d84` to `bad9683`, before the carbon track; the carbon track raised it to 462 (`cc03f19`), 468 (`9565fe9`) and 480 (`8863f2f`, unchanged to `212caf1`). `8dbfc6d` is a pre-rewrite hash that is not on the remote, so the 412 run cannot be traced.

## 3. Verified findings so far

- Subsidy build: independent verifier judged it sound.
- Carbon build: sound after two fix rounds. One major finding remained, a decision (the central reference is the published IEAGHG intensity). The owner has since accepted the IEAGHG-based specification, so it is settled (see `CARBON_AND_SUBSIDY.md`).
- Nine minor verifier findings remain open (listed individually in `CARBON_AND_SUBSIDY.md`).
- Headline figures at `8dbfc6d` (central reference is the published IEAGHG intensity, INL steam and power lines): L5 about S$203/t for BOAK 25 y. `8dbfc6d` is a pre-rewrite hash; the committed tables on the remote carry this value from `9565fe9` to `212caf1` (row `AC_L5_BOAK_25Y_N1P00`). These belong to the previous reference and move after the work in this package. Do not quote them as final.

## 4. The reference-plant specification

Not yet implemented beyond the snapshot commit. The verified specification (sections A to F) and a completeness critique are in `reference/IEAGHG_REFERENCE_SPEC.md` and `reference/IEAGHG_REFERENCE_SPEC_CRITIQUE.md`. Decision ids in the specification (D13, D14) map to D18 to D20 in this package (see `CARBON_AND_SUBSIDY.md`).

## 5. Outstanding task inventory

Engine: reference module, registry (103 stream entries, scalar entries), PSA tail-gas balance, steam and power lines, integration, ninth table, 14 `REF_` rows. Carbon and subsidy: definition C, decision log, U6 test, nine stale SymPy checks, nine minor findings. Economics: regenerate and reconcile. Frontend: reference PFD, scenario page, labels, tour, site scripts, two tour defects. Documents: derivation 03, PDF, equation-to-Rust table, register, README, validation matrix, presentation fixes. Verification: full independent run and the 17-item report. Housekeeping: see `COMMIT_AND_DEPLOYMENT.md`.

## 6. Update of 2026-11-10 (history scan)

The local history scan blocked the first push because one phrase in the subsidy derivation (in two early commits) and one commit message matched a scheduling rule. The 18 unpushed local commits were rewritten once, before any push, to reword that phrase; their hashes changed. The backup branch `backup/pre-scan-fix-84dca45-prompt` and the old branch head keep the original hashes locally. Commit `84dca45` named elsewhere in this package now has a different hash on the pushed branch: find it with `git log --grep='Freeze the eight result tables'`. Verify by message, not by hash.

The public archive weewzr/raystorage (commit `3a25c4a7`) is a complementary source of historical evidence (see `RAYSTORAGE_INTEGRATION.md`).

## 7. Corrections of 2026-11-24 (milestone 1)

| 2026-11-10 record | Verified state |
|---|---|
| Development line `feature/independent-abatement-subsidy` at `84dca45` | Branch never pushed. Snapshot commit is `212caf1`; the pushed line ends at `1a1e822` on `docs/prompt-completion-spec`. |
| `8dbfc6d` (spec audit commit, 412 tests, S$203/t) | Not on the remote and not named in raystorage at `3a25c4a7`. By message and content the matching pushed state is `8863f2f` ("Record carbon fix round 2 in the derivation and the log"), the parent of `212caf1`, which differs from it only by the snapshot folder (8 files added). This mapping is inferred; the hash itself cannot be inspected. |
| `0665b8e`, `9471f9f` (archive index and log) | Pre-rewrite hashes, not on the remote. The log describes `9471f9f` as the head that already held the co-funding model before the carbon track; on the remote that is one of `f0e8d84`, `dcae481` or `bad9683`, which one cannot be determined. |
| `main` at `b4817b3`, 17 ahead and 8 behind | `main` at `cfef6d5`; the package tip is 19 ahead and 9 behind. |
| Expected merge conflicts in five frontend files and the interface review | Only `README.md` conflicted (section 1). |
| 412 workspace tests | 480 (section 2). |
| SymPy "115 of 115" and "120 of 120" (log and derivations 01 and 02) | 108 of 115 and 118 of 120 at `212caf1` and on the completion branch. Both scripts pass in full at `a9cd939`; the failures appear at `cc03f19` (carbon 7 and subsidy C15b) and `9565fe9` (subsidy C19). Causes in `PROGRESS.md`. |
| PR 1 merge `be1e57f` on `main` | `main` was force-pushed to `fbfe631` on 2026-11-06; `be1e57f` stays reachable only through the PR page. |
| A local pre-push history scan | Not available. Substitute: `scripts/check_clean.py`, the Rust text guard and `scripts/scan_history.py`, recorded in `PROGRESS.md`. |
