> Working copy of the specification written before this package. Decision ids D13 and D14 inside it correspond to D18, D19 and D20 of `prompt/CARBON_AND_SUBSIDY.md`; its section F defaults are D21 to D23. Where `IEAGHG_REFERENCE_SPEC_CRITIQUE.md` disagrees, the critique wins.

# NUS-H2 implementation specificatio<working folder>IEAGHG Base Case reference plant and total-CAPEX subsidy (definition C)

Repository worktree the repository worktree, branch `feature/independent-abatement-subsidy`, commit `8dbfc6d`. Written 2026-11-07 by the lead engineer from four read-only audits plus my own re-reading of the report pages and re-computation of every number in section A. No repository file was changed by the audit or by this spec.

Labels on every numbe<working folder>**P** published (read from the source, with page), **C** calculated (equation and substitution shown, reproduced by `working folder/spec_check.py`, run with `python -I`), **A** assumed (a convention or constant the report does not state), **U** unavailable (not in the report and not derivable).

Page references give the PDF page of `2017-02 Techno - Economic Evaluation of SMR Based Standalone (Merchant) Hydrogen Plant with CCS.pdf` and the printed sheet or table. The overview pages, the heat and material balance, the utility sheet and the equipment lists carry no printed page number, so they are cited by PDF page plus table, sheet or stream number. Table numbers repeat across parts of the report, so "Overview Table N" means PDF p12 to p18.

The numbers in section D that come from the committed tables (BOAK 25 y, S$203.18/t and so on) belong to the reference of commit `8dbfc6d`. They move after WP3 (section E). Tests must derive such numbers from formulas, never from literals copied from this file.

---

## 0. Rules for the engineers who build this

1. Read `engine/plant/src/economics.rs` (lines 290 to 435), `abatement.rs` (lines 830 to 990), `emissions.rs` (lines 76 to 152), `dcf.rs` (lines 150 to 290) and `subsidy.rs` (lines 28 to 200 and 370 to 548) before editing. Line numbers are as at `8dbfc6d`.
2. Repository text rules. Source strings in `parameters.toml` and in `results/` cite primary sources, decision ids or public repository paths only (`engine/plant/tests/no_internal_pointers.rs`). Public Markdown files are listed in `PUBLIC_MD` there; a new derivation file must be added to that list. `python scripts/check_clean.py` must stay clea<working folder>no local absolute paths, no working-note names, no personal or course identifiers. Prose in docs follows the project styl<working folder>no em dashes, no triples used for rhetoric, no "it is not A, it is B" phrasing.
3. Engine constants that this spec relies on (all in `engine/props/src/units.rs` unless stated): `MW_CO2` 44.0095 kg/kmol, `LHV_H2` 241.8, `LHV_CO` 283.0, `LHV_CH4` 802.3 kJ/mol, `SCF_LBMOL` 379.49, `MMBTU_GJ` 1.05505585, `MW_H2` 2.01588. `NG_HHV` 1044 Btu/scf, `GAS_CENTRAL` S$15/GJ (HHV), `ELECTRICITY_PRICE` S$0.20/kWh, `AVAILABILITY` 0.85 (hours 7446, days 310.25) are registry values. One new constant is neede<working folder>KCAL_KJ = 4.1868` (International Table calorie), added to `units.rs`.
4. Every new registry entry must be read by `Mode<working folder>:build` or the `PARAMS_COMPLETE` validation row fails (`engine/vv/src/harness.rs` lines 330 to 345). The registry parser accepts scalar fields only (no arrays), so a stream table is stored as one entry per number.
5. Decision id<working folder>this work is recorded as **D13** (conventional reference plant, IEAGHG Base Case reconstruction) and **D14** (total-CAPEX subsidy definition C). D12 is the last id in `docs/ABATEMENT_SUBSIDY_LOG.md`.

---

## A. Verified IEAGHG Base Case data

### A.1 Answers in short

1. All twelve owner target values match the report (table A.3, rows R01 to R16). The report prints no per-kg figur<working folder>8.996 kg/kg H2 is calculated, 0.8091 x 100,000 / 8,994 = 8.995997 (C).
2. **The published 0.8091 kg/Nm3 H2 already contains the combustion of the PSA tail gas** and the CO2 that passes through the tail gas unburnt. It is the whole stack CO2 (flue gas, stream 9). A tail-gas combustion term must never be added on top. Section A.6 gives the carbon breakdown.
3. Stream 9 reproduces Table 2: 8,659.4 kmol/h x 0.2123 x 44.0095 = 80.9067 t/h = 0.80907 kg/Nm3 = 8.99563 kg/kg (C), residual -0.0041 % against 0.8091. The rounding of the four-decimal CO2 fraction alone is +/-0.024 %.
4. The Base Case exports **no steam** across the battery limit. Electricity (9.918 MWe net) is the only export besides hydrogen. Stream 10 ("HP steam export", 46,053 kg/h) is the feed of the internal condensing turbine.
5. Tail-gas LHV, furnace duty split and combustion air are not published. The first two are derived (A.5) with the engine's LHV constants. A furnace thermal efficiency and every convective-section duty are **unavailable**; the design never needs them.

### A.2 Disagreements between the auditors and how I resolved them

| # | Disagreement | Source reading | Resolution |
|---|---|---|---|
| 1 | Stack CO2 80.906 t/h (streams audit) against 80.908 t/h (utilities and repo audits) | Both are 8,659.4 x 0.2123 x M. The streams audit used 44.009, the others 44.01. My ru<working folder>80.9057 (44.009), 80.9067 (44.0095, the engine constant), 80.9076 (44.01) | A molar-mass convention, no data conflict. The spec uses the engine constant 44.0095: 80.9067 t/h, 0.80907 kg/Nm3, 8.99563 kg/kg. |
| 2 | Repo audi<working folder>treams 13, 14 and 15 "have no conditions in the table" | I rendered PDF p45 (image read): stream 13 H2 recycle 40 C, 2.51 MPa, 29.1 kmol/h, 59 kg/h; stream 14 HP steam to process 400 C, 4.29 MPa, 5,290.1 kmol/h, 95,301 kg/h; stream 15 LP steam to deaerator 177 C, 0.44 MPa, 30.0 kmol/h, 540 kg/h | The repo audit was wrong. All fifteen streams are published (PDF p44 streams 1 to 12, PDF p45 streams 13 to 15). |
| 3 | Repo audi<working folder>the IEAGHG Base Case has an HP steam export (stream 10)" so LOST_STEAM must be re-derived from a steam duty. Utilities audi<working folder>no steam is exported | PDF p34 (Overall Block Flow Diagram, Sheet 01 of 01): hydrogen unit sends HP steam to the COGEN unit inside the battery limit; outlets are flue gas, hydrogen, power, waste water. PDF p48 utility sheet TOTAL ro<working folder>LP, MP and HP steam all 0 | Utilities audit is right. Steam to a neighbour is **0 (P)**. The reference's steam value reaches the ledger as electricity. No steam-table duty is needed. |
| 4 | Steam to the turbine 45.9 t/h (utility sheet, Rev. 0, April 2015) against 46.053 t/h (stream 10, H and M balance Rev. 1, June 2015) | Both read on the images (p48, p44) | Both published, 0.33 % apart. The balance uses stream 10. The utility sheet value is a cross-check row. |
| 5 | Reference gas 34.066 MMSCFD (repo audit, by volume with `SCF_LBMOL`) against what the GAS line should use | IEAGHG gas HHV is 51.473 MJ/kg x 18.019 kg/kmol, which is 1,050.75 Btu/scf at `SCF_LBMOL` (C); the engine prices at 1,044 Btu/scf | Convert by energ<working folder>30.563 t/h x 51.473 GJ/t = 1,573.17 GJ/h = 436.99 MW HHV, which is 34.2776 MMSCFD in the engine's gas unit (C). By volume it would be 34.0663. The two differ by 0.62 %. |
| 6 | Radiant duty "60.9 %" (streams audit) against "24.3 %" (utilities audit) | 96.099 MW / 157.905 MW fired = 60.86 %; 96.099 MW / 394.772 MW plant input = 24.34 % (C) | Different denominators, both correct. The spec reports both and treats the radiant duty as an indicator only. |
| 7 | PSA H2 recovery 90.01 % against 89.42 % | (4,461.5 + 29.1) / (6,596.9 x 0.7563) = 90.006 %; 4,461.5 / (6,596.9 x 0.7563) = 89.42 % (C) | Stream 13 is PSA product recompressed to the front of the plant, so the recovery is 90.006 % (product plus recycle). The text range is "typically 85 to 90 %" (Sheet 14, PDF p37). |
| 8 | Repo audit says the floor "12.7 %" and the fuel gas "14.2 %" are "not an identity" | The 14.2 % is the supplementary NG share of NG carbon (260.71 / 1,839.96 = 14.17 %), an identity of the carbon balance. The 12.7 % compares a helium-heated flowsheet with a fired plant | No conflict. The floor stays a lower-bound comparison row. |
| 9 | Stream 14 (95,301 kg/h) is 19.9 t/h larger than the reformer steam implied by H and O balances (streams audit); the utilities audit uses 95.3 t/h as process need | The streams audit re-run gives w2 from H 155.17 and from O 153.70 kmol/h, net reaction water 43.16 t/h against 44.2 t/h on the utility sheet | Unresolved in the report. No carbon, energy or cost output depends on it. It stays in the notes of the stream table. |

### A.3 Verified parameter table (Base Case)

PDF page and printed table or sheet are given for every row. "Overview Table 2" is on PDF p16.

| ID | Quantity | Value | Unit | PDF page | Printed table or sheet | Label |
|---|---|---|---|---|---|---|
| R01 | H2 product, volume | 100,000 | Nm3/h | 16 (also 46) | Overview Table 2; Sheet 20 of 112 | P |
| R02 | H2 product, mass | 8.994 (8,994 kg/h; stream 12 4,461.5 kmol/h) | t/h | 16; 44 | Overview Table 2; stream 12 | P |
| R03 | Nm3 convention | 22.414 (4,461.5 x 22.414 = 100,000.06) | Nm3/kmol | none | not defined in the pages read; confirmed by the Case 2A tail-gas compressor flow 47,204 Nm3/h at MW 28.8 (PDF p152, equipment list sheet 4 of 5) against 47,211 from stream 8 (-0.014 %) | A |
| R04 | NG to feedstock | 26.231 (stream 2: 1,455.8 kmol/h, 26,231 kg/h, 128 C, 3.71 MPa) | t/h | 16; 44 | Overview Table 2; stream 2 | P |
| R05 | NG to fuel (supplementary) | 4.332 (stream 3: 240.4 kmol/h, 4,332 kg/h, 121 C, 0.50 MPa) | t/h | 16; 44 | Overview Table 2; stream 3 | P |
| R06 | NG total | 30.563 (stream 1: 1,696.6 kmol/h, 30,563 kg/h, 9 C, 7.00 MPa) | t/h | 46; 44; 34 | Sheet 20; stream 1; block flow diagram | P (26.231 + 4.332 = 30.563 C) |
| R07 | NG composition, mol/mol | CO2 0.0200, N2 0.0089, CH4 0.8900, C2H6 0.0700, C3H8 0.0100, n-C4H10 0.0010, n-C5H12 0.0001 (sum 1.0000); H2S 5 ppmv design | - | 44; 255 | streams 1 to 3; Annex I Table 1 (Sheet 9 of 24) | P |
| R08 | NG LHV | 46.50 (Overview Table 2); 46.502 (Annex) | MJ/kg | 16; 255 | Overview Table 2; Annex I Table 1 | P |
| R09 | NG HHV | 51.473 | MJ/kg | 255 | Annex I Table 1 (Sheet 9 of 24) | P |
| R10 | Total energy input A | 394.77 (30.563 x 46.50 / 3.6 = 394.772) | MW | 16 | Overview Table 2 | P (check C) |
| R11 | H2 LHV, HHV | 119.96, 141.88 | MJ/kg | 256 | Annex I Table 2 (Sheet 10 of 24) | P |
| R12 | H2 energy B | 299.70 (8.994 x 119.96 / 3.6 = 299.700) | MW | 16 | Overview Table 2 | P (check C) |
| R13 | Gross power, H2 plant load, auxiliaries | 11.500, 1.216, 0.366 (= 19 kW power island + 347 kW utilities and BoP) | MWe | 16; 48 | Overview Table 2; utility sheet FWI-110/15A Rev.1 | P |
| R14 | Net export power | 9.918 (11.500 - 1.216 - 0.366 = 9.918; utility sheet 1,216 + 19 + 347 - 11,500 = -9,918 kW) | MWe | 16; 46; 34 | Overview Table 2; Sheet 20; block flow diagram | P (check C, exact) |
| R15 | Specific CO2 emissions | 0.8091 | kg/Nm3 H2 | 16; 46 | Overview Table 2; Sheet 20 (prints t/1000 Nm3) | P |
| R16 | Specific CO2 per kg H2 | 8.995997 (0.8091 x 100,000 / 8,994) | kg/kg | none | not printed | C |
| R17 | Specific NG consumption, feedstock, fuel, total | 12.197, 2.014, 14.212 | MJ/Nm3 H2 | 16 | Overview Table 2 | P (<working folder>12.197, 2.014, 14.211) |
| R18 | PSA feed (stream 7) | 35 C, 2.58 MPa, 6,596.9 kmol/h, 69,711 kg/h; CO2 0.1627, CO 0.0464, H2 0.7563, N2 0.0020, CH4 0.0302, H2O 0.0024 | - | 44 | stream 7 | P |
| R19 | PSA tail gas (stream 8) | 28 C, 0.13 MPa, 2,106.3 kmol/h, 60,658 kg/h; CO2 0.5095, CO 0.1454, H2 0.2369, N2 0.0062, CH4 0.0945, H2O 0.0076 (sum 1.0001) | - | 44 | stream 8 | P |
| R20 | PSA H2 recovery | "typically 85 to 90 %"; purity 99.9+ % (stream 12 printed 0.9999+) | - | 37; 6 | Sheet 14; Overview basis of design | P (range); 90.006 % C |
| R21 | Tail-gas use | sent to the SMR burners as the primary fuel, topped up with NG | - | 6; 37 | Overview; Sheet 14 | P |
| R22 | Flue gas (stream 9) | 136 C, 0.02 MPa, 8,659.4 kmol/h, 257,698 kg/h; CO2 0.2123, N2 0.6083, O2 0.0102, H2O 0.1692; CO 30 mg/Nm3 max, NOx 120 mg/Nm3 max | - | 44; 34 | stream 9; block flow diagram | P |
| R23 | Furnace radiant duty | 82.63 (96.099 MW at 4.1868 kJ/kcal) | MMkcal/h | 54 | equipment list sheet 5 of 8, flagged "Radiant Duty" | P; MW is C with A conversion |
| R24 | Utility-sheet fuel | 48.2 (hydrogen plant), 48.7 total (56.057 MW C) | MMkcal/h | 48 | utility sheet | P |
| R25 | HP steam to turbine (stream 10) | 46,053 kg/h, 2,556.0 kmol/h, 395 C, 4.23 MPa | - | 44 | stream 10 | P |
| R26 | Utility-sheet steam export | 45.9 | t/h | 48 | utility sheet | P |
| R27 | HP steam to process (stream 14); LP steam (stream 15); demi water (stream 11) | 95,301 kg/h, 400 C, 4.29 MPa; 540 kg/h, 177 C, 0.44 MPa; 91,800 kg/h, 15 C, 0.60 MPa | - | 45; 45; 44 | streams 14, 15, 11 | P |
| R28 | Steam crossing the battery limit | 0 | t/h | 34; 48 | block flow diagram; utility sheet TOTAL row | P |
| R29 | Power island | condensing turbine 12 MWe, generator 15 MVA, condenser 24 MMkcal/h at 8 kPa, sea water 3,420 m3/h at 7 K rise | - | 58; 48 | Power Island equipment list sheet 1 | P |
| R30 | Steam raised in the reformer waste heat boiler | "around 75 %" of the saturated steam | - | 38 | Sheet 15 | P; split of the other 25 % **U** |
| R31 | Steam to carbon ratio | 2.7 to 2.8 | mol/mol | 35 | Sheet 12 | P (<working folder>2.70 on hydrocarbon carbon) |
| R32 | Electricity price and gas price of the report | 80 EUR/MWh (export and import); 6 EUR/GJ LHV | - | 12 | Overview Table 1 | P |
| R33 | Operating hours, electricity revenue | 8,322 h/y; 6,603,008 EUR/y (9.918 x 8,322 x 80 = 6,603,008) | - | 229; 18 | Sheet 92 of 112; Overview Table 4 | P (check C) |
| R34 | Island TPC, TCR | 170.952 M EUR (hydrogen plant 97.212, power island 20.124, utilities and BoP 53.616); TCR 222.887 M EUR | - | 222; 17; 278 | Sheet 85 of 112; Overview Table 3; Annex III | P |
| R35 | Island fixed O&M, raw water, chemicals and catalysts | 7,545,514; 99,365; 420,000 | EUR/y | 18 | Overview Table 4 | P (already in the registry) |
| R36 | Discount rate | 8 % | - | 8; 236; 267 | design basis; Sheet 99; Sheet 21 of 24 | P (the registry cites p12, wrong; correct the locator in WP0) |
| R37 | Furnace thermal efficiency | none | - | - | no flue-gas or process enthalpies are given | **U** |
| R38 | Convective-section duties, duty of every exchanger and boiler, split of the 1.216 MWe | blank cells | - | 52, 53, 56 | equipment list sheets 3, 4 | **U** |
| R39 | Combustion air flow and composition | none (derived 6,648.5 kmol/h dry plus 62.7 kmol/h humidity, 192,944 kg/h, assuming 21 % O2; the O balance closes exactly at 20.91 %) | - | - | no number in the report | C and A, display only |
| R40 | Gauge or absolute marking of H and M pressures | none | - | 44, 45 | Annex I Table 4 prints MPa(g) (PDF p263, Sheet 17 of 24); 4.13 MPa(g) + 0.1 suggests absolute | **U** |

### A.4 Published stream table (source of the registry entries)

Pressure unit as printed (MPa, gauge or absolute not marked). Compositions are mole fractions, wet. Printed values, PDF p44 (streams 1 to 12) and PDF p45 (streams 13 to 15). I read these from rendered images and compared them with the streams audit table.

| Stream | Name | T, C | P, MPa | kmol/h | kg/h | Composition (nonzero printed entries) |
|---|---|---|---|---|---|---|
| 1 | Natural gas from B.L. | 9 | 7.00 | 1696.6 | 30563 | as R07 |
| 2 | NG feedstock to H2 plant | 128 | 3.71 | 1455.8 | 26231 | as stream 1 |
| 3 | NG fuel to burners | 121 | 0.50 | 240.4 | 4332 | as stream 1 |
| 4 | Purified feedstock to pre-reformer | 500 | 3.39 | 5514.0 | 98874 | CO2 .0053, H2 .0053, N2 .0023, CH4 .2350, C2H6 .0185, C3H8 .0026, nC4 .0003, H2O .7307 |
| 5 | HTS reactor inlet (reformer effluent) | 320 | 2.80 | 8370.3 | 101667 | CO2 .0492, CO .1156, H2 .5171, N2 .0015, CH4 .0238, H2O .2927 |
| 6 | HTS reactor outlet | 412 | 2.77 | 8370.3 | 101667 | CO2 .1283, CO .0366, H2 .5961, N2 .0015, CH4 .0238, H2O .2137 |
| 7 | PSA inlet | 35 | 2.58 | 6596.9 | 69711 | as R18 |
| 8 | PSA tail gas | 28 | 0.13 | 2106.3 | 60658 | as R19 |
| 9 | Flue gas to ATM | 136 | 0.02 | 8659.4 | 257698 | as R22 |
| 10 | HP steam export (to the COGEN turbine) | 395 | 4.23 | 2556.0 | 46053 | H2O 1.0 |
| 11 | Demi water make-up and turbine condensate | 15 | 0.60 | 5095.7 | 91800 | H2O 1.0 |
| 12 | Hydrogen to B.L. | 40 | 2.50 | 4461.5 | 8994 | H2 0.9999+ (taken as 1.0, A) |
| 13 | H2 recycle | 40 | 2.51 | 29.1 | 59 | H2 0.9999+ (taken as 1.0, A) |
| 14 | HP steam to process | 400 | 4.29 | 5290.1 | 95301 | H2O 1.0 |
| 15 | LP steam to deaerator | 177 | 0.44 | 30.0 | 540 | H2O 1.0 |

Integrity checks (C, my run): the composition sums are 1.0000 within 1e-4 (stream 5: 0.9999, stream 8: 1.0001); molar mass m/n against the composition agrees within 0.03 % except streams 13 and 15, where rounding of 59 kg/h and 30.0 kmol/h gives -0.57 % and +0.08 %. Stream 1 is 0.40 kmol/h above streams 2 plus 3 (0.024 %) while the masses add exactly.

### A.5 Reconstructed balances and residuals

Residual is out minus in, relative to in. Rows marked "mine" were recomputed by `spec_check.py`; rows marked "audit" were regenerated by re-running the streams audit script (`ieaghg_balances.py`, deterministic) and are not independent re-derivations.

| Balance | Equation | In | Out | Residual | Relative | Tolerance | Source |
|---|---|---|---|---|---|---|---|
| Carbon, NG to stack | C(stream 1) vs n9 y9,CO2 | 1,839.963 kmol C/h | 1,838.391 | -1.572 | -0.0854 % | 1.9 kmol/h worst-case rounding band | mine |
| Carbon, NG split | C(1) vs C(2) + C(3) | 1,839.963 | 1,839.529 | -0.434 | -0.024 % | 2.9 | audit |
| Carbon, feedstock to tail gas | C(2) vs C(8) | 1,578.815 | 1,578.461 | -0.354 | -0.0224 % | 1.6 | mine |
| Carbon, PSA | C(7) vs C(8) | 1,578.638 | 1,578.461 | -0.177 | -0.011 % | 1.4 | mine |
| Carbon, tail gas plus fuel to stack | C(8) + C(3) vs stack | 1,839.175 | 1,838.391 | -0.784 | -0.0426 % | 1.0 | mine |
| PSA mass | m7 vs m8 + m12 + m13 | 69,711 kg/h | 69,711 | 0 | 0 | exact | mine |
| PSA moles | n7 y vs model tail gas | - | 2,106.30 vs 2,106.3 | 0 | 0 | - | mine |
| PSA species, model tail gas minus stream 8 | CO2 +0.156, CO -0.160, H2 -0.347, N2 +0.135, CH4 +0.181, H2O -0.175 kmol/h | | | max 0.347 | | band 0.44 kmol/h | mine |
| Reformer C, stream 4 to 5 | | 1,578.66 | 1,578.64 | -0.020 | -0.0012 % | | audit |
| Shift C and O, stream 5 to 6 | | 1,578.64; 4,241.2 | 1,579.48; 4,242.9 | +0.837; +1.674 | +0.053 %; +0.040 % | | audit |
| Overall control volume (reformer, shift, PSA, furnace), derived steam and air | C, H, O, N, mass | 1,839.529; 15,399.527; 7,107.196; 10,534.821 kmol/h; 298,886.6 kg/h | 1,838.391; 15,399.142; 7,091.504; 10,535.026; 298,624.3 | -1.138; -0.385; -15.692; +0.205; -262.3 | -0.062 %; -0.0025 %; -0.221 %; +0.0019 %; -0.088 % | C is the only independent element; H and N fix steam and air; O is the air-composition check | audit (re-run) |
| Water, net reaction | steam fed minus condensate minus PSA-feed water | 43.16 t/h | utility sheet 44.2 t/h | 1.0 | | deaerator vent size | audit (re-run) |
| Table 2 energy and power | 30.563 x 46.50 / 3.6; 8.994 x 119.96 / 3.6; 11.500 - 1.216 - 0.366 | 394.772; 299.700; 9.918 | 394.77; 299.70; 9.918 | +0.002; 0.000; 0 | < 0.001 % | printed digits | mine |
| Stack CO2 against Table 2 | 8,659.4 x 0.2123 x 44.0095 | 0.80907 kg/Nm3 | 0.8091 | -0.00003 | -0.0041 % | +/-0.024 % (rounding of y) | mine |

Derived quantities (C unless stated). Tail-gas LHV with the engine constant<working folder>H2 33.515 MW, CO 24.075 MW, CH4 44.359 MW, total **101.950 MW** = 174.248 MJ/kmol = 6.0506 MJ/kg = 7.7741 MJ/Nm3. Supplementary NG fuel (stream 3 at the published LHV): 4.332 x 46.50 / 3.6 = **55.955 MW**. Feedstock NG energy 338.817 MW. Burner heat release (fired duty) **157.905 MW** = 40.00 % of the plant input; tail gas supplies 64.56 %. The utility-sheet fuel (56.057 MW) is stream 3 within +0.18 %, so the utility fuel excludes the tail gas. Radiant duty 96.099 MW = 60.86 % of the fired duty.

### A.6 Does 8.996 kg/kg already include tail-gas combustion? Carbon breakdown

**Yes.** The Base Case has no capture and one carbon outlet, stream 9. The report states (PDF p6) that "all of the CO2 is emitted from the flue gas of the steam reformer", produced in reforming and shift and in the combustion of the residual CO in the PSA tail gas and of the supplementary NG. The tail gas is "its primary fuel" (PDF p6). Stream 9 reproduces 0.8091 (A.5). The Case 2A tail-gas capture removes 0.4556 kg/Nm3, which is 96.5 % of the pass-through CO2 below, so Table 2 counts that CO2 inside the 0.8091 (streams audit F14, checked against Table 2 on PDF p16).

Split of the stack CO2 (C; "raw" from carbon in streams 8 and 3 with M = 44.0095; "rescaled" by 0.999574 so that the parts add to the stack value exactly):

| Component | Basis | t CO2/h raw | t CO2/h rescaled | kg/Nm3 rescaled | kg/kg H2 rescaled | Share |
|---|---|---|---|---|---|---|
| A1 process CO2 in the tail gas, passes through unburnt | n8 y(CO2) = 1,073.16 kmol/h | 47.229 | 47.209 | 0.47209 | 5.249 | |
| A2 CO in the tail gas, burnt | 306.26 kmol/h | 13.478 | 13.472 | 0.13472 | 1.498 | |
| A3 CH4 in the tail gas, burnt | 199.05 kmol/h | 8.760 | 8.756 | 0.08756 | 0.974 | |
| **A feedstock carbon via the tail gas** | 1,578.46 kmol C/h | **69.467** | **69.438** | **0.69438** | **7.720** | **85.82 %** |
| B1 CO2 in the supplementary NG | 4.808 kmol/h | 0.212 | 0.212 | 0.00212 | 0.024 | |
| B2 hydrocarbon carbon of the supplementary NG, burnt | 255.9 kmol C/h | 11.262 | 11.258 | 0.11257 | 1.252 | |
| **B supplementary NG** | 260.71 kmol C/h | **11.474** | **11.469** | **0.11469** | **1.275** | **14.18 %** |
| A + B | | 80.941 | 80.907 | 0.80907 | 8.9956 | stack from stream 9 is 80.9067, so A + B raw is +0.0426 % |

Rules that follow (they become the double-counting tests of section C):

* The central unabated intensity is the stream-9 stack CO2 divided by the H2 mass (N.R.9). It is one route. The explicit route A + B differs by +0.0426 % and serves as a check only; the two routes are never added.
* If a scenario changes the tail-gas composition or the supplementary NG flow relative to the Base Case, only the change relative to A and B is added to the central value.
* A capture step acting on the tail gas can remove at most A (0.694 kg/Nm3, 85.8 %). A step acting on the flue gas sees A + B.

---

## B. Reference-plant module in the Rust engine

### B.1 Purpose and switch semantics

The module reconstructs the IEAGHG Base Case from the published streams, validates the reconstruction against Table 2, and supplies the unabated reference of the independent basi<working folder>intensity, gas energy, steam and power. It replaces three INL-based or ratio-based inputs on the independent basis when the reference kind is IEAGHG.

* New registry flag `REF_BASIS` (owner decision, default 1). 1 = reconstructed IEAGHG module feeds the intensity, the GAS line, LOST_STEAM, BASE_POWER, K.18 and the upstream credit. 0 = the independent basis exactly as at `8dbfc6d` (intensity ratio 0.8091 / 0.08994, carbon-implied gas with the INL natural gas composition, INL steam and power lines). `REF_BASIS = 0` is the regression guard and the source of the historical comparison rows.
* `CARBON_BASELINE` keeps its three values (0 IEAGHG, 1 INL, 2 Cho). Only kind 0 uses the module. Kinds 1 and 2 keep the carbon-implied gas and the INL lines, so the `BASE_INL` and `BASE_CHO` corners of the BASELINE sensitivity group stay unchanged.
* The legacy carbon basis (`CARBON_BASIS = 0`, `CarbonSr<working folder>:Anchored`) never calls the module. Its branches stay as they are.
* The id string `IEAGHG_9p00` is a persistent key (CSV rows, tests, `build_site.py` lines 99 to 103). It stays; only printed labels change (WP6).

### B.2 Files, structs and API

New file `engine/plant/src/refplant.rs`, module name `refplant`, declared in `engine/plant/src/lib.rs`. The registry reader lives in the same file and is called from `Mode<working folder>:build` in `anchored.rs` (one new field `ref<working folder>RefPlant`, like `subsid<working folder>SubsidyParams`). Species order everywher<working folder>CO2, CO, H2, N2, O2, CH4, C2H6, C3H8, NC4H10, NC5H12, H2O]`, `NSP_REF = 11`.

```rust
pub struct RefStream { pub i<working folder>u8, pub nam<working folder>&'static str, pub t_<working folder>f64, pub p_mp<working folder>f64,
                       pub kmol_<working folder>f64, pub kg_<working folder>f64, pub <working folder>f64; NSP_REF] }   // published rows, 15 of them

pub struct RefPlantParams {            // everything read from the registry (B.4)
    pub stream<working folder>Vec<RefStream>,       // ids 1..=15
    pub h2_nm3_<working folder>f64, pub nm3_kmo<working folder>f64,
    pub ng_lhv_mj_k<working folder>f64, pub ng_hhv_mj_k<working folder>f64, pub h2_lhv_mj_k<working folder>f64,
    pub energy_in_m<working folder>f64, pub energy_h2_m<working folder>f64,            // validation targets
    pub powe<working folder>RefPower,               // gross, h2_plant, aux_island_kw, aux_bop_kw, net_published
    pub radiant_mmkcal_<working folder>f64, pub utility_fuel_mmkcal_<working folder>f64,
    pub steam_neighbour_t_<working folder>f64,      // published 0
    pub steam_utility_t_<working folder>f64,        // 45.9, cross-check
    pub export_price_rati<working folder>f64,       // 1.0
    pub revenue_eur_<working folder>f64, pub price_eur_mw<working folder>f64, pub hour<working folder>f64,   // validation of the power value
    pub psa_recovery_rang<working folder>(f64, f64),
    pub co2_kg_nm3_publishe<working folder>f64,     // BASELINE_IEAGHG_KG_NM3, existing entry
    pub basi<working folder>RefBasis,               // from REF_BASI<working folder>Reconstructed | Historical
}
impl RefPlantParams { pub fn read(<working folder>&mut Reader<'_>) -> Result<Self>; }

pub struct RefPlant { pub <working folder>RefPlantParams }
impl RefPlant {
    pub fn new(<working folder>RefPlantParams) -> Result<Self>;          // runs balance() once, refuses blocking check failures
    pub fn balance(&self) -> Result<RefBalance>;            // at the published scale
    pub fn psa_tail(&self, recover<working folder>f64) -> PsaBalance;    // N.R.2, used by sensitivities and tests
    pub fn supp_ng_t_h(&self, q_tail_m<working folder>f64) -> f64;       // N.R.5, constant fired duty
    pub fn scaled(&self, h2_kg_<working folder>f64) -> Result<RefScaled>;// N.R.12 and N.R.13
    pub fn stream_table(&self, h2_kg_<working folder>f64) -> Vec<RefStreamRow>;  // published + derived rows with class
}
pub struct RefBalance { pub ps<working folder>PsaBalance, pub furnac<working folder>FurnaceBalance, pub co2: Co2Breakdown,
                        pub ga<working folder>GasEnergy, pub steam_powe<working folder>SteamPower, pub check<working folder>Vec<RefCheck> }
pub struct RefCheck { pub i<working folder>&'static str, pub labe<working folder>&'static str, pub publishe<working folder>f64,
                      pub calculate<working folder>f64, pub rel_residua<working folder>f64, pub tol_re<working folder>f64, pub blockin<working folder>bool }
pub struct RefScaled { pub <working folder>f64, pub intensity_kg_k<working folder>f64, pub gas_mmscfd_e<working folder>f64,
                       pub steam_neighbour_mmbtu_<working folder>f64, pub power_buy_mw<working folder>f64 /* negative = net export */ }
```

`impl Model { pub fn reference_system(&self, <working folder>&Carbon, <working folder>f64, basi<working folder>SystemBasis) -> Result<RefSystem> }` returns `{ gas_mmscfd, steam_export_mmbtu_h, power_buy_mwe, sourc<working folder>RefSourc<working folder>:{Ieaghg, Inl} }`. It is the only switch point. It returns the IEAGHG values when `c.src == Plant`, `carbon_params.baseline == Ieaghg`, `refp.p.basis == Reconstructed` and `basis != InlHistorical`. Otherwise it returns the legacy values with the legacy expressions written exactly as today (`self.export_conv * h_s`, `self.e_conv * h_s`) so that floating-point results are bit-identical.

`CostQuery` keeps `system_line<working folder>bool` and gains `system_basi<working folder>SystemBasis { Reference, InlHistorical }` (default `Reference`). `system_lines = false` still zeroes both lines (`NOSYS_` rows).

Blocking checks (`RefPlan<working folder>:new` fails with `EngineErro<working folder>:Range` if one is violated, which protects against a bad registry edit): carbon NG to stack within 0.2 %; PSA species residuals within 0.5 kmol/h; stack CO2 against 0.8091 within 0.1 %; net power equals gross minus internal within 1e-9; PSA mass closure within 1 kg/h; stream 1 to stream 2 plus 3 mass within 1 kg/h. Non-blocking (reported in `Reference_Plant.csv` and the validation rows): element O balance, stream 10 against 45.9 t/h, utility fuel against stream 3, Annex versus Table 2 LHV, PSA recovery inside 85 to 90.1 %.

### B.3 Equations (numbered N.R.x, to be written in `docs/derivation/03_reference_plant.md`)

Notatio<working folder>treams k = 1..15; n_k kmol/h; m_k kg/h; y_ki mole fraction; c_i carbon atoms per molecule (CO2 1, CO 1, CH4 1, C2H6 2, C3H8 3, nC4H10 4, nC5H12 5); x the scale factor of N.R.12. Heating values in MW use `kmol/h x MJ/kmol / 3600` and `kg/h x MJ/kg / 3600`.

| Id | Equation | Meaning and published or calculated value | Rust item | Test (file `engine/plant/tests/m12_reference_plant.rs`) |
|---|---|---|---|---|
| N.R.1 | ṅ_ki = n_k y_ki; C_k = sum_i ṅ_ki c_i | species and carbon flows of the published table. C1 1,839.963; C2 1,578.815; C3 260.714; C7 1,578.638; C8 1,578.461 kmol C/h | `RefPlan<working folder>:species`, `carbon_of` | `eq_n_r_1_published_table_rows_are_as_printed` |
| N.R.2 | R = (n12 + n13) / (n7 y7,H2); ṅ_tail,i = ṅ_7,i for i other than H2; ṅ_tail,H2 = (1 - R) ṅ_7,H2 | component-wise PSA balance. R = 0.90006 (C). All impurities are rejected, the product is taken as pure H2. | `psa_tail` | `eq_n_r_2_psa_balance_reproduces_stream_8` |
| N.R.3 | Q_tail = sum over i in {H2, CO, CH4} of ṅ_tail,i LHV_i / 3600 | tail-gas heating value with the engine constants. 101.950 MW; 6.0506 MJ/kg; 7.7741 MJ/Nm3 | `FurnaceBalanc<working folder>:q_tail_mw` | `eq_n_r_3_tail_gas_lhv_uses_the_engine_constants` |
| N.R.4 | Q_NG,supp = m3 LHV_NG / 3600 | supplementary NG heat, published LHV 46.50. 55.955 MW | `q_ng_supp_mw` | `eq_n_r_4_ng_fuel_heat_matches_the_utility_sheet_fuel` |
| N.R.5 | Q_fired = Q_tail + Q_NG,supp; for a changed tail gas m_NG,supp(Q_tail') = max(0, Q_fired - Q_tail') x 3600 / LHV_NG | furnace duty as burner heat release. 157.905 MW; tail-gas share 64.56 %. The second form assumes a constant fired duty per kg of H2 (A) and is used only by sensitivities and tests. | `q_fired_mw`, `supp_ng_t_h` | `eq_n_r_5_fired_duty_and_tail_gas_share` |
| N.R.6 | phi_rad = Q_rad / Q_fired, Q_rad = REF_RADIANT x 1e6 x KCAL_KJ / 3.6e6 | indicator onl<working folder>96.099 MW, 0.6086. No efficiency is derived or stored. | `radiant_fraction` | `eq_n_r_6_radiant_duty_is_an_indicator_only` (asserts that no registry id `REF_FURNACE_EFF` exists) |
| N.R.7 | ṅ_CO2,stack = n9 y9,CO2; ṁ_stack = ṅ_CO2,stack x MW_CO2; eps_C = ṅ_CO2,stack / C1 - 1 | stack CO2 80.9067 t/h; eps_C = -0.0854 % | `Co2Breakdow<working folder>:stack_t_h`, `c_residual_rel` | `eq_n_r_7_carbon_closes_from_gas_to_stack` |
| N.R.8 | A1 = n8 y8,CO2; A2 = n8 y8,CO; A3 = n8 y8,CH4; B1 = n3 y3,CO2; B2 = C3 - B1; rho = ṁ_stack / ((A1 + A2 + A3 + B1 + B2) MW_CO2) | CO2 breakdown of A.6 from the published streams 8 and 3 (taken from the published streams, with the model tail gas as the check). rho = 0.999574 | `Co2Breakdown` | `eq_n_r_8_co2_breakdown_adds_to_the_stack` |
| N.R.9 | e_ref = ṁ_stack / m12 (kg/kg); e_ref,Nm3 = ṁ_stack / (n12 x NM3_KMOL) | central unabated intensit<working folder>8.99563 kg/kg, 0.80907 kg/Nm3. Validation targets 8.995997 and 0.8091 within 0.05 %. | `RefBalanc<working folder>:intensity_kg_kg` | `eq_n_r_9_intensity_reproduces_table_2` |
| N.R.10 | Q_gas,LHV = m1 LHV_NG / 3600; E_HHV = m1 HHV_NG / 1000 (GJ/h); n_ref = x E_HHV x 24 / (NG_HHV x MMBTU_GJ) | gas energy converted by energy; the volume route is a cross-check only. 394.772 MW LHV (validates R10); 436.99 MW HHV; n_ref = 34.2776 MMSCFD at x = 1, in the engine's gas unit | `GasEnergy`, `RefScale<working folder>:gas_mmscfd_eq` | `eq_n_r_10_gas_energy_matches_table_2_and_converts_by_energy` |
| N.R.11 | W_net = W_gross - W_H2 - W_aux with W_aux = (19 + 347) / 1000; w_spec = W_gross x 1000 / (m10 / 1000) kWh/t; revenue = W_net x hours x price | power balanc<working folder>9.918 MWe exact; w_spec 249.71 kWh/t (250.54 with the utility-sheet 45.9 t/h); revenue 6,603,008 EUR/y; steam to the neighbour 0 | `SteamPower` | `eq_n_r_11_power_balance_closes` |
| N.R.12 | x = ṁ_H2,plant / m12, with ṁ_H2,plant = `h2_a_kgh` x h_s and h_s = s x `c.h_scale` | every extensive reference quantity scales with x. At Case B (8,988 kg/h) x = 0.999333; intensive ones (kg/kg) do not move | `scaled` | `eq_n_r_12_scaling_is_linear_and_intensive_quantities_do_not_move` |
| N.R.13 | steam_ref = x REF_STEAM_NEIGHBOUR (0, other values refused); P_buy = -x W_net (MWe, negative = export); smr_power_y = P_buy x hours x ELECTRICITY_PRICE x kappa / 1e3 with kappa = REF_POWER_EXPORT_PRICE_RATIO when P_buy < 0 and 1 otherwise; LOST_STEAM = (steam_ref - steam_plant) x steam price; BASE_POWER = -smr_power_y | the two ledger lines from the published balances. Case <working folder>BASE_POWER = +14.760 S$M/y at S$0.20/kWh (C), LOST_STEAM = -steam_plant x price, which is zero within the rounding of the printed line because the plant exports no steam in the headline case (189.2 x 0.683281 = 129.276 MMBtu/h against 15.2339 / 0.117839 = 129.276, C) | `reference_system`, `cost_items` | `eq_n_r_13_system_lines_follow_the_reference` |
| N.R.14 | K.18: lost = steam_ref - export_kept - tg_surplus (MMBtu/h); power term = P_buy x hours x GRID_EF | same signed inputs in the carbon term | `system_adjust` | `eq_n_r_14_k18_terms_follow_the_reference` |
| N.R.15 | tail-gas heat register of the plant (section C): LHV(S14) = LHV(S03) + LHV(S21) + LHV(S15); claims on the steam system limited to Q_N2_BURN | no double count of the plant's own tail gas | `heat_register` (in `refplant.rs`, reads the flowsheet) | `eq_n_r_15_tail_gas_heat_is_claimed_once` |
| N.R.16 | N.9.2 with reference_carbon = E_ref / k + P + (1 - v) K'_ref, E_ref = e_ref x h2_t_y, K'_ref = 0 | the decomposition still sums to the direct abatement. K' cancels in the identity (checked algebraically), so K'_ref = 0 is a convention, A (the published carbon balance leaves at most 0.86 kmol/h unaccounted) | `reference_carbon` | `eq_n_r_16_decomposition_sums_to_direct_abatement_on_the_reference` |
| N.R.17 | with REF_BASIS = 0 every independent-basis output equals the pre-change snapshot; the legacy basis equals the legacy snapshot | regression equation | `Model` | `eq_n_r_17_historical_basis_and_legacy_basis_are_bit_identical` |

### B.4 Parameters to add to `engine/data/parameters.toml`

All entries use `group = "reference"`, `rigour = "fixed_assumption"`, `decision = "D13"`. Source strings cite the report by short title, table or stream, and PDF page (the registry convention), never a working file. The generator rule for the stream entries should be a throw-away Python script run once from the table in A.4; the test `eq_n_r_1_published_table_rows_are_as_printed` holds the literals as the independent check.

**Stream entries (103).** Id pattern `REF_S{nn}_{FIELD}` with nn = 01..15 and FIELD one of `T_C`, `P_MPA`, `KMOL_H`, `KG_H` (60 entries), plus `REF_S{nn}_Y_{SPECIES}` for every nonzero printed mole fraction of streams 1, 4, 5, 6, 7, 8, 9 (7 + 8 + 6 + 6 + 6 + 6 + 4 = 43 entries; streams 2 and 3 copy stream 1 in code, as printed; streams 10, 11, 14, 15 are pure water and streams 12, 13 pure H2 by identity in code). Class `published`, source `IEAGHG 2017-02 Heat and Material Balance, Base case (rev. 1, June 2015), stream {n}`, locator `PDF p44, stream {n}` (streams 1 to 12) or `PDF p45, stream {n}` (13 to 15), status `primary`. Species id<working folder>CO2 CO H2 N2 O2 CH4 C2H6 C3H8 NC4H10 NC5H12 H2O`.

**Scalar entries.**

| Id | Value | Unit | Class | Source (short) | Locator | Status |
|---|---|---|---|---|---|---|
| `REF_BASIS` | 1 | flag | user_decision | D13: 1 reconstructed IEAGHG reference, 0 the independent basis of the previous commit (kept as regression guard) | decision D13 | primary |
| `REF_H2_NM3_H` | 100000 | Nm3/h | published | IEAGHG 2017-02 Overview Table 2, hydrogen to B.L. | Table 2, PDF p16 | primary |
| `REF_NM3_KMOL` | 22.414 | Nm3/kmol | assumed | 0 C and 101.325 kPa; the report gives no definition, 4,461.5 kmol/h x 22.414 reproduces 100,000 Nm3/h (stream 12, PDF p44) | stream 12, PDF p44 | primary |
| `REF_NG_LHV_MJ_KG` | 46.50 | MJ/kg | published | IEAGHG 2017-02 Overview Table 2 (Annex I Table 1 gives 46.502) | Table 2, PDF p16 | primary |
| `REF_NG_HHV_MJ_KG` | 51.473 | MJ/kg | published | IEAGHG 2017-02 Annex I Table 1 natural gas specification | Annex I Table 1, PDF p255 (Sheet 9 of 24) | primary |
| `REF_H2_LHV_MJ_KG` | 119.96 | MJ/kg | published | IEAGHG 2017-02 Annex I Table 2 hydrogen specification | Annex I Table 2, PDF p256 (Sheet 10 of 24) | primary |
| `REF_ENERGY_IN_MW` | 394.77 | MW | published | IEAGHG 2017-02 Overview Table 2, total energy input A; validation target | Table 2, PDF p16 | primary |
| `REF_ENERGY_H2_MW` | 299.70 | MW | published | IEAGHG 2017-02 Overview Table 2, total energy in product B; validation target | Table 2, PDF p16 | primary |
| `REF_POWER_GROSS_MWE` | 11.500 | MWe | published | IEAGHG 2017-02 Overview Table 2, gross power from the COGEN plant | Table 2, PDF p16 | primary |
| `REF_POWER_H2_PLANT_MWE` | 1.216 | MWe | published | IEAGHG 2017-02 Overview Table 2, H2 plant load | Table 2, PDF p16 | primary |
| `REF_POWER_AUX_ISLAND_KW` | 19 | kW | published | IEAGHG 2017-02 utility sheet, power island row | Estimated Utility Consumptions, PDF p48 | primary |
| `REF_POWER_AUX_BOP_KW` | 347 | kW | published | IEAGHG 2017-02 utility sheet, utilities and BoP row | Estimated Utility Consumptions, PDF p48 | primary |
| `REF_POWER_NET_MWE` | 9.918 | MWe | published | IEAGHG 2017-02 Overview Table 2, export power to the grid; validation target | Table 2, PDF p16 | primary |
| `REF_RADIANT_MMKCAL_H` | 82.63 | MMkcal/h | published | IEAGHG 2017-02 equipment list, steam reformer furnace radiant duty | equipment list sheet 5 of 8, PDF p54 | primary |
| `REF_UTILITY_FUEL_MMKCAL_H` | 48.2 | MMkcal/h | published | IEAGHG 2017-02 utility sheet, hydrogen plant fuel | Estimated Utility Consumptions, PDF p48 | primary |
| `REF_STEAM_NEIGHBOUR_T_H` | 0 | t/h | published | IEAGHG 2017-02 block flow diagram battery limits and utility sheet total<working folder>no steam crosses the battery limit | PDF p34; PDF p48 | primary |
| `REF_STEAM_UTILITY_T_H` | 45.9 | t/h | published | IEAGHG 2017-02 utility sheet, HP steam to the power island; cross-check of stream 10 | Estimated Utility Consumptions, PDF p48 | primary |
| `REF_POWER_EXPORT_PRICE_RATIO` | 1.0 | - | published | IEAGHG 2017-02 Overview Table 1: one electricity price of 80 EUR/MWh for export and import | Table 1, PDF p12 | primary |
| `REF_POWER_PRICE_EUR_MWH` | 80 | EUR/MWh | published | IEAGHG 2017-02 Overview Table 1; used only to validate the report's own revenue line | Table 1, PDF p12 | primary |
| `REF_OPERATING_HOURS` | 8322 | h/y | published | IEAGHG 2017-02 Sheet 92 of 112, yearly operating hours | Sheet 92, PDF p229 | primary |
| `REF_POWER_REVENUE_EUR_Y` | 6603008 | EUR/y | published | IEAGHG 2017-02 Overview Table 4, electricity export revenue | Table 4, PDF p18 | primary |
| `REF_PSA_H2_RECOVERY_RANGE` | 0.90 (low 0.85, high 0.90) | - | published | IEAGHG 2017-02 Sheet 14: "typically 85 to 90 %" of the PSA feed H2 is recovered | Sheet 14, PDF p37 | primary |
| `SMR_ISLAND_POWER_ISLAND_TPC_EURM` | 20.124 | M EUR | published | IEAGHG 2017-02 Base Case total plant cost, power island column | Sheet 85 of 112, PDF p222 | primary |
| `SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND` | 0 | flag | user_decision | D13: 0 keeps the whole island on both sides of the DCF (as before); 1 removes the power-island share from the NUS-H2 side only (sensitivity) | decision D13 | primary |

Existing entries that stay and are reuse<working folder>BASELINE_IEAGHG_KG_NM3` (validation target for N.R.9), `BASELINE_IEAGHG_RHO` (comparator only), `SMR_ISLAND_TCR_EURM`, `SMR_ISLAND_TPC_EURM`. Correct the `DISCOUNT` locator to `Design basis, PDF p8; Sheet 99, PDF p236` (value unchanged). Change the `SMR_ISLAND_*` source text that calls the whole island common to both plants to state the power-island overlap (section D.3). `REF_FURNACE_EFF` and `REF_STEAM_DUTY` named in the derivation text are **not** create<working folder>both are unavailable, and the published flows replace them.

### B.5 How the outputs feed the engine

**(a) Baseline CO2 intensity.** `baseline_kg_kg(Ieaghg)` returns `refp.intensity_kg_kg` (N.R.9, 8.99563 kg/kg) when the module is active. `Emissions.baseline_central = baseline_kg_kg(kind) x h2_t_y` is unchanged in form. Table 2's 8.995997 is a validation row (`REF_INTENSITY_VS_TABLE_2`, tolerance 0.05 %, value -0.0041 %). The two H2 densities no longer meet in the abatemen<working folder>the reference intensity is mass per mass (m_stack / 8,994 kg/h) and h2_t_y is the plant's own H2 mass. Leave `RHO_H2_NORMAL` at 0.08988 and keep the review item P08 open as a plant-side item. At Case B the baseline moves from 602,053.95 to 602,029.04 t/y (C, -24.9 t/y).

**(b) Unabated gas energy of the GAS line.** `reference_gas_mmscfd(c, s)` returns `reference_system(...).gas_mmscfd` for the module (N.R.10): the reference gas energy in HHV GJ/h, converted into the engine's MMSCFD unit with the engine's `NG_HHV x MMBTU_GJ`, so that the plant side (`c.ng`) and the reference side are expressed in one currency. This is the single number behind the GAS line, the upstream credit of `lifecycle_terms` (energ<working folder>n_ref x gj x ef_up`) and the `n_ref` of the N.9.2 decomposition (informational). The carbon of the decomposition no longer follows from `n_ref x cbar`; it follows from N.R.16. Rule 1 of N.7 is restate<working folder>one reference gas, two conversions (energy for cost and upstream credit, carbon for abatement). At Case B n_ref = 34.2548 MMSCFD against 35.0923 today (C, x = 0.999333).

**(c) LOST_STEAM and BASE_POWER.** Both lines are recomputed from the published steam and electricity balances (N.R.13): the reference sells no steam (0, P) and exports 9.918 MWe. The NUS-H2 prices stay separate assumption<working folder>gas price for the plant's own exported steam (formula as today, no boiler efficiency in the cost ledger), `ELECTRICITY_PRICE` S$0.20/kWh for the exported power, the export ratio kappa = 1 because the report uses one price both ways. The ledger line ids `LOST_STEAM` and `BASE_POWER` and the DCF ids `STEAM_CREDIT` and `PURCHASED_POWER` stay (CSV and token stability). Their meaning changes, and the line text and source strings change with them (`export.rs` `ledger_line_text` and `line_meta`, lines 3644 to 3667 and 3744 to 3750): BASE_POWER reads "Export revenue of the unabated reference that this plant does not earn (IEAGHG Base Case, Table 2)", a positive cost; the DCF `PURCHASED_POWER` value is negative on the reference side (revenue). The INL line pair (S$15.819/t at BOAK 25 y, <working folder>(15.2339 - 6.4105) x 1e6 / 557,767) is kept only as labelled historical rows `HIST_INL_SYSLINES_L5_{stage}_{life}Y_N1P00`, computed with `system_basis = InlHistorical`. The `SYSLINES_` rows now isolate the IEAGHG lines (headline minus `NOSYS_`), and the `NOSYS_` rows keep their meaning.

K.18 (`system_adjust`) takes the same signed inputs. Case <working folder>grid term -29.67 kt/y of CO2 (the reference's exported power displaces grid emissions that this plant does not) against +12.89 kt/y for the INL purchase (C, GRID_EF 0.402 t/MWh); the lost-steam term goes from -63.3 kt/y to 0.

**Floor reference.** The helium-heated no-capture run (`PLANT_FLOOR`, 7.850175 kg/kg) stays as a labelled lower-bound row. Its tests (floor below the IEAGHG value) remain true. No `FLOOR_` id changes.

**Sensitivity groups.** `BASELINE` keeps the three corners; the IEAGHG corner is the central point. `GAS_SAVING_AHN` and the other consumers of `reference_gas_mmscfd` follow automatically. The Ahn pair base 30,563 kg/h equals the IEAGHG total gas (26.231 + 4.332 t/h, C), so the pair now applies to the matching reference.

**Validation rows (harness, `Validation_Results.csv`).** Each uses the `REF_` prefix. Tolerances in parentheses are relative unless state<working folder>REF_TABLE2_ENERGY_IN` (1e-4), `REF_TABLE2_ENERGY_H2` (1e-4), `REF_TABLE2_NET_POWER` (1e-9), `REF_INTENSITY_VS_TABLE_2` (5e-4), `REF_CARBON_CLOSURE_NG_TO_STACK` (2e-3), `REF_PSA_SPECIES_MAX_RESIDUAL` (0.5 kmol/h absolute), `REF_PSA_RECOVERY_IN_RANGE` (inside 0.85 to 0.901), `REF_UTILITY_FUEL_VS_STREAM_3` (5e-3), `REF_STEAM_STREAM_10_VS_UTILITY` (5e-3), `REF_POWER_REVENUE` (1e-6), `REF_NG_STREAM_SPLIT_MASS` (exact), `REF_STEAM_TO_NEIGHBOUR_ZERO` (exact), `REF_LCOH_BRIDGE` (3e-2: LCOH_SMR at BOAK 25 y against the first-order bridge from the report LCOH, 3.47 S$/kg, <working folder>2.954 + 0.422 gas price + 0.111 availability - 0.015 electricity credit re-based to S$0.20/kWh; today's value is 3.618), `IND_FLOOR_VS_IEAGHG` (existing, re-pointed). The bridge row is an independent check on the whole reconstruction, because it does not use the module.

### B.6 Keeping the legacy basis bit-identical

* `reference_system` returns the legacy expressions unchanged for `CarbonSr<working folder>:Anchored`, for `REF_BASIS = 0`, for kinds Inl and Cho and for `SystemBasi<working folder>:InlHistorical`. The order of multiplication in `steam_smr_mm` and `smr_power_y` must be preserved.
* No existing registry value changes. New entries are appended; `Final_Design_Basis.csv` gains rows, which the legacy comparison (`--subset`) tolerates.
* The plant flowsheet, MEA, cryogenic and anchored models are not touched. `Integrated_Mass_Balance.csv` and `Integrated_Energy_Balance.csv` must stay byte-identical on both bases (the tail-gas heat register goes to the new `Reference_Plant.csv`, section C).
* Gate G2 and G3 in section E check this by file comparison.

### B.7 Expected numerical effect at Case B, BOAK 25 y (calculated what-if, engine not run)

Closed forms from committed value<working folder>k_gas = 1,101.48 GJ/MMSCF x 310.25 d x S$15/GJ / 1e6 = 5.1260 S$M per MMSCFD-y; hours 7,446; A_net 557,767.4 t/y.

| Item | Today | After WP3 | Change |
|---|---|---|---|
| Reference gas, MMSCFD | 35.0923 | 34.2548 | -0.8375 |
| GAS line, S$M/y | -61.4798 | about -57.19 | +4.293 (+7.70 S$/t) |
| LOST_STEAM, S$M/y | +15.2339 | about 0.000 | -15.234 |
| BASE_POWER, S$M/y | -6.4105 | +14.7600 | +21.171 |
| Both system lines, S$M/y | +8.8234 (15.819 S$/t) | +14.7600 (26.46 S$/t) | +5.937 (+10.64 S$/t) |
| L5 headline | 203.180 S$/t | about 221.5 S$/t | about +18.3 S$/t |
| LCOH_SMR (25 y) | 3.618 S$/kg | about 3.47 S$/kg | about -0.15 |
| Baseline, t/y | 602,053.95 | 602,029.04 | -24.9 |

At the report-equivalent electricity price S$0.186/kWh the power line is +13.73 and the L5 shift +16.5 S$/t (C). The engine result must agree with the closed forms within 0.05 S$/t; a larger gap means an unintended interaction and stops the work package.

---

## C. The plant's own PSA tail gas (reject S14), double counting, PFD and simulation changes

### C.1 What the engine does today (read from the audits, `flowsheet.rs` lines 1150 to 1285, `anchored.rs` lines 1124 to 1306)

The PSA is an unsimulated external block (88 % H2 recovery). The reject S14 (B_REC_F, Case <working folder>11,690.7 kg/h, 1,017.35 kmol/h, 55 C, 1.72 bar) goes through the recycle compressor to the cryogenic unit REC-CRYO, which sends the carbon share f = 0.752178 to the reformer feed S03, the H2 share r = 0.92163 to S21 ahead of the PSA, and the rest with N2 and H2O to the nitrogen-rich cut S15 (171.9 kmol/h). N2-BURN burns S15 with stoichiometric O2 and gives the stack S16 (set state 148.9 C, 1.034 bar). The heat Q_N2_BURN = H(S15) - H(S16) = 16.262 MW at Case B is booked to the node STEAM_SYSTEM by closure; no unit consumes it. The reformer is helium-heate<working folder>there is no furnace and no furnace duty. LHV (repo constants, H2 + CO + CH4 only, C): S14 94.775 MW, S15 16.567 MW. In the anchored balance the tail-gas LHV reduces the reboiler gap only in the burn-on-site scenarios CombH and CombN (q_tg = LHV x ETA_BOILER 0.85), and in the recycle scenarios the burned-cut heat is not in the balance because INL's export steam of 83.7 MMBtu/h already contains it.

### C.2 Accounting rules (no double counting)

1. **Reference side.** The tail gas is internal. Its energy (101.95 MW) is feedstock energy that returns to the furnace and is never added to the reference gas energy; the GAS line prices the NG only (394.77 MW LHV, 436.99 MW HHV, x-scaled). Its carbon is inside the stack CO2 (A.6) and is never added again.
2. **Plant side.** The plant has no furnace. Every MW of LHV in S14 belongs to exactly one clai<working folder>cycled as reformer feed (S03) or as PSA feed (S21), or burned in N2-BURN (S15). The only claim on the steam system is Q_N2_BURN, bounded by LHV(S15). The recycled part counts as feed and never as fuel. The saving of fresh gas from the recycle is already in `c.ng`; no gas credit for S14 is added anywhere.
3. **Scenario exclusivity.** In the burn-on-site scenarios the term q_tg replaces the recycle claim for the same stream; in the recycle scenarios the INL export steam already contains the burned-cut heat. The two are alternatives. A test refuses any scenario in which the sum of claims on S14 exceeds its LHV.
4. **Reboiler duty.** No numerical chang<working folder>the reboiler gap and the STEAM_SYSTEM closure stay exactly as at `8dbfc6d` so the two integrated balance tables stay byte-identical. The register below documents the claim.
5. **Carbon.** The plant's S14 carbon is counted once in the plant carbon ledge<working folder>carbon returned to S03 stays in the loop, carbon in S15 leaves through S16. The unabated reference counts the same carbon in its stack. The abatement is the difference of the two stack totals; neither side gets a tail-gas term.

Register rows (in `Reference_Plant.csv`, row kind `register`, run `FS_B_REC_F`; computed from the flowsheet streams on export): `TG_LHV_S14_MW`, `TG_LHV_S03_MW`, `TG_LHV_S21_MW`, `TG_LHV_S15_MW`, `TG_CLAIM_RECYCLED_MW` (S03 + S21), `TG_CLAIM_BURNED_MW` (S15), `TG_CLAIM_STEAM_SYSTEM_MW` (Q_N2_BURN), `TG_CLAIM_SUM_OVER_S14` (must equal 1 within 1e-9 for species flows; the combustible species of S14 equal those of S03 + S21 + S15 because REC-CRYO conserves them). Test `eq_n_r_15_tail_gas_heat_is_claimed_once` assert<working folder>the species identity within 1e-9; Q_N2_BURN <= LHV(S15) (16.262 <= 16.567 at Case B, C); no scenario uses both q_tg and the recycle claim.

### C.3 Simulation changes that follow

1. `RefPlan<working folder>:stream_table(h2_kg_h)` returns the published rows I01 to I15 (published values times x) and three derived rows, all with a class colum<working folder>I16 combustion air (derived, <working folder>21 % O2; 6,648.5 kmol/h dry plus 62.7 kmol/h humidity at x = 1), I17 second steam addition to the reformer (155.17 kmol/h from the H balance, 153.70 from O; use the H value, C), I18 process condensate removed ahead of the PSA (1,772.9 kmol/h water = 31.94 t/h, C). Derived rows are marked `status = derived_unpublished` and never enter any cost or carbon equation.
2. New ninth results table `results/Reference_Plant.csv`, written by `nush2_cli export` (`engine/cli/src/main.rs` next to line 91), built by new `engine/plant/src/export_ref.rs`. Column<working folder>un_id` (`REF_IEAGHG_BASE` at published scale; `REF_IEAGHG_B_REC_F` at Case B scale), `row_kind` (`stream`, `balance`, `check`, `breakdown`, `register`), `id`, `name`, `from_unit`, `to_unit`, `t_c`, `p_mpa`, `kmol_h`, `kg_h`, `y_co2`, `y_co`, `y_h2`, `y_n2`, `y_o2`, `y_ch4`, `y_c2h6`, `y_c3h8`, `y_nc4h10`, `y_nc5h12`, `y_h2o`, `value`, `unit`, `published`, `rel_residual`, `class`, `pdf_page`, `printed_ref`, `equation`, `status`. A new file leaves the two integrated balance schemas untouched (the mass balance has no O2 column). Register the file in `scripts/compare_results.py` (`EXPECTED` line 39, `KEY_COLUMNS` line 115 with the value 3 for the key columns `run_id,row_kind,id`), `scripts/build_numbers.py` (schema set at line 70), `scripts/build_site.py` (publish list, line 53). Leave the legacy comparison step name list in CI unchanged (the new file has no legacy snapshot).
3. Final_Design_Basis tokens (rows `REF_*`): `REF_TAIL_GAS_LHV_MW`, `REF_FIRED_DUTY_MW`, `REF_SUPP_NG_T_H`, `REF_TAIL_GAS_SHARE_OF_FIRED`, `REF_STACK_CO2_KG_KG`, `REF_CO2_A_KG_KG`, `REF_CO2_B_KG_KG`, `REF_GAS_MMSCFD_EQ`, `REF_NET_POWER_MWE_SCALED`, `REF_STEAM_TO_NEIGHBOUR_MMBTU_H` (0), plus the historical `HIST_INL_SYSLINES_*` rows.
4. Wasm (WP6): a `reference` section in the reply (`gas_mmscfd_eq`, `supp_ng_t_h`, `tail_gas_lhv_mw`, `fired_duty_mw`, `stack_co2_kg_kg`, `net_power_mwe`, `steam_to_neighbour_t_h`) with bind paths for the reference view; `sol.emissions.baseline` is replaced by `baseline_central` in the live balance (defect D1).

### C.4 PFD changes

**Plant PFD (existing drawing).** Add the unit `n2_burn` between the cryogenic unit and the stack with streams S15 (cryogenic to n2_burn) and S16 (n2_burn to atmosphere), and a drawing-only heat tag `H3` from `n2_burn` to the power block ("heat to steam system, no unit consumes it"). Edit `frontend/index.html`, `frontend/units.json` (unit entry and stream from/to fields; the schema is documented in `pfd.js` lines 29 to 43 and `selftest.js` lines 325 to 335), and the tour text that says "the burner is not drawn" (`tour.js` lines 337 to 342) in WP6.

**Conventional-reference view (new).** A second drawing on its own tab of `index.html`, with a separate stream prefix `I` (IEAGHG numbers 1 to 15 become `I01` to `I15`; `I16` to `I18` derived). Units (ids prefixed `r_`), from PFD PDF p42: `r_desulph` sulphur adsorber, `r_prereformer`, `r_furnace` (steam reformer furnace with the convective coils and burners), `r_air_fan` (combustion air fan and air/flue gas exchanger), `r_stack` (induced draft fan and stack), `r_rwhb` reformer waste heat boiler, `r_hts` HT shift reactor, `r_swhb` shift waste heat boiler and syngas cooling train, `r_condsep` process condensate separator, `r_psa`, `r_recycle_comp` hydrogen recycle compressor, `r_drum` steam drum with `r_dea` deaerator, `r_turbine` condensing turbine and generator (power island), `r_condenser`. Drawn stream<working folder>I01 to I15 plus the electricity export (tag `E1`, 9.918 MWe, drawing-only). The drawing shows the tail gas line I08 to the burners as the primary fuel, I03 NG to the burners as supplementary fuel, I09 flue gas to the stack, I10 steam to the turbine (internal, labelled "no steam leaves the plant"), and I12 hydrogen. New file<working folder>frontend/refpfd.js` (reuses the selection and tooltip helpers of `pfd.js`, no model arithmetic), `frontend/refpfd.css`, `frontend/units_ref.json` (per uni<working folder>ummary, in, out, equations N.R.x, source pages, limitations; per strea<working folder>name, from, to, engine flag, class). `scripts/build_site.py` writes `data/ref_streams.json` and `data/ref_summary.json` from `Reference_Plant.csv`; `scripts/check_site.py` and `selftest.js` get the checks `auditRefStreams` (every drawn I-tag has a row and a units_ref entry, every `stream` row of the CSV is drawn or listed as derived) and a count check. The reference view needs no live recomputatio<working folder>the plant is fixed and scaled by x; the optional `reference` wasm section feeds the summary strip.

### C.5 Stream table shown on the reference view

Column<working folder>id, name, from, to, T, P, kmol/h, kg/h, main composition, class (published, derived), PDF page. Rows are those of A.4 plus I16 to I18. A footer lists the unresolved item "stream 14 is 19.9 t/h above the reformer steam implied by the H and O balances" (A.2 row 9).

---

## D. Total-CAPEX subsidy design (definition C)

### D.1 Definition

Definition C grants a uniform share f of the total first-installation capital of the integrated project, f from 0 to 100 percent. The share applies to every capital line the NUS-H2 project carries at first installation, with no equipment exclusions. Cost, break-even and the owner's cost per tonne use the incremental part (the six lines) and report the integrated base beside it.

Notation (S$M, first units, ledger basis): K_R reactor overnight capital; K_IHX; K_LIC; K_PI (incremental power island); K_CAP (capture, TCR basis); K_REC (recycle train, TCR-equivalent); K_incr = K_R + K_IHX + K_LIC + K_PI + K_CAP + K_REC (today's `total_capital_sgdm`); K_I the IEAGHG island at TCR basis (`smr_island(h_s).0`, 517.551 S$M at BOAK, <working folder>222.887 M EUR x 2.322); K_int = K_incr + K_I.

### D.2 Lines in the base and why

| Line | In the base of C | Reason |
|---|---|---|
| Reactor overnight capital | yes | first thing a grant pays (S.3c, S.6a); interest during construction is a financing result |
| IHX, licence, incremental power island | yes | year-0 items, no interest inside |
| Capture plant (TCR basis) | yes | on TCR basis it already contains interest, owner cost, spares, working capital and start-up; granting f x TCR includes f x its embedded interest, stated in the definition |
| Recycle train (TCR-equivalent) | yes | same reason; engine multiplies the TPC value by `TCR_OVER_TPC` |
| SMR island (TCR basis) | yes, reported on K_int; mirrored on the reference so it cancels from LAC (D.3) | the only capital shared with the unabated reference |
| Interest during construction line (`IDC`) | no | a progressive grant returns f x IDC as interest saved; granting it again double counts (wrong ways W1, W2 of S.5<working folder>W2 alone is 11.9 S$/t at BOAK 25 y, s = 0.5, definition A) |
| Decommissioning | no | an end-of-life liability, outside capital |
| Replacement units | no (labelled sensitivity S3 only) | convention 5: first installation only |
| Fixed O&M, raw water, chemicals | no | convention 7: OPEX does not change; island fixed O&M stays on the gross island |
| Working capital, depreciation, tax | no | working capital is inside the TCR lines; the model is real and pre-tax, so no depreciation base exists (searched `engine/plant/src` and `parameters.toml`) |

### D.3 The SMR-island decision and the labelled sensitivity

Under a grant on the island on the NUS-H2 side only, LAC falls by f x K_I x CRF(life) x 1e6 / <working folder>43.46 S$/t at f = 0.5 and 86.92 S$/t at f = 1 (BOAK 25 y, <working folder>517.551 x 0.0936788 x 1e6 / 557,767), turning the cost negative above f of about 0.70. The island is identical in the reference, so that fall measures a hydrogen-plant subsidy and has no abatement meaning.

**Primary (C-LL, like for like).** The island share is mirrored on the reference and cancels exactly from the abatement cost. The abatement cost and break-even use f x K_incr. Gross government expenditure and private investment are reported on K_int with the incremental part beside them.

**Labelled sensitivity S1 (project side only).** The literal readin<working folder>f x K_I granted on the NUS-H2 side, the reference unsubsidised. Reported in its own column `cost_net_project_side_s_per_t`, marked "includes a windfall on shared equipment". Mechanic<working folder>the grant on the island is a year-0 negative line on the NUS-H2 side (ledger `NetBlock.grant_credit` annualised once with CRF(life); DCF a year-0 `GRANT_ISLAND` line), added only when share is not zero. Other labelled sensitivities, kept as columns or run id<working folder>S2 definition A (reactor only), which stays as a secondary definition; S3 grants on replacement units with the unused-life credit scaled by (1 - f); S4 grant at close (existing timing).

**Island composition (a flag, with no change of the default).** The island TPC includes the IEAGHG power island (20.124 M EUR, 11.8 % of TPC), carried on both sides while the NUS-H2 plant also carries its own incremental power island and does not use the island turbine. A consistent alternative removes the power-island share from the NUS-H2 side only. With a pro-rata TCR share (<working folder>20.124 / 170.952 x 222.887 = 26.24 M EUR = 60.9 S$M) the LAC would fall by 10.2 S$/t (C, DCF only). Defaul<working folder>unchanged, with the switch `SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND` and a sensitivity row (open question question 3).

### D.4 Share grid and sweep

* The share range is 0 to 1 (`SUBSIDY_SHARE_MAX` already 1.0; range checks, bracket and knots already accept s = 1).
* New entry `SUBSIDY_SWEEP_STEP = 0.05`. Sweep nodes are k x 0.05 for k = 0..20 (21 shares; 0, 0.25, 0.5, 0.75 are nodes, so the existing matrix is a subset). Generate nodes as `k as f64 * step` from the integer k to avoid drift. Definition C uses the sweep. Definitions A and B keep the four-point matrix (`SUBSIDY_SHARE_GRID_1..4`), so their rows are unchanged.
* Row coun<working folder>today 4 stages x 2 lives x 2 definitions x 2 timings x (4 shares x 2 methods + 3 break-even) = 352 rows. Afte<working folder>A and B 352 rows unchanged plus C 4 x 2 x 1 x 2 x (21 x 2 + 3) = 720 rows, 1,072 rows in total (C).
* Token tags `S{round(100 x share):02d}` stay unique at 5 percent steps (`S00` to `S100`).
* The relative gap limit between ledger and DCF is restated against the gross L5 cos<working folder>gap_share_of_l5 = gap / cost_gross`, limit `SUBSIDY_GAP_LIMIT` 0.01 (value unchanged). At share 0 gross equals net, so the zero-share rows are unchanged. Reaso<working folder>near f = 1 the net cost is about zero (C-LL BOAK 25 y f = 1: -0.044 S$/t) or small (BOAK 60 y f = 1: +2.914 S$/t with an absolute gap of 0.0687 S$/t, which is 2.4 percent of the net cost and 0.04 percent of the gross cost of 178.5 S$/t at the old reference, C).
* `SUBSIDY_DEFAULT_DEFINITION` becomes a code 0 = A, 1 = B, 2 = C (new reader `read_code(r, id, max)` beside `read_switch`), value 2. `default_case` therefore marks C rows.

### D.5 Report quantities

Existing columns that sta<working folder>cost_gross_s_per_t` (unsubsidised abatement cost, equals the share-0 row and `AC_L5`), `cost_net_s_per_t` (subsidised private cost net of the grant and the interest it saves), `grant_sgdm`, `private_capital_sgdm`, `interest_*`, `state_pv_sgdm`, `public_outlay_s_per_t`, `gap_*`, `target_met`, break-even rows.

New columns appended to `SUBSIDY_HEADER` (35 to 43 columns) before the meta column<working folder>island_capital_sgdm` (K_I), `total_capital_integrated_sgdm` (K_int), `grant_incremental_sgdm` (f x K_incr), `grant_integrated_sgdm` (f x K_int, gross government expenditure, nominal), `state_pv_integrated_sgdm` (year-0 value of the integrated grant), `private_investment_integrated_sgdm` (K_int - f x K_int), `private_investment_incl_idc_sgdm` (that plus `interest_private_sgdm`, the private IDC on the reactor; the TCR-basis lines carry their own interest already), `public_outlay_integrated_s_per_t`, `cost_net_project_side_s_per_t` (S1; blank for A and B). Government expenditure is reported nominal and as year-0 value, labelled.

### D.6 Double-counting safeguards (each has a named test)

1. No grant on the IDC line and no f x f_IDC x K_R ter<working folder>q_s_5b_no_second_idc_credit_under_c`.
2. The island enters the base once and is mirrored onc<working folder>q_s_10a_island_cancels_when_mirrored` (the C-LL cost equals the cost computed with the island granted on both sides in the DCF, within 1e-6 S$/t).
3. S1 closed for<working folder>q_s_10b_project_side_island_lowers_cost_by_the_closed_form` (f x K_I x CRF x 1e6 / A on the ledger; the DCF effect equals it because the island grant is a year-0 flow).
4. Replacement units are never granted in the primary; S3 scales the unused-life credit by (1 - f): `eq_s_5c_replacement_credit_is_scaled_by_one_minus_f`.
5. Fixed O&M and decommissioning stay on the gross value<working folder>new test `eq_s_8c_c_leaves_opex_and_decom_alone` (every non-capital ledger line equal to the share-0 value at every sweep node).
6. Grant at close beyond the reactor spen<working folder>the excess is paid at year 0 (`grant_year0`); the ledger form has a slope kink at u = 1 (new test `eq_s_9a_the_cost_of_c_bends_once_past_the_reactor_spend`, at share K_R / K_incr = 0.699 at BOAK 25 y, <working folder>742.767 / 1,063.188; the existing `eq_s_9a_the_cost_falls_strictly_with_the_share_and_bends_one_way` extends to C).
7. A zero share pushes no GRANT line and sets no non-default field (`DcfResult` derives `PartialEq` including lines): `eq_s_9b_a_zero_share_changes_nothing_bit_for_bit` already loops `Definitio<working folder>:ALL` and so covers C.
8. Legacy `rx_mult` co-funding stays independen<working folder>q_s_8_a_grant_leaves_the_rx_mult_lever_alone`.
9. Overlaps of the licence and the incremental power island with INL's overnight cost and with the island's own power island are footnoted in the table notes; no deduction is made because their size is unavailable.

### D.7 Files to change (subsidy track)

`engine/plant/src/subsidy.rs` (enum `Definition` add `C`, `ALL`, `id()`; `Eligibility` flags for C; `SubsidyParams.swee<working folder>Vec<f64>`, `eligible_c`, `read_code`; `grant_plan` and `GrantPlan` new reporting fields; `subsidy_eval_at`, `MethodEval` fields; unit tests L1048 to L1140). `engine/plant/src/economics.rs` (`CostItems.smr_island_capital`; `NetBlock` S1 credit gated on share not zero). `engine/plant/src/dcf.rs` (`GRANT_ISLAND` line for S1, the power-island switch for the island on the NUS-H2 side, both gated). `engine/plant/src/export.rs` (`SUBSIDY_HEADER`, scenario loop L4130 to L4209, `Definitio<working folder>:ALL` loop L4152, default case L4162, sweep loop L4189, `FLOOR_BREAKEVEN_` rows L1392 to L1416). `engine/vv/src/harness.rs` (L330 to 345, L645 to 674: exhaustive matches and `IND_BREAKEVEN_` rows). `engine/data/parameters.toml` (`SUBSIDY_ELIG_C_{REACTOR,IHX,LICENCE,POWER_ISLAND,CAPTURE,RECYCLE}` = 1 each, `SUBSIDY_SWEEP_STEP`, default code). Test<working folder>ngine/plant/tests/m10_subsidy.rs` (`ctx_s1` L56 to 68, `x_e` L132 to 136, `eq_s_1`, `eq_s_5a`, `eq_s_6a`, `eq_s_7c` to `7e`, `eq_s_8`, `eq_s_9a`, `eq_s_9b`, table tests L2067 and L2222 row counts, L2076 to 2080 definition parsing where any label other than "A" maps to B, L2091 `default_case`, L2187 `grid.contains`, break-even tests L1477 to 1500), `engine/plant/tests/m11_carbon.rs` (L1565 to 1600, L1643), `engine/vv/tests/benchmark_rows.rs` (L257 to 263). Script<working folder>cripts/build_numbers.py` (L350 to 357 `subsidy_tag`, L418 to 463 token loops for A, B and now C), `math/verify/sympy_subsidy.py` (L663 to 672 check C10d stays valid because B stays). Regenerated artefact<working folder>ults/Subsidy_Scenarios.csv`, `results/Final_Design_Basis.csv`, `results/Validation_Results.csv`, `build/numbers_tokens.csv`, `build/numbers.json`.

Definition B is kept as a labelled legacy definition (not deleted). Its six registry entries stay, so `PARAMS_COMPLETE` keeps passing, and the derivation S.1 text is corrected to the registry membership (reactor, IHX, licence, capture, recycle).

### D.8 Equation blocks to rewrite in `docs/derivation/02_cost_and_subsidy.md`

Line numbers as at `8dbfc6d`.

| Block | Lines | Change |
|---|---|---|
| Variables table, equations list | L13 to L93 | add K_I, K_int, G_incr, G_int, f_sweep, `SUBSIDY_SWEEP_STEP`, `SUBSIDY_ELIG_C_*`; replace the non-registry name `SUBSIDY_ELIGIBLE_B_ITEMS` (L1006, L1029) |
| 2.2 Conventions (decision D7) | L349 to L361 | add convention 9: total CAPEX definition (ledger basis, island treatment, TCR lines hold their own interest); state that decommissioning, replacements and working capital are outside |
| S.1 eligible capital | L378 to L411 | rewrit<working folder>definition C as primary, A secondary, B legacy; fix the B membership mismatch (doc says power island, registry says capture and recycle, admitted at L409); the recycle train is on a TCR-equivalent basis (the engine multiplies its TPC value by `TCR_OVER_TPC`) |
| S.2 grant | around L412 to L440 | G = f x eligible for C; add G_int |
| S.5b wrong ways | L634 to L653 | add W5 island on one side only (windfall), W6 replacement grants without scaling the unused-life credit |
| S.7e break-even | around L823 | closed forms with eligible = K_incr; reachability when f* > share_max |
| S.8 reported quantities | L892 to L905 | new columns (D.5); nominal against year-0 value |
| S.9a, S.9b limiting cases | around L909 to L955 | f = 1 and the kink past the reactor for C |
| New S.10a, S.10b | after S.9 | island like for like (cancels) and project side only |
| Remaining B mentions | L72, L360, L365 to L366, L431, L570, L663, L697, L823, L909, L929, L955 | align with C primary |
| `docs/ABATEMENT_SUBSIDY_LOG.md` | D7 (L44), D7b (L45), D11 (L50) | supersede with D14; add D13 |
| `docs/ABATEMENT_SUBSIDY_REFERENCES.md` L43 to L44; `docs/FINAL_DESIGN_DECISIONS.md` L533 to L545 | | shares, grid and column table |
| `docs/ABATEMENT_SUBSIDY_REVIEW.md` L13, L132 | | historica<working folder>add a "superseded by D14" note, no rewrite |

Docs for the reference trac<working folder>new `docs/derivation/03_reference_plant.md` (N.R.1 to N.R.17 in the project block forma<working folder>what it calculates, variables, where it comes from, substitution, intermediate calculations, final formula, implementation, validation test); edit `01_carbon_and_abatement.md` at the variable block L202 to L208, N.7.1 to N.7.4 (L688 to L780; N.7.2 gains "the central result no longer needs eta_<working folder>the published fired duty replaces it; `REF_FURNACE_EFF` and `REF_STEAM_DUTY` are unavailable and not created", closing open issue 2), the boundary text L781 (rule 1 restated), N.9.2 (L888), N.9.5 (L959), the table row L132 ("K.1 to K.3 ... N.7.3 (legacy)"), and L1126 (the floor and slope statements). `docs/VALIDATION_MATRIX.md` rows for N.R.x; `README.md` L17 and `docs/FINAL_DESIGN_DECISIONS.md` L19, L55, L489 (stale INL 9.21 headline text).

---

## E. Work packages

### E.1 Regression guards (every package ends with the relevant ones; WP7 runs all)

| Gate | Check | Command or test | Pass |
|---|---|---|---|
| G0 | Frozen snapshot of the current results | before WP0: copy `results/*.csv` (1.6 MB) to `engine/vv/data/pre_reference_results/` and commit | exists |
| G1 | Zero subsidy | `eq_s_9b_a_zero_share_changes_nothing_bit_for_bit`, `eq_s_9b_zero_subsidy_reproduces_the_committed_ledger_and_dcf`, `eq_s_8_the_committed_table_reproduces_the_economic_ledger_at_a_zero_share`, and `subsidy_zero_share_equals_economic_ledger` in `build_numbers.py` (all loop `Definitio<working folder>:ALL`, so C is included) | green |
| G2 | Legacy basis | `cargo run --release -p nush2_cli -- export --out <dir> --basis legacy`, then `python scripts/compare_results.py engine/vv/data/legacy_results <dir> --subset --names <the seven CI names> --allow-cell Validation_Results.cs<working folder>PARAMS_COMPLET<working folder>conditions --allow-cell Validation_Results.cs<working folder>EXPORT_BYTE_IDENTICA<working folder>conditions` | exit 0 |
| G3 | Independent basis before the change | WP3 uses an in-process test (`ParamSe<working folder>:set("REF_BASIS", 0.0)`, then `tables_with`) against `include_str!` of `engine/vv/data/pre_reference_results/`. WP4 adds the CLI flag `--ref-basis` with the values `reconstructed` and `historical` (`params_for` in `engine/cli/src/main.rs`, sets `REF_BASIS`) and the full file comparison with `compare_results.py --subset` for all eight tables, allowing only new rows and the new `Subsidy_Scenarios.csv` columns and rows of C | exit 0; rows of definitions A and B of `Subsidy_Scenarios.csv` equal on the old columns |
| G3b | Gate outcomes | `nush2_cli export` refuses to write when the gate rows differ from `gate<working folder>:RECORDED` (`engine/plant/src/gates.rs` line 295; `engine/cli/src/main.rs` `export`). New `REF_` validation rows must not start with a gate prefix listed in `gates.rs`. After WP3 and WP4 run `cargo test -p nush2_plant flip_requires_gate` and one export; if a gate flips, stop and report instead of editing `RECORDED` | no flip |
| G4 | Integrated balances untouched | `Integrated_Mass_Balance.csv` and `Integrated_Energy_Balance.csv` byte-identical to the snapshot on both bases (default basis too) | identical |
| G5 | Default basis against committed | `export` then `compare_results.py results <dir>` | exit 0 after WP7 regenerates `results/` |
| G6 | Text guards and gates | `python scripts/check_clean.py`; `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `python scripts/check_site.py` | clean |

The eight results tables and what is expected to change on the default basi<working folder>Economic_Ledger.csv` (GAS, LOST_STEAM, BASE_POWER lines, their text and sources; per-tonne values through the new A_net; DCF reference-side lines), `Emissions_Ledger.csv` (IEAGHG rows -4.1e-5 relative; INL, Cho and floor rows unchanged), `Final_Design_Basis.csv` (new `REF_*` and `HIST_INL_*` rows, reference rows re-valued), `Integrated_Energy_Balance.csv` and `Integrated_Mass_Balance.csv` (no change), `Sensitivity_Results.csv` (values move through the reference; the BASELINE INL and Cho corners unchanged), `Subsidy_Scenarios.csv` (new columns, C rows, reference-dependent costs), `Validation_Results.csv` (new `REF_` rows, `IND_` rows re-pointed). The ninth table `Reference_Plant.csv` is new.

### E.2 Packages

| WP | Title | Owns these files (no other package edits them while it runs) | Depends on | Expected size |
|---|---|---|---|---|
| WP0 | Foundation and registry (serial, first) | `engine/data/parameters.toml`; `engine/props/src/units.rs` (`KCAL_KJ`); `engine/plant/src/refplant.rs` (ne<working folder>tructs and `read` only); `engine/plant/src/lib.rs`; `engine/plant/src/anchored.rs` (field `refp`, read in `build`); `engine/plant/src/subsidy.rs` (enum `C`, flags, sweep vector, `read_code`, default code); `engine/plant/src/economics.rs` (`CostItems.smr_island_capital`, `SystemBasis`, `CostQuery.system_basis`); `engine/vv/src/harness.rs` and `engine/plant/tests/m10_subsidy.rs` (compile-guard matches only); `engine/plant/tests/no_internal_pointers.rs` (`PUBLIC_MD` += `docs/derivation/03_reference_plant.md`) | G0 | about 1,700 lines of TOML, 300 lines of Rust, 6 tests (`eq_n_r_0_registry_reads_every_reference_entry`, `PARAMS_COMPLETE`, default-definition code, sweep nodes, S.1 membership of C, `read_code` range). Nothing numerical change<working folder>G1, G2, G4 must pass. |
| WP1 | Reference core | `engine/plant/src/refplant.rs` (logic), `engine/plant/tests/m12_reference_plant.rs` (new), `docs/derivation/03_reference_plant.md` (new), `math/verify/sympy_refplant.py` (new, optional) | WP0 | about 700 lines of Rust, 17 tests `eq_n_r_1` to `eq_n_r_12`, `eq_n_r_15` core, 500 lines of doc. Parallel with WP2. |
| WP2 | Subsidy core for C | `engine/plant/src/subsidy.rs`, `engine/plant/src/dcf.rs`, `engine/plant/src/economics.rs` (NetBlock only), `engine/plant/tests/m10_subsidy.rs`, `docs/derivation/02_cost_and_subsidy.md`, `math/verify/sympy_subsidy.py` | WP0 | about 450 lines of Rust, about 22 tests changed or added (names in D.6 and D.7), 200 lines of doc. Parallel with WP1. G1 must stay green. |
| WP3 | Reference integration | `engine/plant/src/abatement.rs`, `engine/plant/src/economics.rs` (`cost_items` reference lines), `engine/plant/src/emissions.rs`, `engine/plant/src/anchored.rs` (baseline fields), `engine/plant/src/sens.rs`, `engine/plant/tests/m11_carbon.rs` | WP1 and WP2 (shares `economics.rs` with WP2) | about 350 lines changed, tests `eq_n_r_13`, `14`, `16`, `17`, G3 test, floor and baseline tests re-pointed (`m11_carbon.rs` L409 to 421, 492 to 496, 1083 to 1163, 1415 to 1580, 1624 to 1652). G3 and G4 must pass with `REF_BASIS = 0`. |
| WP4 | Exports, validation rows, scripts | `engine/plant/src/export.rs`, `engine/plant/src/export_ref.rs` (new), `engine/cli/src/main.rs`, `engine/vv/src/harness.rs`, `engine/vv/tests/benchmark_rows.rs`, `scripts/build_numbers.py`, `scripts/compare_results.py`, `scripts/derivations_tools.py` (L1070 to L1073 re-implementation of the three reference lines), `engine/plant/tests/m10_subsidy.rs` (table tests only) | WP3 | about 500 lines of Rust, 14 `REF_` validation rows, `Reference_Plant.csv` about 150 rows per run |
| WP5 | PFD and site | `frontend/refpfd.js`, `frontend/refpfd.css`, `frontend/units_ref.json`, `frontend/index.html`, `frontend/units.json`, `frontend/pfd.js` (hooks only), `frontend/selftest.js`, `scripts/build_site.py`, `scripts/check_site.py`, `docs/figures/` | schema only (start after WP1); final check after WP4 | about 600 lines (SVG 400, JS 150, JSON 300) |
| WP6 | Labels, live reply, tour | `engine/wasm/src/solve.rs`, `engine/wasm/src/registry.rs`, `engine/wasm/src/tests.rs`, `frontend/views.js`, `frontend/tour.js`, `README.md`, `docs/FINAL_DESIGN_DECISIONS.md` | WP3 | about 120 lines of text and cod<working folder>defects D1 (`solve.rs` L2059 to 2061, `tour.js` L465, L476), D2 (KPI labels say gross or net, `registry.rs` L861 to 866), D3 (stale labels at `views.js` L384, 385, 415, 939, `registry.rs` L567, 568, `README.md` L17), the `reference` section of the reply |
| WP7 | Docs, regeneration and final guard | `docs/derivation/01_carbon_and_abatement.md`, `docs/ABATEMENT_SUBSIDY_LOG.md`, `docs/ABATEMENT_SUBSIDY_REFERENCES.md`, `docs/VALIDATION_MATRIX.md`, `docs/ABATEMENT_SUBSIDY_REVIEW.md`, `results/*`, `build/*`, `math/generated/*`, `math/eq_register.csv` if cards are added | WP4, WP5, WP6 | docs about 400 lines; regenerated CSVs; runs G1 to G6 |

Parallel groups, by disjoint file set<working folder>WP1 and WP2 together after WP0. WP5 can start next to WP3. WP4, WP5 and WP6 run together after WP3 (WP5 finishes its data hookup after WP4). WP3 waits for both WP1 and WP2 because it shares `economics.rs` and `anchored.rs` with them. Packages never run in parallel on `parameters.toml` (only WP0 edits it).

Dropped from scope on purpos<working folder>the report and slide deck (legacy-basis deliverables, no mention of definition B or the grid), the plant flowsheet, the MEA and cryogenic models, deleting definition B, changing `RHO_H2_NORMAL`, and any steam-table calculation.

---

## F. Open questions for the owner (one shot)

Only questions that sources and sensible defaults cannot settle. Each has the default I apply if no answer arrives.

1. **Island treatment in the headline subsidy cost.** Should the headline abatement cost under definition C mirror the island grant on the reference (like for like, the island cancels), or grant the island on the NUS-H2 side only? The literal reading lowers the cost by up to 86.9 S$/t at 100 % (BOAK 25 y, C) and turns it negative above about 70 %, which is a hydrogen-plant subsidy with no abatement meaning. **Defaul<working folder>like for like as the headline; the project-side reading is reported in its own labelled column.**
2. **Electricity value for the reference's 9.918 MWe export.** The report uses one price of 80 EUR/MWh (S$0.186/kWh at the repository factor 2.322) for export and import. The repository price S$0.20/kWh is an unverified author assumption, and it moves L5 by +18.3 S$/t at S$0.20 against +16.5 S$/t at S$0.186 (C). Is there a sourced Singapore export price to use instead? **Defaul<working folder>keep S$0.20/kWh for both directions (ratio 1.0, as the report does) and show the report-equivalent price as a sensitivity.**
3. **Power island inside the island.** The island of the report includes its 12 MWe condensing turbine (about 60.9 S$M pro rata, C). The NUS-H2 side carries it without using it and also carries its own power island. Remove that share from the NUS-H2 side only? It would lower LAC by about 10.2 S$/t (C). **Defaul<working folder>keep the island identical on both sides, with the switch `SMR_ISLAND_NUSH2_EXCLUDES_POWER_ISLAND` and a sensitivity row reporting the effect.**

Settled without asking (recorded in D13 and D14): the IEAGHG reconstruction is the central reference of the independent basis; NUS-H2 gas, electricity and steam prices stay separate assumptions; definition B stays as a labelled legacy definition; total CAPEX is on the ledger basis with IDC reported as financing and `private_investment_incl_idc` shown beside it; the licence and power-island overlaps are footnoted without deduction; the 1 percent gap limit is stated against the gross cost; the plant keeps `RHO_H2_NORMAL` 0.08988 and the reference is mass based; stream 14 steam stays unexplained because nothing depends on it; the blocking 578,000 t/y drift gate stays on the legacy basis only.

---

## G. Reproducibility

`working folder/spec_check.py` (output `spec_check_out.json`) recomputes every number marked C or "mine" above with `python -I`. `ieaghg_balances.py` (streams audit) was re-run for the element balances marked "audit". Page images read by m<working folder>PDF p16, p34, p44 (two crops), p45 and p48; other pages were read by the auditors and cross-checked between at least two of them (p17, p18, p54, p255, p256).
