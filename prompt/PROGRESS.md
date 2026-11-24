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
| 1 | Repository audit and baseline lock | done, with open findings F1 to F3 below | see "Milestone 1" | see "Milestone 1" |
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
3. `python -I scripts/scan_history.py` (new at this milestone): every commit message, author and committer address and every blob reachable from `HEAD`, with the rules of `check_clean.py` for local absolute paths, profile folders, matriculation numbers, group numbers, schedule, allocation and declaration text, plus personal e-mail addresses (anything other than the non-personal service addresses that `check_site.py` accepts) and module-style course codes (reviewed technical identifiers such as AP1000 are listed in the script). Office files are opened and their XML scanned; PDF and other binaries get the path rules on raw bytes only, which is a known gap (compressed PDF streams are not decoded). `--self-test` proves each rule fires on a planted sample.

Known content of the pushed history that the scan accepts: 53 commit messages (all 34 of `main` and the 19 of `docs/prompt-completion-spec` beyond it) carry an assistant co-author trailer with a non-personal service address. They are pushed history, and rewriting them is not authorised. Commits made from 2026-11-24 on carry no such trailer.

### Toolchain

- `rustup toolchain install 1.96.0 --profile minimal --component rustfmt,clippy --target wasm32-unknown-unknown --no-self-update` (per-user rustup; default toolchain unchanged, still stable 1.100.0). In the repository `rust-toolchain.toml` selects `rustc 1.96.0 (ac68faa20 2026-05-25)`, `cargo 1.96.0`, `rustfmt 1.9.0`, `clippy 0.1.96`, host `x86_64-pc-windows-gnu`. Install time 35 s.
- Python 3.12.10 in a virtual environment outside the repository with SymPy 1.14.0; every script runs with `python -I`.
- Browser self-test: Microsoft Edge through `scripts/selftest.py --chrome <msedge.exe>`.
- Fork-error settings (`TESTING_AND_VERIFICATION.md` section 6): one cargo job at a time, `CARGO_BUILD_JOBS=2`, `cargo test -- --test-threads=2`, steps run sequentially by one script that logs each exit code and wall time; no browser open during cargo runs. No fork or resource error occurred.

### Test and verification results

Filled in below by the gate run on the milestone commit.

## Test and verification results

See milestone sections.

## Outstanding assumptions

D14 (cryogenic f and r, no source), D15 (shipping distance and unverified coordinates), D16 (unknown streams U1, U2, U4b, U6; upper bound confirmed), D21 (island mirrored), D22 (electricity S$0.20/kWh, no sourced tariff), D23 (power island kept in island); D21 to D23 confirmed as labelled assumptions on 2026-11-24. Dependence on INL Case 6 through the heat and power balance.

## Blockers

None blocking milestone 2. Candidate: no sourced Singapore electricity export price; unverified site coordinates.

## Next actions

1. Milestone 2: WP0 of `ENGINE_REFERENCE_PLANT.md` (registry entries, `refplant.rs` structs and reader; nothing numerical may change).
2. Re-derive every IEAGHG value from the report PDF with its page: the two files in `reference/` contain 214 `<working folder>` substitutions (175 in the specification, 39 in the critique), each of which also removed the character before a colon, so any value next to one is unreadable.
3. Open findings F1 to F3 of milestone 1 (below).

## Update 2026-11-10

- Package branch `docs/prompt-completion-spec` carries the package plus the raystorage integration files. Unpushed local history was rewritten once to pass the history scan (see `CURRENT_STATE_AND_BASELINE.md` section 6).
- raystorage pinned at `3a25c4a7`. Privacy findings recorded in `RAYSTORAGE_INTEGRATION.md`; owner action pending (remove browser-profile folders from raystorage).
