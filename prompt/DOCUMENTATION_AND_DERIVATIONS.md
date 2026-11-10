# Workstream 5: derivations and documents

## 1. Derivation 03 (reference plant)

Write `docs/derivation/03_reference_plant.md` in the layout of `01_carbon_and_abatement.md` and `02_cost_and_subsidy.md`, and add it to `PUBLIC_MD` in `engine/plant/tests/no_internal_pointers.rs`.

- Start with a Reference Frame: every variable (symbol, definition, units), every equation (N.R.1 to N.R.17), assumptions and parameter ranges.
- Then for each equation, in order: what it calculates; where it comes from; its variables and units; the step-by-step derivation for an advanced secondary-school reader; a numerical substitution with the published IEAGHG values; assumptions and limits; the Rust implementation (function names); the validation test (test names).
- Content: physical definitions; material and energy balances (element balances C, H, O, N); reference-case assumptions; PSA tail-gas treatment; steam and power accounting; engineering boundaries (battery limit, no steam export); source references with PDF page and table; limitations (no furnace efficiency, no published tail-gas LHV, stream 14 steam unexplained by the H and O balances, combustion air derived).
- It must show the derivation and its link to the implemented equations. It must not restate code.
- SymPy check script for the module (path under `math/verify/`, following the text guards), with dimension checks of units.
- Also update derivations 01 and 02 (Rust and test lines, definition C blocks D.8 of the specification, the new decomposition with the reconstructed reference) and the open-issues section.

## 2. Derivation book and PDF

- Update `math/` sections, `eq_register.csv` (every new equation id with a matching LaTeX section), `frame_*.csv` as the existing tooling requires. Add rows only with matching sections.
- Compile `math/NUSH2_First_Principles_Derivations.pdf` with Tectonic. Tectonic 0.17 was used before; install it into a local tools folder (do not install globally), fetch its bundle once, and record the version and command in this folder's `PROGRESS.md`. The build must be reproducible from the repository.
- Units in LaTeX use `\mathrm{...}`.

## 3. Equation-to-Rust table

Final deliverable table: Equation | Pseudocode | Actual Rust with file and line. Generate it with a script from the code and the register, then verify every cited line number against the final code (a test or script that opens each file and checks that the named function is on the cited line). Line numbers are added last, once the code is stable.

## 4. Documents to regenerate or update

README (numerical claims derive from the final regenerated exports via `build_numbers.py` tokens, none hand-typed), `docs/ABATEMENT_SUBSIDY_LOG.md` (decisions, phases, changed-cell groups, history of withdrawn figures), `docs/ABATEMENT_SUBSIDY_REFERENCES.md` (verified references only: title, issuer, year, DOI or URL, page and table, parameter supported; no invented citations), `docs/FINAL_DESIGN_DECISIONS.md`, `docs/VALIDATION_MATRIX.md` (new rows, status words that satisfy `concept_checks.rs`: write "checked against" and "reproduces", never "validated against"), `docs/UI_UX_ASSESSMENT.md`, the report and deck inputs, and `THIRD_PARTY_NOTICES.md` if any dependency changes.

## 5. Source and style rules

References: primary sources only, each marked fetched_and_read, search_snippet_only or not_verified; an unread source supports no parameter. Text: no em dashes, no lists of exactly three items for rhetoric, no "it is not A, it is B", only the owner named, no course or schedule text, no local paths. Run `python scripts/check_clean.py` and the Rust test `no_internal_pointers` on every documentation commit.

## 6. Presentation and report

Regenerate numbers from the final tokens. The 12 typed context constants and the Figure 2 legend size are listed in `FRONTEND_AND_VISUAL_DESIGN.md` section 6. Office files stay out of Git. Keep the existing deliverable conventions (flat deliverables where the repository already uses them).

## 7. Archive material

A Tectonic build log is at `derivations/session-b29325b8/deriv3/b3/out/tectonic_stdout.txt`. Archived equation registers are older copies; use `math/eq_register.csv`. Never copy a historical Rust line number.
