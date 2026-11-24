# Original brief 3: merge, release gate and owner confirmations of 2026-11-24

Record of the owner's third instruction for this package, given on 2026-11-24 through the coordinating session of the multi-repository work and transcribed here with its wording kept where it was given as rules. It supersedes the merge and deployment rules of briefs 1 and 2. Every other rule of those briefs stands. Briefs 1 and 2 are left unedited as historical records.

## Merge and deployment

- Implementation of the completion work is authorised. Work branches may be committed and pushed.
- The coordinator alone merges into `main`, by fast-forward push after the gates pass. Never the GitHub merge button, never force.
- Automatic merge and deploy is approved for NUS-H2.
- One merge method: fast-forward push after CI, no web button, no force.
- Replacing the live headline cost with the regenerated IEAGHG-reference result is accepted once the release gate passes.

## Release gate

The coordinator merges only when all of these hold:

1. Gates G0 to G6 of `TESTING_AND_VERIFICATION.md` ran and passed on the final commit, and the results are recorded in `PROGRESS.md`.
2. The final verification report (`docs/FINAL_VERIFICATION_REPORT.md`) has no item marked FAIL.
3. The independent verifier has no open major (or blocking) finding.
4. CI is green on the pull request.

The merge is done by merging `main` into the work branch and fast-forward pushing the work branch to `main`.

## Owner confirmations recorded with this brief

- Reference plant: IEAGHG 2017-02 Base Case.
- Subsidy definitions: Definition C primary, Definition B labelled legacy, Definition A labelled secondary.
- D21 (island grant mirrored on the reference), D22 (electricity S$0.20/kWh both ways) and D23 (IEAGHG power island kept inside the island on both sides) are confirmed as labelled assumptions, each with its sensitivity row.
- The switch `SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND` removes capital only; the island's O&M share is stated in the row text.
- Legacy registry text stays unchanged; correct locators go into the new rows.
- The guard that forbids web fonts stays.
- Unknown streams stay unknown, and abatement keeps its upper-bound qualification.
- The work branch is `feature/reference-plant-completion`, created at `1a1e822` with `origin/main` merged in by a normal merge commit.
- The external pre-push history scan named in this package is replaced by a recorded substitute: `scripts/check_clean.py`, the Rust text guard and the in-repository history scan `scripts/scan_history.py` (local absolute paths, personal e-mail addresses, private course or matriculation identifiers).
- Commits are attributed to the owner only. No assistant attribution line is added to commits, pull requests or files.
