# Workstream 4: frontend and visual design

Keep the existing architecture: vanilla JavaScript, SVG and CSS in `frontend/`, the Rust engine compiled to WebAssembly (`engine/wasm`), and the existing engine reply contract. No framework migration (no React) without a documented technical justification and an owner decision. No engineering calculation moves into JavaScript. Every displayed number comes from the live engine reply or from a table the exporter wrote.

Existing behaviour to keep: interactive PFD with unit panels, guided tour, SVG charts, scenario controls, responsive layouts, reduced-motion mode, keyboard access, the accessible equipment and stream list (PR 3), `scripts/selftest.py`, the content guard and the external-host guard (no request to any outside host).

## 1. Reference PFD (WP5)

Add a conventional-reference view (the IEAGHG base case) beside the existing NUS-H2 PFD, for example a tab or switch, built in new files (`frontend/refpfd.js`, `refpfd.css`, `units_ref.json`) with hooks only in `pfd.js` and `index.html`. Draw and label:

- Reformer, shift, gas cooling, PSA, hydrogen product, PSA tail-gas outlet, tail-gas route to the furnace burners as primary fuel, supplementary natural gas, combustion air, flue gas and stack CO2, waste-heat recovery, steam generation to the internal turbine, electricity generation, consumption and export. Show that no steam crosses the battery limit.
- Distinct line styles for process gas, fuel gas, steam, power and flue gas, with a legend. Correct flow directions and arrowheads. Correct plant boundary.
- Stream ids match the engine's stream ids and the `Reference_Plant.csv` rows (published streams 1 to 15 plus derived rows). A stream table beside the drawing shows flow, temperature, pressure, composition and energy, with unavailable values marked unavailable (no placeholders and no invented numbers).
- Values come from the live reply (add a `reference` section to the reply in `engine/wasm/src/solve.rs`; keep every existing key). Do not hard-code numbers in the SVG.
- The NUS-H2 PFD also shows its own PSA reject (S14), the cryogenic unit, the recycle and the nitrogen-rich cut explicitly, never as an unaccounted waste stream. Fix remaining drawing discrepancies from `docs/UI_UX_ASSESSMENT.md` and `docs/FINAL_DESIGN_DECISIONS.md` (flow directions, equipment and stream labels, consistency with the calculation).

## 2. Scenario comparison page (WP5b)

New view with these features (original brief section 11):

- Selectors: capital route (FOAK, BOAK, tenth-of-a-kind from BOAK, from FOAK), life (25 y, 60 y), subsidy share (0 to 100 percent, 5 percent steps), eligibility definition (C primary; B and A shown as labelled legacy or secondary), grant timing (progressive, at close), discount rate.
- Shows baseline cost, subsidised private cost, government contribution, private investment, annual abatement (labelled upper bound), L5, DCF, and the verdict.
- Charts across the share range with a permanently visible S$100/t line (labelled), a marker at the break-even share, L5 and DCF drawn as distinct series, definition C and definition B distinguishable by line style and legend and not by colour alone, and the central versus sensitivity range (island treatment, electricity price, shipping distance).
- Cases that meet or miss the target are marked in text and shape. A pass that relies on the upper-bound abatement says conditional. Print "hypothetical government risk-sharing policy" on the page. Show the assumptions (D21 to D23) in a collapsible block.
- Data: the live engine reply for the selected case; the committed `Subsidy_Scenarios.csv` is the regression reference. The page must not compute costs itself.
- Desktop and mobile layouts, keyboard operation of every control, visible focus, readable contrast, `prefers-reduced-motion` respected, chart text alternatives and a data table equivalent for each chart.

## 3. Labels, tour and live reply

Update equipment names, stream labels, tour descriptions, live-reply KPI labels (the headline rows say "generic reference intensity" today and must now name the reconstructed IEAGHG base case), scenario-dependent captions and keyboard interactions. The known label defects: `solve.rs` labels "Lost steam export" and "Baseline purchased power avoided" (the sign of the second flips with the new meaning), `views.js` and `registry.rs` stale labels, `tour.js` steps that describe the INL headline. Keep the tour audit and the required KPI ids passing. When the reference PFD arrives, add tour steps for it.

## 4. Site scripts and withdrawn figures

`scripts/build_numbers.py`, `build_site.py` and `check_site.py` still describe the INL headline and do not show the floor, the without-lines rows or the reference rows. Update them to read the regenerated tables. Add every superseded headline figure to the withdrawn-figures list in `check_site.py` so public site output cannot print it, and keep a record of the withdrawn results in internal documentation (the log). Never silently alter historical data.

## 5. Known defects to fix

- Keyboard self-test fails at 1024 x 600.
- Previous-to-step-1 card heading visibility at 1440 x 900.
- Any regression from the reference PFD or the new page.

Retain the good behaviour at 1440 x 900, 375 x 812 (mobile), reduced motion, keyboard accessibility, live-reply values and no external requests.

## 6. Presentation and drawing deliverables (WP10)

The deck and report are legacy-basis deliverables until regenerated. Correct the 12 typed context constants in the deck (they must come from the final tokens, not typed text) and the Figure 2 legend size on slides 7 and 26 (the legend text is 8.4 pt and must reach the deck's minimum). Align the drawn PFD in the deck and report with the actual reference-plant calculation. Do not publish intermediate or unverified figures as final. Office and PDF working copies are never committed.

## 7. Archive material

Self-test logs at 1024 x 600 and 1024 x 768 are under `validation/session-b29325b8/stagef/uiux/fix_tour_3/`. Older PFD files in `PFD-versions/` are superseded by the repository after PR 3. Never replace the current frontend with an archived version; diff only.
