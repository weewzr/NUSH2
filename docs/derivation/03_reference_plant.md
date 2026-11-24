# Derivation 03: the IEAGHG 2017-02 Base Case reference plant

This file derives the conventional reference plant of the independent carbon basis (decision D18): the unabated steam methane reformer of IEAGHG Report 2017-02, *Techno-Economic Evaluation of SMR Based Standalone (Merchant) Hydrogen Plant with CCS*, Base Case. It is reconstructed from the report's own published balances. Every page number below is a PDF page of that report, which is public on the IEAGHG publications page; the report is not stored in this repository.

Result labels at the end of each equation: **published** (P, read from the report with its page), **calculated** (C, obtained from published values by the equation shown), **assumed** (A, a convention or constant that the report does not state) and **unavailable** (U, not in the report and not derivable).

Status of this file. Equations N.R.1 to N.R.12 and N.R.15 are implemented in `engine/plant/src/refplant.rs` and tested in `engine/plant/tests/m12_reference_plant.rs`. Equations N.R.13, N.R.14, N.R.16 and N.R.17 (the two system lines, the K.18 terms, the decomposition and the regression equation) belong to the integration step and are listed here only so the numbering is complete. Until that step, the registry switch `REF_BASIS` is 0 and no results table reads the reference module: every number in `results/` is unchanged by this file.

---

# Reference Frame

## Variables

| Symbol | Definition | Units |
|---|---|---|
| $k$ | Stream number of the published Heat and Material Balance, 1 to 15 (PDF p44 and p45) | $1$ |
| $n_k$ | Molar flow of stream $k$ | $\mathrm{kmol\ h^{-1}}$ |
| $m_k$ | Mass flow of stream $k$ | $\mathrm{kg\ h^{-1}}$ |
| $y_{k,i}$ | Mole fraction of species $i$ in stream $k$, wet | $1$ |
| $\dot n_{k,i}$ | Species flow, $n_k y_{k,i}$ | $\mathrm{kmol\ h^{-1}}$ |
| $c_i,\ h_i,\ o_i,\ a_i$ | Carbon, hydrogen, oxygen and nitrogen atoms per molecule of species $i$ | $1$ |
| $C_k$ | Carbon flow of stream $k$ | $\mathrm{kmol\ C\ h^{-1}}$ |
| $R$ | PSA hydrogen recovery, product plus recycle over feed hydrogen | $1$ |
| $LHV_i$ | Lower heating value of species $i$ (engine constants: H2 241.8, CO 283.0, CH4 802.3) | $\mathrm{kJ\ mol^{-1}} = \mathrm{MJ\ kmol^{-1}}$ |
| $LHV_{NG},\ HHV_{NG}$ | Heating values of the natural gas (46.50 and 51.473) | $\mathrm{MJ\ kg^{-1}}$ |
| $LHV_{H_2}$ | Lower heating value of hydrogen (119.96) | $\mathrm{MJ\ kg^{-1}}$ |
| $Q_{tail}$ | Heat released by burning the PSA tail gas (stream 8) | $\mathrm{MW}$ |
| $Q_{NG,supp}$ | Heat released by the supplementary natural gas (stream 3) | $\mathrm{MW}$ |
| $Q_{fired}$ | Burner heat release, $Q_{tail}+Q_{NG,supp}$ | $\mathrm{MW}$ |
| $Q_{rad}$ | Radiant duty of the reformer furnace (82.63 MMkcal/h) | $\mathrm{MW}$ |
| $\varphi_{rad}$ | Radiant duty over fired duty, an indicator | $1$ |
| $\dot n_{CO_2,stack},\ \dot m_{stack}$ | CO2 leaving in the flue gas (stream 9) | $\mathrm{kmol\ h^{-1}}$, $\mathrm{t\ h^{-1}}$ |
| $\varepsilon_C$ | Carbon residual, natural gas to stack | $1$ |
| $A_1,\ A_2,\ A_3$ | CO2, CO and CH4 of the tail gas | $\mathrm{kmol\ h^{-1}}$ |
| $B_1,\ B_2$ | CO2 and hydrocarbon carbon of the supplementary natural gas | $\mathrm{kmol\ h^{-1}}$ |
| $\rho$ | Rescaling factor that makes the parts add to the stack | $1$ |
| $V_N$ | Normal volume per kmol (22.414, assumed) | $\mathrm{Nm^3\ kmol^{-1}}$ |
| $e_{ref}$ | Unabated CO2 intensity of the reference | $\mathrm{kg\ kg^{-1}}$, $\mathrm{kg\ Nm^{-3}}$ |
| $E_{HHV}$ | Natural gas energy on the higher heating value | $\mathrm{GJ\ h^{-1}}$ |
| $n_{ref}$ | The same energy in the engine's gas unit | $\mathrm{MMSCFD}$ |
| $W_{gross},\ W_{H_2},\ W_{aux},\ W_{net}$ | Gross power, hydrogen plant load, auxiliaries and export power | $\mathrm{MWe}$ |
| $x$ | Scale factor, plant hydrogen over reference hydrogen | $1$ |

## Equations

| Id | Equation | Label |
|---|---|---|
| N.R.1 | $\dot n_{k,i}=n_k y_{k,i}$; $C_k=\sum_i c_i\dot n_{k,i}$ | C |
| N.R.2 | $R=(n_{12}+n_{13})/\dot n_{7,H_2}$; tail $\dot n_i=\dot n_{7,i}$ for $i\neq H_2$, $(1-R)\dot n_{7,H_2}$ for H2 | C |
| N.R.3 | $Q_{tail}=\sum_{i\in\{H_2,CO,CH_4\}}\dot n_{8,i}LHV_i/3600$ | C |
| N.R.4 | $Q_{NG,supp}=m_3 LHV_{NG}/3600$ | C |
| N.R.5 | $Q_{fired}=Q_{tail}+Q_{NG,supp}$; test form $m_{NG,supp}(Q')=\max(0,Q_{fired}-Q')\,3600/LHV_{NG}$ | C (test form A) |
| N.R.6 | $\varphi_{rad}=Q_{rad}/Q_{fired}$, $Q_{rad}=82.63\cdot 10^{6}\cdot 4.1868/(3.6\cdot 10^{6})$ | C, indicator |
| N.R.7 | $\dot n_{CO_2,stack}=n_9y_{9,CO_2}$; $\dot m_{stack}=\dot n_{CO_2,stack}M_{CO_2}/1000$; $\varepsilon_C=\dot n_{CO_2,stack}/C_1-1$ | C |
| N.R.8 | $A_1=\dot n_{8,CO_2}$, $A_2=\dot n_{8,CO}$, $A_3=\dot n_{8,CH_4}$, $B_1=\dot n_{3,CO_2}$, $B_2=C_3-B_1$, $\rho=\dot n_{CO_2,stack}/(A_1+A_2+A_3+B_1+B_2)$ | C |
| N.R.9 | $e_{ref}=1000\,\dot m_{stack}/m_{12}$ (kg/kg); $e_{ref,N}=1000\,\dot m_{stack}/(n_{12}V_N)$ (kg/Nm3) | C |
| N.R.10 | $Q_{LHV}=m_1LHV_{NG}/3600$; $E_{HHV}=m_1HHV_{NG}/1000$; $n_{ref}=x\,E_{HHV}\cdot 24/(HHV_{eng}\cdot 1.05505585)$ | C |
| N.R.11 | $W_{net}=W_{gross}-W_{H_2}-W_{aux}$, $W_{aux}=(19+347)/1000$; revenue $=W_{net}\,t\,p$ | C |
| N.R.12 | $x=\dot m_{H_2,plant}/m_{12}$; extensive quantities times $x$, intensive ones unchanged | C |
| N.R.13 | System lines LOST_STEAM and BASE_POWER from the reference steam (0) and power | integration step |
| N.R.14 | K.18 terms from the same signed inputs | integration step |
| N.R.15 | $LHV(S14)=LHV(S03)+LHV(S21)+LHV(S15)$; the steam-system claim $\le LHV(S15)$ | C |
| N.R.16 | Decomposition N.9.2 on the reconstructed reference | integration step |
| N.R.17 | With `REF_BASIS` = 0 every independent-basis output equals the frozen snapshot; the legacy basis equals the legacy snapshot | integration step |

## Key assumptions and parameter ranges

| Item | Value | Label | Why |
|---|---|---|---|
| Normal volume $V_N$ | 22.414 Nm3/kmol (0 C, 101.325 kPa) | A | The report defines no normal state; $4{,}461.5\times 22.414=100{,}000.06$ Nm3/h reproduces Table 2 (PDF p16) |
| Product and recycle purity | streams 12 and 13 printed 0.9999+ H2, taken as pure | A | Rounding of the printed fraction |
| CO in the flue gas | printed as a limit, 30 mg/Nm3 max (stream 9, note 2) | U, read as 0 | No mole fraction is printed |
| Gauge or absolute pressure | not marked on PDF p44 and p45 | U | Only displayed |
| Calorie | 4.1868 kJ/kcal, the International Table calorie (`KCAL_KJ`) | A | Converts the MMkcal/h duties |
| Dry air | 21 percent O2, 79 percent N2 | A | Only for the derived air row I16 |
| Furnace thermal efficiency, convective duties, combustion air | none | U | The report prints none; none is stored or derived |
| Tolerances of the checks | listed with N.R.2, N.R.7, N.R.9 and N.R.11 | A | Each about twice the band of the printed digits |

## Published inputs (registry block `REF_*`)

| Quantity | Value | PDF page and table |
|---|---|---|
| Streams 1 to 12: T, P, kmol/h, kg/h and composition | as printed | p44, Heat and Material Balance, Base case, rev. 1 (June 2015) |
| Streams 13 to 15 | as printed | p45 |
| H2 product | 100,000 Nm3/h, 8.994 t/h | p16, Table 2 |
| NG LHV | 46.50 MJ/kg (Table 2); 46.502 MJ/kg (specification) | p16; p255, Table 1, Sheet 9 of 24 |
| NG HHV | 51.473 MJ/kg | p255, Table 1, Sheet 9 of 24 |
| H2 LHV | 119.96 MJ/kg | p16, Table 2; p256, Table 2, Sheet 10 of 24 |
| Energy input, energy in product | 394.77 MW, 299.70 MW | p16, Table 2 |
| Gross power, H2 plant load, export | 11.500, 1.216, 9.918 MWe | p16, Table 2 |
| Power island and utilities loads | 19 kW and 347 kW | p48, Estimated Utility Consumptions |
| Radiant duty | 82.63 MMkcal/h | p54, equipment list sheet 5 of 8 |
| Utility-sheet fuel, HP steam to the power island | 48.2 MMkcal/h, 45.9 t/h | p48 |
| Steam across the battery limit | 0 | p34, Overall Block Flow Diagram; p48, total row |
| Electricity price, hours, revenue | 80 EUR/MWh; 8,322 h/y; 6,603,008 EUR/y | p12, Table 1; p229, Sheet 92 of 112; p18, Table 4 |
| PSA recovery | typically 85 to 90 percent | p37, Sheet 14 of 112 (also p6) |
| Power island total plant cost | 20.124 M EUR of 170.952 | p222, Sheet 85 of 112 |
| Discount rate | 8 percent | p8 (the legacy registry row `DISCOUNT` cites p12; its text is kept unchanged, critique C01) |

---

# Equations N.R.1 to N.R.12

## N.R.1: species and carbon flows of the published table

**What it calculates.** The flow of each species and of carbon in every published stream.

**Where it comes from.** A mole fraction is the share of the molecules; multiplying the total molar flow by it gives the molar flow of that species. Each molecule carries a fixed number of carbon atoms, so the carbon flow is the sum of species flows weighted by their carbon atoms.

**Derivation.** If a stream carries $n_k$ kmol of molecules per hour and a fraction $y_{k,i}$ of them are species $i$, then $\dot n_{k,i}=n_ky_{k,i}$. Methane has one carbon atom, ethane two, propane three, n-butane four, n-pentane five, and CO2 and CO one each. Hence $C_k=\sum_i c_i\dot n_{k,i}$.

**Substitution.** For the natural gas the carbon per molecule is $0.02+0.89+2(0.07)+3(0.01)+4(0.001)+5(0.0001)=1.0845$. Stream 1: $C_1=1{,}696.6\times 1.0845=1{,}839.963$ kmol C/h. Stream 2: 1,578.815. Stream 3: 260.714. Stream 7: $6{,}596.9\times(0.1627+0.0464+0.0302)=1{,}578.638$. Stream 8: $2{,}106.3\times(0.5095+0.1454+0.0945)=1{,}578.461$.

**Limits.** The printed fractions carry four decimals and add to 1 within 1e-4. Stream 1 is 0.4 kmol/h above streams 2 and 3 in moles while the masses add exactly (30,563 = 26,231 + 4,332).

**Implementation.** `refplant::RefStream::species`, `refplant::elements`, `refplant::carbon_of`.

**Test.** `eq_n_r_1_published_table_rows_are_as_printed` holds the printed table as literals, independent of the registry.

## N.R.2: component-wise PSA balance

**What it calculates.** The hydrogen recovery of the PSA and the tail gas that a PSA with that recovery would produce from the published feed.

**Where it comes from.** The PSA keeps every impurity on the adsorbent and releases it with the hydrogen that is not recovered, so for every species $\dot n_{feed,i}=\dot n_{product,i}+\dot n_{tail,i}$. The product (stream 12) and the recycle to the front end (stream 13, recompressed PSA product, Sheet 14, PDF p37) are both PSA product.

**Derivation.** $R=(n_{12}+n_{13})/(n_7y_{7,H_2})$. The model tail gas is the feed for every species other than hydrogen, and $(1-R)$ of the feed hydrogen.

**Substitution.** $R=(4{,}461.5+29.1)/(6{,}596.9\times 0.7563)=4{,}490.6/4{,}989.236=0.900058$. Model tail gas minus stream 8, kmol/h: CO2 +0.156, CO -0.160, H2 -0.347, N2 +0.135, CH4 +0.181, H2O -0.175; the largest is 0.347. Mass: $69{,}711-60{,}658-8{,}994-59=0$ kg/h.

**Limits.** The printed range is "typically 85 to 90 %" (PDF p37); the derived 0.900058 lies 5.8e-5 above 0.90 and inside the range read to the whole percent (below 90.5 percent, critique C11). The form with another recovery keeps the feed fixed, so it moves the hydrogen product off the 100,000 Nm3/h basis. It is a test aid only and is never wired to a sensitivity or to the headline (critique C09); a recovery sensitivity must first scale the whole plant to constant hydrogen.

**Checks.** Species residual within 0.5 kmol/h (blocking, A; rounding band about 0.44 kmol/h). Mass closure within 1 kg/h (blocking).

**Implementation.** `RefPlant::balance` (field `psa`), `RefPlant::psa_tail` (test only).

**Test.** `eq_n_r_2_psa_balance_reproduces_stream_8`.

## N.R.3: tail-gas heating value

**What it calculates.** The heat the tail gas releases in the burners.

**Where it comes from.** The report prints no tail-gas heating value. Its combustibles are H2, CO and CH4; each releases its lower heating value per mole when burnt to CO2 and water vapour.

**Derivation.** $Q_{tail}=\sum\dot n_{8,i}LHV_i/3600$ with $\dot n$ in kmol/h and $LHV$ in MJ/kmol, so the result is MJ/s, that is MW.

**Substitution.** $2{,}106.3\times(0.2369\times 241.8+0.1454\times 283.0+0.0945\times 802.3)/3600=101.950$ MW, which is 174.25 MJ/kmol, 6.0506 MJ/kg and 7.7741 MJ/Nm3.

**Limits.** Calculated with the engine constants (`LHV_H2`, `LHV_CO`, `LHV_CH4` in `engine/props/src/units.rs`), not published.

**Implementation.** `refplant::lhv_mw`, `FurnaceBalance::q_tail_mw`. **Test.** `eq_n_r_3_tail_gas_lhv_uses_the_engine_constants`.

## N.R.4: supplementary natural gas heat

**Derivation and substitution.** $Q_{NG,supp}=m_3LHV_{NG}/3600=4{,}332\times 46.50/3600=55.955$ MW.

**Check.** The utility sheet prints 48.2 MMkcal/h of fuel for the hydrogen plant (PDF p48), $48.2\times 4.1868/3.6=56.057$ MW, 0.18 percent above stream 3. So the utility fuel is the natural gas alone and excludes the tail gas (non-blocking, tolerance 0.5 percent, A).

**Implementation.** `FurnaceBalance::q_ng_supp_mw`, `q_utility_fuel_mw`. **Test.** `eq_n_r_4_ng_fuel_heat_matches_the_utility_sheet_fuel`.

## N.R.5: fired duty and tail-gas share

**Derivation.** The burner heat release is $Q_{fired}=Q_{tail}+Q_{NG,supp}=101.950+55.955=157.905$ MW, 40.0 percent of the plant input of 394.77 MW. The tail gas supplies $101.950/157.905=64.56$ percent, which is why the report calls it the primary fuel (PDF p6).

**Test form.** If the tail gas changed to $Q'$, the natural gas that keeps the same fired duty would be $\max(0,Q_{fired}-Q')\cdot 3600/LHV_{NG}$. Holding the fired duty constant is an assumption (A): the reformer duty and the steam raised would both change. The form is a test aid only (critique C09).

**Implementation.** `FurnaceBalance::q_fired_mw`, `RefPlant::supp_ng_t_h` (test only). **Test.** `eq_n_r_5_fired_duty_and_tail_gas_share`.

## N.R.6: radiant duty, an indicator only

**Derivation.** $Q_{rad}=82.63\times 10^{6}\ \mathrm{kcal\ h^{-1}}\times 4.1868\ \mathrm{kJ\ kcal^{-1}}/3.6\times 10^{6}=96.099$ MW, so $\varphi_{rad}=96.099/157.905=0.6086$. Against the plant input the same duty is 24.3 percent; both are correct with their own denominators.

**Limits.** No furnace efficiency is published and none is derived or stored (U). The registry holds no `REF_FURNACE_EFF` and no `REF_STEAM_DUTY`; the published flows replace both.

**Implementation.** `FurnaceBalance::radiant_fraction`. **Test.** `eq_n_r_6_radiant_duty_is_an_indicator_only`.

## N.R.7: carbon from gas to stack

**Where it comes from.** The Base Case has no capture and one carbon outlet, the flue gas (stream 9): "all of the CO2 is emitted from the flue gas of the steam reformer" (PDF p6). Hydrogen leaves carbon-free, so the carbon of stream 1 must leave in stream 9.

**Substitution.** $\dot n_{CO_2,stack}=8{,}659.4\times 0.2123=1{,}838.391$ kmol/h, $\dot m_{stack}=1{,}838.391\times 44.0095/1000=80.9067$ t/h. $\varepsilon_C=1{,}838.391/1{,}839.963-1=-0.085$ percent.

**Check.** Within 0.2 percent (blocking, A; the rounding band of the printed fractions and flows is about 1.8 kmol C/h, 0.10 percent).

**Implementation.** `Co2Breakdown::stack_t_h`, `c_residual_rel`. **Test.** `eq_n_r_7_carbon_closes_from_gas_to_stack`.

## N.R.8: where the stack CO2 comes from

**Derivation.** Carbon enters the burners in two streams: the tail gas (stream 8) and the supplementary gas (stream 3). In the tail gas, CO2 ($A_1$) passes the burners unburnt, and CO ($A_2$) and CH4 ($A_3$) burn to CO2. In stream 3, CO2 ($B_1$) passes and the hydrocarbon carbon ($B_2=C_3-B_1$) burns. Each kmol of carbon gives one kmol of CO2. The parts add to 1,839.18 kmol/h, 0.043 percent above the stack, so they are multiplied by $\rho=0.999574$ to add up exactly.

**Substitution, rescaled, kg CO2 per Nm3 H2.** $A_1$ 0.47209, $A_2$ 0.13472, $A_3$ 0.08756; $A=0.69438$ (85.8 percent of the stack). $B_1$ 0.00212, $B_2$ 0.11257; $B=0.11469$ (14.2 percent).

**Rules that follow.** The published 0.8091 kg/Nm3 already contains the combustion of the tail gas; a tail-gas term is never added to it. A step that washes CO2 out of the tail gas ahead of the burners removes at most $A_1$, 0.47209 kg/Nm3 (critique C08); Case 2A captures 0.4556 kg/Nm3 (Table 2, PDF p16), 96.5 percent of $A_1$. A step that also converts the CO and CH4 first can reach $A$. A step on the flue gas sees $A+B$.

**Implementation.** `Co2Breakdown` and its `bound_*` methods. **Test.** `eq_n_r_8_co2_breakdown_adds_to_the_stack`.

## N.R.9: the unabated intensity

**Derivation.** Intensity is the stack CO2 per unit of hydrogen product: $e_{ref}=1000\,\dot m_{stack}/m_{12}$. It is one route; the route through the parts of N.R.8 differs by +0.043 percent and serves as a check only. The two are never added.

**Substitution.** $80{,}906.65/8{,}994=8.995625$ kg/kg and $80{,}906.65/(4{,}461.5\times 22.414)=0.809066$ kg/Nm3. Table 2 prints 0.8091 kg/Nm3 (PDF p16), which is $0.8091\times 100{,}000/8{,}994=8.995997$ kg/kg; the reconstruction is 4.1e-5 below it. The four-decimal CO2 fraction of stream 9 alone allows 2.4e-4.

**Checks.** Against 0.8091 kg/Nm3 within 0.1 percent (blocking, A) and per kg within 0.05 percent (reported, A).

**Implementation.** `RefBalance::intensity_kg_kg`, `intensity_kg_nm3`. **Test.** `eq_n_r_9_intensity_reproduces_table_2`.

## N.R.10: gas energy, converted by energy

**Derivation.** $Q_{LHV}=m_1LHV_{NG}/3600=30{,}563\times 46.50/3600=394.772$ MW against 394.77 in Table 2. On the higher heating value $E_{HHV}=30.563\times 51.473=1{,}573.17$ GJ/h (436.99 MW). The engine prices gas in MMSCFD at 1,044 Btu/scf (`NG_HHV`); one MMSCF then holds $1{,}044\times 1.05505585=1{,}101.48$ GJ, so $n_{ref}=1{,}573.17\times 24/1{,}101.48=34.2776$ MMSCFD.

**Why by energy.** The IEAGHG gas is richer than the engine's gas, about 1,050 Btu/scf against 1,044. Converting by volume would price 0.6 percent less energy than the reference burns; converting by energy keeps the gas line in one currency on both sides.

**Implementation.** `GasEnergy`. **Test.** `eq_n_r_10_gas_energy_matches_table_2_and_converts_by_energy`.

## N.R.11: steam and power

**Derivation.** $W_{net}=11.500-1.216-(19+347)/1000=9.918$ MWe, equal to the printed export (identity, tolerance 1e-9). Gross power per tonne of turbine steam is $11{,}500/46.053=249.71$ kWh/t with stream 10 and 250.54 kWh/t with the 45.9 t/h of the utility sheet; both values are printed (PDF p44 and p48) and differ by 0.33 percent. The revenue check $9.918\times 8{,}322\times 80=6{,}603{,}007.68$ EUR/y reproduces the 6,603,008 EUR/y of Table 4 (PDF p18).

**Boundary.** No steam crosses the battery limit (PDF p34 and the total row of PDF p48): stream 10, called "HP Steam export", feeds the internal condensing turbine of the COGEN unit. The reference's steam reaches the ledger only as electricity. The registry refuses any other value of `REF_STEAM_NEIGHBOUR_T_H`.

**Implementation.** `SteamPower`. **Test.** `eq_n_r_11_power_balance_closes`.

## N.R.12: scaling to the plant's hydrogen

**Derivation.** The reference is sized for 8,994 kg/h. For a plant hydrogen output $\dot m_{H_2,plant}$ the factor $x=\dot m_{H_2,plant}/8{,}994$ multiplies every flow, energy and power; intensities do not change. At Case B, $100{,}000\times 0.08988=8{,}988$ kg/h, so $x=0.999333$ and $n_{ref}=34.2548$ MMSCFD.

**Stream table.** The published rows I01 to I15 are the printed streams times $x$. Three derived rows are added and labelled derived and unpublished; they never enter a cost or carbon equation: I16 combustion air (nitrogen balance with 21 percent O2 in dry air, A, giving 6,648.5 kmol/h dry plus 62.7 kmol/h of humidity from the hydrogen balance), I17 the second steam addition ahead of the reformer (hydrogen balance of streams 4 and 5, 155.17 kmol/h; the oxygen balance gives 153.70) and I18 the process condensate ahead of the PSA (1,772.9 kmol/h). Stream 14 (95,301 kg/h) is 19.9 t/h larger than the reformer steam these balances imply; the report does not resolve it and no output depends on it.

**Implementation.** `RefPlant::scale`, `RefPlant::scaled`, `RefPlant::stream_table`. **Tests.** `eq_n_r_12_scaling_is_linear_and_intensive_quantities_do_not_move`, `eq_n_r_12_stream_table_scales_published_rows_and_labels_derived_ones`.

# Equation N.R.15: the plant's own tail gas is claimed once

**What it calculates.** Where the heating value of the NUS-H2 PSA reject S14 goes.

**Where it comes from.** The NUS-H2 plant has no fired furnace. The cryogenic unit splits S14 into the recycle to the reformer feed (S03), the hydrogen return ahead of the PSA (S21) and the nitrogen-rich cut (S15), which N2-BURN burns. Every species of S14 leaves in one of the three, so the heating value of S14 is claimed exactly once: as feed (S03 and S21) or as fuel (S15). The only claim on the steam system is the burner duty $Q_{N2,BURN}$, which cannot exceed the heating value of S15.

**Equation.** $LHV(S14)=LHV(S03)+LHV(S21)+LHV(S15)$ and $Q_{N2,BURN}\le LHV(S15)$.

**Reference side.** The IEAGHG tail gas is internal: its 101.95 MW is feedstock energy that returns to the furnace and is not added to the reference gas energy, and its carbon is already in the stack CO2 of N.R.7.

**Implementation.** `refplant::heat_register`. **Test.** `eq_n_r_15_tail_gas_heat_is_claimed_once`, on the recycle run `FS_B_REC_F`. The check that no anchored scenario uses both the burn-on-site term and the recycle claim for the same stream belongs to the integration step.

# Blocking and reported checks

A failed blocking check refuses to build the model, which protects every result against a bad edit of the registry. The test `a_bad_registry_edit_is_refused_by_a_blocking_check` edits one entry per check and expects the named check to refuse it.

| Check | Tolerance | Blocking |
|---|---|---|
| Carbon, natural gas to stack | 0.2 percent | yes |
| PSA species, model tail gas against stream 8 | 0.5 kmol/h | yes |
| Stack CO2 against 0.8091 kg/Nm3 | 0.1 percent | yes |
| Net power, gross minus internal loads | 1e-9 | yes |
| PSA mass closure | 1 kg/h | yes |
| Natural gas split, stream 1 against streams 2 and 3 | 1 kg/h | yes |
| Intensity per kg against Table 2 | 0.05 percent | no |
| Energy input and energy in product against Table 2 | 1e-4 | no |
| Utility fuel against stream 3; stream 10 against the utility steam | 0.5 percent | no |
| Electricity revenue against Table 4 | 1e-6 | no |
| Specification LHV against Table 2 LHV | half a printed unit | no |
| PSA recovery inside the printed range, read to the whole percent | below 90.5 percent | no |
| Steam to the neighbour | exactly 0 | no |
| Furnace oxygen with the assumed air | 0.5 percent | no |

# Limitations

No furnace efficiency, convective-section duty or tail-gas heating value is published; the tail-gas heating value is calculated with the engine constants and the others are not used. Combustion air is derived only for display. Stream 14 steam is not explained by the H and O balances. Pressures are not marked gauge or absolute. The reconstruction reproduces the report's own tables; it says nothing about how the NUS-H2 plant compares with a real plant, and agreement with the printed values checks transcription and arithmetic, not physics.
