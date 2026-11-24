# Commit and deployment policy

## 1. Branches

The completion branch `feature/reference-plant-completion` was created on 2026-11-24 at `1a1e822`, the tip of `docs/prompt-completion-spec`, which is the only pushed ref that holds both the development line and this package (the local branch `feature/independent-abatement-subsidy` named in the 2026-11-10 record was never pushed). `origin/main` was then merged into it with the ordinary merge commit `9fc36ff`. `docs/prompt-completion-spec` stays untouched as a record. Never silently switch to `main`. Never reset, force-push, delete or overwrite commits. Keep `feature/ui-ux`, `develop`, `fix/selftest-chrome-start` and `feat/astryx-nush2` untouched; they are merged or ancestors and must not be deleted automatically.

## 2. Identity and message rules

Commits use user `weewzr` and email `288510855+weewzr@users.noreply.github.com`. Never use any other address. Never merge with the GitHub web button or `gh pr merge` (GitHub stamps the account's primary mail on the merge commit). The only merge into `main` is the one in section 5: the coordinating session merges `origin/main` into the completion branch locally (an ordinary merge commit under the identity above), re-runs the gates on that commit, then runs `git push origin feature/reference-plant-completion:main`, which the remote accepts only as a fast-forward. Never `--force` or `--force-with-lease`. Commit messages follow the existing imperative style, end with the attribution line given in the session's instructions, and follow the text rules (no em dashes). Never use `--no-verify`. The external pre-push hook of 2026-11-10 is not available on the machines of the 2026-11-24 work; the owner approved a recorded substitute (`PROGRESS.md`, "History scan substitute"): `scripts/check_clean.py`, the Rust text guard (`cargo test -p nush2_plant --test no_internal_pointers`) and `python scripts/scan_history.py`, all three before every push.

## 3. Milestones (each: tests pass, accurate message, `PROGRESS.md` updated, push)

1. Repository audit and baseline lock (merge `origin/main` into the completion branch with a normal merge; record state).
2. Reference-plant equations and implementation (WP0, WP1).
3. Reference integration and ninth export (WP3, WP4 up to regeneration).
4. Carbon and subsidy completion (WP2, U6 test, SymPy, nine minor findings, decision log).
5. Economic reconciliation and result regeneration (defaults flipped, tables regenerated, headlines reconciled).
6. Frontend reference PFD.
7. Scenario comparison and tour updates (labels, site scripts, withdrawn list, tour defects).
8. Derivation 03 and PDF.
9. Equation register and README.
10. Presentation and PFD correction.
11. Independent verification (and fixes).
12. Final report and release candidate.

Do not commit a feature as complete while its acceptance tests fail. Commit partial, honest progress as such ("work in progress") only on the completion branch and never as a milestone.

## 4. What never enters Git

Secrets, credentials, privacy-flagged material, local-only working documents, Office and PDF working files, rebuildable build output (`target/`), copyrighted PDFs (the IEAGHG report, papers), personal or local absolute paths, other people's names, course or schedule text. Run `scripts/check_clean.py`, the Rust text-guard test and the history scan (`scripts/scan_history.py`) before every push.

## 5. Final steps

Rebuild all results; run all engineering tests; run the complete independent verification; build the website; run the browser tests at all four viewports; build the derivation PDF; write `docs/FINAL_VERIFICATION_REPORT.md`. Open the final pull request from the completion branch to `main` with the 17-item summary and the honest limitations in the body, ending with the attribution line required by the session instructions (the instruction of 2026-11-24 requires none).

**Release gate** (owner instruction of 2026-11-24, `ORIGINAL_BRIEF_3_MERGE_POLICY.md`). The coordinating session merges only when all of these hold: (a) every gate G0 to G6 in `TESTING_AND_VERIFICATION.md` ran and passed on the final commit and is recorded in `PROGRESS.md`; (b) no item of the 17-item report in `docs/FINAL_VERIFICATION_REPORT.md` is FAIL, and every BLOCKED item is stated as a limitation in the report, the README and the pull request; (c) the independent verifier has no open blocking or major finding; (d) the `ci` and `site` workflows are green on the pull request's final commit; (e) `origin/main` has not moved since the last merge of `main` into the completion branch, or it is merged again and the gate is re-run. Then merge as section 2 says (merge `main` into the completion branch, fast-forward push to `main`), wait for the `site` workflow on the new `main` commit, and check the published pages (build.json commit, headline values, no withdrawn figure). If `site` fails after the push, nothing deploys, because the deploy job needs the build; fix forward with a new commit on the completion branch and repeat the gate. Never revert by force-push. Do not deploy partially validated results: no merge before the gate passes. Replacing the live headline cost with the regenerated IEAGHG-reference result is accepted once the gate passes.

## 6. Housekeeping boundaries (document, never execute)

- **Archive.** A private archive repository holds text-type working files, verified by checksum. The 947 privacy-flagged files and the 140 Office and PDF files stay local, are not backed up automatically, and must not be inspected, uploaded or published without the owner's explicit permission for that action.
- **Remote branches.** `develop`, `feature/ui-ux`, `fix/selftest-chrome-start` and `feat/astryx-nush2` are merged or ancestors of current work. Inspect and record status; do not delete. Offer the owner a deletion list at the end.
- **Old public repository.** The earlier public repository named "Implementation-of-Nuclear-Assisted-SMR-with-Carbon-Capture-in-Singapore" holds lecture PDFs in its history and on merged branches. Do not rewrite or erase public history. Document the exposure and recommend that the owner decides between making the repository private, or deleting and recreating it after confirming nothing else depends on it, with GitHub Support for cached views.
- **Commit exposure.** Commit `be1e57f` (GitHub-stamped merge of PR 1) stays reachable through the PR page. Removal needs GitHub Support. Do not rewrite PR history and never claim it has been removed. It is no longer an ancestor of `main`, which was force-pushed once to `fbfe631` on 2026-11-06 (`CURRENT_STATE_AND_BASELINE.md` section 1).

## 7. History scan note

The local pre-push scan rejects phrases about schedules and similar identifiers even in commit messages and old blobs. `scripts/scan_history.py`, its substitute since 2026-11-24, applies the same rules as `scripts/check_clean.py` to every commit message and every blob of the branch history, and adds personal e-mail addresses and course codes. Keep new text free of them and run `scripts/check_clean.py` first. Do not rewrite pushed history; rewriting unpushed local commits to satisfy the scan is acceptable once, with a backup branch, and must be recorded in `PROGRESS.md`. Before any further public release run a repository-wide secret and privacy audit including relevant history.
