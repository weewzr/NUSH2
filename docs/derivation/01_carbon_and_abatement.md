# Independent CO2 accounting for the NUS-H2 plant: equations N.1 to N.10

Branch `feature/independent-abatement-subsidy`, Phase 2 (independent derivation), derivation file 01. This file rebuilds the CO2 accounting of the MEA-capture plant from its own atom balances and stream flows. The published INL and IEAGHG values stay in the engine as the legacy basis and as validation references. The verification script is `math/verify/sympy_carbon.py`. Phase 3 implemented every equation in Rust. Each equation ends with the function that carries it and the test that checks it by id.

How to read this file. You need algebra and the idea that a flow is an amount per time. Every symbol is defined in the Reference Frame before it is used. Each equation is derived from atom balances in small steps, then the next equation is built from it. Numbers are quoted to the precision printed by the script, and every number comes from a file in `results/`, from `engine/data/parameters.toml` or from a constant in `engine/`. A quantity that none of those files supplies is shown as a symbol and listed as an open input.

Each result carries one of four labels: **analytical** for an algebraic formula, **numerical** when it needs an iteration or a solver, **series** for an expansion and **approximation** when it drops a term. Every equation below is analytical unless its label says otherwise.

# Reference Frame

## Variables

Streams use the tags of `engine/plant/src/flowsheet.rs`: S01 fresh gas, S03 recycle, S04 reformer feed, S05 reformer outlet, S09 shift outlet, S10 condensate, S11 absorber feed, S12 treated gas, S13 hydrogen product, S14 PSA reject, S15 nitrogen-rich cut, S16 stack gas, S19 CO2 from the stripper, S20 CO2 to storage, S21 cryogenic hydrogen return.

| Symbol | Definition | Units |
|---|---|---|
| $\dot m_{H_2}$ | hydrogen product mass flow at the battery limit | $\mathrm{kg\ h^{-1}}$ |
| $\dot n_{H_2}$ | hydrogen product molar flow, $\dot m_{H_2}/M_{H_2}$ | $\mathrm{kmol\ h^{-1}}$ |
| $M_{H_2}$, $M_{CO_2}$ | molar masses, 2.01588 and 44.0095 | $\mathrm{kg\ kmol^{-1}}$ |
| $\dot n_{NG}$ | fresh natural gas rate | $\mathrm{kmol\ h^{-1}}$ |
| $y_j$ | mole fraction of species $j$ in the gas (the lumped set sums to 1.00002) | $\mathrm{kmol\ kmol^{-1}}$ |
| $a_{C,j}$, $a_{H,j}$, $a_{O,j}$ | atoms of C, H and O in one molecule of species $j$ | 1 |
| $\bar c$ | carbon carried by one kmol of gas | $\mathrm{kmol\ C\ kmol^{-1}}$ |
| $Z_j$ | hydrogen potential of species $j$, $2a_C-a_O+a_H/2$ | $\mathrm{kmol\ H_2\ kmol^{-1}}$ |
| $Z_{NG}$ | hydrogen potential of the gas, $\sum_j y_jZ_j$ | $\mathrm{kmol\ H_2\ kmol^{-1}}$ |
| $\xi$ | PSA hydrogen recovery | 1 |
| $r$ | share of the reject hydrogen that the cryogenic unit returns ahead of the PSA | 1 |
| $\xi_{eff}$ | recovery of the S12 hydrogen into the product with the return loop closed | 1 |
| $\pi$ | CH4 per H2 in the product, $y_{CH_4}/(1-y_{CH_4})$ | $\mathrm{kmol\ kmol^{-1}}$ |
| $F_C$ | carbon in the fresh gas (S01) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $T$ | carbon entering the reformer tubes (S04) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $R$ | recycled carbon (S03) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $G$ | carbon in the PSA reject (S14) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $B$ | carbon in the nitrogen-rich cut (S15), equal to the stack carbon (S16) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $C$ | carbon captured, CO2 to storage (S20) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $P$ | carbon in the hydrogen product, CH4 only (S13) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $K$ | CO2 dissolved in the process condensate (S10) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $\varepsilon_j$ | carbon in an outlet or inlet that the flowsheet does not model (status unknown) | $\mathrm{kmol\ C\ h^{-1}}$ |
| $\varphi$, $\psi$, $\omega$ | shares of $T$ that reach the absorber as CO2, CH4 and CO | 1 |
| $\kappa$ | share of $T$ that leaves as CO2 dissolved in the condensate, $K/T$ | 1 |
| $\eta$ | per-pass capture, the share of the absorber-inlet CO2 that the solvent removes | 1 |
| $\eta_{pl}$ | plant capture, $C/F_C$ | 1 |
| $f$ | cryogenic carbon recovery, the share of the reject carbon sent back to the reformer | 1 |
| $D$ | loop denominator, $1-f(1-\kappa-\eta\varphi)$ | 1 |
| $\chi$ | hydrogen deficit coefficient, $(1-f)(4\psi+\omega)/D$ | 1 |
| $\beta$ | carbon accompanying each kmol of feed N2 in the cut, $B/\dot n_{N_2}$ | $\mathrm{kmol\ C\ kmol^{-1}}$ |
| $y_{N_2,cut}$ | N2 mole fraction of the cut | 1 |
| $v$ | share of the condensate CO2 that reaches the atmosphere | 1 |
| $R_C$, $\rho$, $\tau_C$ | carbon closure residual, relative residual and tolerance | $\mathrm{kmol\ C\ h^{-1}}$, 1, 1 |
| $f_{av}$ | availability | 1 |
| $t_h$ | operating hours per year, $8760f_{av}$ | $\mathrm{h\ y^{-1}}$ |
| $\dot n_{proc}$, $\dot n_{fuel}$, $\dot n_{ref}$ | process gas, supplementary firing gas and total gas of the unabated reference plant | $\mathrm{kmol\ h^{-1}}$ |
| $\psi'$, $\omega'$, $\kappa'$ | the carbon shares $\psi$, $\omega$, $\kappa$ of the reference flowsheet | 1 |
| $Q_F$ | heat the reference furnace must deliver to the process and steam system | $\mathrm{MJ\ h^{-1}}$ |
| $Q_{rej}$ | lower heating value of the reject gas burned in the reference | $\mathrm{MJ\ h^{-1}}$ |
| $\eta_F$ | furnace efficiency, share of fuel heating value delivered as $Q_F$ | 1 |
| $LHV_j$, $LHV_{NG}$ | lower heating value of species $j$ and of the gas | $\mathrm{MJ\ kmol^{-1}}$ |
| $\dot Q_{exp,ref}$, $\dot Q_{exp,pl}$ | steam exported by the reference and by the plant | $\mathrm{GJ\ h^{-1}}$ |
| $\dot W_{ref}$ | electric power the reference consumes | $\mathrm{MW}$ |
| $e_{NG}$, $\eta_b$, $EF_{grid}$ | CO2 per heat of burned gas (LHV), replacement boiler efficiency, grid emission factor | $\mathrm{t\ CO_2\ GJ^{-1}}$, 1, $\mathrm{t\ CO_2\ MWh^{-1}}$ |
| $E_{plant}$, $E_{ref}$ | annual direct CO2 to the atmosphere of the plant and of the reference | $\mathrm{t\ CO_2\ y^{-1}}$ |
| $A_{dir}$, $A_{cap}$, $A_{TS}$, $A_{LC}$ | direct, capture-based, net-after-transport-and-storage and lifecycle abatement | $\mathrm{t\ CO_2\ y^{-1}}$ |
| $\tau_{TS}$ | CO2 emitted or lost per tonne of CO2 delivered to storage | $\mathrm{t\ t^{-1}}$ |
| $\lambda$, $e_{ship}$, $w_{TS}$, $EF_{el}$ | loss fraction, transport fuel CO2 per tonne, electricity per tonne stored, electricity emission factor | 1, $\mathrm{t\ t^{-1}}$, $\mathrm{MWh\ t^{-1}}$, $\mathrm{t\ MWh^{-1}}$ |
| $e_{up,CO_2}$, $e_{up,CH_4}$, $GWP_H$ | upstream CO2 and CH4 per heat of gas, and the global warming potential of CH4 over horizon $H$ | $\mathrm{t\ GJ^{-1}}$, $\mathrm{t\ GJ^{-1}}$, $\mathrm{t\ CO_2e\ t^{-1}\ CH_4}$ |
| $\Delta G$, $E_{nuc}$ | heat of gas the plant does not buy, and nuclear lifecycle emission | $\mathrm{GJ\ y^{-1}}$, $\mathrm{t\ CO_2\ y^{-1}}$ |
| $a_{H_2}$, $\rho_{red}$, $ci$, $e_{ref}$ | abatement per kg H2, fractional reduction, emission intensity of the plant, intensity of the reference | $\mathrm{t\ t^{-1}}$, 1, $\mathrm{t\ t^{-1}}$, $\mathrm{t\ t^{-1}}$ |

## Equations

1. **N.1.1** Carbon per kmol of gas: $\bar c=\sum_jy_ja_{C,j}$ and $F_C=\bar c\,\dot n_{NG}$.
2. **N.1.2** Hydrogen potential: $Z_j=2a_{C,j}-a_{O,j}+a_{H,j}/2$, conserved by every reaction; $Z_{NG}=\sum_jy_jZ_j$.
3. **N.1.3** Hydrogen at the absorber: $\dot n_{H_2,12}=\dot Z_{in}-4\dot n_{CH_4,09}-\dot n_{CO,09}$.
4. **N.1.4** PSA product with the return loop: $\xi_{eff}=\xi/[1-r(1-\xi)]$, $\dot n_{H_2}=\xi_{eff}\dot n_{H_2,12}$, $P=\pi\dot n_{H_2}$.
5. **N.1.5** Gas rate from the hydrogen target: $\dot n_{NG}=\dot n_{H_2}\,[1/\xi_{eff}+f\pi(4-\chi)]/(Z_{NG}-\chi\bar c)$.
6. **N.2.1** Tube carbon: $T=(F_C-fP)/D$.
7. **N.2.2** Carbon of every modelled stream: $G=(1-\kappa-\eta\varphi)T-P$, $R=fG$, $B=(1-f)G$, $K=\kappa T$.
8. **N.3.1** Captured CO2: $C=\eta\varphi T$, with S19 = S20 = $C$ in the modelled plant.
9. **N.3.2** Plant capture: $\eta_{pl}=\eta\varphi(1-fP/F_C)/D$.
10. **N.4.1** Recycle fraction read from the flowsheet: $f=R/G=(T-F_C)/G$.
11. **N.4.2** Nitrogen-cut diagnostics: $B=\beta\,y_{N_2}\dot n_{NG}$, $f=1-\beta\dot n_{N_2}/G$.
12. **N.4.3** Sensitivity of the INL back-calculation to $\varphi$: $\mathrm{d}f_{INL}/\mathrm{d}\varphi=-(T_6/\varphi)(F_{C,6}-C_6-P_A)/G_6^2$.
13. **N.4.4** Stack carbon against the recycle fraction: $B(f)=(1-f)(mF_C-P)/(1-mf)$ with $m=1-\kappa-\eta\varphi$.
14. **N.5.1** Complete combustion of the cut: $\dot n_{CO_2}=n_C$, $\dot n_{H_2O}=n_H/2$, $\dot n_{N_2}=n_N/2$, $\dot n_{O_2}=Z_{15}/2$.
15. **N.5.2** Plant direct emission: $E_{plant}=(M_{CO_2}t_h/1000)\,[B+vK+\sum_j\varepsilon_j^{atm}]$.
16. **N.6.1** Closure residual and tolerance: $R_C=F_C-(C+B+P+K+\sum_j\varepsilon_j)$, $\rho=R_C/F_C$, $\tau_C=10^{-9}$.
17. **N.6.2** Analytical check of the tear: $T_{closed}$ from N.2.1 against $T_{flow}$ from the iterated flowsheet.
18. **N.7.1** Reference process gas: $\dot n_{proc}=\dot n_{H_2}/\{\xi[Z_{NG}-(4\psi'+\omega')\bar c]\}$.
19. **N.7.2** Reference firing gas: $\dot n_{fuel}=\max\{0,\,Q_F/\eta_F-Q_{rej}\}/LHV_{NG}$.
20. **N.7.3** Reference direct emission: $E_{ref}=(M_{CO_2}t_h/1000)\,[\bar c(\dot n_{proc}+\dot n_{fuel})-P-(1-v)K']$.
21. **N.7.4** Steam and power terms of the system view: $E_{lost}=(\dot Q_{exp,ref}-\dot Q_{exp,pl})t_he_{NG}/\eta_b$ and $E_{pow}=\dot W_{ref}t_hEF_{grid}$.
22. **N.8.1** Operating hours: $t_h=8760f_{av}$.
23. **N.8.2** Annual mass of CO2: $\dot E=\dot n_C\,M_{CO_2}\,t_h/1000$.
24. **N.8.3** Annual hydrogen and unit conversions of external references.
25. **N.9.1** Direct emissions avoided: $A_{dir}=E_{ref}-E_{plant}$.
26. **N.9.2** Decomposition: $A_{dir}=(M_{CO_2}t_h/1000)[C+\bar c(\dot n_{ref}-\dot n_{NG})-(1-v)(K'-K)]$.
27. **N.9.3** Capture-based abatement: $A_{cap}=M_{CO_2}Ct_h/1000$.
28. **N.9.4** Net after transport and storage: $A_{TS}=A_{dir}-\tau_{TS}A_{cap}$, $\tau_{TS}=\lambda+e_{ship}+w_{TS}EF_{el}$.
29. **N.9.5** Lifecycle view: $A_{LC}=A_{TS}+\Delta G(e_{up,CO_2}+GWP_He_{up,CH_4})-E_{nuc}$.
30. **N.10.1** Abatement per kg of hydrogen: $a_{H_2}=A/M_{H_2,y}$.
31. **N.10.2** Fractional reduction: $\rho_{red}=A_{dir}/E_{ref}$.
32. **N.10.3** Emission intensity: $ci=E_{plant}/M_{H_2,y}$.
33. **N.10.4** Reported carbon balance residual: $\rho$ of N.6.1 and the status of every unmodelled stream.

## Key assumptions and parameter ranges

- Steady state, one hydrogen basis. The flowsheet is set by intensive specifications only (S/C, temperatures, pressures, $\eta$, $f$, $r$, $\xi$), so every stream flow is proportional to $\dot n_{H_2}$ (`flowsheet.rs:1724` to `1726` and `1735`).
- Reformer outlet 871 C, 439 psia, S/C 3.00 (`flowsheet.rs:118`, `126`); heavier hydrocarbons are converted completely at the tube inlet (D.4); the reformer follows equilibrium (D.11) and the shift beds follow the approach to equilibrium (E.4).
- The flash holds CH4, CO and H2 insoluble and dissolves CO2 by the gas-solubility law of F.6; the absorber removes CO2 only; the PSA and the cryogenic unit are specified splits (J.12, class external boundary and fixed assumption).
- The default routing returns no H2 to the reformer (`h2_rest_to_recycle` false), so the recycle holds only CH4, CO and CO2.
- Combustion of the cut is complete. Excess air does not change any carbon flow.
- $\eta$ (`ETA_MEA`) 0.90, sweep 0.80 to 0.95. $\xi$ (`PSA_H2_RECOVERY`) 0.88. $y_{CH_4}$ in product (`CH4_IN_H2`) 0.001. $f_{av}$ (`AVAILABILITY`) 0.85, which gives $t_h=7446\ \mathrm{h\ y^{-1}}$. $v\in[0,1]$, central value 1. $f\in[0,1]$ and $r\in[0,1]$; the current flowsheet values 0.752178 and 0.921630 are calibrated to INL numbers and are design inputs (see N.4).
- Legacy basis only: `PHI_RECYCLE` 0.775 (range 0.76 to 0.79), `CO2_CAPTURED_C6` 1927 st/d, `ETA_SELEXOL` 0.99, `NG_CASE6` 34 MMSCFD, `CO2_CONV_NOCC` 3205 st/d, `NG_CONV_NOCC` 52.5 MMSCFD.

## Settled equations relied on

These are quoted by id from `math/eq_register.csv` and `math/sections/*.tex` and are not derived again here.

| Id | Content | Used in |
|---|---|---|
| A.3, A.5, A.6 | component balance, element balance, independent reactions | N.1.2, N.2.1, N.5.1, N.6.1 |
| A.15 | closure residual limits ($10^{-8}$ per unit, $10^{-6}$ flowsheet) | N.6.1 |
| D.1 to D.4 | reformer reactions and heavier-gas conversion | N.1.2 |
| D.5, D.6, D.11 | S/C, mole table from extents, reformer equilibrium | N.1.5, N.2.2 (gives $\psi$, $\omega$, $\varphi$) |
| E.2, E.4 | shift species and approach to equilibrium | N.1.5, N.2.2 |
| F.6, F.12 | CO2 solubility in condensate, condensate CO2 | N.2.2, N.5.2 |
| J.2, J.3 | $\dot m=\rho_N\dot V_N$, annual hydrogen | N.8.3, N.10 |
| J.4, J.5 | MMSCFD conversion, feed carbon 1.05379 | N.1.1, N.8.3 |
| J.6 to J.11 | legacy anchored carbon ledger | worked example, N.4.3 |
| J.12, J.13 | PSA split and hydrogen ledger | N.1.4 |
| J.33, K.14 | per-pass and plant capture printed together | N.3.2 |
| K.1 to K.3 | baseline intensity and annual baseline | N.7.3 (legacy) |
| K.5, K.6 | stoichiometric and feed-carbon intensity bounds | N.10 |
| K.7, K.8, K.10, K.11, K.13 | captured, residual, hours, net abatement, intensity | N.8, N.9, N.10 |
| K.17 | tail-gas fates | N.5.2 |
| K.18, K.19 | system view and lifecycle view | N.7.4, N.9.5 |

# Variables explained one by one

Typical values are those of the Level 3 run `FS_B_REC_F` in `results/Integrated_Mass_Balance.csv` (Case B, 8988 kg/h of hydrogen, MEA capture) unless the entry says otherwise. That run uses the cryogenic inputs $f=0.752178$ and $r=0.921630$, which are calibrated to INL numbers; the entries on $f$ and $r$ explain why this matters.

## Variable: hydrogen product ($\dot m_{H_2}$, $\dot n_{H_2}$)

The hydrogen leaving the PSA, the quantity the plant is built to make. $\dot m_{H_2}=H2\_B\times\rho_N=100000\ \mathrm{Nm^3\ h^{-1}}\times0.08988\ \mathrm{kg\ Nm^{-3}}=8988\ \mathrm{kg\ h^{-1}}$ (J.2), so $\dot n_{H_2}=8988/2.01588=4458.60\ \mathrm{kmol\ h^{-1}}$. It is the input of the whole derivation: the flowsheet is solved on this basis and everything else follows. Caveat: `RHO_H2_NORMAL` is marked not verified in `parameters.toml`; a 1 percent change in it moves every absolute tonnage by 1 percent and leaves every ratio unchanged. The product also carries 0.1 mol percent CH4, which is the carbon $P$.

## Variable: molar masses ($M_{H_2}$, $M_{CO_2}$)

Conversion factors between kmol and kg: 2.01588 and 44.0095 $\mathrm{kg\ kmol^{-1}}$ (`engine/props/src/units.rs:16`, `18`). One kmol of carbon that ends as CO2 becomes 44.0095 kg of CO2. They are exact definitions for this study, not fitted values.

## Variable: fresh gas rate ($\dot n_{NG}$)

The natural gas the plant buys, in kmol/h, equal to $F_C/\bar c$. In the central run it is 1150.37 $\mathrm{kmol\ h^{-1}}$, which is 23.10 MMSCFD (1 MMSCFD is 49.80 $\mathrm{kmol\ h^{-1}}$ by J.4). It is an output of N.1.5, not an input: the hydrogen target fixes it. The legacy ledger instead fixed it at INL's 34 MMSCFD scaled by 0.6833, which is 23.23 MMSCFD.

## Variable: composition ($y_j$) and atom counts ($a_{k,j}$)

Mole fractions from the published analysis (TEV-953 A-5, TEV-961 A-50), lumped as in `flowsheet.rs:175` to `185`: CH4 0.93571, C2H6 0.03749, C3H8 0.00920, C4H10 0.00260, C5H12 0.00052 (C6 counted as 1.2 C5), N2 0.01200 (O2 joined to N2), CO2 0.00250. The sum is 1.00002 because the C6 lumping adds a trace of extra moles; this has no effect on carbon. The row S01 of the CSV lists the lumped moles, so its total (1150.3967 in `FS_B_REC_F`) is 1.00002 times $\dot n_{NG}$ (1150.3737), and $\dot n_{NG}$ is the physical gas rate. Putting the C6 and the O2 back as themselves leaves $\bar c$ at 1.05379 and lowers $Z_{NG}$ from 4.13939 to 4.13917 (a relative change of $5.3\times10^{-5}$), which would move the gas rate of N.1.5 by about $6\times10^{-5}$ in relative terms. The flowsheet is solved on the lumped basis, so every check against it uses the lumped $Z_{NG}$. In `parameters.toml` the ids `C_PER_MOL_NG_*` hold these mole fractions despite their names. N2 and O2 are not in the file; they live only in `flowsheet.rs`, so the new ids `NG_Y_N2` and `NG_Y_O2` are proposed. The atom counts $a_{k,j}$ are the integers of the molecular formulas.

## Variable: carbon per kmol of gas ($\bar c$)

Carbon atoms carried by one kmol of feed gas: 1.05379 $\mathrm{kmol\ C\ kmol^{-1}}$ (J.5). It converts a gas rate into a carbon rate. Because the gas is mostly CH4 with a little ethane and heavier gas, $\bar c$ sits slightly above 1.

## Variable: hydrogen potential ($Z_j$, $Z_{NG}$)

A counting device introduced here. $Z_j$ is the number of H2 molecules that one molecule of $j$ can give when its carbon is fully oxidised to CO2 by steam. CH4 has 4, CO has 1, CO2 has 0, H2 has 1, H2O has 0, N2 has 0, C2H6 has 7. The gas as a whole has $Z_{NG}=4.13939$. The reason for inventing it is that $Z$ is conserved by every reaction (N.1.2), which lets the hydrogen yield be found without solving the reactor equations again. O2 is given $Z=-2$ so that combustion also conserves $Z$.

## Variable: PSA recovery ($\xi$), product methane ($\pi$), cryogenic hydrogen return ($r$), effective recovery ($\xi_{eff}$)

$\xi=0.88$ (`PSA_H2_RECOVERY`, published) is the share of the hydrogen entering the PSA that leaves in the product. $\pi=0.001/0.999=0.001001$ comes from the published product purity (99.9 mol percent H2, 0.1 mol percent CH4; `CH4_IN_H2`). $r$ is the share of the reject hydrogen that the cryogenic unit sends back ahead of the PSA; in `FS_B_REC_F` it is 560.342/607.991 = 0.921630, found by `close_h2_return` (`flowsheet.rs:1782`) so that INL's 34 MMSCFD makes INL's 29000 lb/h. $\xi_{eff}=0.88/[1-0.921630\times0.12]=0.989426$ is the net recovery once the return loop is closed. Caveat: the PSA is an unsimulated external block, so $\xi$ and $\pi$ are specifications. $r$ is a design input of the cryogenic unit that is currently calibrated to an INL pair; it fixes the gas rate directly (without the return loop, $\xi_{eff}=\xi$ and the gas rate would be 12.4 percent higher).

## Variable: carbon flows ($F_C$, $T$, $R$, $G$, $B$, $C$, $P$, $K$)

Carbon is the one element that the CO2 accounting must follow, so each stream is measured in kmol of carbon atoms per hour whatever molecule carries it. Central values in $\mathrm{kmol\ C\ h^{-1}}$: $F_C=1212.252$ (S01), $T=1497.327$ (S04), $R=285.075$ (S03), $G=378.999$ (S14), $B=93.924$ (S15 and S16), $C=1108.584$ (S20), $P=4.463$ (S13), $K=5.281$ (S10). The tube carbon $T$ exceeds $F_C$ by the recycle, a factor 1.235. Caveat: the engine's `CarbonLedger` (`carbon.rs:70`) names these `feed`, `tube`, `recycled`, `tail_gas`, `burned`, `captured`, `product` and `condensate`.

## Variable: carbon shares at the absorber ($\varphi$, $\psi$, $\omega$, $\kappa$)

Of every unit of carbon entering the tubes, $\varphi=0.822639$ leaves the shift train as CO2 on its way to the absorber, $\psi=0.159597$ as unconverted CH4, $\omega=0.014236$ as CO and $\kappa=0.003527$ is dissolved in the condensate. They sum to 1 because carbon is conserved through the reaction train and the flash. They are numerical results of the reformer equilibrium (D.11) and the shift approach (E.4), followed by the flash (F.6), at the specified operating point. The legacy ledger used a single number $\phi_c=0.775$ (`PHI_RECYCLE`, reconstructed from INL tables) in place of $\varphi$ and had no $\kappa$. Caveat: they depend weakly on the feed composition at S04, hence on $f$.

## Variable: per-pass capture ($\eta$) and plant capture ($\eta_{pl}$)

$\eta=0.90$ is the share of the CO2 in S11 that the MEA solvent removes in one pass through the absorber (`ETA_MEA`, author design value, sweep 0.80 to 0.95). $\eta_{pl}=C/F_C=0.9145$ is the share of all fresh carbon that ends in storage. They differ because $\varphi<1$ and because the uncaptured CO2 returns through the recycle only in part; the cut and the condensate take some carbon out, and the product takes a little more. The engine prints both (J.33, K.14). Caveat: the absorber and stripper models fail their own gate (M6 for term T02: 6 of 13 rows fail in `results/Validation_Results.csv`, `GATE_T02`), so $\eta$ is a specification and not a result.

## Variable: recycle fraction ($f$), loop denominator ($D$), coefficient ($\chi$)

$f$ is the share of the reject carbon that the cryogenic unit returns to the reformer; the rest goes to the cut. Central value 0.752178, back-calculated from INL Case 6 (J.7). $D=1-f(1-\kappa-\eta\varphi)=0.807369$ is the factor by which the recycle loop amplifies the fresh carbon, so $T=(F_C-fP)/D$. $\chi=(1-f)(4\psi+\omega)/D=0.200323$ collects the hydrogen that unconverted CH4 and CO leave behind. Caveat: $f$ is the one carbon quantity that the plant's atom balances cannot fix on their own, because it is a property of the separator. Equation N.4.1 shows how it is read back from the streams, N.4.3 shows how fragile the INL back-calculation is and N.4.4 gives the response of the stack to $f$.

## Variable: nitrogen-cut diagnostics ($\beta$, $y_{N_2,cut}$)

All N2 in the feed leaves in the cut, because the recycle carries none and no other outlet exists. $\beta=B/\dot n_{N_2}=93.924/13.804=6.804\ \mathrm{kmol\ C\ kmol^{-1}}$ says how much carbon travels with each kmol of purged nitrogen. $y_{N_2,cut}=0.0803$ says that the "nitrogen-rich" cut is 8.0 mol percent N2. Both are diagnostics of the cryogenic specification.

## Variable: condensate vent fraction ($v$) and unmodelled streams ($\varepsilon_j$)

$v$ is the share of the CO2 dissolved in the condensate (S10) that reaches the atmosphere, for instance in a deaerator. The legacy headline ledger left it inside the captured CO2 ($v=0$); decision S14 treats it as vented in a sensitivity. The independent basis sets $v=1$ (a stated convention that maximises the plant emission, new id `COND_CO2_VENT`) because an unknown fate must not count as zero emission. The same $v$ applies to the reference, so by N.9.2 raising $v$ from 0 to 1 changes $A_{dir}$ by $+k(K'-K)$: it lowers the abatement only if the reference condensate CO2 $K'$ is smaller than the plant's $K$, which is known only after the reference run. $\varepsilon_j$ stands for carbon in a stream the flowsheet does not model; their register is in the section on unmodelled streams below. Each carries a status in {modelled, specified, unknown}.

## Variable: closure residual ($R_C$, $\rho$, $\tau_C$)

$R_C$ is the carbon that enters minus the carbon that leaves, $\rho=R_C/F_C$ its relative size and $\tau_C=10^{-9}$ the tolerance. The exported envelope residual of `FS_B_REC_F` is $5.26\times10^{-9}\ \mathrm{kmol\ C\ h^{-1}}$, or $4.3\times10^{-12}$ of the feed.

## Variable: availability ($f_{av}$) and hours ($t_h$)

$f_{av}=0.85$ is an author assumption (decision D05); IEAGHG 2017-02 uses 0.95. Operating hours are $t_h=8760\times0.85=7446\ \mathrm{h\ y^{-1}}$. All annual quantities scale linearly with $f_{av}$ while every kg-per-kg ratio is unchanged (K.16).

## Variable: reference gas ($\dot n_{proc}$, $\dot n_{fuel}$, $\dot n_{ref}$) and reference carbon shares ($\psi'$, $\omega'$, $\kappa'$)

The unabated steam methane reforming plant making the same hydrogen. $\dot n_{proc}$ is the gas that goes through its reformer tubes; $\dot n_{fuel}$ is extra gas burned in its furnace; $\dot n_{ref}=\dot n_{proc}+\dot n_{fuel}$. Its carbon shares come from the same flowsheet run without capture and recycle. For scale only: INL's reference at Case B size is 52.5 MMSCFD $\times0.6833=35.87$ MMSCFD. No independent value exists yet (see N.7).

## Variable: firing balance ($Q_F$, $Q_{rej}$, $\eta_F$, $LHV$)

$Q_F$ is the heat the reference furnace has to deliver; $Q_{rej}$ the heating value of the PSA reject that it burns first; $\eta_F$ the share of fuel heat that reaches the process and the steam system. Lower heating values: H2 241.8, CH4 802.3, CO 283.0 $\mathrm{kJ\ mol^{-1}}$ (`units.rs:30` to `34`), numerically equal to $\mathrm{MJ\ kmol^{-1}}$. $Q_F$ and $\eta_F$ are design inputs of the reference; $Q_{rej}$ follows from the reference flowsheet.

## Variable: steam and power of the two plants ($\dot Q_{exp}$, $\dot W_{ref}$, $e_{NG}$, $\eta_b$, $EF_{grid}$)

INL's reference exports 189.2 MMBtu/h of steam and consumes 6.3 MWe (`EXPORT_STEAM_CONV`, `E_CONV_NOCC`, both at Case A size). The plant exports a smaller steam flow and gets its power from the reactor. $e_{NG}=56.1\ \mathrm{kg\ GJ^{-1}}$ (`EF_NG_LHV`), $\eta_b=0.9$ (`SYSTEM_BOILER_EFF`) and $EF_{grid}=0.402\ \mathrm{t\ MWh^{-1}}$ (`GRID_EF`) are the legacy values for the system view; the first and the third are marked not verified in `parameters.toml` and the second is an assumption.

## Variable: annual emissions and abatements ($E$, $A$)

$E_{plant}$ and $E_{ref}$ are the CO2 each plant puts into the air in a year. $A_{dir}=E_{ref}-E_{plant}$ is the direct abatement. $A_{cap}$ is the CO2 delivered to storage. $A_{TS}$ takes off the CO2 emitted or lost between the plant and the permanent store. $A_{LC}$ adds the life-cycle terms. All are in tonnes of CO2 per year. Headline quantities are CO2; a CO2e quantity appears only in $A_{LC}$ with its horizon (decision D4).

## Variable: transport and storage inputs ($\lambda$, $e_{ship}$, $w_{TS}$, $EF_{el}$, $\tau_{TS}$)

Symbolic physical inputs to be filled from the transport and storage research: $\lambda$ is the share of delivered CO2 that returns to the air before permanent storage; $e_{ship}$ the CO2 from transport fuel per tonne delivered; $w_{TS}$ the electricity used for liquefaction and injection per tonne stored; $EF_{el}$ the emission factor of that electricity. Their sum is $\tau_{TS}$. The legacy ledger lumps all of it into `TS_BURDEN` = 0.025 $\mathrm{t\ t^{-1}}$, an author assumption marked not verified.

## Variable: lifecycle inputs ($e_{up,CO_2}$, $e_{up,CH_4}$, $GWP_H$, $\Delta G$, $E_{nuc}$)

The gas the plant does not buy ($\Delta G$) also avoids its upstream supply emissions: CO2 from energy use ($e_{up,CO_2}$) and methane leakage ($e_{up,CH_4}$). Methane enters a CO2e total only through the global warming potential $GWP_H$ over a stated horizon $H$. The legacy factor `EF_UPSTREAM` = 11.5 $\mathrm{kg\ GJ^{-1}}$ does not say which gases it covers, so it is split into two ids. $E_{nuc}$ is the nuclear lifecycle emission (`EF_NUCLEAR` 5.5 $\mathrm{kg\ MWh^{-1}}$, not verified).

## Variable: intensities ($a_{H_2}$, $\rho_{red}$, $ci$, $e_{ref}$)

Per tonne of hydrogen the abatement is $a_{H_2}$, and the emissions of the plant and of the reference are $ci$ and $e_{ref}$; all are in $\mathrm{t\ CO_2\ t^{-1}\ H_2}$ (numerically kg per kg). $\rho_{red}$ is the fraction of the reference emission that is avoided. Bounds: the chemistry fixes a floor of $M_{CO_2}/(4M_{H_2})=5.458$ (K.5) for any reforming route that makes H2 from CH4, and the feed carbon of the central run gives 5.936 (K.6).

# Equations N.1 to N.5: feed, loop, capture, recycle and stack

## Equation N.1.1: carbon per kmol of natural gas

**What it calculates.** The kmol of carbon atoms carried by one kmol of fresh natural gas, and from it the carbon feed rate $F_C$.

**Variables.** $y_j$ mole fraction of species $j$ ($\mathrm{kmol\ kmol^{-1}}$); $a_{C,j}$ carbon atoms per molecule of $j$ (1); $\bar c$ carbon per kmol of gas ($\mathrm{kmol\ C\ kmol^{-1}}$); $\dot n_{NG}$ gas rate ($\mathrm{kmol\ h^{-1}}$); $F_C$ carbon feed ($\mathrm{kmol\ C\ h^{-1}}$).

**Where it comes from.** The element balance for carbon (A.5) says that carbon atoms are counted by adding over species. One kmol of gas holds $y_j$ kmol of species $j$, and each kmol of $j$ holds $a_{C,j}$ kmol of carbon, so species $j$ brings $y_ja_{C,j}$. Adding over $j$ gives $\bar c$. This is card J.5.

**Substitution into the next equation.** $F_C=\bar c\,\dot n_{NG}$ is the input of the loop equation N.2.1. $\bar c$ also appears in N.1.5 and in N.7.3.

**Intermediate calculations.**

| Species | $y_j$ | $a_{C,j}$ | $y_ja_{C,j}$ |
|---|---|---|---|
| CH4 | 0.93571 | 1 | 0.93571 |
| C2H6 | 0.03749 | 2 | 0.07498 |
| C3H8 | 0.00920 | 3 | 0.02760 |
| C4H10 | 0.00260 | 4 | 0.01040 |
| C5H12 (with C6 as 1.2 C5) | 0.00052 | 5 | 0.00260 |
| CO2 | 0.00250 | 1 | 0.00250 |
| N2 (with O2) | 0.01200 | 0 | 0 |
| Sum | 1.00002 |  | 1.05379 |

**Final implementable formula.**

$$\bar c=\sum_jy_ja_{C,j}=1.05379\ \mathrm{kmol\ C\ kmol^{-1}},\qquad F_C=\bar c\,\dot n_{NG}$$

Result type: analytical.

Rust implementation: `closed_form::gas_constants` and `flowsheet::ng_carbon_per_mol` in `engine/plant/src/abatement.rs` and `engine/plant/src/flowsheet.rs`; the fresh carbon is the `feed` line of `carbon::CarbonLedger`.

Validation test: `eq_n_1_1_carbon_per_kmol_and_feed_carbon` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.1.2: hydrogen potential and its conservation

**What it calculates.** A number $Z_j$ for each species that counts how many H2 molecules the species can give when all its carbon is turned into CO2 with steam. The flow of $Z$ is the same before and after any reaction, mixer or splitter. Picture: $Z$ counts coins, where one H2 molecule is worth one coin and a CH4 molecule is worth four; a reactor exchanges molecules for other molecules of equal total value and never creates or destroys coins.

**Variables.** $a_{C,j}$, $a_{H,j}$, $a_{O,j}$ atoms per molecule (1); $Z_j$ ($\mathrm{kmol\ H_2\ kmol^{-1}}$); $\nu_i$ stoichiometric coefficient of species $i$ in a reaction, positive for products (1); $F_i$ molar flow ($\mathrm{kmol\ h^{-1}}$).

**Where it comes from.** For every reaction the element balances hold (A.5): $\sum_i\nu_ia_{k,i}=0$ for each element $k$ in {C, H, O}. Take twice the carbon balance, subtract the oxygen balance and add half of the hydrogen balance: $\sum_i\nu_i(2a_{C,i}-a_{O,i}+a_{H,i}/2)=0$. The bracket is $Z_i$, so $\sum_i\nu_iZ_i=0$. A reaction of extent $\xi_r$ changes each flow by $\nu_i\xi_r$, so $\sum_iZ_iF_i$ does not change; mixing and splitting move the same species around and do not change it either. Check on reaction D.1: $Z_{CH_4}+Z_{H_2O}=4+0$ and $Z_{CO}+3Z_{H_2}=1+3$.

**Substitution into the next equation.** N.1.3 uses the conserved flow $\dot Z$ across the reformer and the shift beds. N.5.1 uses it to get the oxygen demand of the cut.

**Intermediate calculations.**

| Species | CH4 | CO | CO2 | H2 | H2O | N2 | C2H6 | C3H8 | C4H10 | C5H12 | O2 |
|---|---|---|---|---|---|---|---|---|---|---|---|
| $Z$ | 4 | 1 | 0 | 1 | 0 | 0 | 7 | 10 | 13 | 16 | $-2$ |

$Z_{NG}=0.93571\times4+0.03749\times7+0.00920\times10+0.00260\times13+0.00052\times16=3.74284+0.26243+0.09200+0.03380+0.00832=4.13939$.

**Final implementable formula.**

$$Z_j=2a_{C,j}-a_{O,j}+\tfrac12a_{H,j},\qquad Z_{NG}=\sum_jy_jZ_j=4.13939$$

Result type: analytical. The script also proves the conservation for D.1, D.2, D.3, the general reaction D.4 for any $\mathrm{C}_n\mathrm{H}_m$ and three combustion reactions.

Rust implementation: `closed_form::hydrogen_potential` in `engine/plant/src/abatement.rs`, evaluated on any species vector through the element counts of `nush2_props::mixture::elements`.

Validation test: `eq_n_1_2_hydrogen_potential_table_and_conservation` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.1.3: hydrogen at the absorber

**What it calculates.** The H2 flow in the treated gas S12 from the carbon speciation alone, without solving the reactor equations again.

**Variables.** $\dot Z_{in}$ hydrogen potential flow entering the reformer ($\mathrm{kmol\ H_2\ h^{-1}}$); $\dot n_{CH_4,09}$, $\dot n_{CO,09}$ CH4 and CO leaving the shift train ($\mathrm{kmol\ h^{-1}}$); $\dot n_{H_2,12}$ H2 in S12 ($\mathrm{kmol\ h^{-1}}$).

**Where it comes from.** Between S04 and S09 only reactions occur (D.1 to D.4, E.2), so $\dot Z$ is conserved (N.1.2). The inlet S04 holds the fresh gas and the recycle, together with the process steam. Steam has $Z=0$ and so do CO2 and N2. The outlet S09 holds H2, CH4, CO, CO2, H2O and N2, so only H2, CH4 and CO count. The flash removes water and dissolves CO2, which have $Z=0$, and the model holds H2, CH4 and CO insoluble, so S11 and S12 keep the H2, CH4 and CO of S09 (the absorber removes CO2 only). Therefore
$\dot Z_{in}=\dot n_{H_2,12}+4\dot n_{CH_4,09}+\dot n_{CO,09}$.

**Substitution into the next equation.** In N.1.5 the CH4 and CO flows are replaced by $\psi T$ and $\omega T$, and the recycle part of $\dot Z_{in}$ by the S14 composition.

**Intermediate calculations (FS_B_REC_F).** $\dot Z_{in}=4761.845$ (S01) $+\,4\times176.3905+16.0337=721.596$ (S03) $=5483.441$. The outlet losses are $4\times238.9694+21.3164=977.194$. So $\dot n_{H_2,12}=5483.441-977.194=4506.247\ \mathrm{kmol\ h^{-1}}$, against 4506.247 in the CSV.

**Final implementable formula.**

$$\dot n_{H_2,12}=\dot Z_{in}-4\dot n_{CH_4,09}-\dot n_{CO,09},\qquad \dot Z_{in}=Z_{NG}\dot n_{NG}+4\dot n_{CH_4,03}+\dot n_{CO,03}$$

Result type: analytical, exact for any converged flowsheet; it uses the numerical outputs $\dot n_{CH_4,09}$ and $\dot n_{CO,09}$ of D.11 and E.4.

Rust implementation: `closed_form::hydrogen_potential` in `engine/plant/src/abatement.rs` on S01, S03 and S09; the flowsheet itself solves the reformer and shift beds (`flowsheet::pass`).

Validation test: `eq_n_1_3_hydrogen_at_the_absorber` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.1.4: PSA product with the cryogenic return loop

**What it calculates.** The H2 product and the product CH4 for a given H2 flow in S12, with the cryogenic hydrogen return loop closed around the PSA.

**Variables.** $\xi$ PSA recovery (1); $r$ return share (1); $H_{feed}$ H2 into the PSA ($\mathrm{kmol\ h^{-1}}$); $\xi_{eff}$ (1); $\pi$ ($\mathrm{kmol\ kmol^{-1}}$); $y_{CH_4}$ product CH4 mole fraction (1); $P$ product carbon ($\mathrm{kmol\ C\ h^{-1}}$).

**Where it comes from.** The PSA is a specified split (J.12): the product takes $\xi H_{feed}$ and the reject takes $(1-\xi)H_{feed}$. The cryogenic unit sends back the share $r$ of the reject H2, so the PSA feed is the new H2 from S12 plus the return: $H_{feed}=\dot n_{H_2,12}+r(1-\xi)H_{feed}$. Solving for $H_{feed}$ gives $H_{feed}=\dot n_{H_2,12}/[1-r(1-\xi)]$, and the product is $\xi H_{feed}$. The product contains CH4 at mole fraction $y_{CH_4}$, so per kmol of H2 it holds $y_{CH_4}/(1-y_{CH_4})$ kmol of CH4, which is $\pi$. Its carbon is $P=\pi\dot n_{H_2}$ because CH4 has one carbon atom.

**Substitution into the next equation.** $\xi_{eff}$ and $\pi$ enter N.1.5. $P$ enters N.2.1 and N.6.1.

**Intermediate calculations.** $r=560.342/607.991=0.921630$. $1-r(1-\xi)=1-0.921630\times0.12=0.889404$. $\xi_{eff}=0.88/0.889404=0.989426$, equal to the CSV ratio $4458.599/4506.247=0.989426$. $\pi=0.001/0.999=0.001001$ and $P=0.001001\times4458.599=4.463\ \mathrm{kmol\ C\ h^{-1}}$.

**Final implementable formula.**

$$\xi_{eff}=\frac{\xi}{1-r(1-\xi)},\qquad\dot n_{H_2}=\xi_{eff}\,\dot n_{H_2,12},\qquad P=\pi\,\dot n_{H_2}$$

Result type: analytical. Design input: $r$ (see N.4). Specified, not simulated: $\xi$ and $y_{CH_4}$.

Rust implementation: `closed_form::xi_eff` in `engine/plant/src/abatement.rs` and `flowsheet::psa_boundary`.

Validation test: `eq_n_1_4_psa_loop_with_the_hydrogen_return` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.1.5: gas rate from the hydrogen target

**What it calculates.** The fresh gas rate that makes the specified hydrogen output, written with the reformer and shift conversions that the flowsheet already solves.

**Variables.** $\dot n_{NG}$ ($\mathrm{kmol\ h^{-1}}$); $\dot n_{H_2}$ target ($\mathrm{kmol\ h^{-1}}$); $\xi_{eff}$, $f$, $\pi$, $\psi$, $\omega$, $\kappa$, $\eta$, $\varphi$ (1); $Z_{NG}$ ($\mathrm{kmol\ H_2\ kmol^{-1}}$); $\bar c$ ($\mathrm{kmol\ C\ kmol^{-1}}$); $D$, $\chi$ (1).

**Where it comes from.** Build $\dot Z_{in}$ and the outlet terms of N.1.3 from the carbon shares. The recycle S03 holds the fraction $f$ of every carbon species of the reject S14 (CH4 $\psi T-P$, CO $\omega T$, CO2 $(1-\eta)\varphi T$) and no H2. So $\dot Z_{in}=Z_{NG}\dot n_{NG}+4f(\psi T-P)+f\omega T$. The outlet terms are $4\psi T+\omega T$. Their difference is
$\dot n_{H_2,12}=Z_{NG}\dot n_{NG}-(1-f)(4\psi+\omega)T-4fP$.
Now insert N.2.1, $T=(\bar c\dot n_{NG}-fP)/D$, and use the symbol $\chi=(1-f)(4\psi+\omega)/D$:
$\dot n_{H_2,12}=(Z_{NG}-\chi\bar c)\dot n_{NG}-f(4-\chi)P$.
Apply N.1.4, $\dot n_{H_2}=\xi_{eff}\dot n_{H_2,12}$ and $P=\pi\dot n_{H_2}$:
$\dot n_{H_2}/\xi_{eff}=(Z_{NG}-\chi\bar c)\dot n_{NG}-f(4-\chi)\pi\dot n_{H_2}$.
Solve for $\dot n_{NG}$.

**Substitution into the next equation.** $\dot n_{NG}$ gives $F_C=\bar c\dot n_{NG}$ for N.2.1. The same formula with $f=0$ and $r=0$ becomes N.7.1.

**Intermediate calculations.** With $f=0.752178$: $1-f=0.247822$, $4\psi+\omega=0.638389+0.014236=0.652625$ and $D=0.807369$, so $\chi=0.247822\times0.652625/0.807369=0.200323$. Numerator: $1/\xi_{eff}=1.010687$ and $f\pi(4-\chi)=0.752178\times0.001001\times3.799677=0.002861$, sum 1.013548. Denominator: $4.13939-0.200323\times1.05379=3.928293$. Ratio 0.258012 $\mathrm{kmol\ gas\ kmol^{-1}\ H_2}$. Then $\dot n_{NG}=0.258012\times4458.599=1150.37\ \mathrm{kmol\ h^{-1}}$, which is the gas rate of `FS_B_REC_F` to a relative $5\times10^{-10}$.

**Final implementable formula.**

$$\dot n_{NG}=\dot n_{H_2}\,\frac{1/\xi_{eff}+f\,\pi\,(4-\chi)}{Z_{NG}-\chi\,\bar c},\qquad\chi=\frac{(1-f)(4\psi+\omega)}{D},\quad D=1-f(1-\kappa-\eta\varphi)$$

Result type: analytical given $\psi$, $\omega$, $\kappa$ and $\varphi$. Those four are numerical outputs of the equilibrium reformer (D.11), the shift approach (E.4) and the flash (F.6); the formula does not replace the flowsheet solver, it is the exact relation that the solver result must satisfy. Because every term is proportional to $\dot n_{H_2}$, the gas rate scales linearly with the hydrogen basis. Lower bound from $\psi=\omega=0$: $\dot n_{NG}\ge\dot n_{H_2}/(\xi_{eff}Z_{NG})=1088.6\ \mathrm{kmol\ h^{-1}}$.

Rust implementation: `closed_form::gas_rate_from_h2`, `closed_form::chi` and `closed_form::loop_denominator` in `engine/plant/src/abatement.rs`; the flowsheet reaches the same rate by solving on the hydrogen basis (`flowsheet::solve` with `FeedBasis::Hydrogen`).

Validation test: `eq_n_1_5_gas_rate_from_the_hydrogen_target` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.2.1: tube carbon (the recycle loop in closed form)

**What it calculates.** The carbon entering the reformer tubes, which is the fresh carbon plus the recycled carbon, without iterating. Picture: a tank that is topped up with fresh water while a pipe returns part of the outflow to the same tank. After many turns of the loop the level settles, and the closed form below is that settled level.

**Variables.** $T$, $F_C$, $R$, $G$, $P$ ($\mathrm{kmol\ C\ h^{-1}}$); $f$, $\kappa$, $\eta$, $\varphi$, $D$ (1).

**Where it comes from.** The loop follows from the mixer, the split and the reject balance. (i) The mixer in front of the reformer: $T=F_C+R$. (ii) The cryogenic split: $R=fG$. (iii) The reject: carbon reaching S12 is $T$ less the condensate share $\kappa T$ less the captured share $\eta\varphi T$ (N.3.1), and the PSA removes the product carbon $P$, so $G=(1-\kappa-\eta\varphi)T-P$. Substitute (ii) and (iii) into (i): $T=F_C+f[(1-\kappa-\eta\varphi)T-P]$. Collect the terms in $T$ on the left: $T[1-f(1-\kappa-\eta\varphi)]=F_C-fP$.

**Substitution into the next equation.** $T$ feeds N.2.2, which lists every other stream, and N.3.1.

**Intermediate calculations.** $F_C-fP=1212.252-0.752178\times4.463=1208.895$. $D=1-0.752178\times0.256098=0.807369$. $T=1208.895/0.807369=1497.327$, equal to S04. The loop amplifies the fresh carbon by $T/F_C=1.2352$.

**Final implementable formula.**

$$T=\frac{F_C-fP}{D},\qquad D=1-f\,(1-\kappa-\eta\varphi)$$

Result type: analytical. Series view: write $q=f(1-\kappa-\eta\varphi)$, which is below 1. Each turn of the loop returns the fraction $q$ of the carbon that entered it, so $T=(F_C-fP)(1+q+q^2+\dots)=(F_C-fP)/(1-q)$, which is the formula above; the flowsheet's accelerated tear iteration converges to the same sum. $D>0$ for all $0\le f\le1$ whenever $\kappa+\eta\varphi>0$, so the loop always has a solution. Card J.8 is the special case $\kappa=0$ with $\varphi$ replaced by the legacy $\phi_c$.

Rust implementation: `closed_form::loop_denominator` and `closed_form::tube_carbon` in `engine/plant/src/abatement.rs`; the iterated tear is `flowsheet::solve_tears`.

Validation test: `eq_n_2_1_tube_carbon_closed_form_for_three_tear_starts` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.2.2: the fate of carbon, stream by stream

**What it calculates.** The carbon flow of every modelled carbon-bearing stream from $T$ and the carbon shares, with the two design inputs $\eta$ and $f$.

**Variables.** As in N.2.1, plus $B$ cut and stack carbon, $K$ condensate carbon ($\mathrm{kmol\ C\ h^{-1}}$).

**Where it comes from.** Carbon is conserved in each unit (A.5), so a stream's carbon is the carbon of its parent minus what the unit sends elsewhere. The reformer and the shift beds, with their coolers, keep all of $T$. The flash sends $\kappa T$ to S10 and the rest, $(1-\kappa)T=(\varphi+\psi+\omega)T$, to S11. The absorber moves $\eta\varphi T$ to the solvent. The PSA moves $P$ to the product. The cryogenic unit splits the reject by $f$.

**Substitution into the next equation.** $C=\eta\varphi T$ is N.3.1. $B$ and $K$ enter N.5.2 and N.6.1.

**Intermediate calculations.** Every modelled carbon-bearing stream, with the central values in $\mathrm{kmol\ C\ h^{-1}}$ from `FS_B_REC_F`:

| Stream | Content | Carbon expression | Value |
|---|---|---|---|
| S01 | fresh gas | $F_C=\bar c\dot n_{NG}$ | 1212.252 |
| S03 | recycle (CH4, CO, CO2) | $R=fG$ | 285.075 |
| S04 | reformer feed | $T=F_C+R$ | 1497.327 |
| S05 to S09C | reformer outlet to flash feed (CH4, CO, CO2) | $T$ | 1497.327 |
| S10 | CO2 in condensate | $K=\kappa T$ | 5.281 |
| S11 | absorber feed | $(1-\kappa)T$ | 1492.046 |
| S12 | treated gas (slip CO2, CH4, CO) | $(1-\kappa-\eta\varphi)T$ | 383.462 |
| S13 | product CH4 | $P=\pi\dot n_{H_2}$ | 4.463 |
| S14 | reject | $G=(1-\kappa-\eta\varphi)T-P$ | 378.999 |
| S15 | cut | $B=(1-f)G$ | 93.924 |
| S16 | stack gas | $B$ (complete combustion) | 93.924 |
| S17, S18 | rich and lean solvent (CO2 loading) | internal, difference is $C$ | 2217.168, 1108.584 |
| S19, S20 | CO2 from the stripper, CO2 to storage | $C=\eta\varphi T$ | 1108.584 |
| S02, S21, combustion O2 | steam, hydrogen return, air O2 | none | 0 |

Inside S14 the species are CH4 $\psi T-P=234.506$, CO $\omega T=21.316$ and CO2 $(1-\eta)\varphi T=123.176$, which sum to 378.999. The slip CO2 is why the recycle carries more CO2 than INL's Selexol plant did.

**Final implementable formula.**

$$G=(1-\kappa-\eta\varphi)T-P,\quad R=fG,\quad B=(1-f)G,\quad K=\kappa T,\quad C=\eta\varphi T$$

Result type: analytical given N.2.1. Streams that the flowsheet does not model are not set to zero; they appear in the register of unmodelled streams and in N.6.1.

Rust implementation: `closed_form::fates` and `PlantLedger::shares` in `engine/plant/src/abatement.rs`; the stream carbon is `Model::plant_ledger`, which runs the flowsheet with the registry design inputs (`Model::ledger_spec`).

Validation test: `eq_n_2_2_stream_vector_of_the_modelled_streams` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.3.1: captured CO2

**What it calculates.** The CO2 delivered to storage as the specified per-pass capture times the CO2 entering the absorber.

**Variables.** $C$ ($\mathrm{kmol\ C\ h^{-1}}$ equal to $\mathrm{kmol\ CO_2\ h^{-1}}$); $\eta$ (1); $\varphi T$ the CO2 in S11 ($\mathrm{kmol\ h^{-1}}$); $\dot n_{MEA}$ solvent circulation ($\mathrm{kmol\ h^{-1}}$); $\alpha_{rich}$, $\alpha_{lean}$ CO2 loadings ($\mathrm{kmol\ CO_2\ kmol^{-1}\ MEA}$).

**Where it comes from.** The definition of per-pass capture is $\eta=(\text{CO}_2\text{ removed from the gas})/(\text{CO}_2\text{ in S11})$, so $C=\eta\varphi T$ with $\varphi T=\dot n_{CO_2,11}$. In steady state the solvent carries exactly this CO2 from the absorber to the stripper: $\dot n_{MEA}(\alpha_{rich}-\alpha_{lean})=C$ (the flowsheet sets the circulation this way, `flowsheet.rs:528` and `533`). The stripper delivers $C$ as S19, and compression passes it unchanged to S20 because no vent is modelled.

**Substitution into the next equation.** $C$ enters N.3.2, N.6.1, N.9.2 and N.9.3.

**Intermediate calculations.** $\varphi T=0.822639\times1497.327=1231.760$ (the CO2 of S11). $C=0.9\times1231.760=1108.584$. Solvent check: rows S17 and S18 of the CSV hold 2217.168 and 1108.584 kmol CO2/h on $\dot n_{MEA}=4434.336\ \mathrm{kmol\ h^{-1}}$, so $\alpha_{rich}=0.50$, $\alpha_{lean}=0.25$ and $\dot n_{MEA}(\alpha_{rich}-\alpha_{lean})=1108.584=C$. The loadings change the circulation and the energy and leave the carbon flows as they are.

**Final implementable formula.**

$$C=\eta\,\varphi\,T=\eta\,\dot n_{CO_2,11},\qquad\dot n_{19}=\dot n_{20}=C$$

Result type: analytical.

**Physics, design input, literature.**
- Physics: the CO2 available to the absorber, $\varphi T$ (reformer and shift equilibrium and the flash), and the carbon balances above.
- Design input: $\eta$. The absorber and stripper models fail their gate (M6, term T02), so no column result replaces it. Parameter id `ETA_MEA` (existing, 0.90, range 0.80 to 0.95, class assumed).
- Literature: none for $\eta$. INL's 99 percent Selexol capture (`ETA_SELEXOL`) applies only to the legacy basis.
- Not modelled and therefore unknown: any CO2 vented between S19 and S20 (dehydration, compression), new id `CO2_PROC_VENT`, and MEA carbon (make-up and degradation), new id `MEA_CARBON`. They are in the register below with status unknown.

Rust implementation: `closed_form::fates` (field `c`) in `engine/plant/src/abatement.rs`; the capture island is `flowsheet::AnchoredMea` selected by `CaptureSel::Eta` at the specified per-pass capture.

Validation test: `eq_n_3_1_captured_co2_is_the_specified_capture_times_the_absorber_co2` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.3.2: plant capture

**What it calculates.** The share of all fresh carbon that ends in storage, to print beside $\eta$ (J.33).

**Variables.** $\eta_{pl}$ (1); $C$, $F_C$ ($\mathrm{kmol\ C\ h^{-1}}$); other symbols as before.

**Where it comes from.** Divide N.3.1 by $F_C$ and insert N.2.1: $\eta_{pl}=\eta\varphi T/F_C=\eta\varphi(F_C-fP)/(F_CD)$.

**Substitution into the next equation.** N.10.2 and the plant-capture rows of the output tables use $\eta_{pl}$.

**Intermediate calculations.** $1-fP/F_C=1-3.357/1212.252=0.997231$. $\eta\varphi=0.740375$. $\eta_{pl}=0.740375\times0.997231/0.807369=0.914483$, equal to $1108.584/1212.252$. In the limit $f\to1$ the denominator becomes $\kappa+\eta\varphi=0.743902$ and $\eta_{pl}\to0.99159$: even with complete recycling, plant capture stays below 1 because the condensate and the product take their share.

**Final implementable formula.**

$$\eta_{pl}=\frac{C}{F_C}=\frac{\eta\varphi\,(1-fP/F_C)}{D}$$

Result type: analytical. This $\eta_{pl}$ is not the legacy column `plant_capture`, which is $C/(C+B)$ (0.903767 in the legacy B_REC_F row).

Rust implementation: `closed_form::plant_capture` in `engine/plant/src/abatement.rs`; the ledger value is `captured / feed` of `CarbonLedger`.

Validation test: `eq_n_3_2_plant_capture_stays_below_one` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.4.1: the recycle fraction from the plant's own streams

**What it calculates.** The share of reject carbon that is recycled, read from the carbon flows of the plant's own flowsheet, in place of the INL back-calculated 0.752.

**Variables.** $f$ (1); $R$, $G$, $T$, $F_C$ ($\mathrm{kmol\ C\ h^{-1}}$).

**Where it comes from.** The recycle is the reject share that the cryogenic unit returns, $R=fG$, and the mixer gives $T=F_C+R$. Therefore $f=R/G=(T-F_C)/G$. The legacy card J.7 evaluates exactly this second form with INL's numbers: $f_{INL}=(T_6-F_{C,6})/G_6$ with $T_6=C_6/(\eta_{Sel}\phi_c)$ and $G_6=T_6-C_6-P_A$. The independent basis takes $T$ and $G$ from the plant's own S04 and S14, and $F_C$ from S01.

**Substitution into the next equation.** $f$ fixes $D$ in N.2.1 and the split of N.2.2.

**Intermediate calculations.** $(1497.327-1212.252)/378.999=285.075/378.999=0.752178$. The legacy value is $0.752178$ as well, because the flowsheet takes the INL value as its input (`flowsheet.rs:328`); the stream-based reading therefore returns the input and proves the tear is converged, but it does not yet make $f$ independent.

**Final implementable formula.**

$$f=\frac{R}{G}=\frac{T-F_C}{G}$$

Result type: analytical (an identity of the converged loop).

**Physics, design input, literature.**
- Physics: the identity above, the loop solution N.2.1, and the consequences of any $f$ on $T$, $G$, $B$, $C$ (everything downstream is exact once $f$ is given).
- Design input: $f$ itself. It describes how sharply the cryogenic unit separates the PSA reject into a CH4 and CO rich recycle and a nitrogen-rich cut. The unit is an unsimulated block (class fixed assumption), so the plant's atom balances cannot determine $f$. New id `CRYO_C_RECOVERY` with provenance assumed; its central value is a owner decision (open issue 1). The companion input is $r$, new id `CRYO_H2_RETURN`; it is currently found by `close_h2_return` on INL's 34 to 130 MMSCFD pair and shares the same dependence on INL.
- Literature (legacy basis and validation row only): INL Case 6 captured CO2 1927 st/d, Selexol 99 percent, $\phi_c=0.775$ (ids `CO2_CAPTURED_C6`, `ETA_SELEXOL`, `PHI_RECYCLE`).
- A physical bound that follows from the equations: $0\le f<1$, and N.4.2 ties $f$ to the purge of the feed nitrogen.

Rust implementation: `closed_form::recycle_read_back` in `engine/plant/src/abatement.rs`; the design input is `CarbonParams::cryo_f` read from `CRYO_C_RECOVERY`.

Validation test: `eq_n_4_1_recycle_fraction_read_back_from_the_streams`, and the validation rows `IND_CRYO_F_READBACK` and `IND_CRYO_F_VS_J7` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.4.2: nitrogen-cut diagnostics

**What it calculates.** The stack carbon expressed through the nitrogen that must be purged, and the N2 content of the cut.

**Variables.** $\dot n_{N_2}$ feed N2 ($\mathrm{kmol\ h^{-1}}$); $y_{N_2}$ N2 mole fraction of the gas (1); $\beta$ ($\mathrm{kmol\ C\ kmol^{-1}}$); $y_{N_2,cut}$ (1); $\dot n_{15}$ total cut flow ($\mathrm{kmol\ h^{-1}}$).

**Where it comes from.** The recycle S03 carries no N2 and no other outlet takes N2, so all feed N2 leaves in S15: $\dot n_{N_2}=y_{N_2}\dot n_{NG}$. Define $\beta=B/\dot n_{N_2}$ and $y_{N_2,cut}=\dot n_{N_2}/\dot n_{15}$. Then $B=\beta y_{N_2}\dot n_{NG}$, and from $B=(1-f)G$ it follows that $f=1-\beta\dot n_{N_2}/G$.

**Substitution into the next equation.** None. These are reported beside $f$ so that a user can see what the cryogenic specification implies for the stack.

**Intermediate calculations.** $\dot n_{N_2}=0.0120\times1150.37=13.8045$. $\beta=93.9242/13.8045=6.8039$. $\dot n_{15}=171.9356$ and $y_{N_2,cut}=0.08029$. Check: $f=1-6.8039\times13.8045/378.999=0.752178$.

**Final implementable formula.**

$$B=\beta\,y_{N_2}\,\dot n_{NG},\qquad f=1-\frac{\beta\,y_{N_2}\,\dot n_{NG}}{G},\qquad y_{N_2,cut}=\frac{y_{N_2}\dot n_{NG}}{\dot n_{15}}$$

Result type: analytical. Observation: the cut named "nitrogen-rich" holds only 8.0 mol percent N2 at the current inputs, so the cryogenic specification in `REC-CRYO` is not a sharp N2 separation.

Rust implementation: `closed_form::n2_cut` in `engine/plant/src/abatement.rs`; the feed nitrogen is the `n2_cut` line of `CarbonLedger`.

Validation test: `eq_n_4_2_nitrogen_balance_of_the_cut` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.4.3: how fragile the INL back-calculation is

**What it calculates.** The sensitivity of the legacy recycle fraction to the carbon split $\phi_c$ that is used to build it, to show why it is replaced.

**Variables.** $T_6$, $G_6$, $F_{C,6}$, $C_6$, $P_A$ (all $\mathrm{kmol\ C\ h^{-1}}$ at Case A scale: 2157.163, 495.605, 1784.379, 1655.083 and 6.474); $\phi$ (1); $f_{INL}$ (1).

**Where it comes from.** $f_{INL}=(T_6-F_{C,6})/G_6$ with $T_6=C_6/(\eta_{Sel}\phi)$ and $G_6=T_6-C_6-P_A$. Both $T_6$ and $G_6$ move by the same amount when $\phi$ changes, $\mathrm{d}T_6/\mathrm{d}\phi=\mathrm{d}G_6/\mathrm{d}\phi=-T_6/\phi$. The quotient rule gives $\mathrm{d}f_{INL}/\mathrm{d}\phi=(\mathrm{d}T_6/\mathrm{d}\phi)\,(G_6-T_6+F_{C,6})/G_6^2$, and $G_6-T_6+F_{C,6}=F_{C,6}-C_6-P_A$.

**Substitution into the next equation.** None; it supports open issue 1.

**Intermediate calculations.** $F_{C,6}-C_6-P_A=1784.379-1655.083-6.474=122.822$, which is only 6.9 percent of $F_{C,6}$: $f_{INL}$ is a ratio built from a small difference of large numbers. $\mathrm{d}f_{INL}/\mathrm{d}\phi=-(2157.163/0.775)\times122.822/495.605^2=-1.3918$. Evaluating the formula at the flowsheet's own $\varphi=0.8226$ instead of 0.775 gives $f_{INL}=0.6687$ in place of 0.7522, a drop of 0.0835 for a 0.0476 rise in $\varphi$. This is an evaluation of card J.7, not a plant result.

**Final implementable formula.**

$$\frac{\mathrm{d}f_{INL}}{\mathrm{d}\phi}=-\frac{T_6}{\phi}\,\frac{F_{C,6}-C_6-P_A}{G_6^{2}}$$

Result type: analytical (exact derivative; the large-step value 0.6687 is a direct evaluation).

Rust implementation: `closed_form::d_f_inl_d_phi` in `engine/plant/src/abatement.rs`, evaluated against the legacy `Model::carbon`.

Validation test: `eq_n_4_3_inl_back_calculation_is_fragile` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.4.4: stack carbon as a function of the recycle fraction

**What it calculates.** The stack carbon $B$ for any recycle fraction $f$ at a fixed fresh gas rate (fixed $F_C$), and the inverse (the $f$ that gives a chosen stack carbon). It shows the response of the carbon streams to the one carbon input that the atom balances cannot fix. The plant is specified by its hydrogen output, and at a fixed hydrogen output the gas rate itself changes with $f$ (N.1.5), so the second form of the final formula below is the one to use for the design decision.

**Variables.** $m=1-\kappa-\eta\varphi$ (1); $B_0=mF_C-P$ stack carbon when the whole reject is burned ($\mathrm{kmol\ C\ h^{-1}}$); $B$, $F_C$, $P$ ($\mathrm{kmol\ C\ h^{-1}}$); $f$ (1); $B_H(f)$ the stack carbon at a fixed hydrogen output ($\mathrm{kmol\ C\ h^{-1}}$); $\dot n_{NG}(f)$ the gas rate of N.1.5 as a function of $f$ ($\mathrm{kmol\ h^{-1}}$).

**Where it comes from.** From N.2.2, $G=mT-P$, and from N.2.1, $T=(F_C-fP)/D$ with $D=1-fm$. Then $G=[m(F_C-fP)-P(1-fm)]/D$. The two terms in $fmP$ cancel and $G=(mF_C-P)/D$. The stack carbon is $B=(1-f)G$.

**Substitution into the next equation.** None. The inverse turns a target stack carbon, or a target carbon-to-nitrogen ratio $\beta$ through N.4.2, into the value of $f$ that the cryogenic unit must achieve (open issue 1).

**Intermediate calculations.** $m=0.256098$, $B_0=0.256098\times1212.252-4.463=305.992\ \mathrm{kmol\ C\ h^{-1}}$ (100272 $\mathrm{t\ y^{-1}}$ of CO2). Check at the flowsheet value: $B(0.752178)=0.247822\times305.992/0.807369=93.924$, equal to S15. The slope is $\mathrm{d}B/\mathrm{d}f=-B_0(1-m)/(1-mf)^2=-305.992\times0.743902/0.651848=-349.2\ \mathrm{kmol\ C\ h^{-1}}$ per unit of $f$, that is $-1144\ \mathrm{t\ y^{-1}}$ for each 0.01 of $f$. Evaluation points, with the carbon shares held at the `FS_B_REC_F` values:

| $f$ | 0 | 0.5 | 0.752178 | 0.9 | 1 |
|---|---|---|---|---|---|
| $B$ ($\mathrm{kmol\ C\ h^{-1}}$) | 305.99 | 175.46 | 93.92 | 39.76 | 0 |
| $B$ ($\mathrm{t\ y^{-1}}$) | 100272 | 57499 | 30778 | 13031 | 0 |

Halving the present stack carbon needs $f=(B_0-B)/(B_0-mB)=0.8812$ at fixed gas.

Fixed hydrogen output. The hydrogen target $\dot n_{H_2}=4458.599\ \mathrm{kmol\ h^{-1}}$ fixes $P$, and N.1.5 then gives a different fresh gas rate for every $f$, because a larger $f$ recycles more carbon and needs less fresh gas. Put $F_C=\bar c\,\dot n_{NG}(f)$ into $B(f)$. With $\varphi$, $\psi$, $\omega$, $\kappa$ and $\xi_{eff}$ held at the `FS_B_REC_F` values:

| $f$ | 0 | 0.5 | 0.752178 | 0.9 | 1 |
|---|---|---|---|---|---|
| $\dot n_{NG}(f)$ ($\mathrm{kmol\ h^{-1}}$) | 1305.53 | 1205.42 | 1150.37 | 1116.53 | 1092.94 |
| $B_H$ ($\mathrm{kmol\ C\ h^{-1}}$) | 347.87 | 183.98 | 93.92 | 38.58 | 0 |
| $B_H$ ($\mathrm{t\ y^{-1}}$) | 113993 | 60290 | 30778 | 12642 | 0 |

The slope at the present point is $-1205\ \mathrm{t\ y^{-1}}$ for each 0.01 of $f$, against $-1144$ at fixed gas, so the fixed-gas value understates the response by 5 percent. Halving the present stack carbon needs $f=0.8779$ at fixed hydrogen output. The two curves meet at the run's own $f$, which anchors both. The verification script asserts these values.

**Final implementable formula.**

$$B(f)=\frac{(1-f)\,(mF_C-P)}{1-m\,f},\qquad f(B)=\frac{B_0-B}{B_0-m\,B}\qquad(\text{fixed }F_C)$$

$$B_H(f)=\frac{(1-f)\,\big(m\,\bar c\,\dot n_{NG}(f)-P\big)}{1-m\,f},\qquad\dot n_{NG}(f)\ \text{from N.1.5}\qquad(\text{fixed }\dot n_{H_2})$$

Result type: approximation when $f$ differs from the run it is anchored to, because $\varphi$, $\psi$, $\omega$ and $\kappa$ are held fixed although they change weakly with the recycle composition; analytical (exact) at the converged $f$ of any run. The table is an evaluation of the formula and not a flowsheet result.

Rust implementation: `closed_form::stack_vs_f`, `closed_form::f_for_stack` and `closed_form::stack_at_fixed_h2` in `engine/plant/src/abatement.rs`.

Validation test: `eq_n_4_4_stack_carbon_curve` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.5.1: complete combustion of the nitrogen-rich cut

**What it calculates.** The stack gas of the cut burner, in particular its CO2, and the oxygen the burner needs.

**Variables.** $n_C$, $n_H$, $n_O$, $n_N$ atoms entering with S15 ($\mathrm{kmol\ h^{-1}}$ of atoms); $\dot n_{CO_2}$, $\dot n_{H_2O}$, $\dot n_{N_2}$ stack species and $\dot n_{O_2}$ air oxygen ($\mathrm{kmol\ h^{-1}}$); $Z_{15}$ hydrogen potential flow of the cut ($\mathrm{kmol\ H_2\ h^{-1}}$).

**Where it comes from.** Complete combustion means the products are CO2, H2O and N2 only. The four element balances read: C: $\dot n_{CO_2}=n_C$. H: $2\dot n_{H_2O}=n_H$. N: $2\dot n_{N_2}=n_N$. O: $n_O+2\dot n_{O_2}=2\dot n_{CO_2}+\dot n_{H_2O}$. The last gives $\dot n_{O_2}=(2n_C+n_H/2-n_O)/2$, and the numerator is the hydrogen potential flow $Z_{15}$ by its definition in N.1.2. Excess air adds N2 and O2 to the stack and does not change any carbon flow.

**Substitution into the next equation.** $\dot n_{CO_2}=B$ enters N.5.2.

**Intermediate calculations.** S15 holds (in $\mathrm{kmol\ h^{-1}}$) CO2 30.5257, CH4 58.1158, CO 5.2827, H2 47.6484, N2 13.8045 and H2O 16.5585. Then $n_C=93.9242$, $n_H=360.8771$, $n_O=82.8926$, $n_N=27.6090$. Products: CO2 93.9242, H2O 180.4385, N2 13.8045. Oxygen: $(187.8484+180.4385-82.8926)/2=142.6972$, equal to $Z_{15}/2=285.3943/2$ and to the CSV row `COMBUSTION_O2`.

**Final implementable formula.**

$$\dot n_{CO_2,16}=n_C,\quad\dot n_{H_2O,16}=\tfrac12n_H,\quad\dot n_{N_2,16}=\tfrac12n_N,\quad\dot n_{O_2}=\tfrac12Z_{15}$$

Result type: analytical.

Rust implementation: unit N2-BURN of `flowsheet::pass` in `engine/plant/src/flowsheet.rs` (complete combustion with the oxygen of the element balance); `closed_form::hydrogen_potential` gives the oxygen demand check.

Validation test: `eq_n_5_1_complete_combustion_of_the_cut` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.5.2: direct emission of the plant

**What it calculates.** The CO2 the plant puts into the air in a year, from every outlet that reaches the atmosphere.

**Variables.** $E_{plant}$ ($\mathrm{t\ CO_2\ y^{-1}}$); $B$, $K$, $\varepsilon_j^{atm}$ ($\mathrm{kmol\ C\ h^{-1}}$); $v$ (1); $M_{CO_2}$ ($\mathrm{kg\ kmol^{-1}}$); $t_h$ ($\mathrm{h\ y^{-1}}$).

**Where it comes from.** The modelled atmospheric outlets are the stack ($B$, N.5.1) and the vented part of the condensate CO2 ($vK$). Unmodelled outlets that reach the air add $\varepsilon_j^{atm}$. Convert the sum with N.8.2. The product carbon $P$ leaves with the hydrogen and is excluded on both plants (it is equal on both), and the tail gas is burned on site, so none is exported (K.17).

**Substitution into the next equation.** $E_{plant}$ enters N.9.1 and N.10.3.

**Intermediate calculations.** $B$: $93.9242\times44.0095\times7446/1000=30778.47\ \mathrm{t\ y^{-1}}$. $K$: $5.2810\times327.6947=1730.57\ \mathrm{t\ y^{-1}}$. With $v=1$ and no unknown atmospheric outlet counted yet: $32509.03\ \mathrm{t\ y^{-1}}$. The legacy ledger reports 38309.01 $\mathrm{t\ y^{-1}}$ (card K.8), which carries INL's recycle split and $\phi_c$.

**Final implementable formula.**

$$E_{plant}=\frac{M_{CO_2}\,t_h}{1000}\Big[B+vK+\sum_j\varepsilon_j^{atm}\Big]$$

Result type: analytical. The sum over $\varepsilon_j^{atm}$ is empty only when every listed stream is modelled or specified; otherwise the quantity is reported as a lower bound with the unknown streams named.

Rust implementation: `Model::emissions` in `engine/plant/src/emissions.rs` (fields `atm`, `cond_vented`) and `Model::abatement` in `engine/plant/src/abatement.rs`; the carbon lines come from `Model::plant_carbon`.

Validation test: `eq_n_5_2_direct_emission_is_the_stack_plus_the_vented_condensate` in `engine/plant/tests/m11_carbon.rs`.

# Equations N.6 to N.10: closure, reference plant, annual quantities and abatement

## Equation N.6.1: carbon closure residual and tolerance

**What it calculates.** The carbon that enters minus the carbon that leaves, also expressed as a fraction of the feed, with a verdict that is never "closed" while a stream is unknown. Picture: a bank statement. If the opening balance equals deposits and withdrawals combined, the books balance; that proves nothing about whether each single entry is correct.

**Variables.** $R_C$ ($\mathrm{kmol\ C\ h^{-1}}$); $\rho=R_C/F_C$ (1); $\tau_C$ (1); $F_C$, $C$, $B$, $P$, $K$ ($\mathrm{kmol\ C\ h^{-1}}$); $\varepsilon_j^{in}$, $\varepsilon_j^{out}$ unmodelled inlet and outlet carbon ($\mathrm{kmol\ C\ h^{-1}}$).

**Where it comes from.** Carbon is conserved over the plant envelope (A.5, J.11). The fresh gas is the only modelled carbon inlet. The modelled outlets are S20, S15 (burned to S16), S13 and S10. Adding the unmodelled streams gives the residual
$R_C=F_C+\sum_j\varepsilon_j^{in}-(C+B+P+K+\sum_j\varepsilon_j^{out})$.
Each term carries a status: modelled, specified or unknown. If any term is unknown, $R_C$ is not defined and the verdict is "undetermined". If all terms are known, the verdict is "closed" when $|\rho|\le\tau_C$ and "open" otherwise. A missing stream is never entered as 0.

**Tolerance.** $\tau_C=10^{-9}$ of the feed carbon, equal to `ELEMENT_TOL` (`flowsheet.rs:193`). Four facts justify it. (a) Numerical floor: the tear tolerance is $10^{-11}$ (`flowsheet.rs:187`), and the exported envelope residual of `FS_B_REC_F` is $5.26\times10^{-9}\ \mathrm{kmol\ C\ h^{-1}}$, which is $4.3\times10^{-12}$ of the feed, so a good run sits 230 times below the tolerance. (b) Detection: the smallest modelled stream, $P$, is $3.68\times10^{-3}$ of the feed, so omitting it gives $\rho=3.7\times10^{-3}$, which exceeds the tolerance by a factor $3.7\times10^{6}$; any omitted stream larger than $10^{-9}F_C$ is caught. (c) Consequence: $10^{-9}F_C=1.21\times10^{-6}\ \mathrm{kmol\ C\ h^{-1}}$ is $3.97\times10^{-4}\ \mathrm{t\ y^{-1}}$ of CO2, which is $7\times10^{-10}$ of an abatement of $5.8\times10^{5}\ \mathrm{t\ y^{-1}}$. (d) A bound looser than the card limit $10^{-6}$ of A.15 would be meaningless, and one tighter than the floor would fail good runs.

**Substitution into the next equation.** $\rho$ and the statuses are printed by N.10.4. The identity behind the residual is used in N.9.2.

**Intermediate calculations.** From the CSV: $F_C=1212.252$, $C+B+P+K=1108.584+93.924+4.463+5.281=1212.252$, residual below the six-decimal print precision (the full-precision exported value is $5.26\times10^{-9}$). Symbolic check: substituting N.2.1 and N.2.2 into $F_C-(C+B+P+K)$ simplifies to 0 for any $F_C$, $f$, $\kappa$, $\eta$, $\varphi$, $P$.

**Final implementable formula.**

$$R_C=F_C+\sum_j\varepsilon_j^{in}-\Big(C+B+P+K+\sum_j\varepsilon_j^{out}\Big),\qquad\rho=\frac{R_C}{F_C},\qquad\tau_C=10^{-9}$$

Verdict: undetermined if any status is unknown; else closed if $|\rho|\le\tau_C$; else open.

Result type: analytical. Caveat: because every unit is a split, closure is a bookkeeping test. It cannot reveal an error in $\eta$, $f$ or $\varphi$; N.6.2 and the Z-balance of N.1.3 test other things.

Rust implementation: `carbon::CarbonLedger::account` and `carbon::closure_report` in `engine/plant/src/carbon.rs`, built on `carbon::close_carbon`; the tolerance is `CarbonParams::closure_tol` read from `CARBON_CLOSURE_TOL`.

Validation test: `eq_n_6_1_verdicts_are_never_closed_while_a_stream_is_unknown` and `eq_n_6_1_tolerance_and_the_exported_residual` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.6.2: analytical check of the iterated tear

**What it calculates.** The difference between the tube carbon found by the iterated flowsheet and the closed form of N.2.1, and the difference between the flowsheet gas rate and N.1.5.

**Variables.** $T_{flow}$ tube carbon of the converged run ($\mathrm{kmol\ C\ h^{-1}}$); $T_{closed}$ from N.2.1 ($\mathrm{kmol\ C\ h^{-1}}$); $\delta_T=(T_{closed}-T_{flow})/T_{flow}$ (1); $\delta_n$ the same for the gas rate (1).

**Where it comes from.** The flowsheet iterates the recycle tear with accelerated fixed-point steps until the scaled residual is below `TEAR_TOL`. N.2.1 solves the same linear loop in one step. At convergence the two must agree to the tear tolerance. The check uses the input $f$ (not the ratio read back), so it tests the iteration and the mixer.

**Substitution into the next equation.** None.

**Intermediate calculations.** With the printed streams of `FS_B_REC_F` and the input $f=0.7521781$: $T_{closed}=1497.326806$ against $T_{flow}=1497.326806$, $\delta_T$ below $10^{-6}$ (the print precision). $\dot n_{NG}$ from N.1.5 is 1150.3737 against $F_C/\bar c=1150.3737$, $\delta_n=-5\times10^{-10}$.

**Final implementable formula.**

$$\delta_T=\frac{T_{closed}-T_{flow}}{T_{flow}},\qquad\delta_n=\frac{\dot n_{NG}^{(N.1.5)}-\dot n_{NG}^{flow}}{\dot n_{NG}^{flow}},\qquad\text{pass if }|\delta|\le\tau_C$$

Result type: analytical comparison against a numerical result.

Rust implementation: `closed_form::tube_carbon` and `closed_form::gas_rate_from_h2` in `engine/plant/src/abatement.rs` against the converged flowsheet.

Validation test: `eq_n_6_2_the_tear_check_against_the_closed_forms` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.7.1: process gas of the unabated reference plant

**What it calculates.** The gas that goes through the reformer tubes of a steam methane reforming plant with the same feed composition and the same hydrogen output, and no capture.

**Variables.** $\dot n_{proc}$ ($\mathrm{kmol\ h^{-1}}$); $\dot n_{H_2}$ ($\mathrm{kmol\ h^{-1}}$); $\xi$ (1); $\psi'$, $\omega'$ (1); $Z_{NG}$ ($\mathrm{kmol\ H_2\ kmol^{-1}}$); $\bar c$ ($\mathrm{kmol\ C\ kmol^{-1}}$).

**Where it comes from.** The reference is the same flowsheet with the capture switched off ($\eta=0$) and the tail gas burned instead of recycled ($f=0$). With no cryogenic unit there is no hydrogen return, so $r=0$ and $\xi_{eff}=\xi$. Put these values into N.1.5: $D=1$, $\chi=4\psi'+\omega'$ and the $f\pi$ term vanishes. Same feed means the same gas composition, the same S/C 3.00, the same reformer exit 871 C and 439 psia, and the same shift train, so the reference carbon shares $\psi'$, $\omega'$ and $\kappa'$ come from the same equations D.11, E.4 and F.6, on a tube feed that holds no recycle.

**Substitution into the next equation.** $\dot n_{proc}$ is the first part of $\dot n_{ref}$ in N.7.3. Its reject stream gives $Q_{rej}$ in N.7.2.

**Intermediate calculations.** The bound with complete conversion ($\psi'=\omega'=0$) is $\dot n_{proc}\ge4458.599/(0.88\times4.13939)=1223.99\ \mathrm{kmol\ h^{-1}}=24.577$ MMSCFD. This follows from the equations alone. For orientation: the plant uses 23.10 MMSCFD; INL's whole reference (process gas and firing gas) at Case B size is $52.5\times0.6833=35.87$ MMSCFD, and INL's nuclear-heated plant without capture is $44.5\times0.6833=30.41$ MMSCFD. The actual $\psi'$ and $\omega'$ need a flowsheet run in the no-capture, no-recycle configuration (Phase 3); no value is guessed here.

**Final implementable formula.**

$$\dot n_{proc}=\frac{\dot n_{H_2}}{\xi\,[\,Z_{NG}-(4\psi'+\omega')\bar c\,]},\qquad T'=\bar c\,\dot n_{proc}$$

Result type: analytical given $\psi'$ and $\omega'$, which are numerical outputs.

Rust implementation: `Model::reference_ledger` and `LedgerKind::Reference` in `engine/plant/src/abatement.rs`: the flowsheet run with `CaptureSel::Off` (`flowsheet::NoCapture`), no recycle and no hydrogen return.

Validation test: `eq_n_7_1_reference_process_gas_and_its_limits` and `eq_n_7_floors_lie_below_every_published_reference` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.7.2: firing gas of the reference furnace

**What it calculates.** The extra natural gas that the reference burns in its furnace after burning all its PSA reject, so that the heat demands of the process (tubes and feed preheat) and of the steam system are met.

**Variables.** $\dot n_{fuel}$ ($\mathrm{kmol\ h^{-1}}$); $Q_F$ ($\mathrm{MJ\ h^{-1}}$); $\eta_F$ (1); $Q_{rej}$ ($\mathrm{MJ\ h^{-1}}$); $LHV_{NG}$ ($\mathrm{MJ\ kmol^{-1}}$); $\dot n_{H_2,14'}$, $\dot n_{CH_4,14'}$, $\dot n_{CO,14'}$ reject species of the reference ($\mathrm{kmol\ h^{-1}}$); $LHV_{H_2}$, $LHV_{CH_4}$, $LHV_{CO}$ ($\mathrm{MJ\ kmol^{-1}}$, numerically 241.8, 802.3 and 283.0).

**Where it comes from.** The furnace is an energy balance on the fuel: the usable heat from burning the reject and the firing gas is $\eta_F(Q_{rej}+\dot n_{fuel}LHV_{NG})$, and it must equal the heat $Q_F$ that the process takes. Solve for $\dot n_{fuel}$ and keep it non-negative. The reject heating value is the sum over its combustible species of flow times lower heating value; CO2 and N2 contribute nothing. $Q_F$ is the sum of the reformer tube duty and feed preheat duty of the reference flowsheet (the flowsheet computes these as `Q_SMR` and `Q_FEED_PREHEAT`, 93.57 and 16.07 MW in `FS_B_REC_F`) plus the net duty of the steam system, which includes the steam the reference exports.

**Substitution into the next equation.** $\dot n_{fuel}$ enters $\dot n_{ref}$ in N.7.3.

**Intermediate calculations.** None can be evaluated without the reference run. A consistency expectation: the legacy factor `EF_NG_LHV` = 56.1 $\mathrm{kg\ GJ^{-1}}$ together with $\bar c$ implies $LHV_{NG}=M_{CO_2}\bar c/e_{NG}=44.0095\times1.05379/56.1=0.8267\ \mathrm{GJ\ kmol^{-1}}$, which sits above $LHV_{CH_4}=802.3$ as expected for a gas with heavier hydrocarbons. The factor is not verified, so this is a cross-check to run in Phase 3 against a heating value computed from formation enthalpies (A.9, A.10).

**Final implementable formula.**

$$\dot n_{fuel}=\frac{1}{LHV_{NG}}\max\!\Big\{0,\ \frac{Q_F}{\eta_F}-Q_{rej}\Big\},\qquad Q_{rej}=\dot n_{H_2,14'}LHV_{H_2}+\dot n_{CH_4,14'}LHV_{CH_4}+\dot n_{CO,14'}LHV_{CO}$$

Result type: analytical in $\eta_F$ and $Q_F$.

**Physics, design input, literature.**
- Physics: the energy balance form, the reject composition from N.7.1, the heating values of H2, CH4 and CO, and the carbon identity of N.7.3.
- Design inputs: the furnace efficiency $\eta_F$ and the net steam-system duty inside $Q_F$ (new ids `REF_FURNACE_EFF`, `REF_STEAM_DUTY`, provenance assumed, no value proposed here).
- Literature values (validation rows only): INL 52.5 MMSCFD total gas, 3205 st/d CO2, 189.2 MMBtu/h exported steam and 6.3 MWe (ids `NG_CONV_NOCC`, `CO2_CONV_NOCC`, `EXPORT_STEAM_CONV`, `E_CONV_NOCC`), the 44.5 MMSCFD and 2713 st/d nuclear-heated case without capture (`NG_NORECYCLE`, `CO2_HTGR_NOCC`), the IEAGHG 8.996 kg/kg bound and the fleet-average 9.35 kg/kg (`BASELINE_IEAGHG_*`, `BASELINE_CHO`).

Rust implementation: `closed_form::firing_gas` in `engine/plant/src/abatement.rs`. The central result does not call it: the furnace efficiency and the steam duty have no source in the repository, so the reference carries no firing gas and every figure against it is a floor (open issue 2).

Validation test: `eq_n_7_2_firing_gas_is_not_negative_and_needs_no_invented_efficiency` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.7.3: direct emission of the reference plant

**What it calculates.** The CO2 the unabated plant puts into the air in a year.

**Variables.** $E_{ref}$ ($\mathrm{t\ CO_2\ y^{-1}}$); $\dot n_{proc}$, $\dot n_{fuel}$ ($\mathrm{kmol\ h^{-1}}$); $\bar c$; $P$, $K'$ ($\mathrm{kmol\ C\ h^{-1}}$); $v$ (1); $M_{CO_2}$; $t_h$.

**Where it comes from.** The reference has no capture and burns everything combustible that it does not sell, so every carbon atom that enters, in the process gas and in the firing gas, leaves as CO2 in the flue gas, except the carbon in the hydrogen product ($P$, the same product as the plant's) and the part of the condensate CO2 that stays in the water ($(1-v)K'$). This is one carbon balance with the stack as the only gas outlet. Convert with N.8.2.

**Substitution into the next equation.** $E_{ref}$ enters N.9.1 and N.10.2.

**Intermediate calculations (validation of the identity).** Applying the identity to INL's published gas flows, with the product CH4 of INL Case A (6.474 kmol/h) and no condensate term: 52.5 MMSCFD gives $(2614.6\times1.05379-6.474)$ kmol C/h, which is 3200.4 st/d against the published 3205 st/d (difference $-0.143$ percent); 44.5 MMSCFD gives 2711.6 st/d against 2713 st/d ($-0.052$ percent). Inverting, INL's 3205 st/d corresponds to 52.575 MMSCFD under the identity, against 52.5 published. The carbon identity is therefore consistent with INL's reference to 0.15 percent, and the only open quantity of the reference is its gas flow.

**Final implementable formula.**

$$E_{ref}=\frac{M_{CO_2}\,t_h}{1000}\Big[\bar c\,(\dot n_{proc}+\dot n_{fuel})-P-(1-v)K'\Big]$$

Result type: analytical.

Rust implementation: `Model::abatement` in `engine/plant/src/abatement.rs` (row `PLANT_FLOOR` from the reference ledger, rows `IEAGHG_9p00`, `INL_9p21` and `CHO_9p35` from the published intensities times the annual hydrogen) and `Model::emissions` (fields `baseline_central`, `baseline_id`). `Model::reference_carbon` and `Model::reference_gas_mmscfd` turn the central intensity into the carbon and the gas of the unabated reference (the carbon identity of this equation solved for the gas); the GAS line of the cost ledger, the saved upstream gas of the lifecycle view and the avoided-gas term of N.9.2 all read that gas. INL's 52.5 MMSCFD is a comparison input of the cost (`CostQuery::ng_smr`) and the legacy value. `Model::floor_reference` gives the plant-built reference (the floor of N.7.1 and N.7.3 without firing gas) as a cost comparison: its gas to price through `CostQuery::ng_smr` and its net abatement to divide by. The central result stays on the generic reference intensity of decision D2; `reference_cost_block` in `engine/plant/src/export.rs` prints the cost and the break-even shares under the floor beside the headline (rows `FLOOR_L5_*`, `FLOOR_LAC_DCF_*` and `FLOOR_BREAKEVEN_*` of `Final_Design_Basis.csv`).

Validation test: `eq_n_7_3_reference_identity_on_the_inl_gas_flows` `the_implied_gas_reproduces_inls_gas_when_given_inls_intensity`, `inls_reference_gas_is_a_comparison_with_a_reported_cost_effect`, `rule_1_of_n_7_holds_for_every_central_baseline`, `the_plant_built_floor_is_one_reference_for_gas_and_abatement`, `the_exported_floor_and_system_line_rows_are_the_engine_values`, `the_exported_floor_break_even_shares_meet_the_target_strictly` and the validation rows `IND_N73_INL_UNABATED`, `IND_N73_INL_HTGR_NOCC`, `IND_GAS_LINE_UNDER_INL_GAS`, `IND_L5_UNDER_INL_GAS`, `IND_LAC_DCF_UNDER_INL_GAS`, `IND_L5_UNDER_PLANT_FLOOR`, `IND_LAC_DCF_UNDER_PLANT_FLOOR`, `IND_BREAKEVEN_A_UNDER_PLANT_FLOOR` and `IND_BREAKEVEN_B_UNDER_PLANT_FLOOR` in `engine/plant/tests/m11_carbon.rs` and `engine/vv/src/harness.rs`.

## Equation N.7.4: steam and power terms of the system view

**What it calculates.** The CO2 consequence of the two differences between the plants that the direct view leaves out: the steam the plant no longer exports and the electric power the reference needed.

**Variables.** $E_{lost}$, $E_{pow}$ ($\mathrm{t\ CO_2\ y^{-1}}$); $\dot Q_{exp,ref}$, $\dot Q_{exp,pl}$ ($\mathrm{GJ\ h^{-1}}$); $e_{NG}$ ($\mathrm{t\ CO_2\ GJ^{-1}}$); $\eta_b$ (1); $\dot W_{ref}$ (MW); $EF_{grid}$ ($\mathrm{t\ MWh^{-1}}$); $t_h$.

**Where it comes from.** If the plant exports less steam than the reference, a consumer must raise the difference elsewhere. At the heat of a gas-fired boiler of efficiency $\eta_b$ this emits $(\dot Q_{exp,ref}-\dot Q_{exp,pl})\,e_{NG}/\eta_b$ per hour. The reference's electricity is a grid import with the grid factor $EF_{grid}$, which the plant avoids because its power comes from the reactor. This is card K.18 written with the quantities named here.

**Substitution into the next equation.** $A_{sys}=A_{dir}-E_{lost}+E_{pow}$, printed beside $A_{dir}$.

**Intermediate calculations (legacy basis).** Power: $6.3\times0.6833\times7446\times0.402=12885\ \mathrm{t\ y^{-1}}$. The CSV system row is $527631.56$ against $578051.61$ for the direct row, a difference of $-50420.05$; so the lost-steam term implied by the ledger is $12885.13+50420.05=63305.19\ \mathrm{t\ y^{-1}}$. When both plants export the same steam, $E_{lost}=0$ (the script proves it).

**Final implementable formula.**

$$E_{lost}=(\dot Q_{exp,ref}-\dot Q_{exp,pl})\,t_h\,\frac{e_{NG}}{\eta_b},\qquad E_{pow}=\dot W_{ref}\,t_h\,EF_{grid},\qquad A_{sys}=A_{dir}-E_{lost}+E_{pow}$$

Result type: analytical.

Rust implementation: `closed_form::system_terms` in `engine/plant/src/abatement.rs` and the legacy `Model::system_adjust` in `engine/plant/src/emissions.rs`, which prints the system view. The two cost lines that correspond to these terms, LOST_STEAM and BASE_POWER, price the steam export and the purchased power of INL's conventional plant (`EXPORT_STEAM_CONV` and `E_CONV_NOCC`, TEV-953 Table 2) scaled to the plant. They are comparison inputs, because no source gives the steam duty or the power of a plant-built reference. `CostQuery::system_lines` (default true, the ladder as committed) switches both off in `Model::cost_items`, so the ledger and the cash flow read the same cost without them; `Final_Design_Basis.csv` prints that cost beside the headline (rows `NOSYS_*` and `SYSLINES_*`).

Validation test: `eq_n_7_4_system_terms_are_added_once`, `the_two_system_lines_vanish_together_when_they_are_switched_off`, `the_system_lines_are_on_by_default_and_unchanged_on_the_legacy_basis` and the validation rows `IND_L5_WITHOUT_SYSTEM_LINES` and `IND_LAC_DCF_WITHOUT_SYSTEM_LINES` in `engine/plant/tests/m11_carbon.rs` and `engine/vv/src/harness.rs`.

## Boundary of the reference and the cost ladder lines (N.7)

Both plants take the same gas composition at the battery limit and deliver the same hydrogen flow at the same purity. Both are counted gate to gate: CO2 that reaches the air from the site is an emission, CO2 delivered at the compression outlet (S20) is captured and counted before transport and storage. The hydrogen product carries the same carbon $P$ on both sides, so it cancels. The rules below say where each physical difference is counted, so that nothing is counted twice.

| Item | Reference plant | Plant | Carbon, direct view | Carbon, system view | Cost ladder line |
|---|---|---|---|---|---|
| Fuel for the reformer duty | fired furnace: reject plus firing gas $\dot n_{fuel}$ | reactor heat through helium, no fuel | fuel CO2 is inside $E_{ref}$ (N.7.3) | already counted | GAS prices the gas difference $\dot n_{ref}-\dot n_{NG}$ in Singapore dollars |
| Process gas | $\dot n_{proc}$ | $\dot n_{NG}$ | inside $E_{ref}$ and $E_{plant}$ | already counted | GAS (same difference) |
| Exported steam | $\dot Q_{exp,ref}$, its fuel is inside $E_{ref}$ | $\dot Q_{exp,pl}$ | not netted | $E_{lost}$ added once | LOST_STEAM |
| Electric power | $\dot W_{ref}$ imported | from the reactor | no entry | $E_{pow}$ added once | BASE_POWER |
| Hydrogen product | $P$ in the product | $P$ in the product | cancels | cancels | none |
| CO2 to storage | none | $C$ | through N.9.2 | through N.9.2 | capture and transport and storage lines |

Four rules follow.
1. One source for the gas difference. The GAS line is the price times $\dot n_{ref}-\dot n_{NG}$ (`Model::cost_items` and `Model::ledger` in `engine/plant/src/economics.rs`) and the avoided-gas carbon in N.9.2 is $\bar c(\dot n_{ref}-\dot n_{NG})$. Both must read the same two numbers, taken from N.1.5 and N.7. The test `rule_1_of_n_7_the_gas_line_the_decomposition_and_the_lifecycle_read_one_gas_difference` compares them, and the saved upstream gas of the lifecycle view (N.9.5) reads the same difference. The rule holds at every sensitivity point that changes the baseline: such a point selects the baseline through `CARBON_BASELINE` (`solve` in `engine/plant/src/sens.rs`), so the gas and the abatement follow it together and no point shifts the abatement alone. The tests `rule_1_of_n_7_holds_at_every_sensitivity_point_that_changes_the_baseline` and `the_baseline_sensitivity_group_equals_the_carbon_baseline_group` check the points of the BASELINE and CARBON_BASELINE groups.
2. The system terms $E_{lost}$ and $E_{pow}$ are added once, and only in the system view. They are never folded into $E_{ref}$. The fuel burned for the reference's exported steam stays inside $E_{ref}$ in the direct view, and $E_{lost}$ removes the same energy once in the system view.
3. Pair the cost level with the carbon view. The ladder from L0 to L5 reaches L5 only after adding LOST_STEAM and BASE_POWER, whose carbon consequences exist only in $A_{sys}$. The legacy headline divides L5 by the direct abatement, which uses a carbon boundary that leaves those two items out while the cost includes them. On the legacy B_REC_F row the system abatement is 527631.56 against 578051.61, 8.72 percent smaller, so the same cost over $A_{sys}$ is 9.6 percent higher. This is a mismatch of boundaries and leaves nothing counted twice; it is listed as open issue 3 for a decision.
4. Equal products. The two plants must share the same hydrogen flow and the same product CH4, so that $P$ cancels in N.9.2. A reference built for a different purity would add a term $k(P_{ref}-P)$ that the formula does not contain.

## Equation N.8.1: operating hours

**What it calculates.** The hours per year at full output.

**Variables.** $t_h$ ($\mathrm{h\ y^{-1}}$); $f_{av}$ (1); 8760 $\mathrm{h\ y^{-1}}$ (the calendar year).

**Where it comes from.** A year has $24\times365=8760$ h. The plant makes its design output for the fraction $f_{av}$ of them (K.10).

**Substitution into the next equation.** $t_h$ enters N.8.2 and every annual quantity.

**Intermediate calculations.** $8760\times0.85=7446\ \mathrm{h\ y^{-1}}$, which equals $24\times(0.85\times365)=24\times310.25$, the form used for the INL baseline in the engine.

**Final implementable formula.**

$$t_h=8760\,f_{av}$$

Result type: analytical.

Rust implementation: `Model::new` in `engine/plant/src/anchored.rs` (field `hours`).

Validation test: `eq_n_8_1_operating_hours` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.8.2: kmol per hour to tonnes per year

**What it calculates.** The annual mass of CO2 for a carbon flow given in kmol of C per hour.

**Variables.** $\dot n_C$ carbon flow ($\mathrm{kmol\ C\ h^{-1}}$); $M_{CO_2}$ ($\mathrm{kg\ CO_2\ kmol^{-1}\ CO_2}$); $t_h$ ($\mathrm{h\ y^{-1}}$); $\dot E$ ($\mathrm{t\ CO_2\ y^{-1}}$).

**Where it comes from.** One kmol of C atoms in CO2 gives one kmol of CO2, which has mass $M_{CO_2}$ in kg. Mass per hour is $\dot n_CM_{CO_2}$ in $\mathrm{kg\ h^{-1}}$. Multiply by $t_h$ to get kg per year and divide by 1000 to get tonnes.

**Substitution into the next equation.** Used by N.5.2, N.7.3, N.9.3 and in every row of the output tables.

**Intermediate calculations.** The factor is $44.0095\times7446/1000=327.695\ \mathrm{t\ y^{-1}}$ per $\mathrm{kmol\ h^{-1}}$. Check: $1108.584\times327.695=363277\ \mathrm{t\ y^{-1}}$.

**Final implementable formula.**

$$\dot E=\dot n_C\,M_{CO_2}\,\frac{t_h}{1000}=\dot n_C\times327.695\quad[\mathrm{t\ y^{-1}}]\ \text{at}\ f_{av}=0.85$$

Result type: analytical. The legacy function `tpy` multiplies by a scale $s$ (Case B over Case A); the independent flowsheet is solved on the Case B basis directly, so no scale appears.

Rust implementation: `Model::tpy` in `engine/plant/src/anchored.rs` and `closed_form::tonnes_per_kmol_h` in `engine/plant/src/abatement.rs`.

Validation test: `eq_n_8_2_annual_mass_of_a_carbon_flow` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.8.3: annual hydrogen and conversion of external references

**What it calculates.** The annual hydrogen mass for the per-kg quantities, and the conversions that bring INL's gas and CO2 numbers to kmol per hour for the validation rows.

**Variables.** $M_{H_2,y}$ ($\mathrm{t\ y^{-1}}$); $\dot m_{H_2}$ ($\mathrm{kg\ h^{-1}}$); $X_{st}$ CO2 in short tons per day ($\mathrm{st\ d^{-1}}$); $X_g$ gas in $\mathrm{MMSCFD}$; $ST=0.90718474\ \mathrm{t\ st^{-1}}$; $LB=0.45359237\ \mathrm{kg\ lb^{-1}}$; $SCF=379.49\ \mathrm{scf\ lbmol^{-1}}$.

**Where it comes from.** Annual mass is hourly mass times hours (J.3): $M_{H_2,y}=\dot m_{H_2}t_h/1000$. A short ton per day is $ST\times1000/24\ \mathrm{kg\ h^{-1}}$, so $\dot n=X_{st}\,ST\times1000/(24M_{CO_2})$. A million standard cubic feet per day is $10^6/(SCF\times24)$ lbmol per hour, and a lbmol is $LB$ kmol (J.4).

**Substitution into the next equation.** $M_{H_2,y}$ is the denominator of N.10.

**Intermediate calculations.** $M_{H_2,y}=8988\times7446/1000=66924.65\ \mathrm{t\ y^{-1}}$. 1927 st/d gives 1655.08 kmol/h. 1 MMSCFD gives 49.803 kmol/h. 

**Final implementable formula.**

$$M_{H_2,y}=\frac{\dot m_{H_2}\,t_h}{1000},\quad\dot n[\mathrm{kmol\ h^{-1}}]=\frac{X_{st}\,ST\,1000}{24\,M_{CO_2}},\quad\dot n_{gas}=\frac{X_g\,10^6\,LB}{SCF\times24}$$

Result type: analytical.

Rust implementation: `nush2_props::units::{st_d_to_kmol_h, mmscfd_to_kmol_h}` and the field `h2_t_y` of `Emissions` in `engine/plant/src/emissions.rs`.

Validation test: `eq_n_8_3_annual_hydrogen_and_the_unit_conversions` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.9.1: direct emissions avoided

**What it calculates.** The CO2 that the plant keeps out of the air compared with the unabated reference making the same hydrogen. This is the headline abatement in kg CO2 (decision D4).

**Variables.** $A_{dir}$, $E_{ref}$, $E_{plant}$ ($\mathrm{t\ CO_2\ y^{-1}}$).

**Where it comes from.** Definition: the difference of the two annual direct emissions, N.7.3 minus N.5.2.

**Substitution into the next equation.** N.9.2 rewrites $A_{dir}$ in terms of the captured CO2 and the avoided gas. N.9.4 subtracts the transport and storage burden.

**Intermediate calculations.** Legacy B_REC_F at the INL baseline: $616360.62-38309.01=578051.61\ \mathrm{t\ y^{-1}}$ (card K.11), the value in `results/Emissions_Ledger.csv`. The independent value needs N.7.2 inputs and is not computed here.

**Final implementable formula.**

$$A_{dir}=E_{ref}-E_{plant}$$

Result type: analytical.

Rust implementation: `Model::emissions` (field `a_direct`) in `engine/plant/src/emissions.rs` and `BaselineRow::a_direct` in `engine/plant/src/abatement.rs`.

Validation test: `eq_n_9_1_hand_ledger_of_the_direct_abatement` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.9.2: where the abatement comes from

**What it calculates.** A split of $A_{dir}$ into the CO2 that is captured and the carbon of the gas that the plant does not buy. It also shows which terms the cost ladder prices. Picture: the plant saves CO2 in two ways, by catching carbon that the reference would release and by never buying the gas that the reference burns for heat.

**Variables.** $k=M_{CO_2}t_h/1000$ ($\mathrm{t\ y^{-1}}$ per $\mathrm{kmol\ h^{-1}}$); $C$, $\bar c$, $\dot n_{ref}$, $\dot n_{NG}$, $K$, $K'$, $v$ as before.

**Where it comes from.** From N.7.3, $E_{ref}=k[\bar c\dot n_{ref}-P-(1-v)K']$. From N.5.2, $E_{plant}=k[B+vK]$. The closure of N.6.1 with no unknown streams gives $B=\bar c\dot n_{NG}-C-P-K$. Put this into $E_{plant}$: $E_{plant}=k[\bar c\dot n_{NG}-C-P-(1-v)K]$. Subtract: the product carbon $P$ cancels, and
$A_{dir}=k[\bar c(\dot n_{ref}-\dot n_{NG})+C-(1-v)(K'-K)]$.

**Substitution into the next equation.** The first term is priced by GAS; the second is the capture-based quantity of N.9.3.

**Intermediate calculations.** With $v=1$ the condensate terms vanish. Raising $v$ from 0 to 1 raises $E_{plant}$ by exactly $kK$ (proved in the script), which for `FS_B_REC_F` is 1730.6 $\mathrm{t\ y^{-1}}$. Legacy B_REC_F: $C=1097.904\ \mathrm{kmol\ h^{-1}}$ (captured) and $\bar c\Delta\dot n=1.05379\times629.4=663.41\ \mathrm{kmol\ C\ h^{-1}}$, where $\Delta\dot n=(52.5-34)\times0.6833=12.641$ MMSCFD. So $A_{dir}=327.695\times(1097.904+663.406)=577172\ \mathrm{t\ y^{-1}}$ against the ledger's 578052, a difference of $-0.152$ percent. The gap is INL's rounding of 3205 st/d and 52.5 MMSCFD (N.7.3). The split is 359777 t/y captured (62.3 percent) and 217395 t/y avoided gas carbon (37.7 percent).

**Final implementable formula.**

$$A_{dir}=k\Big[C+\bar c\,(\dot n_{ref}-\dot n_{NG})-(1-v)(K'-K)\Big]$$

Sensitivity that follows directly: each MMSCFD of reference gas (49.803 $\mathrm{kmol\ h^{-1}}$) moves $A_{dir}$ by $49.803\times1.05379\times327.695=17.2\ \mathrm{kt\ y^{-1}}$. Result type: analytical.

Scope of the decomposition. It uses the closure of the modelled streams. If an unmodelled inlet $\varepsilon^{in}$ or an unmodelled outlet that does not reach the air $\varepsilon^{out,other}$ is present, the carbon balance becomes $\bar c\dot n_{NG}+\sum\varepsilon^{in}=C+B+P+K+\sum\varepsilon^{out}$ and $A_{dir}$ of N.9.1 differs from the formula above by exactly $k\,(\sum\varepsilon^{out,other}-\sum\varepsilon^{in})$. For instance MEA carbon that enters and leaves to the air makes the formula overstate $A_{dir}$ by $k\,\varepsilon_{MEA}$. The definition to implement is N.9.1; the decomposition is a consistency test that holds only while those streams are zero or balanced (the script proves the difference symbolically).

Lower bound for the present flowsheet run: with $\dot n_{ref}\ge1223.99\ \mathrm{kmol\ h^{-1}}$ (N.7.1) and $v=1$, $A_{dir}\ge327.695\times[1108.584+1.05379\times(1223.99-1150.37)]=388700\ \mathrm{t\ y^{-1}}$. It assumes the cryogenic inputs of that run and a reference with no firing gas and complete conversion; it is a floor and not an estimate.

Rust implementation: `Decomposition` built in `Model::abatement` in `engine/plant/src/abatement.rs`, with the reference carbon and gas from `Model::reference_carbon` and `Model::reference_gas_mmscfd` (the gas of the GAS line, rule 1 of N.7).

Validation test: `eq_n_9_2_decomposition_of_the_direct_abatement` and `rule_1_of_n_7_the_gas_line_the_decomposition_and_the_lifecycle_read_one_gas_difference` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.9.3: capture-based abatement

**What it calculates.** The CO2 delivered to storage in a year, before transport and storage.

**Variables.** $A_{cap}$ ($\mathrm{t\ CO_2\ y^{-1}}$); $C$ ($\mathrm{kmol\ h^{-1}}$).

**Where it comes from.** N.8.2 applied to the captured flow, K.7.

**Substitution into the next equation.** $A_{cap}$ is the base of the transport and storage burden in N.9.4 and the first term of N.9.2.

**Intermediate calculations.** Legacy: $1097.904\times327.695=359777.35\ \mathrm{t\ y^{-1}}$. Present flowsheet run: $1108.584\times327.695=363277\ \mathrm{t\ y^{-1}}$, which is 0.97 percent above the legacy value.

**Final implementable formula.**

$$A_{cap}=k\,C=\frac{M_{CO_2}\,t_h}{1000}\,\eta\,\varphi\,T$$

Result type: analytical.

Rust implementation: `Emissions::captured` and `Abatement::captured_t_y` in `engine/plant/src/emissions.rs` and `engine/plant/src/abatement.rs`.

Validation test: `eq_n_9_3_capture_based_abatement` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.9.4: net abatement after transport and storage

**What it calculates.** The direct abatement less the CO2 that returns to the air between the plant and the permanent store.

**Variables.** $A_{TS}$ ($\mathrm{t\ CO_2\ y^{-1}}$); $\tau_{TS}$ ($\mathrm{t\ t^{-1}}$); $\lambda$ (1); $e_{ship}$ ($\mathrm{t\ t^{-1}}$); $w_{TS}$ ($\mathrm{MWh\ t^{-1}}$); $EF_{el}$ ($\mathrm{t\ MWh^{-1}}$).

**Where it comes from.** Two kinds of effect act on each tonne delivered at S20. A loss: a fraction $\lambda$ escapes on the way to the store and reaches the air again. An energy use: the transport fuel emits $e_{ship}$ tonnes of CO2 per tonne moved, and liquefaction and injection use $w_{TS}$ MWh of electricity per tonne at the emission factor $EF_{el}$. Adding the two kinds gives the burden per tonne, $\tau_{TS}=\lambda+e_{ship}+w_{TS}EF_{el}$, and it applies to $A_{cap}$.

**Substitution into the next equation.** $A_{TS}$ is the base of the lifecycle view N.9.5.

**Intermediate calculations.** The four inputs are symbolic and come from the transport and storage research; none is guessed. The legacy ledger lumps them into one number, `TS_BURDEN` = 0.025 $\mathrm{t\ t^{-1}}$ (assumed, not verified), giving $0.025\times359777.35=8994.43\ \mathrm{t\ y^{-1}}$ and $A_{TS}=569057.18\ \mathrm{t\ y^{-1}}$ on the legacy row. With all inputs zero $A_{TS}=A_{dir}$ (proved in the script).

**Final implementable formula.**

$$A_{TS}=A_{dir}-\tau_{TS}\,A_{cap},\qquad\tau_{TS}=\lambda+e_{ship}+w_{TS}\,EF_{el}$$

Result type: analytical. The plant's own CO2 compression (S19 to S20) is inside the plant; its electricity belongs to the system view (card K.9, not implemented).

Rust implementation: `Model::ts_parts`, `closed_form::ts_ratio` and `Model::emissions` (fields `ts_emission`, `a_net`) in `engine/plant/src/abatement.rs` and `engine/plant/src/emissions.rs`; the inputs are the `TS_*` registry entries.

Validation test: `eq_n_9_4_net_abatement_after_transport_and_storage` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.9.5: lifecycle view and the gas basis rule

**What it calculates.** The abatement with the supply chain of the natural gas and the nuclear plant added, in CO2 and, when requested, in CO2e.

**Variables.** $A_{LC}$ ($\mathrm{t\ y^{-1}}$); $\Delta G$ heat of the gas not bought ($\mathrm{GJ\ y^{-1}}$); $e_{up,CO_2}$, $e_{up,CH_4}$ ($\mathrm{t\ GJ^{-1}}$); $GWP_H$ ($\mathrm{t\ CO_2e\ t^{-1}\ CH_4}$) for the horizon $H$; $E_{nuc}$ ($\mathrm{t\ CO_2\ y^{-1}}$).

**Where it comes from.** The plant buys less gas than the reference, by $\dot n_{ref}-\dot n_{NG}$. The saved gas also saves its upstream emissions: CO2 from the energy used in production and transport, and leaked methane. Methane enters a CO2e total only through $GWP_H$. The nuclear plant adds its own life-cycle emission $E_{nuc}$. This is card K.19 with the gas basis made explicit. By decision D4 the headline quantities are kg CO2: the CO2 view sets $GWP_H=0$. A CO2e quantity appears only here, with the horizon and the source of $GWP_H$ printed beside it.

**Substitution into the next equation.** $A_{LC}$ is reported with a label for its gas basis. N.10.1 can divide any view by the annual hydrogen.

**Intermediate calculations.** Legacy B_REC_F: $578051.61-8994.43-2902.01=566155.16\ \mathrm{t\ y^{-1}}$ (CSV, burdens view); the saved upstream gas is $\Delta G=4.32\times10^{6}\ \mathrm{GJ\ y^{-1}}$ on the legacy HHV basis (the gas difference turned into heat: $\Delta G=(\dot n_{ref}-\dot n_{NG})\,H_{NG}\,t_h/1000$ with $H_{NG}$ the heating value per kmol of gas in $\mathrm{MJ\ kmol^{-1}}$, the basis, higher or lower, being stated beside it; the legacy ledger evaluates it as $12.6407\ \mathrm{MMSCFD}\times1044\ \mathrm{Btu\ scf^{-1}}\times1.05505585\ \mathrm{GJ\ MMBtu^{-1}}\times310.25\ \mathrm{d\ y^{-1}}=4319752\ \mathrm{GJ\ y^{-1}}$), giving $4.32\times10^{6}\times11.5/1000=49677.15\ \mathrm{t\ y^{-1}}$ and a lifecycle total of $615832.31$. The legacy factor 11.5 $\mathrm{kg\ GJ^{-1}}$ does not state its gas basis or its heating-value basis; both must be stated beside any value that replaces it. Setting $GWP_H=0$ returns the CO2 view; the CO2e view exceeds it by exactly $GWP_He_{up,CH_4}\Delta G$ (proved in the script).

**Final implementable formula.**

$$A_{LC}=A_{TS}+\Delta G\,\big(e_{up,CO_2}+GWP_H\,e_{up,CH_4}\big)-E_{nuc}$$

Result type: analytical. No value of $GWP_H$ is assumed in this file; the horizon and the report that supplies it are a owner decision (open issue 6).

Rust implementation: `Model::lifecycle_terms` in `engine/plant/src/emissions.rs` and `closed_form::lifecycle` in `engine/plant/src/abatement.rs`. The sources read state the upstream intensity only as one CO2e total (`EF_UPSTREAM`, 11.5 kg CO2e per GJ at a 100-year potential of 30, IEA 2026, labelled by the parameter `EF_UPSTREAM_GWP100`), so $e_{up,CO_2}$ and $e_{up,CH_4}$ cannot be separated without an invented share. The total enters the CO2 slot of the formula with the methane term at zero. The fields are named `up_credit_co2e` and `nuclear_co2e`, the nuclear factor is also a CO2e range whose horizon the source read does not state, and every row of the two lifecycle views of the emissions table says CO2e with the horizon and the source. The saved gas $\Delta G$ is priced on the reference gas of rule 1 of N.7.

Validation test: `eq_n_9_5_lifecycle_view_counts_transport_and_storage_once`, `eq_n_9_5_the_exported_lifecycle_is_the_closed_form_on_the_aggregate_factor` and `no_co2e_quantity_appears_without_a_label_in_any_view` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.10.1: abatement per kg of hydrogen

**What it calculates.** Tonnes (or kg) of CO2 avoided per tonne (or kg) of H2 delivered.

**Variables.** $a_{H_2}$ ($\mathrm{t\ CO_2\ t^{-1}\ H_2}$); $A$ one of $A_{dir}$, $A_{TS}$, $A_{LC}$ ($\mathrm{t\ y^{-1}}$); $M_{H_2,y}$ ($\mathrm{t\ y^{-1}}$).

**Where it comes from.** Annual CO2 over annual hydrogen (K.13, N.8.3). The numerical value in kg per kg equals the value in t per t.

**Substitution into the next equation.** None.

**Intermediate calculations.** Legacy: $578051.61/66924.65=8.637350$, equal to the ledger column. The label of the view is part of the quantity.

**Final implementable formula.**

$$a_{H_2}=\frac{A}{M_{H_2,y}}$$

Result type: analytical.

Rust implementation: `BaselineRow::a_h2_net` and `a_h2_direct` in `engine/plant/src/abatement.rs`; the summary key `A_net_per_kg` of `Model::plant_summary`.

Validation test: `eq_n_10_intensities_and_the_reduction` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.10.2: fractional reduction

**What it calculates.** The share of the reference emission that the plant avoids.

**Variables.** $\rho_{red}$ (1); $A_{dir}$, $E_{ref}$ ($\mathrm{t\ y^{-1}}$).

**Where it comes from.** $\rho_{red}=A_{dir}/E_{ref}=1-E_{plant}/E_{ref}$ by N.9.1.

**Substitution into the next equation.** None.

**Intermediate calculations.** Legacy: $578051.61/616360.62=0.93785$. The plant-capture ratios of N.3.2 and the legacy column $C/(C+B)=0.903767$ are different quantities and are printed with their own names.

**Final implementable formula.**

$$\rho_{red}=\frac{A_{dir}}{E_{ref}}=1-\frac{E_{plant}}{E_{ref}}$$

Result type: analytical.

Rust implementation: `BaselineRow::reduction` in `engine/plant/src/abatement.rs`.

Validation test: `eq_n_10_intensities_and_the_reduction` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.10.3: emission intensity

**What it calculates.** The plant's direct CO2 per tonne of hydrogen, and the same for the reference.

**Variables.** $ci$, $e_{ref}$ ($\mathrm{t\ CO_2\ t^{-1}\ H_2}$); $E_{plant}$, $E_{ref}$ ($\mathrm{t\ y^{-1}}$).

**Where it comes from.** K.13 with N.8.3: $ci=E_{plant}/M_{H_2,y}$ and $e_{ref}=E_{ref}/M_{H_2,y}$. Then $a_{H_2}=e_{ref}-ci$ for the direct view (proved in the script).

**Substitution into the next equation.** None.

**Intermediate calculations.** Legacy: $ci=38309.01/66924.65=0.572420$ and $e_{ref}=9.20977$. Present flowsheet streams: stack only $30778.47/66924.65=0.45990$, and with the condensate CO2 vented ($v=1$) $32509.03/66924.65=0.48576$. Floors: $M_{CO_2}/(4M_{H_2})=5.4579$ for the chemistry (K.5), and $5.9358$ for the feed carbon of the present run (K.6).

**Final implementable formula.**

$$ci=\frac{E_{plant}}{M_{H_2,y}},\qquad e_{ref}=\frac{E_{ref}}{M_{H_2,y}}$$

Result type: analytical.

Rust implementation: `Abatement::intensity_kg_kg` in `engine/plant/src/abatement.rs`.

Validation test: `eq_n_10_intensities_and_the_reduction` in `engine/plant/tests/m11_carbon.rs`.

## Equation N.10.4: reported carbon balance residual

**What it calculates.** The residual and the stream statuses that the output tables print, replacing the anchored value that prints 0.000000 by construction.

**Variables.** $\rho$ (1); $\tau_C$ (1); status of each stream.

**Where it comes from.** N.6.1, evaluated on the plant's own stream ledger at run time, with the tolerance printed beside it. The anchored closure is an identity of how the legacy ledger is solved and cannot report a physical error.

**Substitution into the next equation.** None; it ends the chain.

**Intermediate calculations.** For `FS_B_REC_F`: $\rho=4.3\times10^{-12}$ against $\tau_C=10^{-9}$, verdict "undetermined" until the streams in the register below receive a status other than unknown (the verdict "closed on modelled streams" is printed beside the list of unknown streams).

**Final implementable formula.**

$$\text{report}\ \big(\rho,\ \tau_C,\ \text{verdict},\ \{\text{stream},\ \text{status}\}\big)$$

Result type: analytical.

Rust implementation: `Abatement::closure` (a `carbon::AccountReport`) in `engine/plant/src/abatement.rs`, written to the emissions ledger as `CLOSURE_ABS`, `CLOSURE_REL` and `CLOSURE_TOL` by `independent_lines` in `engine/plant/src/export.rs`.

Validation test: `eq_n_10_4_reported_residual_carries_its_tolerance_and_statuses` in `engine/plant/tests/m11_carbon.rs`.

# Worked numeric example: the central case

The central case is Case B, 8988 kg/h of hydrogen, MEA capture with recycle (`B_REC_F`), availability 0.85, INL baseline (`INL_9p21`). Part A repeats the legacy anchored basis with the equations above, so that the numbers of `results/Emissions_Ledger.csv` are reproduced. Part B applies the same equations to the plant's own streams (`FS_B_REC_F`). Part C says what follows about the size of the change.

## Part A: legacy anchored basis, reproduced

| Step | Calculation | Result |
|---|---|---|
| Scale | $s_b=(100000\times0.08988)/(29000\times0.45359237)=8988/13154.18$ | 0.683281 |
| Feed carbon, Case A | $34\ \mathrm{MMSCFD}\to1693.3\ \mathrm{kmol\ h^{-1}}$; $\times1.05379$ (N.1.1) | $F_{C,6}=1784.379$ |
| Product carbon, Case A | $130\ \mathrm{MMSCFD}\to6474.4\ \mathrm{kmol\ h^{-1}}$; $\times0.001$ | $P_A=6.4744$ |
| INL capture | $1927\ \mathrm{st\ d^{-1}}\to1655.083\ \mathrm{kmol\ h^{-1}}$ (N.8.3) | $C_6=1655.083$ |
| INL tube carbon (J.7) | $T_6=1655.083/(0.99\times0.775)$ | 2157.163 |
| INL tail gas | $G_6=2157.163-1655.083-6.4744$ | 495.605 |
| INL recycle fraction (N.4.1 form) | $f=(2157.163-1784.379)/495.605$ | 0.752178 |
| MEA tube carbon (N.2.1 with $\kappa=0$, $\varphi=\phi_c$) | $(1784.379-0.752178\times6.4744)/[1-0.752178\times(1-0.9\times0.775)]=1779.509/0.772466$ | $T=2303.67$ |
| Captured (N.3.1) | $0.9\times0.775\times2303.673$ | $C=1606.812$ |
| Tail gas (N.2.2) | $2303.673-1606.812-6.4744$ | $G=690.387$ |
| Burned (N.2.2) | $(1-0.752178)\times690.387$ | $B=171.093$ |
| Closure (N.6.1) | $1784.379-(1606.812+171.093+6.4744)$ | $<10^{-9}$ |
| Case B scale | $\times0.683281$ | $C=1097.904$, $B=116.905$ |
| Annual (N.8.2) | $\times327.695\ \mathrm{t\ y^{-1}}$ per $\mathrm{kmol\ h^{-1}}$ | captured 359777.35, residual 38309.01 |
| Baseline (K.3) | $3205\times0.90718474\times310.25\times0.683281$ | 616360.62 |
| Abatement (N.9.1) | $616360.62-38309.01$ | 578051.61 $\mathrm{t\ y^{-1}}$ |
| Per kg H2 (N.10.1) | $578051.61/66924.65$ | 8.637350 |
| Intensities (N.10.2, N.10.3) | baseline $9.20977$; plant $0.572420$; reduction | 0.93785 |
| Decomposition (N.9.2) | $327.695\times(1097.904+663.406)$ | 577172 (the ledger says 578052, $-0.152$ percent) |
| Transport and storage (N.9.4) | $0.025\times359777.35$ | 8994.43; $A_{TS}=569057.18$ |
| System view (N.7.4) | power credit 12885.13; ledger system row $-50420.05$ | implied lost steam 63305.19 |

All entries of the left half agree with the CSV to the printed digits: captured, residual, baseline, abatement, the per-kg value, the transport and storage burden (8994.434), the saved upstream gas (49677.145) and the Case B kmol/h values 1097.904, 116.905 and 1219.232 of `Validation_Results.csv`. The script asserts every one of them.

## Part B: the same equations on the plant's own streams

`FS_B_REC_F` solves the flowsheet on the 8988 kg/h basis. Its cryogenic inputs are still $f=0.752178$ and $r=0.921630$, so this part measures the change of accounting (streams in place of the anchored algebra) and does not yet include the effect of replacing $f$ and $r$.

| Quantity | Legacy anchored (Case B) | Plant streams | Change |
|---|---|---|---|
| Feed carbon $F_C$ ($\mathrm{kmol\ C\ h^{-1}}$) | 1219.232 | 1212.252 | $-0.57$ percent |
| Tube carbon $T$ | 1574.056 | 1497.327 | $-4.87$ percent |
| Captured $C$ | 1097.904 | 1108.584 | $+0.97$ percent |
| Product carbon $P$ | 4.424 | 4.463 | $+0.89$ percent |
| Reject $G$ | 471.728 | 378.999 | $-19.66$ percent |
| Recycled $R$ | 354.824 | 285.075 | $-19.66$ percent |
| Burned $B$ | 116.905 | 93.924 | $-19.66$ percent |
| Condensate $K$ | not in the ledger | 5.281 | new term |
| Captured, annual ($\mathrm{t\ y^{-1}}$) | 359777 | 363277 | $+0.97$ percent |
| Stack, annual | 38309 | 30778 | $-19.66$ percent |
| Stack plus vented condensate ($v=1$) | 38309 | 32509 | $-15.14$ percent |
| Stack intensity $ci$ ($\mathrm{t\ t^{-1}}$) | 0.572420 | 0.45990 (0.48576 with $v=1$) | |
| Plant capture $C/F_C$ | 0.9005 (legacy $C/F_C$) | 0.91448 | |

The reject, recycle and burned carbon all move by the same 19.66 percent because the legacy ledger and the flowsheet share $f$ and differ in $\varphi$ ($\phi_c=0.775$ against 0.8226). A higher $\varphi$ means more CO2 reaches the absorber and less carbon is left in the reject.

## Part C: what follows about the size of the change

- Captured CO2 rises by 0.97 percent and the stack falls by 19.66 percent (15.14 percent when the condensate CO2 is vented) for the present cryogenic inputs. These follow from N.2 and N.3 with the shares of the existing flowsheet run.
- The abatement $A_{dir}$ of the independent basis cannot be stated yet. N.9.2 shows it equals $k[C+\bar c(\dot n_{ref}-\dot n_{NG})]$ for $v=1$; the reference gas $\dot n_{ref}$ needs the inputs of N.7.2. Two facts do follow: a floor of 388700 $\mathrm{t\ y^{-1}}$ for the present flowsheet inputs (reference without firing gas and with complete conversion), and a slope of 17.2 $\mathrm{kt\ y^{-1}}$ per MMSCFD of reference gas.
- Replacing $f$ and $r$ changes the stack, the recycle and the gas rate; N.1.5, N.2.1 and N.4.4 give the response once the central values are chosen (the stack curve of N.4.4 is exact at the converged $f$ and an approximation away from it). The slope at the present point and at the fixed hydrogen output of the plant is $-1205\ \mathrm{t\ y^{-1}}$ of stack CO2 per 0.01 of $f$ ($-1144$ at a fixed gas rate, see N.4.4). No new central value is proposed here.

# What is physics, design input or literature value (N.3, N.4, N.7)

| Item | Physics (follows from balances and equilibria) | Design input | Literature value (validation or legacy only) | Parameter ids |
|---|---|---|---|---|
| N.3 captured CO2 | CO2 reaching the absorber $\varphi T$ from D.11, E.4, F.6; solvent CO2 balance $\dot n_{MEA}(\alpha_{rich}-\alpha_{lean})=C$; $S19=C$ | per-pass capture $\eta$ (column gate M6 failed) | none for MEA; Selexol 0.99 for the legacy basis | `ETA_MEA` (existing); new `CO2_PROC_VENT`, `MEA_CARBON` |
| N.4 recycle fraction | loop closure N.2.1; read-back $f=R/G$; nitrogen balance N.4.2 | cryogenic carbon recovery $f$ and hydrogen return $r$ | INL Case 6: 1927 st/d, 0.99, $\phi_c=0.775$ giving 0.752178; 34 to 130 MMSCFD giving $r=0.921630$ | new `CRYO_C_RECOVERY`, `CRYO_H2_RETURN`; legacy `CO2_CAPTURED_C6`, `ETA_SELEXOL`, `PHI_RECYCLE`, `NG_CASE6`, `H2_A_MMSCFD` |
| N.7 unabated reference | carbon identity N.7.3; process gas from N.7.1; reject heating values; energy balance form of N.7.2 | furnace efficiency $\eta_F$; net steam-system duty; reference reformer specifications equal to the plant's | INL 52.5 MMSCFD, 3205 st/d, 189.2 MMBtu/h, 6.3 MWe; 44.5 MMSCFD, 2713 st/d; IEAGHG 8.996 kg/kg; fleet average 9.35 kg/kg | `NG_CONV_NOCC`, `CO2_CONV_NOCC`, `EXPORT_STEAM_CONV`, `E_CONV_NOCC`, `NG_NORECYCLE`, `CO2_HTGR_NOCC`, `BASELINE_IEAGHG_*`, `BASELINE_CHO`; new `REF_FURNACE_EFF`, `REF_STEAM_DUTY` |

# Parameter ids

Existing ids reused without change of meaning: `AVAILABILITY` (0.85, assumed), `H2_B` (100000 Nm3/h, published), `RHO_H2_NORMAL` (0.08988 kg/Nm3, published, not verified), `C_PER_MOL_NG_CH4`, `_C2`, `_C3`, `_C4`, `_C5`, `_C6`, `_CO2` (mole fractions, published), `ETA_MEA` (0.90, assumed, 0.80 to 0.95), `PSA_H2_RECOVERY` (0.88, published), `CH4_IN_H2` (0.001, published), `EF_NG_LHV`, `SYSTEM_BOILER_EFF`, `GRID_EF`, `EF_UPSTREAM`, `TS_BURDEN`, `EF_NUCLEAR` (legacy lifecycle and system factors, mostly not verified).

New ids. The column "Provenance" uses the four allowed labels. "Needs" says what must be supplied; no value is proposed where none follows from the equations.

| Id | Symbol | Unit | Provenance | Proposed value | Needs |
|---|---|---|---|---|---|
| `NG_Y_N2` | $y_{N_2}$ | mol/mol | published | 0.0119 (as hard-coded in `flowsheet.rs:175` to `185`) | move into `parameters.toml` with the TEV-953 A-5 and TEV-961 A-50 locators |
| `NG_Y_O2` | $y_{O_2}$ | mol/mol | published | 0.0001 (same place; the flowsheet adds it to N2) | move into `parameters.toml` with the same locators |
| `CRYO_C_RECOVERY` | $f$ | 1 | assumed | none; legacy comparison value 0.752178 is calculated from J.7 | owner decision on the central value (open issue 1) |
| `CRYO_H2_RETURN` | $r$ | 1 | assumed | none; the present 0.921630 is calculated by closure on INL's pair | owner decision (open issue 1) |
| `COND_CO2_VENT` | $v$ | 1 | assumed | 1 (unknown fate is not zero emission; matches the vented sensitivity of decision S14) | confirmation by the user |
| `MEA_CARBON` | $\varepsilon_{MEA}$ | kmol C/h | assumed | none | solvent make-up and degradation rate, with the split between treated gas, CO2 product and waste |
| `CO2_PROC_VENT` | $\varepsilon_{vent}$ | kmol C/h | assumed | none | vent specification of dehydration and compression between S19 and S20 |
| `CARBON_CLOSURE_TOL` | $\tau_C$ | 1 | assumed | $10^{-9}$ (justification in N.6.1, equal to `ELEMENT_TOL`) | none |
| `REF_FURNACE_EFF` | $\eta_F$ | 1 | assumed | none | furnace efficiency of the unabated reference |
| `REF_STEAM_DUTY` | in $Q_F$ | MW | assumed | none | net steam-system duty of the reference, including its steam export |
| `TS_LEAK` | $\lambda$ | 1 | assumed | none | transport and storage research |
| `TS_SHIP_CO2` | $e_{ship}$ | t/t | assumed | none | transport and storage research |
| `TS_ENERGY` | $w_{TS}$ | MWh/t | assumed | none | transport and storage research |
| `EF_UPSTREAM_CO2`, `EF_UPSTREAM_CH4` | $e_{up,CO_2}$, $e_{up,CH_4}$ | t/GJ | assumed | none | split of `EF_UPSTREAM` with its heating-value basis (HHV or LHV) stated |
| `GWP_HORIZON` | $H$ | y | user_decision | none | choice of horizon (open issue 6) |
| `GWP_CH4` | $GWP_H$ | t CO2e/t CH4 | published | none | value and source report for the chosen horizon |

## Implemented ids (Phase 3)

The registry holds fifteen new entries, all read by `CarbonParams::read` in `engine/plant/src/abatement.rs` and all listed with provenance and source in `engine/data/parameters.toml`. `CARBON_BASIS` (0 legacy, 1 independent) and `CARBON_BASELINE` (0 IEAGHG, 1 INL, 2 Cho) are project decisions D1, D12 and D2. `CRYO_C_RECOVERY` and `CRYO_H2_RETURN` hold f and r to twelve digits, each assumed and calibrated to INL's pair (decision D8). `COND_CO2_VENT` is v and `CARBON_CLOSURE_TOL` is $	au_C$. Nine `TS_*` entries replace the symbolic $\lambda$, $e_{ship}$ and $w_{TS}$ of the table above: ship fuel per tonne and kilometre, its energy content and CO2 factor, the shipping distance, conditioning and injection electricity, terminal and purge shares and the grid factor of the electricity. The distance is the one entry with no source, and it carries a band of 200 to 2000 km.

Not implemented, because no source value exists in the repository: `MEA_CARBON`, `CO2_PROC_VENT`, `REF_FURNACE_EFF`, `REF_STEAM_DUTY`, the split of `EF_UPSTREAM` and the two warming-potential entries. The N2 and O2 fractions stay in `flowsheet.rs`, where the lumped feed is defined.

# Register of streams that the flowsheet does not model

None of these is set to zero. Each carries a status, and the closure verdict of N.6.1 is "undetermined" while any status is unknown.

| Id | Stream | Effect on carbon | Status | Treatment |
|---|---|---|---|---|
| U1 | MEA make-up and degradation (2 C per MEA molecule) | inlet, and outlets to treated gas, CO2 product or waste | unknown | `MEA_CARBON`; the solvent carbon enters only the solvent element closure today |
| U2 | CO2 vent between S19 and S20 (water removal, dehydration, compression) | outlet to air | unknown (S20 equals S19 in the model) | `CO2_PROC_VENT` |
| U3 | impurities in S20 | none for carbon while S20 is pure CO2 | specified pure | note only |
| U4 | loss and leakage after S20 | outlet to air | unknown | $\lambda$ in N.9.4 |
| U5 | fuel for steam and grid electricity | outside the direct boundary | out of boundary | system view N.7.4; electricity emission K.9 is not implemented |
| U6 | start-up, shutdown, flaring and fugitive methane | outlet to air | unknown | no parameter yet |
| U7 | CO and CO2 carry-over in the PSA product | outlet with product | specified zero by the 99.9 / 0.1 mol percent product specification | `CH4_IN_H2` |
| U8 | dissolved CH4, CO and H2 in the condensate | outlet to water treatment | assumed zero by the flash model | stated assumption of `flash.rs` |
| U9 | fate of the condensate CO2 (S10) | modelled flow | modelled flow, fate set by $v$ | `COND_CO2_VENT` |

# Dimension table

Each row checks that the left side and every term of the right side carry the same dimensions. The script `math/verify/sympy_carbon.py` performs the same check with SymPy's dimension system. "h/y" is a pure number in SI dimensions.

| Eq. | Left side | Right side | Check |
|---|---|---|---|
| N.1.1 | $\bar c$: $\mathrm{kmol\ C\ kmol^{-1}}$ | $\sum y_ja_{C,j}$: $\mathrm{kmol\ kmol^{-1}}\times1$ | ok |
| N.1.2 | $Z$: $\mathrm{kmol\ H_2\ kmol^{-1}}$ | atom counts | ok |
| N.1.3 | $\mathrm{kmol\ h^{-1}}$ | $\dot Z_{in}$, $4\dot n_{CH_4}$, $\dot n_{CO}$: $\mathrm{kmol\ h^{-1}}$ | ok |
| N.1.4 | $\xi_{eff}$: 1 | $\xi/[1-r(1-\xi)]$: 1 | ok |
| N.1.5 | $\dot n_{NG}$: $\mathrm{kmol\ h^{-1}}$ | $\dot n_{H_2}\,[\mathrm{kmol\ h^{-1}}]\times$ ratio of $\mathrm{kmol\ gas}$ per $\mathrm{kmol\ H_2}$ (1) | ok |
| N.2.1 | $T$: $\mathrm{kmol\ C\ h^{-1}}$ | $(F_C-fP)/D$ | ok |
| N.2.2 | $G$: $\mathrm{kmol\ C\ h^{-1}}$ | $(1-\kappa-\eta\varphi)T-P$ | ok |
| N.3.1 | $C$: $\mathrm{kmol\ C\ h^{-1}}$ | $\eta\varphi T$ | ok |
| N.3.2 | $\eta_{pl}$: 1 | $C/F_C$ | ok |
| N.4.1 | $f$: 1 | $R/G$ | ok |
| N.4.2 | $\beta$: $\mathrm{kmol\ C\ kmol^{-1}\ N_2}$ | $B/\dot n_{N_2}$ | ok |
| N.4.4 | $B$: $\mathrm{kmol\ C\ h^{-1}}$ | $(1-f)(mF_C-P)/(1-mf)$ | ok |
| N.5.1 | $\dot n_{O_2}$: $\mathrm{kmol\ h^{-1}}$ | $Z_{15}/2$: $\mathrm{kmol\ H_2\ h^{-1}}$ (one H2-equivalent needs one half O2) | ok |
| N.5.2 | $E_{plant}$: $\mathrm{t\ y^{-1}}$ | $\mathrm{kmol\ h^{-1}}\times\mathrm{kg\ kmol^{-1}}\times\mathrm{h\ y^{-1}}/1000$ | ok |
| N.6.1 | $\rho$: 1 | $R_C/F_C$ | ok |
| N.7.2 | $\dot n_{fuel}$: $\mathrm{kmol\ h^{-1}}$ | $(\mathrm{MJ\ h^{-1}})/(\mathrm{MJ\ kmol^{-1}})$ | ok |
| N.7.3 | $E_{ref}$: $\mathrm{t\ y^{-1}}$ | as N.5.2 | ok |
| N.7.4 | $E_{lost}$: $\mathrm{t\ y^{-1}}$ | $\mathrm{GJ\ h^{-1}}\times\mathrm{h\ y^{-1}}\times\mathrm{t\ GJ^{-1}}$ | ok |
| N.8.2 | $\dot E$: $\mathrm{t\ y^{-1}}$ | $\mathrm{kmol\ h^{-1}}\times\mathrm{kg\ kmol^{-1}}\times\mathrm{h\ y^{-1}}/1000$ | ok |
| N.9.1 | $\mathrm{t\ y^{-1}}$ | $E_{ref}-E_{plant}$ | ok |
| N.9.4 | $\mathrm{t\ y^{-1}}$ | $A_{dir}-\tau_{TS}A_{cap}$ with $\tau_{TS}$ in $\mathrm{t\ t^{-1}}$ | ok |
| N.9.5 | $\mathrm{t\ y^{-1}}$ | $A_{TS}+\Delta G\,e_{up}$ ($\mathrm{GJ\ y^{-1}}\times\mathrm{t\ GJ^{-1}}$) $-E_{nuc}$ | ok |
| N.10.1 | $a_{H_2}$: $\mathrm{t\ t^{-1}}$ | $A/M_{H_2,y}$ | ok |
| N.10.2 | $\rho_{red}$: 1 | $A_{dir}/E_{ref}$ | ok |

# Tests that Phase 4 must write

Each test names the equation it checks and the tolerance on the quantity. Full-precision values come from the engine, not from the CSV print.

1. **Z table and conservation** (N.1.2): for every unit of the flowsheet, $\sum Z_iF_i$ in and out agree to `ELEMENT_TOL` relative; the table of $Z$ equals the formula for all ten species and O2.
2. **Feed carbon** (N.1.1): $\bar c=1.05379$ to $10^{-6}$ and $F_C$ equals the carbon of S01 to $10^{-12}$ relative.
3. **Hydrogen at the absorber** (N.1.3): the Z-balance H2 of S12 equals the flowsheet H2 to $10^{-9}$ relative.
4. **PSA loop** (N.1.4): $\xi_{eff}$ equals S13 H2 over S12 H2; limits $r=0$ gives $\xi$ and $r=1$ gives 1.
5. **Gas rate** (N.1.5): closed form equals the solved gas rate to $10^{-9}$; homogeneity: doubling the hydrogen basis doubles every stream and leaves every intensity unchanged to $10^{-12}$.
6. **Loop closed form** (N.2.1, N.6.2): $T_{closed}$ equals S04 carbon to $10^{-9}$ relative for three tear starts.
7. **Stream vector** (N.2.2): $G$, $R$, $B$, $K$ and the slip CO2 equal the flowsheet streams to $10^{-9}$.
8. **Capture identity** (N.3.1): S20 CO2 equals $\eta\times$ S11 CO2, S19 equals S20, and the solvent loading difference times circulation equals $C$.
9. **Plant capture** (N.3.2): $C/F_C$ equals the closed form; the value at $f=1$ stays below 1.
10. **Recycle read-back** (N.4.1): $R/G$ equals the input `CRYO_C_RECOVERY`; the validation row reproduces the J.7 value 0.752178 from INL inputs.
11. **Nitrogen balance and stack curve** (N.4.2, N.4.4): feed N2 equals S15 N2 to $10^{-12}$; $f=1-\beta\dot n_{N_2}/G$; $B(f)$ equals S15 carbon at the run's $f$ to $10^{-9}$, $B(0)=B_0$, $B(1)=0$, and $f(B(f))=f$.
12. **Combustion** (N.5.1): S16 CO2 equals the carbon of S15 and the air oxygen equals $Z_{15}/2$, both to $10^{-9}$.
13. **Closure verdicts** (N.6.1): "closed" for the central run; "open" when the product stream is dropped; "undetermined" when any stream is unknown; never "closed" with an unknown stream.
14. **Tolerance sensitivity** (N.6.1): runs with the tear tolerance relaxed up to $10^{-9}$ still close, and a relative loss of $10^{-6}$ in any single stream gives the verdict "open".
15. **Reference limits** (N.7.1): the reference configuration ($\eta=0$, $f=0$, $r=0$) has $\xi_{eff}=\xi$, closes carbon, and its process gas is at least $\dot n_{H_2}/(\xi Z_{NG})$.
16. **Reference identity** (N.7.3): the reference emission equals the carbon of all gas in less the product carbon; the INL pairs 52.5 MMSCFD and 44.5 MMSCFD return 3205 and 2713 st/d within 0.5 percent (validation rows, non-blocking).
17. **Firing balance** (N.7.2): $\dot n_{fuel}\ge0$; a furnace whose reject heat already exceeds $Q_F/\eta_F$ gets zero fuel; the implied $LHV_{NG}$ from `EF_NG_LHV` is within 1 percent of the heating value from formation enthalpies (cross-check).
18. **Decomposition** (N.9.2): $A_{dir}$ computed from N.9.1 equals the N.9.2 form to $10^{-9}$ when no unmodelled stream is present, and differs from it by exactly $k(\sum\varepsilon^{out,other}-\sum\varepsilon^{in})$ otherwise; $v=0$ against $v=1$ changes $E_{plant}$ by exactly $kK$.
19. **Single source of the gas difference** (rule 1 of N.7): the GAS ledger line divided by the gas price equals $\dot n_{ref}-\dot n_{NG}$ used in N.9.2, in MMSCFD, to $10^{-12}$.
20. **System terms once** (N.7.4): two plants with equal steam export give $E_{lost}=0$; $A_{sys}-A_{dir}=-E_{lost}+E_{pow}$ and no other term.
21. **Annual conversion** (N.8): 1 kmol/h of carbon gives 327.695 t/y at $f_{av}=0.85$; the value scales linearly with $f_{av}$; 1927 st/d gives 1655.083 kmol/h; 1 MMSCFD gives 49.803 kmol/h.
22. **Legacy equivalence**: with the legacy basis and zero subsidy the engine reproduces every cell of `results/` at 8750b09 (decision D1), including 359777.35, 38309.01, 616360.62 and 578051.61 t/y.
23. **Transport and storage and lifecycle** (N.9.4, N.9.5): zero inputs give $A_{TS}=A_{dir}$; $GWP_H=0$ returns the CO2 view; the CO2e view differs from the CO2 view by exactly $GWP_He_{up,CH_4}\Delta G$; the CO2e quantity never appears in a headline field.
24. **Intensities** (N.10): $a_{H_2}=e_{ref}-ci$; $\rho_{red}=1-E_{plant}/E_{ref}$; $e_{ref}\ge5.4579$ and the plant intensity is below the reference.
25. **Dimension tests**: each equation of the dimension table has a unit test on a typed quantity.
26. **Input guards**: $f\notin[0,1]$, $r\notin[0,1]$, $v\notin[0,1]$ and $\eta\notin(0,1]$ are rejected, never clamped.

# Open issues

1. **Central values of $f$ and $r$.** The plant's own atom balances determine everything downstream of the cryogenic split and nothing about the split itself. Both inputs are still calibrated to INL numbers, and the back-calculation of $f$ is ill conditioned (N.4.3). The owner has to choose a design basis for the cryogenic unit; N.2.1, N.2.2 and N.4.4 give the response in closed form (at the present carbon shares and the fixed hydrogen output, the stack carbon falls by 1205 $\mathrm{t\ y^{-1}}$ for each 0.01 of $f$; the $B_H(f)$ form of N.4.4 includes the change of the gas rate), so the choice can be scanned before it is fixed, or set through a target stack carbon or a target $\beta$.
2. **Reference furnace.** N.7.2 needs $\eta_F$ and the net steam-system duty, and Phase 3 needs a no-capture, no-recycle configuration of the flowsheet. Until then the reference gas is the gas that the central intensity implies (rule 1 of N.7) or the floor of N.7.1, and INL's gas is a comparison row. Status after fix round 2: the central result is the generic reference intensity of decision D2, which includes the firing gas of a real furnace; the floor leaves it out. The headline rows carry the words generic reference intensity, and the cost and the break-even shares under the floor are printed beside the headline. The owner has not yet accepted decision D2 in this form; if it is not accepted, the plant-built reference becomes the centre only after `REF_FURNACE_EFF` and `REF_STEAM_DUTY` have sources.
3. **Pairing of the cost level with the carbon view.** L5 contains LOST_STEAM and BASE_POWER; the direct abatement does not contain their carbon. Decide which pair is the headline (L5 with $A_{sys}$, or the ladder without those two lines with $A_{dir}$) and report the other as a second row. Status after fix round 2: the cost without the two lines is printed beside the headline (`NOSYS_*`, `SYSLINES_*`, `IND_L5_WITHOUT_SYSTEM_LINES`), and the two lines are labelled comparison inputs because they price INL's conventional plant; the pairing itself is still the owner's decision.
4. **Capture efficiency.** $\eta$ stays a specification while gate M6 (term T02) is failed.
5. **Unknown streams.** U1, U2 and U6 need sourced bounds; the fate $v$ of the condensate CO2 needs confirmation.
6. **Gas basis.** Choose the horizon $H$ and the source of $GWP_H$ (decision D4), split `EF_UPSTREAM` by gas and state its heating-value basis. Status: the aggregate now carries its horizon and source (100-year potential of 30, IEA 2026) and the lifecycle rows say CO2e; the split by gas and the heating-value basis wait for a source that states them.
7. **Transport and storage inputs** wait for the transport and storage research.
8. **Inputs outside `parameters.toml`.** The N2 and O2 fractions, S/C, reformer temperature and pressure live in `flowsheet.rs`. The ids `C_PER_MOL_NG_*` hold mole fractions despite their names.
9. **Blocking checks.** `drift.rs` pins the abatement at 578000 plus or minus 12000 t/y; it would block any independent rebuild and should become a non-blocking validation row. The tests that compare the flowsheet with the anchored ledger will fail by design with the independent basis.
10. **Sources not re-read.** The external source documents behind the legacy and comparison values are not in the repository, so no quoted external value was compared with its page; the values here are those recorded in `parameters.toml`.

# Verification

Script: `math/verify/sympy_carbon.py` (SymPy 1.14, Python 3.10 or later). Run from the repository root with `python -I math/verify/sympy_carbon.py`. The script proves the algebra symbolically (Z conservation, loop closed form and closure identity, PSA return loop, gas-rate formula, combustion balance, abatement decomposition, transport and storage and lifecycle structure, annual conversion and intensity identities, the dimension table), then substitutes values from `results/Integrated_Mass_Balance.csv`, `results/Emissions_Ledger.csv` and `engine/data/parameters.toml` to reproduce the legacy anchored basis and the Level 3 run. It does not recompute plant physics. Result at the time of writing: 115 of 115 checks pass, exit status 0 (106 written with the derivation and 9 added by the independent verification: the fixed-hydrogen form of N.4.4, the heat of the saved gas in N.9.5 and the scope of the decomposition in N.9.2).
