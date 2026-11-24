# raystorage integration

Archive: https://github.com/weewzr/raystorage (public since 2026-11-10). Pinned commit for this audit: `3a25c4a767b67faa3e797944ea9ac63f9265d4a1`. Layout: `NUSH2/<category>/<origin>/<relative path>` with categories derivations, simulations, process-data, PFD-versions, research-and-references, validation, and two origins: `session-b29325b8` (earlier NUS-H2 build, tour and site work) and `session-dee70430` (carbon, subsidy and IEAGHG reference work). `NUSH2/MANIFEST.csv` lists 3,683 files with sizes and SHA-256. Use targeted searches of the manifest and the paths in `RAYSTORAGE_FILE_INDEX.md`; never load the archive indiscriminately. Archived files are historical evidence, not authoritative code. The repository at the current commit always wins.

## 1. Most useful discoveries

1. **IEAGHG base-case reconstruction** (`process-data/session-dee70430/phase3b/ieaghg_streams.md` and `ieaghg_balances.py`). Source-traced extraction of streams 1 to 15, the PSA balance, the carbon breakdown, and the finding that the published 0.8091 kg/Nm3 already contains tail-gas combustion (85.8 percent feedstock carbon through the tail gas, 14.2 percent supplementary gas). The figures the owner listed match the reconstruction: 100,000 Nm3/h hydrogen, 8.994 t/h, natural gas 30.563 t/h (26.231 feedstock and 4.332 burner fuel), PSA tail gas 2,106.3 kmol/h and 60,658 kg/h, 11.500 MWe gross, 9.918 MWe exported. The future session must still verify each against the primary report PDF (pages in `ENGINE_REFERENCE_PLANT.md`) and must write the Rust checks independently of the Python script.
2. **Independent recomputation scripts** (`validation/session-dee70430/phase3b/spec_check.py`, `critic/critic_check*.py`): closed forms for the Case B shifts, gas by energy, the LCOH bridge, K.18 terms. Reuse as the model for independent tests.
3. **Verifier evidence** (`validation/session-dee70430/phase3/reverify2_carbon`, `fix1carbon/sympy_*_now.log`, `verify_subsidy/mutations.log`, `mut_gate.py`): the source of the nine minor findings, the seven stale carbon SymPy checks and the two stale subsidy checks (C19, C15b), and a working mutation harness.
4. **Decision wording** (`derivations/session-dee70430/phase3/carbon_plan.md`): the original text of D13 to D17 (end of the file). D18 to D23 are in `CARBON_AND_SUBSIDY.md`; they were never in a file of the archive.
5. **Frontend self-test history** (`validation/session-b29325b8/stagef/uiux/fix_tour_3/`): logs at 1024 x 600 and 1024 x 768, useful to diagnose the keyboard failure. Older `pfd.js`, `pfd.css`, `selftest.js` copies are superseded by the current repository (PR 3); diff only.
6. **Tectonic build log** (`derivations/session-b29325b8/deriv3/b3/out/tectonic_stdout.txt`): expected output and warnings of the derivation PDF build.

## 2. What the archive does not contain

The reference-plant specification and its critique (now in `prompt/reference/`), the Phase 2 results JSON, any file that matched the earlier privacy scan, images, Office and PDF files, paper PDFs and text extracted from them. The archive holds no current LaTeX sources worth preferring over `math/` in the repository.

## 3. Privacy findings (read before touching the archive)

The archive is public but was only text-scanned for names and identifiers. A quick scan at the pinned commit found:
- 689 files that are browser-profile data (for example `PFD-versions/session-b29325b8/stagef/uiux/audit_pfd/capture/profile/Default/...`, storage and service-worker logs). Do not copy, read or build from them.
- About 178 files containing the owner's Windows user-name path and about 147 files with email-like strings (many are code, not addresses), and 26 files containing the word password (probably tool text, unreviewed).
Recommended owner action (not for the session): remove the profile folders from raystorage in a new commit, and decide whether history needs a purge (GitHub Support for cached views). The session must not edit raystorage. Before any additional public release of NUSH2, run a repository-wide secret and privacy audit over the working tree and the relevant Git history.

## 4. Rules for using archived material

- Review any script before running it. Mutation scripts edit source files; restore with `git checkout` and confirm a clean tree.
- Never copy a historical Rust line number. Line numbers are generated last from the final code.
- Never paste archived code over current files. Port ideas into new tests, with the equation id and the source.
- Treat results in archived logs as historical. Rerun every test on the final code; record the new results, not the old ones.
- Distinguish for each reused item: copied (needs licence and privacy review), ported (re-implemented), or only consulted. Record this in `PROGRESS.md`.
- Pin the archive commit in `PROGRESS.md` for any later session; verify a file's SHA-256 against `MANIFEST.csv` before use.

## 5. Mapping to outstanding tasks

| Task | Archive material | Intended destination |
|---|---|---|
| Reference-plant registry and balances | ieaghg_streams.md, ieaghg_balances.py | `engine/data/parameters.toml`, `m12_reference_plant.rs`, derivation 03 |
| Independent closed-form tests | spec_check.py, critic_check*.py | `engine/plant/tests`, `engine/vv` |
| Decision log D13 to D17 | carbon_plan.md | `docs/ABATEMENT_SUBSIDY_LOG.md` |
| U6 test, nine findings, mutations | reverify2_carbon, mutations.log, mut_gate.py | `m11_carbon.rs`, `scripts/` |
| Seven plus two stale SymPy checks | sympy_*_now.log | `math/verify/*.py` |
| Definition C data checks | impl_subsidy/audit_table.py, recompute2.py | `scripts/` |
| Derivation PDF build | tectonic_stdout.txt | `math/` build notes |
| 1024 x 600 keyboard defect | fix_tour_3 selftest logs | `frontend/selftest.js`, `frontend/*` |
| Reference PFD with PSA tail gas | pfd_facts.json (checklist only) | `frontend/refpfd.js` (new) |
