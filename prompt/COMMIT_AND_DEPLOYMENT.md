# Commit and deployment policy

## 1. Branches

Create the completion branch from the development line at its latest pushed state: `feature/independent-abatement-subsidy` (verify its head first; the package branch `docs/prompt-completion-spec` sits on top of it). Suggested name `feature/reference-plant-completion`. Never silently switch to `main`. Never reset, force-push, delete or overwrite commits. Keep `feature/ui-ux`, `develop`, `fix/selftest-chrome-start` and `feat/astryx-nush2` untouched; they are merged or ancestors and must not be deleted automatically.

## 2. Identity and message rules

Commits use user `weewzr` and email `288510855+weewzr@users.noreply.github.com`. Never use any other address. Never merge with the GitHub web button or `gh pr merge` (GitHub stamps the account's primary mail on the merge commit). Commit messages follow the existing imperative style, end with the attribution line given in the session's instructions, and follow the text rules (no em dashes). Never use `--no-verify`. The local pre-push hook scans the full history (about 5 minutes); let it run.

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

Secrets, credentials, privacy-flagged material, local-only working documents, Office and PDF working files, rebuildable build output (`target/`), copyrighted PDFs (the IEAGHG report, papers), personal or local absolute paths, other people's names, course or schedule text. Run `scripts/check_clean.py`, the Rust text-guard test and the history scan before every push.

## 5. Final steps

Rebuild all results; run all engineering tests; run the complete independent verification; build the website; run the browser tests at all four viewports; build the derivation PDF; write `docs/FINAL_VERIFICATION_REPORT.md`. Do not deploy partially validated results: the live site is built by `site.yml` from `main`, so nothing deploys until the owner merges. Open the final pull request from the completion branch to `main` with the 17-item summary and the honest limitations in the body, ending with the attribution line required by the session instructions. The pull request stays open and unmerged. Verify the site only after the owner merges, by fetching the published pages.

## 6. Housekeeping boundaries (document, never execute)

- **Archive.** A private archive repository holds text-type working files, verified by checksum. The 947 privacy-flagged files and the 140 Office and PDF files stay local, are not backed up automatically, and must not be inspected, uploaded or published without the owner's explicit permission for that action.
- **Remote branches.** `develop`, `feature/ui-ux`, `fix/selftest-chrome-start` and `feat/astryx-nush2` are merged or ancestors of current work. Inspect and record status; do not delete. Offer the owner a deletion list at the end.
- **Old public repository.** The earlier public repository named "Implementation-of-Nuclear-Assisted-SMR-with-Carbon-Capture-in-Singapore" holds lecture PDFs in its history and on merged branches. Do not rewrite or erase public history. Document the exposure and recommend that the owner decides between making the repository private, or deleting and recreating it after confirming nothing else depends on it, with GitHub Support for cached views.
- **Commit exposure.** Commit `be1e57f` (GitHub-stamped merge of PR 1) stays reachable through the PR page. Removal needs GitHub Support. Do not rewrite PR history and never claim it has been removed.

## 7. History scan note

The local pre-push scan rejects phrases about schedules and similar identifiers even in commit messages and old blobs. Keep new text free of them and run `scripts/check_clean.py` first. Do not rewrite pushed history; rewriting unpushed local commits to satisfy the scan is acceptable once, with a backup branch, and must be recorded in `PROGRESS.md`. Before any further public release run a repository-wide secret and privacy audit including relevant history.
