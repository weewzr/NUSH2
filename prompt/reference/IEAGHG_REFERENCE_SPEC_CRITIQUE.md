> Completeness critique of the specification in this folder. Its items C01 to C11 are mandatory corrections.

# Critique of spec.md (completeness critic, read-only)

Scop<working folder>cratchpad/phase3b/spec.md` against the IEAGHG 2017-02 PDF and against the repository worktree the repository worktree at `8dbfc6d`. Nothing in the repository was edited. Every figure below was recomputed with `python -I` (`critic/critic_check.py`, `critic/critic_check2.py`). Label<working folder>P published, C calculated, A assumed, U unavailable.

Verdic<working folder>the Base Case data, the carbon argument and the closed forms in section B.7 are sound. The plan has three blocking defects, all of them conflicts with the owner's rules or with the spec's own gates, and six major gaps (places that read the old reference and are not covered). Severity ke<working folder>blocking means the work cannot pass the spec's own gates or breaks an owner rule if implemented as written; major means a covered-looking area has an uncovered site or a false statement that will cause a wrong or inconsistent result; minor means wording, labelling or a small numeric slip.

## 1. Independent verification (more than five numbers)

Inputs were typed from the PDF text or page images read by the critic, then recomputed. Page numbers are PDF pages.

| # | Quantity | Equation and substitution | Critic value | Spec value | Source read | Result |
|---|---|---|---|---|---|---|
| V1 | Stack CO2 and Table 2 intensity | 8,659.4 x 0.2123 x 44.0095 / 1000 | 80.90665 t/h, 0.809067 kg/Nm3, 8.995625 kg/kg, -0.0041 % vs 0.8091 | 80.9067, 0.80907, 8.99563, -0.0041 % | p44 stream 9, p16 Table 2 | agrees |
| V2 | PSA H2 recovery and mass closure | (4,461.5 + 29.1) / (6,596.9 x 0.7563); 69,711 - 60,658 - 8,994 - 59 | 0.900058; 0 kg/h | 90.006 %; 0 | p44, p45 | agrees (note C11) |
| V3 | Tail-gas LHV and fired duty | sum n8 y LHV / 3600 with 241.8, 283.0, 802.3 kJ/mol; plus 4.332 x 46.50 / 3.6 | 101.9496 MW; 157.9046 MW | 101.950; 157.905 | p44 stream 8, p16, `units.rs` L30-34 | agrees |
| V4 | Power and revenue | 11,500 - 1,216 - 19 - 347; 9.918 x 8,322 x 80 | 9,918 kW; 6,603,007.68 EUR/y | 9.918 MWe; 6,603,008 | p48, p229 (sheet 92, 8,322 h) | agrees |
| V5 | TPC sum and power-island pro rata | 97.212 + 20.124 + 53.616; 20.124 / 170.952 x 222.887 x 2.322 x CRF(25) x 1e6 / 557,767.4 | 170.952; 26.2376 M EUR; 10.232 S$/t | 170.952; 26.24; 10.2 | p222 sheet 85, p17 | agrees |
| V6 | Gas by energy and Case B what-if | 30.563 x 51.473 x 24 / (1,044 x 1.05505585); x = 0.999333 | 34.27763 MMSCFD; 34.25476; k = 5.126005 S$M per MMSCFD-y; GAS after -57.1866; BASE_POWER +14.7600; L5 +18.34 S$/t | 34.2776; 34.2548; 5.1260; about -57.19; +14.7600; about +18.3 | p255, `parameters.toml`, committed ledger | agrees |
| V7 | Heating value of the engine's INL gas | sum y HHV(kJ/mol) x (453.59237 / 379.49) / 1.05505585 with `NG_MOL_FRACTIONS` | 1,043.7 Btu/scf (engine 1,044); IEAGHG composition gives 1,050.24 vs spec 1,050.75 (0.05 %) | 1,044; 1,050.75 | `flowsheet.rs` L170-178, p255 | agrees, so conversion by energy is the like-for-like choice |
| V8 | LCOH bridge | 0.1144 / 0.08994 x 2.322; 8.527 c fuel x (15 / 12.59 - 1); +0.111; credit rebased to S$0.20 | 2.9535 + 0.4221 + 0.1107 - 0.0157 = 3.4706 S$/kg; today 3.618 is 4.2 % above | 3.47; today 3.618 | Annex II p272 (auditor), p18 | agrees; the 3.47 sum is the spec's, the auditor's sentence in section 5.6 of the utilities file is muddled |
| V9 | K.18 terms | 9.918 x 0.999333 x 7,446 x 0.402; 129.276 x 1.05505585 x 7,446 x 56.1 / 0.9 | 29.67 kt/y; 63.3 kt/y | 29.67; 63.3 | committed ledger | agrees |
| V10 | S14 and S15 heat register | LHV of S14, S15 from the committed `Integrated_Mass_Balance.csv` rows FS_B_REC_F; S03 + S21 + S15 - S14 | 94.775 MW; 16.567 MW; residual 1.5e-7 MW | 94.775; 16.567 | `results/Integrated_Mass_Balance.csv` L192-199 | agrees |
| V11 | Cross-case closure of Table 2 | extra fuel (5.597 - 4.332) t/h as CO2 per Nm3 vs (0.3870 + 0.4556 - 0.8091) | 0.03351 vs 0.03350 | not in spec | p16 | independent confirmation that 0.8091 already holds all tail-gas CO2 |
| V12 | Row counts | 4 x 2 x 2 x 2 x 11; 4 x 2 x 1 x 2 x 45 | 352; 720 | 352; 720 | `m10_subsidy.rs` L2060-2067 | agrees |

Quoted statements checked in the PDF tex<working folder>p6 ("all of the CO2 is emitted from the flue gas of the steam reformer", tail gas as "primary fuel"), p7 (8,322 h/y, excess HP steam converted to electricity), p8 (8 % discount), p34 (battery-limit tabl<working folder>no steam outlet), p35 (S/C 2.7-2.8; the critic gets 2.600 on stream 4 alone and 2.7005 with the derived 155.17 kmol/h), p37 (85 to 90 %), p38 (around 75 %), p45 (streams 13 to 15), p48 (rendered, all columns), p54 (82.63), p58 (12 MWe, 24 x 10^6 kcal/h, 8 kPa), p222, p229, p255, p256. No transcription error was found in section A.

## 2. Corrections

### Blocking

**C01. Editing existing registry text breaks the legacy gate G2 and the CI step.**
Evidenc<working folder>cripts/compare_results.py` header and `compare_subset` (L6-9, L140-160) require every cell of every reference row, text included, to be equal; the only escape is `--allow-cell`. `engine/vv/data/legacy_results/Final_Design_Basis.csv` carries `source` and `locator` columns and holds the rows `DISCOUNT` (L68, locator "PDF p12") and the six `SMR_ISLAND_*` rows (L186-191). `.github/workflows/ci.yml` (L42-52) and `site.yml` allow only the `conditions` cells of `PARAMS_COMPLETE` and `EXPORT_BYTE_IDENTICAL`. Spec B.4 (last paragraph) changes the `DISCOUNT` locator and the `SMR_ISLAND_*` source text.
Effec<working folder>ven cells differ in the legacy export, so G2 and the CI "Legacy basis against the snapshot" step fail. This also violates "legacy bit-identical".
Fi<working folder>do not edit the text of any existing registry row. Record the corrected `DISCOUNT` locator and the power-island overlap in the D13 entry, in `docs/derivation/03_reference_plant.md` and in the new `REF_` rows. If the owner wants the text corrected in place, that is a deliberate loosening of the legacy guard by named cells (`Final_Design_Basis.cs<working folder>DISCOUN<working folder>locator` and the six `SMR_ISLAND_*:source` cells) and needs its own decision and a line in the log; the spec must not do it silently.

**C02. Restating the gap limit against the gross cost weakens an existing tolerance and changes old columns.**
Evidenc<working folder>ubsidy.rs` L883 and `dcf.rs` L376 define `gap_rel = (dcf - l5) / l5` on the net cost; `gap_within` (`subsidy.rs` L595) tests it against `SUBSIDY_GAP_LIMIT` (decision D11, "as a share of the ledger cost"); `m10_subsidy.rs` L1927-1934 asserts it for `s <= 0.75` and `l5.lac_net > 0` on every definition; `export.rs` L4059 prints it as `gap_share_of_l5`. Spec D.4 redefines `gap_share_of_l5` as gap over gross cost for all definitions. Gross is at least net, so the same absolute gap passes more easily at every share above zero, for definitions A and B as well. Critic figures (C): at BOAK 60 y, f = 1 the spec's own example gives 0.0687 / 2.914 = 2.36 % of net but 0.0687 / 178.5 = 0.038 % of gross.
Effec<working folder>a tolerance is weakened, the old column `gap_share_of_l5` changes on A and B rows (so the G3 criterion "A and B rows equal on the old columns" fails), and the owner rule "no tolerance weakened" is broken.
Fi<working folder>leave `gap_rel`, `gap_within`, `SUBSIDY_GAP_LIMIT`, the `s <= 0.75` test and the column `gap_share_of_l5` exactly as they are for A, B and C. Add a new column `gap_share_of_gross_l5` and a new registry entry `SUBSIDY_GAP_LIMIT_GROSS` (0.01) as a separate check that applies to definition C over the full sweep and to any share above 0.75. Record it as a new part of D14, not as a restatement of D11.

**C03. The spec's own gates cannot be green between WP2/WP3 and WP7 because `results/` is regenerated last.**
Evidenc<working folder>m10_subsidy.rs` L474-505 (`eq_s_9b_zero_subsidy_reproduces_the_committed_ledger_and_dcf`, named in G1) builds `ctx_committed()` on the default basis and compares L5 with the committed `results/Economic_Ledger.csv` to 1e-6; L2060-2067 (`eq_s_8_the_committed_table_matches_the_engine`) does the same for the subsidy table; `wasm/src/tests.rs` L347 (`default_solve_matches_the_published_results`) and the CI step "Default basis against results" compare the live default with the committed tables. Spec B.1 sets `REF_BASIS` default 1 in WP0 and WP3 moves L5 by about +18 S$/t; Definition C joins `Definitio<working folder>:ALL` (WP0) and the export loop at `export.rs` L4152 then writes 720 more rows. Spec E.2 regenerates `results/*` only in WP7.
Effec<working folder>G1, G5 and G6 fail by construction in WP3 to WP6, so no package can show the "must stay green" evidence the spec demands.
Fi<working folder>hip `REF_BASIS = 0` in WP0 and let WP1 and WP3 tests set it to 1 explicitly. Move the regeneration of `results/*`, `build/*`, `engine/vv/data` expectations and the committed-table tests from WP7 into WP4 (the exporter owner), flip the default to 1 in that same WP4 commit, and let WP7 only re-run the guards. Until then list the expected-red tests in the spec so engineers do not edit them blind.

### Major

**C04. Direct readers of `Model.base_ieaghg` are not covered, which leaves two IEAGHG intensities in one results set or breaks the legacy basis.**
Evidence (all read at `8dbfc6d`): `anchored.rs` L861 (field), L1516 and L1519 (`A_ieaghg`, `base_ieaghg` keys of `case_summary`); `export.rs` L935 (label), L1032 (`RES_BASELINE_IEAGHG_KG_KG`, label says "the one value every file uses"), L1091 (comparator label), L3341 (Emissions_Ledger `IEAGHG_9p00` rows = `m.base_ieaghg * h2`); `sens.rs` L966 (`BASE_IEAGHG` corner value; the central `central_kg` at L958 is `baseline_kg_kg`) and L247 (Anchored only); `vv/src/golden.rs` L662 (Anchored). Spec B.5(a) changes only `baseline_kg_kg(Ieaghg)`.
Effec<working folder>if the field stays, the ledger central is 8.995625 while `A_ieaghg`, `RES_BASELINE_IEAGHG_KG_KG`, the BASELINE group IEAGHG corner and the Emissions_Ledger rows stay at 8.995997, and "the IEAGHG corner is the central point" is false by 4.1e-5. If the field is overwritten in `Mode<working folder>:build`, `export.rs` L3341 and `case_summary` change on the legacy basis too, which breaks bit-identity.
Fi<working folder>name one accessor, `baseline_kg_kg(Ieaghg)`, that returns the reconstructed value only when `carbon_params.independent()` and `REF_BASIS = 1`, and the old ratio otherwise. Replace the Plant-path reads at `export.rs` L935, L1032, L1091, L3341, `sens.rs` L966 and the `case_summary` keys by it; keep L247 and `golden.rs` L662 on the old field. Add a test that no non-Anchored code reads `base_ieaghg` directly and that the two printed values agree.

**C05. "No existing registry value changes" is false, and G3 lists the wrong pass criterion.**
Evidenc<working folder>pec D.4 sets `SUBSIDY_DEFAULT_DEFINITION` to code 2, but the registry value is 0.0 today (`parameters.toml` L2675-2676) and `results/Subsidy_Scenarios.csv` marks 88 A rows `default_case = true`. After the change those 88 cells become false. Spec B.6 states no existing value changes and G3 requires A and B rows equal on all old columns.
Fi<working folder>tate the one exception, add `default_case` and the registry row to the G3 allowed-difference list by cell (as the CI already does for `conditions`), and let the CLI flag `--ref-basis historical` keep the new default definition so only those named cells differ.

**C06. Existing tests and text guards trip on the new file and on the spec's own vocabulary.**
Evidenc<working folder>ngine/plant/tests/no_internal_pointers.rs` L151 asserts exactly 8 CSVs in `results/`; adding `Reference_Plant.csv` fails it (also the doc comment in `engine/cli/src/main.rs` L3). The same test scans `parameters.toml` and every results CSV for the phrase "owner decision", for words shaped like `question 3` or `s6` (`PATTERNS` L8, `working_key` L47), and for any `.py` path not under `scripts/` (`private_files` L72). `engine/vv/tests/concept_checks.rs` L288 (`no_published_text_claims_validated_status`) rejects "validated against", "is validated" and similar phrases in `README.md`, `docs/` and `frontend/`.
Effec<working folder>the new CSV, `frontend/units_ref.json` and `docs/derivation/03_reference_plant.md` fail CI if they reuse the spec's wording ("validates the reconstruction", "open question question 3", `math/verify/sympy_refplant.py`).
Fi<working folder>add to WP4 the change of the count to 9, and add to section 0 of the spe<working folder>never copy question 1 to question 3, "owner decision" or a `.py` path outside `scripts/` into registry source strings or results; avoid the phrases listed in `concept_checks.rs` in docs and frontend text (write "checked against" and "reproduces").

**C07. Several sites that encode the old meaning are not in the work packages.**
Evidenc<working folder>m11_carbon.rs` L1373-1382 asserts `lost_steam > 0.0 && baseline_power < 0.0` on the plant balance (the spec lists L1083-1163, L1415-1580, L1624-1652 but not this test); `wasm/src/solve.rs` L2135-2136 prints the labels "Lost steam export" and "Baseline purchased power avoided" (the sign of the second flips); `registry.rs` L635 exposes `ELECTRICITY_PRICE` to the live page, where a higher price now raises the cost; `vv/src/harness.rs` L685 carries the note "price the steam export and the purchased power of INL's conventional plant"; `emissions.rs` L154-196 (`backup`, `backup_heater`) keeps the INL unabated plant (`co2_conv`, `ng_conv`) for outage hours and the spec is silent.
Fi<working folder>add these sites to WP3 (test), WP6 (labels, UI text) and WP4 (harness note). State in D13 that the outage rows stay on the INL basis on purpose and label them so.

**C08. The tail-gas capture bound in A.6 is loose, and the evidence in the same section shows it.**
Evidenc<working folder>A.6 says a capture step on the tail gas "can remove at most A (0.694 kg/Nm3, 85.8 %)". A is CO2 plus the carbon of CO and CH4 that only becomes CO2 in the burners. A CO2 wash upstream of the burners can remove at most A1, 0.4721 kg/Nm3 (58.4 % of the stack, <working folder>0.47209 / 0.80907). Case 2A removes 0.4556 kg/Nm3 (p16), which is 96.5 % of A1 and 65.6 % of A. Table 2 closes across case<working folder>0.3870 + 0.4556 - 0.8091 = 0.0335 equals the CO2 of the extra 1.265 t/h of fuel in Case 2A (0.03351, V11).
Fi<working folder>write the bound as A1 for a step that removes CO2 already in the gas, and as A only for a step that also converts or removes the CO and CH4 carbon before combustion. Make the test assert the tighter bound for the CO2-wash case.

**C09. `psa_tail(recovery)` and the constant-fired-duty form of N.R.5 are unsafe for any sensitivity.**
Evidenc<working folder>N.R.2 keeps the published feed (stream 7) fixed, so changing R changes the H2 product (the 100,000 Nm3/h basis moves); N.R.5 holds the fired duty per kg of H2 constant, but the reformer duty and the steam raised both change with R.
Fi<working folder>mark both as test-only in the API, forbid wiring them to a sensitivity group or to the headline, and state in the doc that any recovery sensitivity must scale the whole plant to constant H2 first.

### Minor

**C10. K_I label.** 517.551 S$M is the registry value 222.89 x 2.322 (`dcf.rs` test L527 asserts exactly that); 222.887 x 2.322 gives 517.544 (C). Change the label in D.1 and D.3 to "222.89 M EUR (registry, Table 3) x 2.322".

**C11. Recovery range.** The derived recovery 0.900058 exceeds the registry `high` 0.90 by 5.8e-5. The validation row hides this behind a literal 0.901. Store the check as "printed range 85 to 90 % to the nearest percent, so up to 90.5 %" or set `high` to the printed 0.90 and give the row an explicit rounding note.

**C12. New tolerances carry no stated derivation.** The blocking carbon tolerance is 0.2 % = 3.68 kmol C/h; the critic's crude worst-case rounding band is 1.79 kmol C/h (0.097 %, C). The others (0.1 %, 0.5 kmol/h, 5e-4) are about twice their rounding band. Label them A in B.2 and state the band beside each.

**C13. B.7 closed forms divide by the old A_net.** The baseline falls 24.9 t/y, so the new A_net is 557,742.5 and L5 is 221.531 instead of 221.521 (C). The difference is 0.01 S$/t, inside the 0.05 S$/t stop rule, but the stop rule should say which A_net it divides by.

**C14. Power-island switch is capital only.** Removing 26.24 M EUR of TCR leaves the island's share of maintenance, insurance and local taxes on the NUS-H2 sid<working folder>0.1177 x (2,564,280 + 1,709,520) = 503,100 EUR/y (Table 4, p18), 1.168 S$M/y, 2.09 S$/t (C). State whether `SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND` also removes it; direct labour per unit is U.

**C15. Utility-sheet revision labels.** p48 prints "REV. 0, April 2015" in the header and "FWI-110/15A Rev.1" in the footer. The spec calls the sheet Rev. 0 in A.2 row 4 and Rev. 1 in R13. Do not explain the 0.33 % gap between 45.9 and 46.053 t/h by revision; state only that both are printed.

**C16. S$0.186/kWh.** 80 x 2.322 / 1000 = 0.18576. The spec's +13.73 S$M/y uses 0.186 (13.727); 0.18576 gives 13.709. State the rounding.

**C17. The LCOH bridge shares assumptions with the engine.** It uses the engine's gas price, availability and FX factor, so it detects gross errors of the reference only. Today's 3.618 is 4.2 % above 3.4706 and fails the 3e-2 tolerance; the after-change value 3.465 (3.618 minus (4.293 + 21.171 - 15.234) / 66.92 = 0.153, C) passes. Say this in the row text, and state that 3e-2 is an A.

**C18. System boundary of L5.** The cost ledger now carries the reference's export revenue (BASE_POWER +14.76 S$M/y) while A_net excludes the matching K.18 grid credit (-29.67 kt/y, 5.3 % of A_net, C). The same structure existed before with the opposite sign. Say in D13 which denominator the system lines use, and report one labelled row on the K.18-adjusted abatement.

**C19. Scaling basis.** `smr_island(s)` scales with s / s_b (`dcf.rs` L159-165), so at Case B the island is the full report island, while reference flows scale with x = 0.999333. The 0.07 % mismatch is harmless; document it.

**C20. K'_ref = 0.** `COND_CO2_VENT` has `low = 0.0` (`parameters.toml` L2979-2985). The identity still closes, but the reported reference carbon changes when the sensitivity leaves v = 1. Note it.

**C21. Documents outside the package list.** `math/sections/K_emissions.tex` and `L_economics.tex` (cards K.2, L.16, L.24), `math/frame_seed.csv` L42, `math/frame_assumptions.csv` L14 and `math/generated/values.tex` describe the INL pair. No gate reads them, but WP7 should either update them or label them legacy.

## 3. Owner rules checklist

| Rule | Status after corrections |
|---|---|
| Never invent values | Holds. Every assumed value is labelled (22.414, 4.1868, 21 % O2, K'_ref = 0, kappa = 1). Tolerances need the A label (C12). |
| Legacy bit-identical | Holds only after C01, C04 and C05 are applied. |
| No tolerance weakened | Holds only after C02. New tolerances are additions. |
| Keep PSA tail gas, lost steam and base power in the primary result | Hold<working folder>tail gas stays inside the stack CO2, LOST_STEAM and BASE_POWER stay as ledger and DCF lines with the same ids, the headline includes both. LOST_STEAM is zero in the headline because the published balance exports no steam (p34, p48). |
| Double counting | Stack CO2 versus an explicit tail-gas ter<working folder>closed (A.6, V11). Island overla<working folder>open question 3 stays. No new path found beyond C08 and C18. |
| Units | Volume versus energy (A.2 row 5), LHV versus HHV (V7), kg versus Nm3 (R03), t/h versus kt/y (V9) and EUR versus S$ (C10, C16) were all rechecked. No further slip. |

## 4. Not verifiable here

R03 (22.414) rests on one cross-check (p152), R40 (gauge or absolute pressure) is unavailable, the O balance residual (-0.22 %) depends on the assumed 21 % O2, and the Case B figures in B.7 are closed forms that need the engine run to confirm.
