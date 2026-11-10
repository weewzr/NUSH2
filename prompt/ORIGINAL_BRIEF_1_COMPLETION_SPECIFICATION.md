# Original brief 1: create the autonomous engineering completion and development specification

Faithful copy of the owner's first instruction for this package (lists run together into paragraphs, wording kept), kept so the future session can check the package against it. The package files are the working instructions; this file is the source brief.

Repository: https://github.com/weewzr/NUSH2

Create a comprehensive, persistent project-instruction package in `prompt/`.

This package will be used by a future Claude Code session operating through an authorised collaborator's Claude Max account. The collaborator will be overseas and unavailable to manage routine agent permissions, inspect every change or approve each development action. The objective is to let Claude independently complete the outstanding engineering, carbon, economic, frontend, documentation and verification work.

In this session, create, review, commit and push the instruction package only. Do not begin the substantive NUS-H2 engineering implementation yet.

## 1. First: inspect and preserve the current repository

Before creating any files:

1. Inspect the entire NUSH2 repository.
2. Inspect all branches and the current commit history.
3. Identify the exact head of `main`.
4. Identify the exact head of `feature/ui-ux`.
5. Calculate the actual ahead/behind relationship and merge base.
6. Verify the location of commit `84dca45`.
7. Inspect PR #1 and its merge history.
8. Read the existing engineering specifications, design decisions, mathematical derivations, subsidy log, validation matrix, reference data, results and frontend documentation.
9. Locate existing reference-plant specifications, verifier findings and owner decisions, including any files not found in the initial directory listing.
10. Establish the current state of all outstanding tasks.

The repository already contains substantial work. Do not mistake an absent file on one branch for proof that the file does not exist elsewhere. Treat the actual Git history and repository contents as authoritative over summaries.

Critical branch-preservation requirements: do not reset, force-push, delete or overwrite existing development work. Treat `feature/ui-ux` and all its existing commits as valuable work.

Before any modification: check the working tree; check ignored and untracked files; record the starting branch and commit; identify any local changes; check the remote branch state; preserve the existing merge history; inspect the old commit exposure and archive boundaries.

The remote branch previously inspected was `feature/ui-ux` at `83e4f6d33231b818d6e35765486e1ee5404ba580`. Verify its current state rather than assuming it remains unchanged. Do not merge into `main` during the project without explicit authorisation. A final pull request to `main` is expected, but it must remain unmerged.

## 2. Create the instruction package

Create `prompt/` with: README.md, MASTER_PROMPT.md, CURRENT_STATE_AND_BASELINE.md, ENGINE_REFERENCE_PLANT.md, CARBON_AND_SUBSIDY.md, ECONOMIC_ASSUMPTIONS.md, FRONTEND_AND_VISUAL_DESIGN.md, DOCUMENTATION_AND_DERIVATIONS.md, TESTING_AND_VERIFICATION.md, GITHUB_REFERENCES.md, AUTONOMOUS_EXECUTION.md, COMMIT_AND_DEPLOYMENT.md, PROGRESS.md.

The files must contain the complete actionable specification for the future development session. They must not merely repeat the issue list. Read the actual repository and translate all outstanding work into concrete implementation tasks, acceptance criteria, dependencies and tests.

## 3. Overall objective

Complete the NUS-H2 engineering project and its browser simulator. Preserve the existing scientific Rust engine, browser-side WebAssembly implementation, JavaScript frontend, engineering derivations, literature provenance and existing testing framework. Six connected workstreams: (1) reference-plant engineering engine; (2) carbon-abatement and subsidy implementation; (3) economic assumption reconciliation and results regeneration; (4) frontend process-flow and scenario presentation; (5) engineering derivations, documents and presentation; (6) independent verification and final reporting. All work must be scientifically traceable and reproducible.

## 4. Engineering reference plant (ENGINE_REFERENCE_PLANT.md)

The reference plant follows the accepted IEAGHG-based specification. The engine implementation is incomplete and must be resumed.

4.1 Reference configuration. Use the IEAGHG base-case module, PSA tail-gas balance, steam balance, power balance and correct plant-boundary definitions. The reference plant's PSA tail gas, steam and power lines must be recalculated from the IEAGHG balances, not from copied headline values or unverified approximations. The specification and its critique already exist in project records. Review both, including the existing snapshot work associated with commit `84dca45`. Do not substitute the INL reference as the default plant.

4.2 Implementation. Implement the complete reference engine. Wire its results into carbon-abatement calculations, GAS calculations, lost-steam lines, base-power calculations, and total-CAPEX and subsidy accounting where applicable. Add the ninth exported table `Reference_Plant.csv`. Add the 14 `REF_` validation rows. Regenerate all dependent results and exports.

4.3 Verification. Create independent tests for mass balances, energy balances, PSA tail-gas balance, steam balance, power balance, reference-plant consistency, correct boundary accounting, integration with abatement calculations and integration with subsidy calculations. Use the IEAGHG specification as the basis for acceptance. The build folders under Rust `target/` have been deleted: expect a complete rebuild and distinguish ordinary clean-build time from genuine errors. Do not treat a successful compilation as engineering verification.

## 5. Carbon-abatement and subsidy implementation (CARBON_AND_SUBSIDY.md)

5.1 Accepted reference decision. The owner has accepted the IEAGHG-based reference-plant specification. Record this decision in `docs/ABATEMENT_SUBSIDY_LOG.md`. The previous verifier's major disagreement about the reference direction is settled. Preserve the history of the decision and explain the reason for the adopted direction.

5.2 Subsidy definition. Definition C is the primary total-CAPEX subsidy definition: range 0 to 100 percent, increment 5 percent. Definition B becomes a secondary legacy comparison. Definition C drives the primary scenario comparisons and defaults. Do not continue presenting Definition B as the primary economic definition.

5.3 Outstanding decision log. Record decisions D13 to D23 in `docs/ABATEMENT_SUBSIDY_LOG.md`. Locate the original decision records before transcription. Preserve exact definitions, accepted reasoning, outstanding qualifications, source references and remaining unresolved items. Do not invent missing owner decisions.

5.4 Missing U6 test. Add an explicit unit test for the unknown start-up and flaring stream (U6) being set to zero. The test must exercise the relevant calculation directly and must not rely solely on exported-table comparison. Test relevant limiting cases and verify that the stream's absence is correctly propagated through the carbon balance and abatement calculations.

5.5 Stale SymPy checks. Review the nine stale checks: seven in the carbon script and two in the subsidy script (C19 and C15b). Determine whether to repair, replace with equivalent symbolic or numerical assertions, or remove with documented reasoning. Do not weaken verification merely to obtain passing results.

5.6 Nine minor verifier findings. Locate the exact nine findings, resolve them individually (labels, test coverage, provenance) and record each resolution against its original finding. Do not collapse them into an undocumented general cleanup.

5.7 Abatement upper-bound qualification. The abatement figure must remain explicitly labelled an upper bound while these four streams remain unknown: MEA make-up, CO2 vent before the pipeline, leakage after storage, start-up and flaring. Do not convert an upper bound into an asserted final value. Do not invent values for unknown streams.

## 6. Economic assumptions and results (ECONOMIC_ASSUMPTIONS.md)

6.1 No premature headline claims. The anticipated move in central L5 from approximately S$203/t to approximately S$221/t is a hand estimate only. The engine must determine the actual outcome. Do not publish the estimate as an engine result. All previously quoted headlines must be reconciled after the reference-plant implementation and results regeneration.

6.2 Subsidy and plant-built floor. Preserve Definition C as primary and Definition B as legacy. The plant-built floor reference of approximately S$283/t is a previously stated result requiring re-verification. The observation that Definition A does not reach S$100/t even at 100 percent subsidy is subject to re-verification under the final engine. Do not carry over any numerical conclusion without confirming it against regenerated results.

6.3 Three owner-applied defaults, to be documented and reviewed: (1) island grant mirrored on the reference; (2) electricity at S$0.20/kWh, with no sourced Singapore export price; (3) power island retained within the island boundary. Keep these as transparent assumptions unless an authoritative source or engineering decision supersedes them. Do not fabricate a Singapore electricity export tariff.

6.4 Independence from INL. Cryogenic recovery 0.752 and hydrogen return 0.922 remain assumptions without supporting citations. The cost also remains dependent on INL Case 6 through the heat and power balance. Document precisely which calculations remain dependent on INL and which have been independently reconstructed using the IEAGHG reference. Do not claim a completely independent reference plant while the cost calculation still relies on INL-derived inputs.

6.5 Shipping distance. The 1,000 km distance remains an assumption. The sites have been named, but their coordinates have not been verified. Do not create fabricated geographical coordinates. Research the source locations and define a reproducible distance calculation. Label shipping distances as assumptions until the relevant source and calculation are established.

## 7. Frontend and visual design (FRONTEND_AND_VISUAL_DESIGN.md)

Preserve the existing vanilla JavaScript, SVG, CSS and Rust/WebAssembly architecture unless a documented technical justification supports changing it. Do not automatically migrate to React. The existing frontend has an interactive PFD, equipment unit panels, process tour, SVG charts, scenario controls, live engine replies, WebAssembly integration, responsive layouts, and content and external-host guards. Maintain the existing engine response contract. Do not move engineering calculations into JavaScript.

7.1 Reference PFD. Complete it with the PSA tail-gas route, relevant return and reject streams, steam lines, power lines, material stream labels, a stream table, correct flow direction and correct plant boundary. All displayed stream values come from the live engine response; no inconsistent precomputed numbers.

7.2 Scenario comparison page. Scenario selectors; comparison charts; clear central and sensitivity results; a permanently visible S$100/t target reference line; Definition C and Definition B results clearly distinguished; transparent subsidy assumptions; cases that meet or miss the target clearly identified. Do not misrepresent an upper bound as a final result.

7.3 Labels and process tour. Update equipment names, stream labels, tour descriptions, live-reply fields, scenario-dependent captions, keyboard interactions, accessibility and responsive behaviour. Reconcile the existing tour with the corrected engineering model.

7.4 Site scripts. Update site scripts and the withdrawn-figures list (they still describe the old INL headline). Remove superseded figures from public site outputs. Retain a documented record of withdrawn results in internal documentation. Do not silently alter historical data.

7.5 Existing outstanding tour defects. Investigate and resolve the keyboard self-test failure at 1024 x 600, the previous-to-step-1 card-heading visibility at 1440 x 900, remaining drawing inconsistencies in the existing UI assessment, and any regression from the new reference PFD. Retain the good behaviour at 1440 x 900, 375 x 812 mobile, reduced motion, keyboard accessibility, live-reply-derived values and no unexpected external requests.

## 8. Engineering derivations and documents (DOCUMENTATION_AND_DERIVATIONS.md)

Complete: (1) derivation 03 for the IEAGHG-based reference plant; (2) compile the derivation PDF using Tectonic; (3) the equation-to-Rust table; (4) verify every cited Rust line number against the final code; (5) update the equation register; (6) update the engineering documentation; (7) regenerate the README from the final verified results; (8) update the validation matrix; (9) update the abatement/subsidy log; (10) update the third-party notices if dependencies change.

The derivation must include physical definitions, material and energy balances, reference-case assumptions, PSA tail-gas treatment, steam and power accounting, engineering boundaries, source references and limitations. It must not merely restate code; it shows the derivation and its relationship to the implemented equations. The README derives its numerical claims from the final regenerated exports.

## 9. GitHub repositories to research (GITHUB_REFERENCES.md)

Process engineering: IDAES (https://github.com/IDAES/idaes-pse), DWSIM (https://github.com/DanWBR/dwsim), CoolProp (https://github.com/CoolProp/CoolProp): process modelling, independent thermodynamic checks, engineering balances, model validation. These are reference tools and optional dependencies, not permission to replace the current Rust engine.

Numerical and property testing: Hypothesis (https://github.com/HypothesisWorks/hypothesis), proptest (https://github.com/proptest-rs/proptest), cargo-nextest (https://github.com/nextest-rs/nextest): generated tests for mass balance, energy balance, emissions accounting, cost definitions, zero streams and boundary cases.

Frontend and charts: D3 (https://github.com/d3/d3), Observable Plot (https://github.com/observablehq/plot), Playwright (https://github.com/microsoft/playwright): interactive SVG, scenario-comparison graphs, accessibility, browser automation, responsive testing. Prefer integration with the current vanilla JavaScript/SVG frontend.

Documentation and build: Tectonic (https://github.com/tectonic-typesetting/tectonic), Ruff (https://github.com/astral-sh/ruff): reproducible document builds and Python quality checks.

For every candidate: check current maintenance, read the licence, check dependency requirements, evaluate compatibility, identify bundle or build-size implications, identify external-host requirements, evaluate maintenance cost. Do not install libraries just because they are listed. Prefer selectively adopting improvements that directly resolve known gaps. Preserve `THIRD_PARTY_NOTICES.md` requirements.

## 10. Testing and final verification (TESTING_AND_VERIFICATION.md)

10.1 Engineering tests: reference-plant mass balance, energy balance, PSA tail-gas balance, steam balance, power balance, reference integration, carbon closure, zero-stream behaviour, subsidy calculations, economic ledger consistency, scenario reproducibility.

10.2 Export tests: the nine exported tables including `Reference_Plant.csv`; all 14 `REF_` validation rows; compare regenerated exports with the expected final engine results. Never update golden files merely to make tests pass.

10.3 Stale symbolic checks: resolve all nine stale SymPy checks and preserve equivalent verification strength.

10.4 Browser testing at 1440 x 900, 375 x 812, 1024 x 600 and 1024 x 768. Test guided tour, keyboard controls, scenario comparison, reference PFD, unit panels, live reply fields, the S$100/t line, reduced motion, mobile layout, no console errors, no failed requests, no external requests.

10.5 Full independent verification. The earlier independent verification run stopped because of process fork errors. Re-run the complete check from the final implementation. Investigate the cause rather than suppressing the error. Where parallel execution is responsible, use controlled sequential or lower-concurrency execution and document the exact change.

10.6 Final 17-item verification report. Each item: requirement, test or evidence, actual result, PASS / FAIL / BLOCKED, relevant file or commit reference, remaining qualification. Do not report an item as passing merely because the code appears to implement it.

## 11. Housekeeping and security boundaries

11.1 Archive. `raystorage` has been verified. The 947 privacy-flagged files and the 140 Office and PDF files remain local and must not be backed up automatically. Do not inspect, upload or publish them unless their owner explicitly authorises the particular action. Do not infer that they are safe to publish because they are included in a local archive.

11.2 Remote branches. `develop`, `feature/ui-ux` and `fix/selftest-chrome-start`: inspect and document their status. Do not delete them automatically.

11.3 Old public repository. The `Implementation-of-Nuclear-Assisted-SMR` repository contains lecture PDFs in its history. Do not attempt to rewrite or erase public history automatically. Document the exposure and recommend a safe repository-housekeeping approach.

11.4 Commit exposure. The older commit `be1e57f` remains reachable through https://github.com/weewzr/NUSH2/pull/1. The owner has been informed that GitHub Support is required for removal. Do not rewrite PR history or claim that this exposure has been removed.

## 12. Presentation and drawing fixes

Presentation: correct 12 typed context constants; correct the Figure 2 legend size on slides 7 and 26. PFD: correct remaining drawing discrepancies, flow directions, equipment and stream labels; align the visible PFD with the actual reference-plant calculation. Retain the existing final-documentation and output conventions. Do not publish intermediate or unverified presentation figures as final.

## 13. Autonomous execution and permissions (AUTONOMOUS_EXECUTION.md)

The repository owner wants the future session to manage routine development independently. The collaborator is overseas and must not be required to approve every ordinary action.

13.1 Within the NUSH2 repository and its development environment the owner pre-authorises the future session to: inspect all project files and Git history; inspect local and remote branches; create and edit project files; install ordinary project dependencies; perform clean Rust rebuilds; run Python scripts; execute verification and testing scripts; build PDFs and figures; build the WebAssembly module; run the browser self-test; create dedicated development branches; commit completed phases; push completed development commits to the authorised remote branch; open pull requests; use independent subagents; review and integrate agent output; continue automatically through completed phases. Use supported workspace permissions and documented Claude Code permission settings. These instructions express the owner's authorisation but cannot grant GitHub credentials, override a protected branch, or change the permissions of the collaborator's account. The collaborator must establish the actual repository access, authentication and permitted tool capabilities before leaving. Do not bypass approval controls, sandbox boundaries, authentication or security restrictions.

13.2 No routine human approval for routine file edits, installing ordinary dependencies, running tests or builds, generating documents, normal calculations, phase commits, or pushing to the authorised development branch. If an action genuinely requires owner-level approval or an unavailable permission, document the blocker and continue with independent work.

13.3 Branch protection. Use a dedicated completion branch based on the correct existing development history. Do not silently switch to `main`. Do not destroy or overwrite existing branch commits. Do not force-push or rewrite history. Do not merge the final pull request. The owner has authorised automatic development commits and pushes to the dedicated authorised branch.

13.4 Scientific decisions. The IEAGHG reference direction and Definition C as the primary subsidy definition are settled: implement them without asking again. For unresolved physical data or new consequential assumptions: research authoritative evidence, record source and uncertainty, keep unsupported assumptions clearly labelled, do not invent evidence, do not halt unrelated work. Do not alter the user's accepted scientific definitions without a documented technical reason.

13.5 Subagent management. Separate agents for reference-plant engine; carbon and subsidy calculations; economics and source verification; frontend and PFD; derivations and documents; independent testing and verification. The coordinating agent assigns non-conflicting responsibilities, reviews all code, reconciles shared interfaces, resolves integration conflicts, runs the full test suite and verifies scientific claims independently. No agent modifies shared files concurrently without coordination.

13.6 Automatic progression. One phase at a time. When a phase passes its acceptance criteria: run the full relevant tests; review the code changes; update documentation; update `prompt/PROGRESS.md`; commit the milestone; push the development branch; continue automatically to the next phase. Do not ask for confirmation between phases. If a phase is blocked by unavailable source data or unresolved evidence, mark that item BLOCKED and continue other work.

## 14. Commit and deployment policy (COMMIT_AND_DEPLOYMENT.md)

Milestones: (1) repository audit and baseline lock; (2) reference-plant equations and implementation; (3) reference integration and ninth export; (4) carbon and subsidy completion; (5) economic reconciliation and result regeneration; (6) frontend reference PFD; (7) scenario comparison and tour updates; (8) derivation 03 and PDF; (9) equation register and README; (10) presentation and PFD correction; (11) independent verification; (12) final report and release candidate.

Each milestone requires passing tests, an accurate commit message, an updated progress record and a push to the authorised development branch. Do not commit a feature as complete while its acceptance tests are failing. Do not push secrets, privacy-flagged materials, local-only working documents or excluded Office/PDF files. Retain the existing site build, content guard, result comparison and browser testing requirements.

At the end: rebuild all results; run all engineering tests; run the complete independent verification; build the website; run all browser viewport tests; build the derivation PDF; generate the final report. Open a final pull request targeting `main`. The pull request must remain open and unmerged. The owner wants the implementation pushed but does not want Claude to merge it into `main`.

## 15. Progress tracking

`prompt/PROGRESS.md` records: initial branch, initial commit, initial ahead/behind status, starting working-tree state, current phase, completed milestones, latest commit hash, latest pushed branch head, test results, engineering verification results, outstanding assumptions, blockers, next actions. The future session resumes from this file rather than repeating completed work.

## 16. Instruction-package completion criteria

Before committing the package verify: (1) all 13 required files exist; (2) the task list covers every outstanding item supplied by the owner; (3) the accepted IEAGHG and Definition C decisions are correctly represented; (4) unresolved assumptions remain explicitly unresolved; (5) the 14 REF validation rows and the ninth export are included; (6) the nine stale SymPy checks are addressed; (7) the missing U6 unit test is explicit; (8) the nine minor verifier findings are separately tracked; (9) the 17-item final verification is specified; (10) the presentation, PFD, browser and document requirements are included; (11) the branch-preservation instructions are consistent; (12) automatic phase commits and pushes are clearly specified; (13) the final PR is required to remain unmerged.

## 17. Commit and push this instruction package

Confirm the current Git state; verify the correct existing branch and baseline; use a dedicated documentation branch if necessary; stage only the intended `prompt/` files; commit with `docs(prompt): add autonomous NUS-H2 completion specification`; push the documentation branch to the authorised NUSH2 remote; verify the pushed files are accessible. Do not commit any substantive engineering changes in this session. Do not force-push. Do not merge into main.

## 18. Final response

Report: branch name, commit SHA, GitHub URL for `prompt/`, files created, current branch baseline, any missing specifications or unresolved questions.

Purpose: allow a future Claude session to complete NUS-H2 using the repository itself as its persistent source of instructions, without requiring the collaborator to manage every development step.
