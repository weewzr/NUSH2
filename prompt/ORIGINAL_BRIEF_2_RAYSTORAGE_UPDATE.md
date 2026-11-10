# Original brief 2: update the master instructions with raystorage integration and autonomous execution

Faithful copy of the owner's second instruction for this package (lists run together into paragraphs, wording kept), kept so the future session can check the package against it.

Repositories: primary project https://github.com/weewzr/NUSH2; historical research archive https://github.com/weewzr/raystorage.

## Objective

Update the existing `prompt/` directory in NUSH2 so the future Claude Code implementation session can complete the outstanding engineering, carbon-abatement, subsidy, frontend, documentation and verification work autonomously.

The owner has made raystorage public. It contains extensive historical NUS-H2 working files, including calculations, verification scripts, derivations, research notes, exported results and frontend artifacts. Use this archive to recover useful work, avoid repeating previous research and strengthen the scientific implementation.

This session is for updating, committing and pushing the instruction package only. Do not begin implementing the engineering work yet.

## 1. Repository inspection

Before modifying anything: inspect the current NUSH2 repository and all relevant branches; read the existing `prompt/` files; read `README.md`, `docs/FINAL_DESIGN_DECISIONS.md`, `docs/VALIDATION_MATRIX.md` and `docs/UI_UX_ASSESSMENT.md`; inspect the existing Rust engine, WebAssembly interface, frontend and testing scripts; confirm the latest development branch and commit; preserve the existing `feature/ui-ux` history and its unmerged work, if any; identify the accepted IEAGHG reference-plant and Definition C decisions; reconcile the existing instructions with the current repository state. Do not reset branches, force-push, discard local changes or overwrite completed work.

## 2. Inspect raystorage

Read `NUSH2/README.md` and `NUSH2/MANIFEST.csv`. The archive has six main categories: `derivations/`, `simulations/`, `process-data/`, `PFD-versions/`, `research-and-references/`, `validation/`. Use targeted searches and the SHA-256 manifest rather than loading thousands of files indiscriminately. Pin the archive commit SHA used for the audit. Treat archived files as historical evidence, not automatically authoritative code.

## 3. Create raystorage integration documentation

Create `prompt/RAYSTORAGE_INTEGRATION.md` and `prompt/RAYSTORAGE_FILE_INDEX.md`. The integration document maps relevant archive materials to outstanding NUS-H2 tasks. For each important file record: exact raystorage path; SHA-256; relevant workstream; purpose; source or provenance; verification status; whether superseded; reuse recommendation; required tests; intended NUSH2 destination, if applicable.

## 4. Prioritise the IEAGHG reference reconstruction

Inspect `NUSH2/process-data/session-dee70430/phase3b/ieaghg_balances.py` and `ieaghg_streams.md`. They contain a source-traced reconstruction of the IEAGHG 2017-02 base case. The historical reconstruction reports: hydrogen production 100,000 Nm3/h; hydrogen product 8.994 t/h; total natural gas 30.563 t/h; natural gas feedstock 26.231 t/h; supplementary burner fuel 4.332 t/h; PSA tail gas 2,106.3 kmol/h; PSA tail-gas mass flow 60,658 kg/h; gross electricity 11.500 MWe; net electricity export 9.918 MWe; published CO2 intensity 0.8091 kg/Nm3 H2.

Verify these against the primary IEAGHG report and the archived reconstruction. Pay particular attention to the warning that the published CO2 intensity already includes tail-gas combustion. Do not double-count tail-gas emissions. Use these findings to strengthen the reference-plant specification, engineering balances and validation tests.

## 5. Recover carbon and subsidy work

Inspect relevant files under `NUSH2/derivations/session-dee70430/phase2/derive_subsidy/`, `NUSH2/research-and-references/session-dee70430/phase3/carbon/`, `.../phase3/impl_subsidy/`, `NUSH2/validation/session-dee70430/phase2/verify_subsidy/` and `.../phase3/verify_subsidy/`. Identify reusable calculations, verification scripts, implementation fragments and historical test results.

Preserve the accepted decisions: IEAGHG-based reference plant; Definition C as the primary total-CAPEX subsidy; Definition B as a secondary legacy comparison; subsidy range 0 to 100 percent in 5 percent increments.

Locate evidence relevant to decisions D13 to D23, the missing U6 zero-stream test, the seven stale carbon SymPy checks, the subsidy checks C19 and C15b, the nine minor verifier findings and the remaining unknown abatement streams. Do not fabricate missing decisions, sources or test results.

## 6. Recover mathematical derivations

Inspect `NUSH2/derivations/`. Identify useful LaTeX sources, equation registers, equation-to-code traceability, Tectonic build scripts, reference-plant calculations and previous derivation corrections. Use these to improve the planned reference-plant derivation 03. Do not copy historical Rust line numbers without checking the final code.

## 7. Recover verification evidence

Inspect `NUSH2/validation/`. Locate previous engineering verification, export comparisons, numerical tests, browser self-tests, failure logs and independent reviewer findings. Distinguish historical results from current passing tests. The future implementation must rerun the final tests.

## 8. Recover frontend and PFD work

Inspect `NUSH2/PFD-versions/`. Identify reusable SVG drawing corrections, stream-routing improvements, guided-tour fixes, responsive layout changes, browser test scripts and scenario display improvements. Pay particular attention to the keyboard test at 1024 x 600, previous-to-step-1 card visibility at 1440 x 900, the reference PFD with PSA tail gas, and the scenario comparison page and S$100/t reference line. Do not replace the current frontend with an older archived version.

## 9. Preserve the full outstanding engineering scope

The master instructions must still require: IEAGHG reference-plant engine; correct PSA tail-gas, steam and power balances; reference integration with abatement, GAS, lost-steam and base-power calculations; `Reference_Plant.csv`; fourteen `REF_` validation rows; Definition C subsidy implementation; results regeneration; carbon and SymPy verification fixes; documentation of unresolved assumptions; updated PFD and scenario comparison; derivation 03 and Tectonic PDF; equation register and verified equation-to-Rust references; updated README and presentation; complete independent verification; final 17-item verification report. Do not remove or weaken any existing requirement.

## 10. Privacy and publication

raystorage is public, but its archived content still requires review. Do not automatically copy browser profiles, session storage, personal information, credentials, local-only documents, third-party copyrighted materials or unnecessary temporary outputs. Review scripts before executing them. Preserve the existing NUSH2 privacy and content guards. Perform a repository-wide secret and privacy audit, including relevant Git history, before any additional public release.

## 11. Autonomous execution and automatic commits

Update `prompt/AUTONOMOUS_EXECUTION.md` and `prompt/COMMIT_AND_DEPLOYMENT.md`. The future session operates with minimal human intervention. The owner authorises routine development activities within the project: reading and editing project files; installing ordinary dependencies; running Rust and Python builds; running tests and verification; building WebAssembly; building scientific documents; creating and coordinating subagents; committing completed phases; pushing commits to the authorised development branch; opening a final pull request. Use documented Claude Code permission settings and actual GitHub credentials. Do not bypass security prompts, sandbox restrictions, branch protection or authentication.

After each successful phase: run tests; verify scientific requirements; update documentation; update `prompt/PROGRESS.md`; commit the milestone; push the authorised branch; continue to the next phase. Do not require approval between routine phases when the necessary permissions have already been granted. Do not force-push, delete branches or merge into `main`. At the end, open a PR to `main` and leave it unmerged.

## 12. Update the instruction files

Update all relevant existing files under `prompt/` (README, MASTER_PROMPT, CURRENT_STATE_AND_BASELINE, ENGINE_REFERENCE_PLANT, CARBON_AND_SUBSIDY, ECONOMIC_ASSUMPTIONS, FRONTEND_AND_VISUAL_DESIGN, DOCUMENTATION_AND_DERIVATIONS, TESTING_AND_VERIFICATION, GITHUB_REFERENCES, AUTONOMOUS_EXECUTION, COMMIT_AND_DEPLOYMENT, PROGRESS). Create `RAYSTORAGE_INTEGRATION.md` and `RAYSTORAGE_FILE_INDEX.md`. Preserve the existing instructions unless they conflict with verified evidence or accepted owner decisions.

## 13. Commit and push

Review the changes; verify all links and paths; check for contradictory instructions; stage only the intended `prompt/` files; commit `docs(prompt): integrate raystorage archive and autonomous execution`; push to the authorised NUSH2 branch; verify the remote commit. Do not start substantive engineering implementation in this session.

## 14. Final response

Report: branch; commit SHA; GitHub link to `prompt/`; files created and updated; most important raystorage discoveries; any unresolved blockers.

The completed instruction package must allow a future Claude Code session to continue NUS-H2 development using the current repository and raystorage as complementary sources of evidence.
