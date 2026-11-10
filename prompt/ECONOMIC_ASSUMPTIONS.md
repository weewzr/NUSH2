# Workstream 3: economic assumptions and results

## 1. No premature headline claims

At commit `8dbfc6d` (before the reference-plant work) the committed headline is: central L5 about S$203/t for BOAK 25 y, and about S$283/t under the plant-built floor. A hand calculation from closed forms, with the engine not run, suggests that recalculating the steam and power lines from the IEAGHG balances would move central L5 to about S$221/t (gas line plus 7.7 S$/t, lost steam to zero, base power from a credit to a cost of about S$14.8M/y). This is an estimate. The engine decides. Do not publish it as a result, do not copy any of these figures into README, site or documents. After regeneration, reconcile every previously quoted headline (log, README, site, tour, documents, presentation) with the regenerated tables and record the old and new values in the log as a history, keeping withdrawn figures out of public site output.

Re-verify, under the final engine, and only then state: the floor reference cost, and the observation that definition A cannot reach S$100/t even at 100 percent. Carry no number forward unconfirmed.

## 2. Ledger facts

- Method 1 is the annual incremental ledger L0 to L5; S$/t equals ladder times 1e6 over net annual abatement. Method 2 is the DCF (5.5 y construction with a part-year bucket, replacement at 25 and 50 y, decommissioning at the horizon). Interest during construction factors differ slightly (ledger 1.19766, DCF 1.20023), kept for regression.
- Capital routes: FOAK, BOAK, tenth-of-a-kind from BOAK, tenth-of-a-kind from FOAK; lives 25 y and 60 y. The existing co-funding multiplier `rx_mult` (target S$96/t, BOAK only) stays but is not the grant model.
- Do not remove a cost item to reach the target.

## 3. Assumptions to document and review (assumption register)

Create a table in `docs/ABATEMENT_SUBSIDY_LOG.md` (or a section of `docs/FINAL_DESIGN_DECISIONS.md`) listing each of these with class, source, uncertainty and the output that depends on it.

1. **Island grant mirrored on the reference (D21).** Like-for-like headline. Reported sensitivity: grant on the NUS-H2 side only (windfall column); effect up to S$87/t at 100 percent for BOAK 25 y and the cost turns negative above about 70 percent.
2. **Electricity S$0.20/kWh (D22).** An author assumption, not a sourced Singapore export price. The IEAGHG report uses EUR 80/MWh both ways (S$0.186/kWh at the repository factor 2.322). Do not fabricate a Singapore tariff. Search for authoritative sources (energy market authority tariff documents); if none supports a value, keep the assumption labelled.
3. **Power island inside the island (D23).** Effect of removing it from the NUS-H2 side only: about S$10/t lower (hand estimate; engine decides).

## 4. Independence from INL: record precisely

Independently reconstructed from IEAGHG: baseline CO2 intensity, unabated gas energy, steam to neighbours (zero), exported power, PSA tail-gas balance, island capital (report TCR).

Still dependent on INL (state this in the docs and in the final report, and do not claim a fully independent reference plant): cryogenic carbon recovery f = 0.752 and hydrogen return r = 0.922 (calibrated, no source); the heat and power balance of the plant keeps the anchored captured CO2 from INL Case 6 (359,777 t/y against the plant's 363,277 t/y), through `CO2_CAPTURED_C6`, `NG_CASE6`, `ETA_SELEXOL`, `PHI_RECYCLE`; the reformer steam ratio, PSA recovery and the Selexol 0.99 specification carried from INL design data; comparison rows INL 9.21 and Cho 9.35 kg/kg. Quantify each dependence by perturbing the input (for example plus 30 percent) and reporting the change in abatement and in L5.

## 5. Shipping distance

1,000 km is an assumption. Named candidate sites: Kasawari (Sarawak) and Sunda Asri (West Java). Their coordinates are not verified; do not write coordinates from memory. Research the operators' announcements or a public gazetteer, record the source and date for each coordinate, then compute great-circle distance from Jurong Island with a stated sea-route factor (the previous estimate used 1.15, assumed) in a committed script that reads the coordinates from the registry. Until sourced, label every distance assumed. Sweep 200, 600, 1,000, 1,400, 2,000 km; keep the net abatement effect (a few kt/y over the range) in the sensitivity table. The cost ladder carries transport and storage as a lumped S$60/t today; document whether distance moves cost, and do not invent a distance-cost relation.

## 6. Currency and conventions to verify against IEAGHG

Currency and price year of the report's costs (EUR), the factor to S$ (2.322 in the repository), discount rate (8 percent in the report), plant life, availability (repository 0.85 against the report's 8,322 h/y), financing assumptions, annualisation. Record each in `reference/` notes or the log with page references. Do not mix EUR and S$.

## 7. Archive material

Finance and emission-factor research notes sit under `research-and-references/session-dee70430/phase2/`. They are research records with a verification status per source; read each source's status before using a number.
