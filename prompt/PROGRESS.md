# Progress record

Resume from this file. Update it at every milestone and before any pause. Keep entries factual: commands run, results, commit hashes.

## Starting state (recorded when this package was written, 2026-11-10)

- Starting branch: `feature/independent-abatement-subsidy`, commit `84dca45` (working tree clean, no local changes).
- Package branch: `docs/prompt-completion-spec`, created from `84dca45`; contains only the `prompt/` folder on top.
- Ahead/behind of `84dca45` against `origin/main` (`b4817b3`): 17 ahead, 8 behind; merge base `8750b09`.
- Remote heads: `main` `b4817b3`; `feature/ui-ux` `83e4f6d` (merged, 0 ahead of main); `develop` `b30664b`; `fix/selftest-chrome-start` `f8d5a34`; `feat/astryx-nush2` `b4817b3`.
- PR 1, 2, 3: all merged. `be1e57f` stays reachable through PR 1.
- Rust `target/` folders deleted: first build is a full rebuild.
- Verification state: 412 workspace tests passing at `8dbfc6d` (verifier run); the coordinating session's own full re-run died on a fork error and must be redone.

## Current phase

Phase 0: baseline lock not started (the session that writes the package does no engineering).

## Milestones

| # | Milestone | Status | Commit | Pushed head |
|---|---|---|---|---|
| 1 | Repository audit and baseline lock | not started | | |
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

## Test and verification results

None recorded yet for this package.

## Outstanding assumptions

D14 (cryogenic f and r, no source), D15 (shipping distance and unverified coordinates), D16 (unknown streams U1, U2, U4b, U6), D21 (island mirrored), D22 (electricity S$0.20/kWh, no sourced tariff), D23 (power island kept in island). Dependence on INL Case 6 through the heat and power balance.

## Blockers

None recorded. Candidate: no sourced Singapore electricity export price; unverified site coordinates.

## Next actions

1. `git fetch`; verify the heads above; record the true starting state in this file.
2. Create `feature/reference-plant-completion` from the development line; merge `origin/main` (normal merge).
3. Run the clean rebuild and the full gate sequentially; record timings and results.
4. Start milestone 2 with WP0 of `ENGINE_REFERENCE_PLANT.md`.

## Update 2026-11-10

- Package branch `docs/prompt-completion-spec` carries the package plus the raystorage integration files. Unpushed local history was rewritten once to pass the history scan (see `CURRENT_STATE_AND_BASELINE.md` section 6).
- raystorage pinned at `3a25c4a7`. Privacy findings recorded in `RAYSTORAGE_INTEGRATION.md`; owner action pending (remove browser-profile folders from raystorage).
