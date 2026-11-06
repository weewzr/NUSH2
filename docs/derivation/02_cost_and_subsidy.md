# Derivation 02: cost model and government co-funding

This file restates the existing cost model, which prices the plant in two ways (an annual ledger called L0 to L5 and a discounted cash flow called the DCF), and then derives the government co-funding model that the redevelopment adds. It is the reference for Phase 3. The annual net CO2 abatement is kept as a symbol, $\mathcal{A}$, that the carbon rebuild will supply later. Nothing below depends on how $\mathcal{A}$ is obtained.

Result labels used at the end of each equation: **analytical** (exact algebra), **numerical** (needs a loop or a root solve), **series** (a truncated expansion) and **approximation** (an estimate with a stated error).

Every worked number comes from `math/verify/sympy_subsidy.py`. That script reads `results/Economic_Ledger.csv` and `engine/data/parameters.toml` and does not call the Rust engine. Where a number needs an abatement, the legacy value $\mathcal{A}=578{,}051.6117\ \mathrm{t\ y^{-1}}$ is used as a placeholder that exercises the algebra. No worked number in this file is a result of the redevelopment.

---

# Reference Frame

## Variables

| Symbol | Definition | Units |
|---|---|---|
| $i$ | Real discount rate, one value for the ledger factors, interest during construction and the DCF (`DISCOUNT`, 0.08) | $1\ \mathrm{y^{-1}}$ (per year) |
| $v$ | One-year discount factor, $v=1/(1+i)$ | $1$ |
| $t_c$ | Construction time from financial close to year 0 (`CONSTRUCTION_YEARS`, 5.5) | $\mathrm{y}$ |
| $w,\ q$ | Whole years and part year of the construction time, $t_c=w+q$; here $w=5$, $q=0.5$ | $\mathrm{y}$ |
| $t_\ell$ | Project life, the operating years after year 0 (25 or 60 in the new matrix) | $\mathrm{y}$ |
| $t_u$ | Replacement interval of the 25 y items (`ITEM_LIFE_Y`) | $\mathrm{y}$ |
| $\mathrm{CRF}(t)$ | Capital recovery factor, $i/[1-(1+i)^{-t}]$ (register L.1) | $\mathrm{y^{-1}}$ |
| $\mathrm{SFF}(t)$ | Sinking fund factor, $i/[(1+i)^t-1]$ (register L.3) | $\mathrm{y^{-1}}$ |
| $\alpha(t)$ | Annuity factor, $\sum_{y=1}^{t}v^{y}=1/\mathrm{CRF}(t)$ | $\mathrm{y}$ |
| $f_{IDC}(t)$ | Ledger interest-during-construction factor for a build of length $t$ (register L.4) | $1$ |
| $f_D$ | DCF bucket factor for the build of length $t_c$ (S.0.1) | $1$ |
| $\Delta f$ | Difference $f_D-f_{IDC}(t_c)$ (S.0.2) | $1$ |
| $\rho_R$ | Present-value factor of an item that is bought again every $t_u$ (S.0.3) | $1$ |
| $C_R$ | Reactor overnight capital, `reactor_capital` of `CostItems` | $\mathrm{S\$M}$ |
| $K_{ihx},\ K_{lic},\ K_{pi}$ | Capital of the intermediate heat exchanger, the licence and the power island | $\mathrm{S\$M}$ |
| $K_{cap},\ K_{rec}$ | Capital of the capture plant and of the recycle train | $\mathrm{S\$M}$ |
| $D$ | Decommissioning total, paid or funded for the end of life (a liability, not a construction cost) | $\mathrm{S\$M}$ |
| $c_f$ | Fixed O&M share of capture and recycle capital (0.027) | $\mathrm{y^{-1}}$ |
| $Y$ | Sum of the annual differential lines: gas, capture fixed O&M, transport and storage, reactor O&M, lost steam, baseline power | $\mathrm{S\$M\ y^{-1}}$ |
| $\Delta C_5$ | Annual incremental cost at rung L5, the sum of all ledger lines | $\mathrm{S\$M\ y^{-1}}$ |
| $\Delta\mathrm{NPV}$ | Present value at year 0 of the NUS-H2 cash flows minus the unabated reformer cash flows | $\mathrm{S\$M}$ |
| $\mathcal{A}$ | Annual net CO2 abatement (placeholder 578,051.6117 until the carbon rebuild) | $\mathrm{t\ y^{-1}}$ |
| $\kappa$ | Conversion from a one-off year-0 amount to a levelised cost, $\kappa=\mathrm{CRF}(t_\ell)\,10^{6}/\mathcal{A}$ | $\mathrm{S\$\ t^{-1}}$ per $\mathrm{S\$M}$ |
| $T$ | Target levelised abatement cost, to be undercut strictly (`TARGET_COST`, 100) | $\mathrm{S\$\ t^{-1}}$ |
| $\mathrm{LAC}_5(s),\ \mathrm{LAC}_D(s)$ | Levelised abatement cost from the ledger (L5) and from the DCF, after a grant share $s$ | $\mathrm{S\$\ t^{-1}}$ |
| $s$ | Grant share of eligible expenditure, $0\le s\le 1$ | $1$ |
| $e_j$ | Eligibility flag of capital item $j$ (1 eligible, 0 not) | $1$ |
| $E$ | Eligible expenditure (S.1) | $\mathrm{S\$M}$ |
| $X_E$ | Eligible expenditure other than the reactor, $E-C_R$ | $\mathrm{S\$M}$ |
| $G$ | Grant, $G=sE$ | $\mathrm{S\$M}$ |
| $K_{tot}$ | Total overnight capital of the incremental project (S.8) | $\mathrm{S\$M}$ |
| $K^{priv}$ | Private overnight capital, $K_{tot}-G$ | $\mathrm{S\$M}$ |
| $b_k,\ t_k$ | Gross reactor spend in DCF bucket $k$ and its date | $\mathrm{S\$M}$, $\mathrm{y}$ |
| $S_k$ | Cumulative gross reactor spend through bucket $k$ | $\mathrm{S\$M}$ |
| $g_k,\ p_k$ | Grant applied and private payment in bucket $k$, with $g_k+p_k=b_k$ | $\mathrm{S\$M}$ |
| $B(t)$ | Private funded balance: private cash paid in plus the interest accrued on it | $\mathrm{S\$M}$ |
| $P_D(G)$ | Year-0 present value of the private reactor payments in the DCF after grant $G$ at close | $\mathrm{S\$M}$ |
| $u$ | Fraction of the reactor spend covered by the grant, $u=\min(G/C_R,1)$ | $1$ |
| $t_p$ | Private construction window at close, $t_p=t_c(1-u)$ | $\mathrm{y}$ |
| $h(u)$ | Ledger private present-value factor at close (S.3c) | $1$ |
| $\mathrm{IDC}^{(1)},\ \mathrm{IDC}^{(2)},\ \mathrm{IDC}^{(3)}$ | Interest during construction on the gross spend, on the private balance (default), and the part saved by the grant (S.5a) | $\mathrm{S\$M}$ |
| $M_5,\ M_D$ | Slope of the levelised cost in $s$ for a progressive grant (S.7a) | $\mathrm{S\$\ t^{-1}}$ |
| $s^{*}$ | Break-even share, the infimum of the shares with cost below $T$ | $1$ |

## Equations

Equations marked **(quoted)** are in `math/eq_register.csv` and are used as they stand. All others are derived in this file.

1. **L.1, L.3, L.4 (quoted).** $\mathrm{CRF}$, $\mathrm{SFF}$ and $f_{IDC}(t)=[(1+i)^{t}-1]/(it)$.
2. **S.0.1.** DCF bucket factor: $f_D=\big[q(1+i)^{w}+((1+i)^{w}-1)/i\big]/t_c$.
3. **S.0.2.** Gap between the two interest factors: $\Delta f=(1+i)^{w}\,[1+qi-(1+i)^{q}]/(t_c\,i)$.
4. **S.0.3.** Replacement factor $\rho_R(t_\ell)$ and the replaced-item gap $K[\rho_R\,\mathrm{CRF}(t_\ell)-\mathrm{CRF}(t_u)]$.
5. **S.0.4.** Decommissioning identity: $D\,v^{t_\ell}\,\mathrm{CRF}(t_\ell)=D\,\mathrm{SFF}(t_\ell)$.
6. **S.0.5.** The two costs per tonne: $\mathrm{LAC}_5=\Delta C_5\,10^{6}/\mathcal{A}$ and $\mathrm{LAC}_D=\Delta\mathrm{NPV}\,10^{6}/(\mathcal{A}\,\alpha(t_\ell))$, in closed form.
7. **S.0.6.** Total gap between the two methods at zero subsidy.
8. **S.1.** Eligible expenditure: $E=\sum_j e_jK_j$, definitions A and B.
9. **S.2.** Grant and private capital: $G=sE$, $K^{priv}=K_{tot}-G$.
10. **S.3a.** Grant at financial close, DCF: $g_k=\min(b_k,\max(0,G-S_{k-1}))$ and the funded balance.
11. **S.3b.** Closed form of $P_D(G)$.
12. **S.3c.** Grant at close, ledger form: $K_p f_{IDC}(t_p)=C_R\,h(u)$ with $h(u)=[(1+i)^{t_c(1-u)}-1]/(it_c)$.
13. **S.3d.** Optional upper bound in which idle grant cash earns the discount rate (not the default).
14. **S.4a.** Progressive grant, DCF: $p_k=(1-s)\,b_k$.
15. **S.4b.** Progressive grant, ledger form: $(1-s)\,C_R\,f_{IDC}(t_c)$.
16. **S.5a.** The interest bases $\mathrm{IDC}^{(1)}$, $\mathrm{IDC}^{(2)}$, $\mathrm{IDC}^{(3)}$ and the default.
17. **S.5b.** No double count: the net-of-grant identity and the size of the wrong ways.
18. **S.6a.** $\mathrm{LAC}_5(s)$ for each timing.
19. **S.6b.** $\mathrm{LAC}_D(s)$ for each timing.
20. **S.7a.** Exact conditions for affinity in $s$ and the slopes $M_5$, $M_D$.
21. **S.7b.** Break-even share for the progressive grant, $s^{*}=(\mathrm{LAC}(0)-T)/M$.
22. **S.7c.** Break-even share at close, ledger form, closed form.
23. **S.7d.** Break-even share at close, DCF: exact piecewise-linear root and the bisection fallback.
24. **S.7e.** Cases of $s^{*}$ and the target flag.
25. **S.8.** Reported quantities.
26. **S.9a.** Monotonicity and convexity.
27. **S.9b.** Limiting cases $s=0$ and $s=1$.
28. **S.9c.** Independent closed-form check of the grant effect.

## Key assumptions and parameter ranges

- All flows are real. One rate, $i=0.08$, serves the ledger factors, interest during construction, the DCF and the discounting of tonnes. The register's discount sweep (L.30) covers 5 to 10 percent. No tax, debt share, escalation or ramp-up exists in either method.
- Year 0 is the end of construction. Construction lasts $t_c=5.5\ \mathrm{y}$ for every stage. Reactor spend is uniform and falls at year ends, with the half-year bucket first (dates $-5,-4,\dots,0$). Operation runs from year 1 to $t_\ell$.
- New matrix: four stages (FOAK, BOAK, 10OAK_P46 and 10OAK_P47, the last two being the tenth-of-a-kind routes); lives 25 and 60 y; shares 0, 0.25, 0.5 and 0.75 plus the continuous solve on $[0,1]$.
- Stage multipliers on the reactor: FOAK 1.6 (range 1.3 to 2.1), learning factor $0.9^{\log_2 10}=0.7047$ (register L.7 to L.9).
- Items with a 25 y life (capture, IHX, recycle train, SMR island) are bought again in full every 25 y, with a straight-line credit for unused life at the horizon.
- Target: levelised cost strictly below $100\ \mathrm{S\$\ t^{-1}}$ in the ledger and in the DCF, tested separately (decision D3).
- A grant changes private financing only. Output, physical emissions, operating cost and construction time stay as they are.
- Grant cash held before it is applied earns nothing, and it is applied to eligible spend in date order (S.3a). This is the one modelling choice that most affects the at-close result.

## Existing equations relied on

| Id | What | Code | Use here |
|---|---|---|---|
| L.1, L.2, L.3 | CRF, annualised capital, SFF | `plant::anchored::crf`, `sff`; `Model::ledger` | quoted |
| L.4, L.21 | $f_{IDC}$ and the IDC line | `Model::idc_factor`, `Model::ledger` | quoted, then compared with the DCF (S.0.2) and reused at a shorter window (S.3c) |
| L.5 to L.10 | Reactor capital, stage, learning, size exponent | `Model::reactor_capex`, `Model::stage_mult` | $C_R$ enters as an input; not rederived |
| L.12, L.13, L.15, L.16, L.18 to L.20, L.22 to L.24 | Capture, fixed O&M, transport and storage, gas, IHX, licence, decommissioning, power island, recycle train, lost steam and baseline power | `Model::cost_items`, `Model::ledger` | inputs; not rederived |
| L.25, L.26 | Ladder L0 to L5 and cost per tonne | `Ledger::ladder`, `Ledger::l5` | quoted |
| L.28, L.34, L.36, L.37 | Levelised form, DCF, levelised cost, gap test | `plant::dcf::uniform_spend`, `with_replacement`, `Model::dcf` | quoted, then restated in closed form |

Not relied on: L.14 and L.17 (not implemented in the engine), L.27 and L.29 to L.33 (tables, sensitivities, levers, comparator, guards), L.35 (hydrogen cost).

---

# Part one: the existing model restated

## 1.1 Two ways to price the same plant

Think of buying a house with a mortgage. One way to state the cost is a yearly bill: the mortgage payment plus the running costs, each year the same. That is the **ledger** (Method 1). The other way is a dated list of every payment, each moved to one common day with the interest it would earn or cost on the way. That is the **DCF** (Method 2). Both use the same cost items (`CostItems`, built by `Model::cost_items`), the same rate and the same tonnes, so they differ only in how they treat time.

Moving a payment to a common day uses the **present value**. If a payment of one dollar falls in year $y$, its value on the opening day (year 0) is $(1+i)^{-y}$ dollars. For a payment made before opening, $y$ is negative, the factor is larger than one, and the extra is the interest that the money has cost by opening day.

## 1.2 Variables explained

- **$i$, discount rate.** The yearly price of money, 8 percent in real terms. It is an assumption (`DISCOUNT`, class assumed, same as the IEAGHG study). It sets every annuity factor and every discounted flow.
- **$t_c$, $w$, $q$, construction time.** 5.5 y from INL NZW09 (`CONSTRUCTION_YEARS`). The DCF has a calendar of whole years only, so it splits 5.5 into $w=5$ whole years and a half year $q=0.5$. The same $t_c$ applies to every stage.
- **$t_\ell$, project life.** The years of operation, 25 or 60. A longer life spreads the same capital over more tonnes, which lowers $\mathrm{CRF}$.
- **$t_u$, replacement interval.** 25 y for capture, IHX, recycle train and SMR island. On a 60 y life they are bought at year 0 and again at years 25 and 50.
- **$\mathrm{CRF}(t)$.** The yearly payment that repays one dollar over $t$ years with interest. $\mathrm{CRF}(25)=0.09367878$ and $\mathrm{CRF}(60)=0.08079795$.
- **$\alpha(t)$.** The sum of the yearly discount factors. A flat yearly amount $a$ for $t$ years is worth $a\,\alpha(t)$ at year 0. Also $\alpha=1/\mathrm{CRF}$, with $\alpha(25)=10.674776$ and $\alpha(60)=12.376552$.
- **$C_R$ and the other $K$ values.** Overnight capital means the price if everything could be built in one instant, so interest is not in it. BOAK at 25 y: $C_R=742.767\ \mathrm{S\$M}$, $K_{cap}=98.760$, $K_{ihx}=89.247$, $K_{lic}=65.325$, $K_{pi}=17.297$, $K_{rec}=48.830$. They are rebuilt from the CSV line values divided by the factor the engine used, so they carry the CSV rounding (about $10^{-6}$ relative).
- **$D$.** The decommissioning total, 109.746 $\mathrm{S\$M}$ for BOAK. It is a future liability and enters the ledger through a sinking fund.
- **$Y$.** The annual lines that have no capital in them. BOAK: $Y=-6.396279\ \mathrm{S\$M\ y^{-1}}$ (the large negative gas saving nearly cancels the positive lines).
- **$\mathcal{A}$.** Net tonnes of CO2 avoided per year. Both costs per tonne divide by it, so it is the only link to the carbon rebuild.

## 1.3 The ledger lines L0 to L5

The ledger (`Model::ledger`) annualises each capital item with its own factor and adds the yearly lines. `Ledger::ladder` sums the lines rung by rung (register L.25). BOAK, 25 y, values from `results/Economic_Ledger.csv` in $\mathrm{S\$M\ y^{-1}}$:

| Line | Rung | Formula in `Model::ledger` | BOAK 25 y |
|---|---|---|---|
| GAS | L0 | $-(\text{gas of the reformer}-\text{gas of the plant})\times$ price | $-64.796277$ |
| REACTOR_CAP | L0 | $C_R\,\mathrm{CRF}(t_\ell)$ | $69.581551$ |
| CAPTURE_CAP | L0 | $K_{cap}\,\mathrm{CRF}(t_u)$ | $9.251752$ |
| CAPTURE_FOM | L0 | $c_fK_{cap}$ | $2.666530$ |
| TS | L0 | tariff $\times$ captured tonnes | $21.586641$ |
| REACTOR_OM | L1 | reactor O&M and fuel | $25.323474$ |
| IHX | L2 | $K_{ihx}\,\mathrm{CRF}(\text{IHX life})$ | $8.360585$ |
| LICENCE | L3 | $K_{lic}\,\mathrm{CRF}(t_\ell)$ | $6.119566$ |
| DECOM | L3 | $D\,\mathrm{SFF}(t_\ell)$ | $1.501191$ |
| IDC | L4 | $C_R\,[f_{IDC}(t_c)-1]\,\mathrm{CRF}(t_\ell)$ | $13.753510$ |
| POWER_ISLAND | L4 | $K_{pi}\,\mathrm{CRF}(t_\ell)$ | $1.620350$ |
| RECYCLE_TRAIN | L4 | $K_{rec}\,\mathrm{CRF}(t_u)+c_fK_{rec}$ | $5.892778$ |
| LOST_STEAM | L5 | steam difference $\times$ gas price | $15.233868$ |
| BASE_POWER | L5 | $-$ power the reformer would buy | $-6.410515$ |
| EXPORT_REVENUE | L5 | $-$ export revenue (zero, so the CSV omits it) | $0$ |
| CAPTURE_ALLIN | L0 | off in the central case (zero, omitted) | $0$ |

The sum is $\Delta C_5=109.685004\ \mathrm{S\$M\ y^{-1}}$. Rung totals in $\mathrm{S\$\ t^{-1}}$ are 66.2, 110.0, 124.5, 137.7, 174.5 and 189.7 for L0 to L5.

## 1.4 Equations

### Equation S.0.1: DCF bucket factor (analytical)

**What it calculates.** The value at year 0 of the reactor spend per dollar of overnight capital, when the spend is placed in the year-end buckets of `uniform_spend`. It is the DCF's version of "capital plus interest during construction".

**Variables.** $C_R$ [$\mathrm{S\$M}$] overnight capital; $t_c=w+q$ [$\mathrm{y}$] construction time; $w$ [$\mathrm{y}$] whole years; $q$ [$\mathrm{y}$] part year; $i$ [$1\ \mathrm{y^{-1}}$] rate; $b_k$ [$\mathrm{S\$M}$] bucket amount; $t_k$ [$\mathrm{y}$] bucket date.

**Where it comes from.** `plant::dcf::uniform_spend` and the function `discount`, which returns $(1+i)^{-\text{year}}$.

**Derivation.**

1. `uniform_spend` puts $C_Rq/t_c$ at year $-w$ (the part year comes first) and $C_R/t_c$ at each of the years $-w+1,\dots,0$. The total is $C_R(q+w)/t_c=C_R$.
2. A bucket at year $-m$ is multiplied by $(1+i)^{m}$. The first bucket has $m=w$ and the others have $m=w-1,\dots,0$.
3. So $\mathrm{PV}=\dfrac{C_R}{t_c}\Big[q(1+i)^{w}+\sum_{m=0}^{w-1}(1+i)^{m}\Big]$.
4. The sum is a geometric series. If $S=\sum_{m=0}^{w-1}(1+i)^m$ then $(1+i)S-S=(1+i)^w-1$, so $S=\big((1+i)^{w}-1\big)/i$.
5. Substitute step 4 into step 3 and divide by $C_R$.

**Final implementable formula.**
$$f_D=\frac{1}{t_c}\left[\,q\,(1+i)^{w}+\frac{(1+i)^{w}-1}{i}\right],\qquad \mathrm{PV}_{reactor}=C_R\,f_D .$$

**Numbers.** $(1+i)^{5}=1.4693280768$, $\big((1+i)^5-1\big)/i=5.8666009600$, $q(1+i)^5=0.7346640384$. The bracket is $6.6012649984$ and $f_D=6.6012649984/5.5=1.2002299997$. For an integer $t_c$ ($q=0$) the formula reduces to $f_{IDC}$.

**Feeds into.** S.0.2 (compare with $f_{IDC}$), S.0.5 (reactor term of $\Delta\mathrm{NPV}$), S.3a and S.4a (the private share of this PV).

Rust implementation: `nush2_plant::subsidy::bucket_factor` (the closed form); the dated buckets are `nush2_plant::dcf::uniform_spend`, used by `Model::dcf_at`

Validation test: `eq_s_0_1_dcf_bucket_factor_equals_the_dated_buckets` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.0.2: the 0.0026 gap between the two interest factors (analytical, with a series check)

**What it calculates.** The difference between the DCF's factor $f_D$ and the ledger's factor $f_{IDC}(t_c)$, which is the whole zero-subsidy gap at 25 y.

**Variables.** As in S.0.1, plus $\Delta f$ [$1$] the difference and $g(q)=1+qi-(1+i)^{q}$ [$1$].

**Where it comes from.** Register L.4 gives $f_{IDC}(t)=[(1+i)^{t}-1]/(it)$. Its derivation in the register is exact for whole $t$ (equal year-end instalments) and is "applied at a non-integer $t_c$ as a smooth extension". The DCF cannot do that, so it uses S.0.1.

**Derivation.**

1. $f_{IDC}(t_c)=\big[(1+i)^{w+q}-1\big]/(it_c)$.
2. Subtract it from S.0.1 over the common denominator $it_c$: $\Delta f=\dfrac{1}{it_c}\Big[qi(1+i)^{w}+(1+i)^{w}-1-(1+i)^{w+q}+1\Big]$.
3. Factor $(1+i)^{w}$ and use $(1+i)^{w+q}=(1+i)^{w}(1+i)^{q}$: $\Delta f=\dfrac{(1+i)^{w}}{it_c}\big[1+qi-(1+i)^{q}\big]$.
4. Sign. $g(q)=1+qi-(1+i)^q$ has $g(0)=0$ and $g(1)=0$, and its second derivative $g''(q)=-\ln(1+i)^{2}(1+i)^{q}$ is negative. A curve that bends downward and is zero at both ends is positive between them, so $\Delta f\ge0$ and $f_D\ge f_{IDC}$.
5. Meaning. The straight line $1+qi$ is a chord of the curve $(1+i)^q$ and lies above it. The DCF treats the half-year bucket as earning interest in proportion to its length, while the ledger lets it compound.
6. Series. Expanding $(1+i)^{q}$ in powers of $i$ gives $g(q)=q(1-q)\,i^{2}/2+O(i^{3})$, so $\Delta f\approx(1+i)^{w}\,q(1-q)\,i/(2t_c)$.

**Final implementable formula (the check form, not needed by the engine).**
$$\Delta f=\frac{(1+i)^{w}\,\big[1+qi-(1+i)^{q}\big]}{i\,t_c}\ \ge 0 .$$

**Numbers.** The ledger factor is $f_{IDC}(5.5)=(1.08^{5.5}-1)/(0.08\times5.5)=0.5269705292/0.44=1.1976602936$. For the gap, $(1+i)^{q}=1.0392304845$ and $1+qi=1.04$, so $g=0.00076951546$. Then $\Delta f=1.4693280768\times0.00076951546/0.44=0.0025697061$, which agrees with $f_D-f_{IDC}=1.2002299997-1.1976602936$. The series gives $0.0026715$, which is 3.96 percent above the exact value (label: series, leading term). Relative to $f_D$ the gap is 0.21 percent.

**Effect on the cost per tonne.** $\Delta\mathrm{LAC}_{reactor}=C_R\,\mathrm{CRF}(t_\ell)\,\Delta f\;10^{6}/\mathcal{A}$. For BOAK at 25 y this is $742.767\times0.0936788\times0.0025697\times10^{6}/578{,}051.6=0.30932\ \mathrm{S\$\ t^{-1}}$, and the CSV stores 0.309322.

**Feeds into.** S.0.6 (total gap) and S.6b. Under a progressive grant this term scales by $(1-s)$ (S.9c).

Rust implementation: none beyond S.0.1: the engine holds the two factors, `subsidy::bucket_factor` and `Model::idc_factor`, and the gap is their difference; the closed form is the check

Validation test: `eq_s_0_2_the_two_interest_factors_differ_by_the_closed_form_gap` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.0.3: replacement factor and the replaced-item gap (analytical)

**What it calculates.** The present value, per dollar, of an item that is bought at year 0 and again every $t_u$ years, and the gap between its DCF cost and its ledger charge.

**Variables.** $K$ [$\mathrm{S\$M}$] capital of the item; $t_u$ [$\mathrm{y}$] interval; $t_\ell$ [$\mathrm{y}$] life; $n=\lceil t_\ell/t_u\rceil$ [$1$] number of units bought; $v$ [$1$] discount factor; $\rho_R$ [$1$] the factor.

**Where it comes from.** `plant::dcf::with_replacement`: units are bought at years $0,t_u,2t_u,\dots$ while the purchase year is below $t_\ell$, and the unused life of the last unit, $(nt_u-t_\ell)/t_u$ of its price, is credited at year $t_\ell$.

**Derivation.**

1. Purchases: $K\sum_{m=0}^{n-1}v^{mt_u}$.
2. Credit: $-K\,\dfrac{nt_u-t_\ell}{t_u}\,v^{t_\ell}$.
3. Add them: $\rho_R(t_\ell)=\sum_{m=0}^{n-1}v^{mt_u}-\dfrac{nt_u-t_\ell}{t_u}\,v^{t_\ell}$.
4. The DCF annualises with $1/\alpha(t_\ell)=\mathrm{CRF}(t_\ell)$ (S.0.5). The ledger charges $K\,\mathrm{CRF}(t_u)$. The gap per tonne is $K\,[\rho_R\mathrm{CRF}(t_\ell)-\mathrm{CRF}(t_u)]\,10^{6}/\mathcal{A}$.
5. If $t_\ell=t_u$ then $n=1$, the credit is zero, $\rho_R=1$ and the gap is zero.

**Final implementable formula.**
$$\rho_R(t_\ell)=\sum_{m=0}^{n-1}v^{mt_u}-\frac{nt_u-t_\ell}{t_u}\,v^{t_\ell},\qquad \text{gap}_K=K\,\big[\rho_R(t_\ell)\,\mathrm{CRF}(t_\ell)-\mathrm{CRF}(t_u)\big]\frac{10^{6}}{\mathcal{A}} .$$

**Numbers.** $t_\ell=60$, $t_u=25$: $n=3$ and the credit fraction is $15/25=0.6$. $v^{25}=0.1460179$, $v^{50}=0.0213212$, $v^{60}=0.0098759$, so $\rho_R(60)=1+0.1460179+0.0213212-0.0059255=1.1614136$. Then $\rho_R\mathrm{CRF}(60)-\mathrm{CRF}(25)=1.6106\times10^{-4}$. With $10^{6}/\mathcal{A}=1.72995$ the gap terms are: capture $0.027517$, IHX $0.024866$ and recycle train $0.013605$, which sum to $0.065989\ \mathrm{S\$\ t^{-1}}$.

**Feeds into.** S.0.5 (the DCF closed form) and S.0.6. Also S.1: only the first unit of an item can be granted.

Rust implementation: `nush2_plant::dcf::with_replacement` (existing, private), called by `Model::dcf_at` for the capture plant, the IHX and the recycle train

Validation test: `eq_s_0_3_replacement_factor_gives_the_replaced_item_gap` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.0.4: decommissioning identity (analytical)

**What it calculates.** Shows that the ledger's sinking-fund charge and the DCF's single payment at the horizon give the same annual cost.

**Variables.** $D$ [$\mathrm{S\$M}$] total; $t_\ell$ [$\mathrm{y}$]; $\mathrm{SFF}$, $\mathrm{CRF}$ [$\mathrm{y^{-1}}$].

**Where it comes from.** Ledger: $D\,\mathrm{SFF}(t_\ell)$ (`Model::ledger`). DCF: one flow $D$ at year $t_\ell$ (`Model::dcf`, line DECOM).

**Derivation.** The DCF value $Dv^{t_\ell}$ is annualised by $\mathrm{CRF}(t_\ell)$. Then $v^{t_\ell}\mathrm{CRF}(t_\ell)=\dfrac{i\,v^{t_\ell}}{1-v^{t_\ell}}=\dfrac{i}{(1+i)^{t_\ell}-1}=\mathrm{SFF}(t_\ell)$, by multiplying the numerator and denominator by $(1+i)^{t_\ell}$.

**Final formula.** $D\,v^{t_\ell}\,\mathrm{CRF}(t_\ell)=D\,\mathrm{SFF}(t_\ell)$. At 25 y: $109.746\times0.0136788=1.501191$.

**Feeds into.** S.0.6: the decommissioning gap term is zero. This holds at the central cost level; at the High level the ledger uses a different fund rate (audit finding), which is out of scope here.

Rust implementation: `Model::ledger` (line DECOM, sinking fund) and `Model::dcf_at` (line DECOM, one payment at the horizon)

Validation test: `eq_s_0_4_decommissioning_sinking_fund_equals_the_discounted_payment` in `engine/plant/tests/m10_subsidy.rs`; the existing `eq_l_03_decommissioning_at_end_of_life_equals_the_sinking_fund` in `engine/plant/src/dcf.rs`

### Equation S.0.5: the two costs per tonne in closed form (analytical)

**What it calculates.** The levelised abatement cost from the ledger (L5) and from the DCF, written with the same symbols so that they can be compared term by term and so that a grant can be inserted.

**Variables.** $\Delta C_5$ [$\mathrm{S\$M\ y^{-1}}$]; $\Delta\mathrm{NPV}$ [$\mathrm{S\$M}$]; $\mathcal{A}$ [$\mathrm{t\ y^{-1}}$]; $Y$ [$\mathrm{S\$M\ y^{-1}}$]; $c_f$ [$\mathrm{y^{-1}}$]; $\alpha(t_\ell)$ [$\mathrm{y}$]; the capitals $K$ [$\mathrm{S\$M}$].

**Where it comes from.** Register L.25 and L.26 (ledger) and L.28 and L.36 (DCF). Code: `Ledger::l5`, `Model::dcf`. In `Model::dcf` the unabated reformer's island (capital, fixed and variable O&M) appears on both plants with equal values, so it cancels in the difference. Only the lines that differ remain.

**Derivation.**

1. Ledger. Add the lines of section 1.3. Collect those with $\mathrm{CRF}(t_\ell)$ (reactor with its interest, licence, power island) and those with $\mathrm{CRF}(t_u)$ (capture, IHX, recycle train):
$$\Delta C_5=Y+c_fK_{rec}+\mathrm{CRF}(t_\ell)\big[C_Rf_{IDC}+K_{lic}+K_{pi}\big]+\mathrm{CRF}(t_u)\big[K_{cap}+K_{ihx}+K_{rec}\big]+D\,\mathrm{SFF}(t_\ell).$$
2. DCF. A flat yearly line $a$ from year 1 to $t_\ell$ is worth $a\,\alpha(t_\ell)$. The reactor is worth $C_Rf_D$ (S.0.1). The replaced items are worth $\rho_R$ times their price (S.0.3). Licence and power island are one payment at year 0. Decommissioning is $Dv^{t_\ell}$.
$$\Delta\mathrm{NPV}=\alpha(t_\ell)\,(Y+c_fK_{rec})+C_Rf_D+\rho_R(t_\ell)\big[K_{cap}+K_{ihx}+K_{rec}\big]+K_{lic}+K_{pi}+Dv^{t_\ell}.$$
3. Divide each by its tonnes. The DCF divides by $\mathcal{A}\,\alpha(t_\ell)$, and $1/\alpha=\mathrm{CRF}(t_\ell)$.

**Final implementable formulas.**
$$\mathrm{LAC}_5=\frac{10^{6}\,\Delta C_5}{\mathcal{A}},\qquad
\mathrm{LAC}_D=\frac{10^{6}}{\mathcal{A}}\Big\{Y+c_fK_{rec}+\mathrm{CRF}(t_\ell)\big[C_Rf_D+\rho_R(K_{cap}+K_{ihx}+K_{rec})+K_{lic}+K_{pi}+Dv^{t_\ell}\big]\Big\}.$$

**Numbers (BOAK, 25 y).** Pieces of $\Delta C_5$: $Y=-6.396279$, $c_fK_{rec}=1.318418$, reactor with interest $83.335061$, licence and power island $7.739916$, capture, IHX and recycle capital $22.186697$, decommissioning $1.501191$. Sum $109.685004\ \mathrm{S\$M\ y^{-1}}$, so $\mathrm{LAC}_5=189.7495\ \mathrm{S\$\ t^{-1}}$ (CSV 189.749501). The DCF gives $\Delta\mathrm{NPV}=1172.771561\ \mathrm{S\$M}$ and $\mathrm{LAC}_D=1172.771561\times10^{6}/(578{,}051.6117\times10.674776)=190.0588\ \mathrm{S\$\ t^{-1}}$ (CSV 190.058823).

**Feeds into.** S.0.6 and every later equation: a grant changes only the capital terms in the braces.

Rust implementation: `Model::ledger` with `Ledger::l5` for the ledger, and `Model::dcf_at` (fields `lac` and `l5`) for the discounted cash flow

Validation test: `eq_s_0_5_closed_forms_reproduce_both_costs_per_tonne` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.0.6: total gap between the methods at zero subsidy (analytical)

**What it calculates.** $\mathrm{LAC}_D-\mathrm{LAC}_5$ term by term, which predicts the stored `LAC_L5_GAP` and each `GAP_*` row.

**Variables.** As in S.0.2 and S.0.3.

**Where it comes from.** Subtract the two formulas of S.0.5. Annual lines, licence and power island enter both with the same weight and cancel. Decommissioning cancels by S.0.4.

**Derivation.** What remains is the reactor term (S.0.2) and the three replaced items (S.0.3).

**Final implementable formula.**
$$\mathrm{LAC}_D-\mathrm{LAC}_5=\frac{10^{6}}{\mathcal{A}}\Big\{\mathrm{CRF}(t_\ell)\,C_R\,\Delta f+\big[\rho_R(t_\ell)\mathrm{CRF}(t_\ell)-\mathrm{CRF}(t_u)\big]\,(K_{cap}+K_{ihx}+K_{rec})\Big\}.$$

**Numbers.** At 25 y the second term is zero and the gap equals the reactor term: BOAK $0.309322$, FOAK $0.494915$, 10OAK_P46 $0.217976$, 10OAK_P47 $0.348761$. At 60 y the replaced items add $0.065989$ for BOAK, giving $0.266790+0.065989=0.332779$ (CSV 0.332779). The largest gap above is 0.204 percent of L5 (FOAK, 60 y). The existing acceptance limit (L.37) is 10 percent of L5, which is about 49 times larger.

**Feeds into.** S.9c: the grant effect on the gap is predicted in closed form.

Rust implementation: `Model::dcf_at` (fields `gap_abs`, `gap_rel` and `gap_terms`)

Validation test: `eq_s_0_6_the_gap_between_the_methods_at_zero_subsidy_is_predicted` in `engine/plant/tests/m10_subsidy.rs`

## 1.5 Verification against the committed ledger

The script rebuilds both methods from the CSV (lines divided by the engine's factors give the capitals; flows are dated as in `Model::dcf`). Values in $\mathrm{S\$\ t^{-1}}$, B_REC_F, $n=1$, 210 MWth, with the legacy $\mathcal{A}$:

| Stage | $t_\ell$ | $C_R$ ($\mathrm{S\$M}$) | $\mathrm{LAC}_5$ rebuilt | $\mathrm{LAC}_5$ CSV | $\mathrm{LAC}_D$ rebuilt | $\mathrm{LAC}_D$ CSV | gap model | gap CSV |
|---|---|---|---|---|---|---|---|---|
| FOAK | 25 | 1188.428 | 277.93063 | 277.930634 | 278.42555 | 278.42555 | 0.494915 | 0.494915 |
| FOAK | 60 | 1188.428 | 241.69642 | 241.696417 | 242.18927 | 242.189271 | 0.492853 | 0.492853 |
| BOAK | 25 | 742.767 | 189.74950 | 189.749501 | 190.05882 | 190.058823 | 0.309322 | 0.309322 |
| BOAK | 60 | 742.767 | 165.64019 | 165.640188 | 165.97297 | 165.972967 | 0.332779 | 0.332779 |
| 10OAK_P46 | 25 | 523.419 | 146.34793 | 146.347930 | 146.56590 | 146.565906 | 0.217976 | 0.217976 |
| 10OAK_P46 | 60 | 523.419 | 128.20633 | 128.206332 | 128.46032 | 128.460325 | 0.253993 | 0.253993 |
| 10OAK_P47 | 25 | 837.471 | 208.48812 | 208.488121 | 208.83688 | 208.836882 | 0.348761 | 0.348761 |
| 10OAK_P47 | 60 | 837.471 | 181.80225 | 181.802248 | 182.16904 | 182.169043 | 0.366795 | 0.366795 |

All rebuilt values match the stored ones to within the 6-decimal rounding of the CSV lines (largest difference $3.8\times10^{-6}\ \mathrm{S\$\ t^{-1}}$). The checks also confirm that the IDC line equals $C_R(f_{IDC}-1)\mathrm{CRF}$, that all gap terms other than the reactor and the three replaced items are zero, that the stage ratios are 1.6 and 0.7047, and that BOAK capital from the registry (2184 $\mathrm{US\$_{2019}\ kWt^{-1}}$, 600 MWth reference, factor 1.6195, 210 MWth) is $742.76748\ \mathrm{S\$M}$.

---

# Part two: the government co-funding model

## 2.1 The picture

A builder is paid in six instalments (a half-size one in the first year and then five equal ones). The owner funds each instalment with borrowed money that costs 8 percent a year, so the loan grows until the plant opens. Now the government offers to pay a share $s$ of the bill. There are two ways to pay it.

- **Progressive.** The government pays a share $s$ of each instalment on the day it falls due. The owner borrows only $(1-s)$ of each instalment, so the loan is $(1-s)$ times as large all the way through.
- **At financial close.** The government pays the whole grant on the first day. The owner's account holds the cash and the builder's first bills are paid from it until it runs out. The owner starts borrowing later, which shortens the borrowing period and cuts the interest by more than a progressive grant would.

Two rules keep the model honest. The grant is a share of the **spend** (overnight cost), never of a figure that already contains interest. The interest saved is computed from how much the owner actually borrows and for how long, never by multiplying an interest figure by $s$.

## 2.2 Conventions decided here (decision D7)

1. **Date of financial close.** Financial close is the start of construction, $t_{fc}=-t_c=-5.5\ \mathrm{y}$ relative to year 0, in both methods. The DCF keeps its year-end buckets at $-5,-4,\dots,0$. With convention 3 below, the exact date of close has no numerical effect, so the DCF needs no flow at $-5.5$ and no change to its whole-year dates.
2. **Treatment of 5.5 y.** The ledger keeps its smooth factor $f_{IDC}$ and the DCF keeps its half-weight first bucket. Both use the same $t_c$ for every stage. Their known gap (S.0.2) is not removed. Under a grant the ledger uses $f_{IDC}$ at the shorter private window (S.3c) and the DCF uses its own buckets, so the same kind of smoothing gap remains and is bounded in S.9c.
3. **Idle grant cash.** Grant cash that has arrived but is not yet spent earns nothing, and it is applied to eligible spend in date order (the "grant first" rule). A project cannot earn the discount rate, which is a risky-project rate, on idle public cash. The grant is one pot of cash: at close, the part of $G$ that belongs to non-reactor items is applied to the earliest spend as well, and the earliest spend is reactor spend. With this rule the private interest of the dated balance (S.3a) can never become negative, and $s=1$ on definition A removes exactly the reactor capital and its interest (S.9b). The smooth ledger form (S.3c) shares this property except for a small smoothing artifact when the private window is shorter than one year (S.3c, step 7). The alternative in which idle cash earns $i$ is kept only as an upper bound (S.3d) because it fails both tests.
4. **Interest base.** Interest during construction is computed on the private balance outstanding (basis 2 of S.5a). It is reported on its own line and is never reduced by applying $s$ to a present-value figure.
5. **One-off grant, no replacement units.** The grant co-funds the first installation of an eligible item. Replacement units (the IHX bought again at years 25 and 50 on a 60 y life) are private, and so is the unused-life credit.
6. **Annualising a one-off grant.** The ledger annualises capital with the item's own factor, but a grant is paid once, so its annual equivalent is spread over the project life with $\mathrm{CRF}(t_\ell)$. In the DCF this is automatic because the whole present value is divided by $\alpha(t_\ell)$.
7. **What the grant leaves alone.** $\mathcal{A}$, the annual lines, the fixed O&M (which stays at $c_f$ times the gross capital), the construction time and the discount rate do not depend on $s$.
8. **Legacy lever.** `rx_mult` stays as the reactor-cost stress lever. If `rx_mult` differs from 1 while $s>0$, the grant applies to the stressed $C_R$. The old co-funding rows that use `rx_mult` as a pro-rata grant are left as they are (decision D5) and must not be combined with $s>0$.
9. **Zero subsidy.** With $s=0$ every new term is exactly zero: $G=0$, $u=0$, $p_k=b_k$, $h(0)=f_{IDC}(t_c)$ and no grant line exists. The engine must take the unchanged code path (skip the grant block when $G=0$) so that results are bit-identical. The script confirms that every route returns the zero-subsidy values (checks C4 and C5).
10. **Defaults.** Eligible set: definition A (reactor only). Timing: progressive, with the at-close result printed beside it in every table. Both are owner decisions to confirm. The verdict for a scenario names its timing.

## 2.3 Variables explained

- **$s$, share.** The fraction of eligible spend that the government pays. Grid 0, 0.25, 0.5 and 0.75, and a continuous solve on $[0,1]$.
- **$e_j$, flag.** One switch per capital item. Definition A switches on only the reactor. Definition B adds three items (S.1).
- **$E$, $X_E$.** The eligible spend, and the part of it that is not the reactor. For BOAK at 25 y: $E_A=742.767$, $X_E=0$ for A; $E_B=914.637$, $X_E=171.869$ for B ($\mathrm{S\$M}$).
- **$G$.** The grant, $sE$. At $s=0.5$ and definition A it is $371.384\ \mathrm{S\$M}$.
- **$b_k$, $t_k$, $S_k$.** The six DCF buckets for BOAK: $67.524$ at year $-5$ and $135.049$ at each of the years $-4$ to $0$ ($\mathrm{S\$M}$). $S_k$ is their running total.
- **$g_k$, $p_k$.** How each bucket is split between the grant and the owner.
- **$B(t)$, funded balance.** The owner's loan: cash paid in, grown at $i$ between instalments. At year 0 it equals the present value of the private payments, and the part above the cash paid in is the interest during construction.
- **$u$, $t_p$.** The share of the reactor spend that the grant covers, and the length of time the owner still has to pay. At $s=0.5$, A: $u=0.5$ and $t_p=2.75\ \mathrm{y}$.
- **$h(u)$.** A shorthand for the owner's present-value factor at close: $h(u)=[(1+i)^{t_c(1-u)}-1]/(it_c)$. It equals $f_{IDC}$ at $u=0$ and $0$ at $u=1$.
- **$\kappa$.** Converts a year-0 amount in $\mathrm{S\$M}$ into $\mathrm{S\$\ t^{-1}}$. BOAK at 25 y: $\kappa=0.16205954$, and $\kappa C_R=120.3726\ \mathrm{S\$\ t^{-1}}$.
- **$s^{*}$.** The share at which the cost equals $T$. It is an infimum: shares above it pass and $s^{*}$ itself does not (decision D3).

## 2.4 Equations

### Equation S.1: eligible expenditure (analytical)

**What it calculates.** The part of the capital spend that the grant is a share of.

**Variables.** $E$ [$\mathrm{S\$M}$]; $K_j$ [$\mathrm{S\$M}$] overnight capital of the first unit of item $j$; $e_j$ [$1$] flag; $X_E=E-C_R$ [$\mathrm{S\$M}$].

**Where it comes from.** The capital fields of `CostItems` built by `Model::cost_items`: `reactor_capital`, `ihx_capital`, `licence_capital`, `power_island_capital`, `capture_capital`, `recycle_capital`. Each is an amount spent once, before interest.

**Derivation.** Take every capital item of the incremental project and decide, item by item, whether it is a construction cost of the nuclear heat source on an overnight basis. Eligibility is a flag on the cost item, so that if an item is later removed from the ledger the eligible base follows it.

| Item | Eligible in A | Eligible in B | Reason |
|---|---|---|---|
| Reactor $C_R$ | yes | yes | The asset the programme co-funds. Its basis (register L.5, L.6) is overnight and has no interest in it. |
| IHX $K_{ihx}$ | no | yes | Sized by the reactor heat flow and useless without the reactor, so it is part of the nuclear heat source. First unit only. Its cost anchor is a scaled published figure that the registry marks as not verified. |
| Licence $K_{lic}$ | no | yes | A one-off regulatory cost of the nuclear plant paid before operation. Caution: the registry notes that INL's overnight cost already contains a licensing share, so the licence line may overlap it. The flag follows the line if the line is removed. |
| Power island $K_{pi}$ | no | yes | It is the add-back of the power-conversion share (9 percent) that the no-turbine recipe removes from the INL cost (`POWER_ISLAND_ADDBACK`), so it is nuclear-plant equipment on the same overnight basis. Caution: the Phase 1 audit found that it is sized on total electricity, which includes the tail-gas turbine, so part of it stands for non-nuclear equipment. |
| Capture $K_{cap}$ | no | no | Its unit cost is an IEAGHG Total Capital Requirement difference, a basis that includes interest during construction and owner costs (the code states this for the reformer island; the IEAGHG table was not opened here). A share of it would grant interest. It is also standard process equipment. Its fixed O&M stays on the gross capital. |
| Recycle train $K_{rec}$ | no | no | Process equipment on the capture side, not part of the nuclear heat source. Its basis is a Total Plant Cost, so adding it later is a one-flag change. |
| Reformer island | no | no | Identical in both plants, so it cancels from the incremental cost, and its Total Capital Requirement already contains its own interest. |
| Decommissioning $D$ | no | no | A liability at the end of life, not construction spend. |
| Interest during construction | never | never | A financing result, not spend (S.5a). |
| Operating lines | never | never | Not capital. |
| Replacement units | never | never | The grant is for the first installation (convention 5). |

**Final implementable formula.**
$$E=\sum_{j}e_j\,K_j,\qquad E_A=C_R,\qquad E_B=C_R+K_{ihx}+K_{lic}+K_{pi}.$$

**Numbers.** BOAK at 25 y: $E_A=742.767$, $E_B=742.767+89.247+65.325+17.297=914.637\ \mathrm{S\$M}$, which is 23.1 percent larger. Total overnight capital is $1062.227\ \mathrm{S\$M}$ (S.8). The ledger line for each non-reactor eligible item has no interest in it, because the engine buys these items at year 0 in both methods. A grant on them therefore has no interest term of its own under a progressive grant.

**Feeds into.** S.2 (substitute $E$ into $G=sE$).

Rust implementation: `Model::grant_plan` (fields `eligible` and `other_eligible`) with `subsidy::SubsidyParams::eligibility`; the flags are the `SUBSIDY_ELIG_` entries of `engine/data/parameters.toml`. Definition B in the registry follows decision D7b (reactor, IHX, capture plant, recycle train and licence). The table above and the worked numbers of S.6a, S.7d and S.8 use another membership (reactor, IHX, licence and power island), which the tests reproduce by setting those flags

Validation test: `eq_s_1_eligible_capital_is_the_flagged_sum_and_b_contains_a` and `eq_s_1_worked_numbers_of_the_derivation_follow_from_its_membership` in `engine/plant/tests/m10_subsidy.rs`; `subsidy::tests::eq_s_1_registry_membership_follows_decision_d7b` and `subsidy::tests::eq_s_1_a_flag_that_is_not_zero_or_one_is_refused` in `engine/plant/src/subsidy.rs`

### Equation S.2: grant and private capital (analytical)

**What it calculates.** The grant and the private overnight capital, together with the quantities that the grant must leave unchanged.

**Variables.** $G$ [$\mathrm{S\$M}$]; $s$ [$1$]; $E$ [$\mathrm{S\$M}$]; $K_{tot}$ [$\mathrm{S\$M}$] total overnight capital; $K^{priv}$ [$\mathrm{S\$M}$].

**Where it comes from.** Definition of a share grant, with the invariants of convention 7.

**Derivation.**

1. $G=sE$, so $0\le G\le E\le K_{tot}$ because $0\le s\le1$.
2. The grant reduces private capital one for one, so $K^{priv}=K_{tot}-G$.
3. How $G$ is spread over items and dates depends on the timing (S.3a and S.4a). The total does not.
4. Invariants: $\mathcal{A}(s)=\mathcal{A}(0)$; $Y(s)=Y(0)$; fixed O&M is $c_f\,K_{cap}$ and $c_f\,K_{rec}$ on the gross capital; $t_c$ and $i$ do not change.

**Final implementable formula.**
$$G=sE,\qquad K^{priv}=K_{tot}-G .$$

**Numbers.** BOAK 25 y, $s=0.5$, A: $G=371.384$ and $K^{priv}=690.844$; B: $G=457.318$ and $K^{priv}=604.909\ \mathrm{S\$M}$.

**Feeds into.** S.3a, S.4a (how $G$ is applied), S.8 (reporting).

Rust implementation: `Model::grant_plan` (fields `grant`, `total_capital` and `private_capital`); a share outside 0 to 1 is refused

Validation test: `eq_s_2_private_capital_never_rises_with_the_share_and_the_grant_stays_within_eligible` and `eq_s_2_the_subsidy_changes_no_physical_flow_and_no_operating_cost` in `engine/plant/tests/m10_subsidy.rs`; `subsidy::tests::eq_s_2_the_grid_defaults_and_limits_are_read_and_checked` in `engine/plant/src/subsidy.rs`

### Equation S.3a: grant at financial close, DCF buckets and funded balance (numerical)

**What it calculates.** How the grant paid at close is applied against the DCF buckets, the private payment in each bucket, and the owner's funded balance.

**Variables.** $b_k$, $S_k$, $g_k$, $p_k$ [$\mathrm{S\$M}$] as in section 2.3 with $S_{-1}=0$; $G$ [$\mathrm{S\$M}$]; $B_k$ [$\mathrm{S\$M}$] balance after bucket $k$; $t_k$ [$\mathrm{y}$]; $i$ [$1\ \mathrm{y^{-1}}$]. Buckets run over the reactor dates $-5,\dots,0$.

**Where it comes from.** The bucket list of `uniform_spend` (S.0.1) plus the rule of convention 3: at close the project account holds $G$, it pays the bills in date order and it earns nothing while it waits.

**Derivation.**

1. Before bucket $k$ the unspent grant is $\max(0,G-S_{k-1})$.
2. The grant pays as much of the bucket as it can: $g_k=\min\big(b_k,\max(0,G-S_{k-1})\big)$.
3. The owner pays the rest: $p_k=b_k-g_k=\min\big(b_k,\max(0,S_k-G)\big)$. (Check the regimes. If $G\le S_{k-1}$ then $g_k=0$ and $p_k=b_k$. If $G\ge S_k$ then $g_k=b_k$ and $p_k=0$. Otherwise $g_k=G-S_{k-1}$ and $p_k=S_k-G$.)
4. Balance. Between buckets the loan grows at $i$ and then the owner's payment is added: $B_k=B_{k-1}(1+i)^{\,t_k-t_{k-1}}+p_k$, with $B_{-1}=0$.
5. Unrolling step 4 gives $B(0)=\sum_kp_k(1+i)^{-t_k}\equiv P_D(G)$, the present value of the private payments.
6. If $G>C_R$ (only possible when $X_E>0$) the reactor is fully covered and the excess $G_X=G-C_R$ pays the eligible items bought at year 0, which carry no interest.
7. The DCF cash lines are unchanged except for one new line GRANT with the negative flows $-g_k$ at the bucket dates (and $-G_X$ at year 0). Gross lines are untouched, so the interest can be reported as present value minus nominal for each line.

**Final implementable formula.**
$$g_k=\min\!\big(b_k,\ \max(0,\,G-S_{k-1})\big),\quad p_k=b_k-g_k,\quad B_k=B_{k-1}(1+i)^{t_k-t_{k-1}}+p_k,\quad P_D(G)=B(0).$$

**Numbers.** BOAK 25 y, $s=0.5$, A, so $G=371.384$ (cumulative bills: $67.52$, $202.57$, $337.62$, $472.67$, $607.72$, $742.77$). The grant pays the buckets at years $-5$ to $-3$ in full and $33.76$ of the bucket at year $-2$. Funded balance in $\mathrm{S\$M}$ at the year ends:

| Year | Gross (no grant) | Progressive (S.4a) | At close (default) | Idle cash at $i$ (S.3d) |
|---|---|---|---|---|
| $-5$ | 67.524 | 33.762 | 0.000 | $-318.429$ |
| $-4$ | 207.975 | 103.987 | 0.000 | $-208.855$ |
| $-3$ | 359.662 | 179.831 | 0.000 | $-90.514$ |
| $-2$ | 523.483 | 261.742 | 101.286 | 37.293 |
| $-1$ | 700.410 | 350.205 | 244.438 | 175.325 |
| $0$ | 891.492 | 445.746 | 399.042 | 324.400 |

Interest during construction in the DCF (balance at year 0 minus cash paid in): gross $891.492-742.767=148.724$; progressive $445.746-371.384=74.362$; at close $399.042-371.384=27.658\ \mathrm{S\$M}$.

**Feeds into.** S.3b (a closed form for $P_D$), S.6b (substitute $P_D(G)$ for the reactor term $C_Rf_D$).

Rust implementation: `nush2_plant::subsidy::funded_balance`, called by `Model::grant_plan` for a grant at close

Validation test: `eq_s_3a_a_grant_at_close_pays_the_earliest_buckets_and_leaves_the_owner_the_rest` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.3b: closed form of $P_D(G)$ (analytical, piecewise)

**What it calculates.** The same $P_D(G)$ without a loop, as an independent check on S.3a.

**Variables.** $\gamma=Gt_c/C_R$ [$1$] the grant in units of one full bucket ($C_R/t_c$); $j$ [$1$] number of full buckets after the first that the grant covers; $r$ [$1$] covered fraction of the next bucket; $n_r$ [$\mathrm{y}$] number of years from the partly covered bucket to year 0; $\varrho$ [$1$] its uncovered fraction.

**Where it comes from.** S.3a. In units of $C_R/t_c$ the first bucket has size $q$ and each later one has size 1.

**Derivation.**

1. If $\gamma\ge t_c$ the reactor is fully covered and $P_D=0$.
2. If $\gamma\le q$ the grant covers part of the first bucket only: the first bucket keeps $\varrho=q-\gamma$ (date $-w$, so $n_r=w$) and every later bucket is whole.
3. If $q<\gamma<t_c$ then $j=\lfloor\gamma-q\rfloor$ whole buckets after the first are covered, $r=\gamma-q-j$, the next bucket keeps $\varrho=1-r$ at $n_r=w-1-j$ years before year 0, and the buckets after it are whole.
4. In every case the buckets from the partly covered one to the end are worth $\varrho(1+i)^{n_r}+\sum_{m=0}^{n_r-1}(1+i)^{m}$ units (geometric sum, as in S.0.1).

**Final implementable formula.**
$$P_D(G)=\frac{C_R}{t_c}\left[\varrho\,(1+i)^{n_r}+\frac{(1+i)^{n_r}-1}{i}\right],\quad (n_r,\varrho)=\begin{cases}(w,\ q-\gamma)&\gamma\le q\\(w-1-j,\ 1-r)&q<\gamma<t_c\end{cases},\quad P_D=0\ \text{if}\ \gamma\ge t_c.$$

**Numbers.** BOAK 25 y, $s=0.5$, A: $\gamma=0.5\times5.5=2.75$, so $\gamma-q=2.25$, $j=2$, $r=0.25$, $n_r=2$, $\varrho=0.75$. Then $P_D=135.04863\times[0.75\times1.1664+(1.1664-1)/0.08]=135.04863\times[0.8748+2.08]=399.0417$, which equals the loop of S.3a.

**Feeds into.** S.6b and S.7d.

Rust implementation: `nush2_plant::subsidy::private_pv_closed_form`, used by the break-even solve at close

Validation test: `eq_s_3b_the_closed_form_equals_the_balance_recursion` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.3c: grant at close, ledger form (analytical)

**What it calculates.** The ledger's counterpart of $P_D$: the owner's reactor capital plus interest after a grant at close, with the ledger's smooth factor.

**Variables.** $u=\min(G/C_R,1)$ [$1$]; $K_p=C_R(1-u)$ [$\mathrm{S\$M}$] private reactor capital; $t_p=t_c(1-u)$ [$\mathrm{y}$] private window; $f_{IDC}(t)$ [$1$]; $h(u)$ [$1$].

**Where it comes from.** Register L.4, applied to the private part. Under the grant-first rule the owner starts paying when the grant is used up, and then pays at the same rate $C_R/t_c$ as before until year 0. So the owner pays $K_p$ over a window of length $t_p$.

**Derivation.**

1. Spend rate is $C_R/t_c$, so a grant $G=uC_R$ pays the first $t_cu$ years of the build and the owner's window is $t_p=t_c(1-u)$.
2. The owner's capital is $K_p=C_R(1-u)$ over the window $t_p$, and L.4 applied to that window gives $K_pf_{IDC}(t_p)$.
3. Substitute $f_{IDC}(t_p)=\big[(1+i)^{t_p}-1\big]/(it_p)$ and $t_p=t_c(1-u)$. The factor $(1-u)$ in $K_p$ cancels the $(1-u)$ in $t_p$:
$$K_pf_{IDC}(t_p)=C_R(1-u)\frac{(1+i)^{t_c(1-u)}-1}{it_c(1-u)}=C_R\,\frac{(1+i)^{t_c(1-u)}-1}{it_c}.$$
4. Define $h(u)=\dfrac{(1+i)^{t_c(1-u)}-1}{it_c}$. Then $h(0)=f_{IDC}(t_c)$, $h(1)=0$, $h'(u)=-\dfrac{\ln(1+i)\,(1+i)^{t_c(1-u)}}{i}<0$ and $h''(u)>0$.
5. The interest on the private balance is $\mathrm{IDC}^{(2)}=C_Rh(u)-C_R(1-u)$.
6. For an integer $t_c$ and a grant that covers a whole number of buckets, the formula agrees with S.3a exactly. In every other case it is the same kind of smooth extension as L.4 itself, so it differs from S.3a by the same kind of small gap.
7. Sign of the interest. $f_{IDC}(t)<1$ for $0<t<1$ (the curve $(1+i)^t$ lies below its chord $1+it$ there), so $\mathrm{IDC}^{(2)}<0$ when the private window is shorter than one year, that is for $u>1-1/t_c$. Setting $\mathrm{d}\,\mathrm{IDC}^{(2)}/\mathrm{d}u=0$ gives $(1+i)^{t_c(1-u)}=i/\ln(1+i)$, so the most negative value sits at $u_{min}=1-\dfrac{\ln\!\big[i/\ln(1+i)\big]}{t_c\ln(1+i)}$. The dated balance of S.3a cannot do this, because it is cash paid in plus interest accrued on it. The slope ratio $\ln(1+i)/i$ of S.9a has the same origin.

**Final implementable formula.**
$$\text{private reactor value}=C_R\,h(u),\qquad h(u)=\frac{(1+i)^{t_c(1-u)}-1}{i\,t_c},\qquad \mathrm{IDC}^{(2)}=C_R\,[h(u)-(1-u)].$$

**Numbers.** BOAK 25 y, $s=0.5$, A: $h(0.5)=(1.08^{2.75}-1)/0.44=0.23570649/0.44=0.53569657$, so $C_Rh=397.898$. S.3b gives $399.042$ for the DCF, which is 0.29 percent higher (the usual smoothing gap). $\mathrm{IDC}^{(2)}=397.898-371.384=26.514\ \mathrm{S\$M}$. For step 7 (BOAK 25 y): $u_{min}=0.90851$ and $\mathrm{IDC}^{(2)}_{min}=-0.0017490\,C_R=-1.299\ \mathrm{S\$M}$, which lowers $\mathrm{LAC}_5$ by at most $0.2105\ \mathrm{S\$\ t^{-1}}$. It occurs only for $s>0.818$ on definition A and for $0.664<s<0.812$ on definition B, above every at-close break-even share of the table in S.7d. An implementation may keep the smooth form (consistent with the closed form of S.7c) or floor the ledger interest at zero. The floor would add a kink and change S.7c in that region, so the choice is left to Phase 3.

**Feeds into.** S.6a (substitute $C_Rh(u)$ for $C_Rf_{IDC}$ in the reactor term) and S.7c.

Rust implementation: `nush2_plant::subsidy::private_factor_at_close`, called by `Model::grant_plan` for a grant at close. The smooth form is kept: the ledger interest can dip below zero by at most 0.18 percent of the reactor capital when the private window is shorter than one year

Validation test: `eq_s_3c_the_ledger_form_at_close_and_its_limits` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.3d: upper bound with idle cash earning the discount rate (analytical, not the default)

**What it calculates.** The benefit of a grant at close if the project could invest idle public cash at the discount rate. It is the textbook "dated negative cash flow at the start of construction". It is kept to show why the default is different.

**Variables.** $G$ [$\mathrm{S\$M}$]; $(1+i)^{t_c}$ [$1$] growth from close to year 0; $s_0$ [$1$] share at which private interest turns negative.

**Where it comes from.** A flow of $-G$ at $t_{fc}=-t_c$ has present value $-G(1+i)^{t_c}$ at year 0. In the DCF this needs a flow at the date $-5.5$, which the whole-year dates cannot hold.

**Derivation.**

1. Private present value: $C_Rf-G(1+i)^{t_c}$.
2. Private interest: $\mathrm{IDC}=C_R(f-1)-G\big[(1+i)^{t_c}-1\big]$, which is zero at $s_0=C_R(f-1)/\{E[(1+i)^{t_c}-1]\}$.
3. Two defects follow. For $s>s_0$ the owner earns interest income on public money. And at $s=1$ (A) the private present value is $C_R[f-(1+i)^{t_c}]<0$, so the grant removes more than the reactor capital and its interest.

**Final formula.** $\mathrm{PV}_{priv}=C_Rf-G(1+i)^{t_c}$, with LAC affine in $s$.

**Numbers.** BOAK 25 y, A: $(1+i)^{t_c}=1.52697$, $s_0=0.37509$. At $s=0.5$ the private interest is $-48.893\ \mathrm{S\$M}$ (ledger form); at $s=1$ the private value is $-244.601\ \mathrm{S\$M}$. The private value crosses zero at $s=f/(1+i)^{t_c}=0.7843$. The break-even share (L5) is 0.48829 against 0.57142 at close and 0.62255 progressive, so the bound gives the lowest share in every row of the table of S.7d.

**Feeds into.** S.9b (the reason for convention 3).

Rust implementation: `Model::idle_cash_bound_share` (a bound that the model does not use); `SUBSIDY_GRANT_CASH_RATE` must be 0 and `subsidy::SubsidyParams::read` refuses any other value

Validation test: `eq_s_3d_idle_cash_earning_the_rate_gives_the_lowest_break_even_share` in `engine/plant/tests/m10_subsidy.rs`; `subsidy::tests::eq_s_3d_and_s_5a_only_the_implemented_conventions_are_accepted` in `engine/plant/src/subsidy.rs`

### Equation S.4a: progressive grant, DCF (analytical)

**What it calculates.** The private payments and funded balance when the grant pays a share of each bill as it falls due.

**Variables.** $s$, $e_j$ [$1$]; $b_k$, $p_k$, $g_k$ [$\mathrm{S\$M}$]; $K_j$ [$\mathrm{S\$M}$] for the items bought at year 0.

**Where it comes from.** "Progressive with the eligible spend": the grant flow of each item has the item's own dates.

**Derivation.**

1. Reactor: $g_k=sb_k$ and $p_k=(1-s)b_k$ on the bucket dates.
2. Items bought at year 0 (definition B): the grant is $sK_j$ at year 0, the owner pays $(1-s)K_j$.
3. The recursion of S.3a is linear, so $B(0)=(1-s)\sum_kb_k(1+i)^{-t_k}=(1-s)C_Rf_D$.
4. In the DCF the GRANT line holds $-sb_k$ at the bucket dates and $-sK_j$ at year 0, so its present value is $-s(C_Rf_D+X_E)$.

**Final implementable formula.**
$$p_k=(1-s)\,b_k,\qquad P_D^{prog}=(1-s)\,C_R\,f_D,\qquad \mathrm{PV}(\text{GRANT})=-s\,(C_R\,f_D+X_E).$$

**Numbers.** BOAK 25 y, $s=0.5$, A: $P_D^{prog}=0.5\times891.492=445.746$, as in the table of S.3a.

**Feeds into.** S.6b.

Rust implementation: `Model::grant_plan`, progressive branch (fields `dcf_flows` and `dcf_private_reactor_pv`); `Model::dcf_at` adds the GRANT line

Validation test: `eq_s_4a_a_progressive_grant_scales_every_private_payment` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.4b: progressive grant, ledger form (analytical)

**What it calculates.** The ledger's owner capital and interest for a progressive grant.

**Variables.** $C_R$, $K_{priv,R}$ [$\mathrm{S\$M}$]; $f_{IDC}(t_c)$ [$1$]; $s$ [$1$].

**Where it comes from.** Register L.4 and L.21 applied to the owner's share: the owner pays $(1-s)$ of every instalment, so L.4 applies to $(1-s)C_R$ over the full $t_c$.

**Derivation.** $K_{priv,R}=(1-s)C_R$. Capital plus interest: $(1-s)C_Rf_{IDC}(t_c)$. Interest alone: $\mathrm{IDC}^{(2)}=(1-s)\,C_R\,[f_{IDC}(t_c)-1]$.

**Final implementable formula.** $\text{private reactor value}=(1-s)\,C_R\,f_{IDC}(t_c)$.

**Numbers.** BOAK 25 y, $s=0.5$: $0.5\times742.767\times0.19766=73.408\ \mathrm{S\$M}$ of interest (the DCF figure is 74.362).

**Feeds into.** S.6a.

Rust implementation: `Model::grant_plan`, progressive branch (ledger fields); `Model::ledger` sets `Ledger::net`

Validation test: `eq_s_4b_a_progressive_grant_scales_the_ledger_reactor_value` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.5a: the interest bases (analytical)

**What it calculates.** Three ways to measure interest during construction when part of the spend is granted, and which one the model uses.

**Variables.** $\mathrm{IDC}^{(1)},\mathrm{IDC}^{(2)},\mathrm{IDC}^{(3)}$ [$\mathrm{S\$M}$]; $C_R$ [$\mathrm{S\$M}$]; $f_{IDC}(t_c)$ [$1$]; $h(u)$ [$1$]; $u$, $s$ [$1$].

**Where it comes from.** Section 2.1: interest is the cost of borrowed money, so it follows the balance that the owner borrows.

**Derivation.**

| Basis | Definition | Progressive | At close (S.3c) |
|---|---|---|---|
| 1. Gross spend | Interest as if nothing were granted | $C_R(f-1)$ | $C_R(f-1)$ |
| 2. Private balance (default) | Interest on the money the owner actually borrows | $(1-s)C_R(f-1)$ | $C_R\,[h(u)-(1-u)]$ |
| 3. Grant share | Interest the grant saves, $\mathrm{IDC}^{(1)}-\mathrm{IDC}^{(2)}$ | $s\,C_R(f-1)$ | $C_R\,[f-h(u)-u]$ |

Here $f=f_{IDC}(t_c)$. Basis 1 is exact only for a grant paid at completion (in arrears), because then the owner borrows the whole bill. Basis 3 is a report of what the grant saved and is never an input. Basis 2 is the default for four reasons. It is the cost of the money the owner actually borrowed. In the dated balance it cannot be negative (the smooth ledger form dips below zero by at most 0.18 percent of $C_R$, S.3c step 7). It depends on when the grant arrives, so the two timings differ. It involves no multiplication of a present-value figure by $s$.

**Final implementable formula.**
$$\mathrm{IDC}^{(2)}=\text{private balance at year 0}-\text{private cash paid in},\qquad \mathrm{IDC}^{(3)}=\mathrm{IDC}^{(1)}-\mathrm{IDC}^{(2)}.$$

**Numbers.** BOAK 25 y, $s=0.5$, A, ledger form in $\mathrm{S\$M}$: basis 1 is $146.816$. Progressive: basis 2 is $73.408$ and basis 3 is $73.408$. At close: basis 2 is $26.514$ and basis 3 is $120.301$.

**Feeds into.** S.5b and S.6.

Rust implementation: `Model::grant_plan` (fields `ledger_interest_gross`, `ledger_interest_private`, `ledger_interest_saved` and the `dcf_interest_` fields)

Validation test: `eq_s_5a_interest_is_charged_on_the_private_balance_and_the_saving_is_reported` in `engine/plant/tests/m10_subsidy.rs`; `subsidy::tests::eq_s_3d_and_s_5a_only_the_implemented_conventions_are_accepted` in `engine/plant/src/subsidy.rs`

### Equation S.5b: no double count and the size of the wrong ways (analytical)

**What it calculates.** The net-of-grant cost as gross cost minus two separately reported credits, and how large the errors would be that the design avoids.

**Variables.** $\Delta C_5(s)$, $\Delta C_5(0)$ [$\mathrm{S\$M\ y^{-1}}$]; $G$, $\mathrm{IDC}^{(3)}$ [$\mathrm{S\$M}$]; $\mathrm{CRF}(t_\ell)$ [$\mathrm{y^{-1}}$].

**Where it comes from.** Each dollar of the private reactor value is either spend or interest. The grant removes spend (credit $G$). The reduced borrowing removes interest (credit $\mathrm{IDC}^{(3)}$). Nothing else changes.

**Derivation.**

1. Gross private value $=C_R+\mathrm{IDC}^{(1)}$.
2. Net private value $=(C_R-G)+\mathrm{IDC}^{(2)}$.
3. Difference $=G+(\mathrm{IDC}^{(1)}-\mathrm{IDC}^{(2)})=G+\mathrm{IDC}^{(3)}$.
4. Annualise a one-off year-0 amount with $\mathrm{CRF}(t_\ell)$ (convention 6).
5. Wrong ways. (W1) A grant base that includes interest, $G'=sC_Rf$, over-grants by $sC_R(f-1)$. (W2) Taking the credit $sC_Rf$ on the present value and also cutting the interest by $s$ counts the same $sC_R(f-1)$ twice. (W3) Using the progressive saving $sf$ for a grant that arrives at close under-states the saving by $C_R[(f-h(u))-sf]$. (W4) Taking the share of the year-0 present value, $sC_Rf$, and then dating that amount $t_f$ years before year 0 (for a grant at financial close $t_f=t_c$), so that it is compounded once more, counts the interest twice: once inside $f$ and once in the factor $(1+i)^{t_f}$. The credit becomes $sC_Rf\,(1+i)^{t_f}$, which is $(1+i)^{t_f}$ times the correct progressive credit $sC_Rf$ and exceeds it by $sC_Rf\,[(1+i)^{t_f}-1]$.

**Final implementable formula.**
$$\Delta C_5(s)=\Delta C_5(0)-\mathrm{CRF}(t_\ell)\,\big[G+\mathrm{IDC}^{(3)}\big].$$

**Numbers.** BOAK 25 y, $s=0.5$, A, with $\kappa C_R=120.3726$: W1 and W2 each lower $\mathrm{LAC}_5$ by $\kappa\,s\,C_R(f-1)=11.896\ \mathrm{S\$\ t^{-1}}$ (6.3 percent of $\mathrm{LAC}_5(0)$). W3 raises it by $\kappa C_R[(f-h)-sf]=7.600\ \mathrm{S\$\ t^{-1}}$. W4 with $t_f=t_c$ lowers it by $\kappa\,s\,C_Rf\,[(1+i)^{t_c}-1]=37.985\ \mathrm{S\$\ t^{-1}}$ (20.0 percent of $\mathrm{LAC}_5(0)$), because the credit is $1.52697$ times the correct one. None of these errors would be caught by the existing 10 percent gap test, because both methods would share it.

**Feeds into.** S.6a, S.8.

Rust implementation: `nush2_plant::economics::NetBlock` and `Ledger::l5_net`, set in `Model::ledger`

Validation test: `eq_s_5b_the_net_cost_is_the_gross_cost_less_the_grant_and_the_interest_it_saves` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.6a: ledger cost as a function of the share (analytical)

**What it calculates.** $\mathrm{LAC}_5(s)$ for each timing and each eligible set.

**Variables.** $\kappa=\mathrm{CRF}(t_\ell)10^{6}/\mathcal{A}$ [$\mathrm{S\$\ t^{-1}}$ per $\mathrm{S\$M}$]; $C_R,X_E,G$ [$\mathrm{S\$M}$]; $f=f_{IDC}(t_c)$; $h(u)$; $u=\min(G/C_R,1)$.

**Where it comes from.** S.0.5 with the capital terms replaced by their private values (S.4b and S.3c). The other ledger lines are untouched. The credit for eligible non-reactor items is annualised with $\mathrm{CRF}(t_\ell)$, including the IHX (convention 6).

**Derivation.**

1. S.0.5 gives $\Delta C_5$ as a sum of fixed parts and the capital charges $\mathrm{CRF}(t_\ell)[C_Rf+\dots]$.
2. Progressive: the reactor value $C_Rf$ becomes $(1-s)C_Rf$ and each eligible $K_j$ becomes $(1-s)K_j$. So $\Delta C_5$ falls by $\mathrm{CRF}(t_\ell)\,s\,(C_Rf+X_E)$.
3. At close: the reactor value $C_Rf$ becomes $C_Rh(u)$ and any grant beyond $C_R$ pays year-0 items one for one. So $\Delta C_5$ falls by $\mathrm{CRF}(t_\ell)\,[C_R(f-h(u))+\max(0,G-C_R)]$.
4. Multiply by $10^{6}/\mathcal{A}$.

**Final implementable formula.**
$$\mathrm{LAC}_5(s)=\mathrm{LAC}_5(0)-\kappa\times\begin{cases}s\,(C_R f+X_E)&\text{progressive}\\ C_R\,[f-h(u)]+\max(0,\,G-C_R)&\text{at close}\end{cases}$$

Net-of-grant block for the ledger (L0 to L5 stay gross): $\text{GRANT\_CREDIT}=-G\,\mathrm{CRF}(t_\ell)$ and $\text{IDC\_EFFECT}=-\mathrm{IDC}^{(3)}\mathrm{CRF}(t_\ell)$, both in $\mathrm{S\$M\ y^{-1}}$.

**Numbers.** BOAK 25 y, $\mathrm{LAC}_5(0)=189.7495$, in $\mathrm{S\$\ t^{-1}}$:

| $s$ | A progressive | A at close | B progressive | B at close |
|---|---|---|---|---|
| 0.25 | 153.7081 | 147.8023 | 146.7449 | 138.7124 |
| 0.50 | 117.6668 | 110.0672 | 103.7403 | 93.9107 |
| 0.75 | 81.6254 | 76.1213 | 60.7356 | 54.5826 |

**Feeds into.** S.7 (break-even) and S.8.

Rust implementation: `Model::subsidy_eval_at` (field `l5` of `SubsidyEval`), which takes the net ledger from `Model::ledger`

Validation test: `eq_s_6a_ledger_cost_follows_the_closed_form_for_each_timing` and `eq_s_6a_derivation_tables_for_definition_b_follow_from_its_membership` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.6b: DCF cost as a function of the share (analytical when progressive, numerical at close)

**What it calculates.** $\mathrm{LAC}_D(s)$ for each timing and each eligible set.

**Variables.** $\kappa$ [$\mathrm{S\$\ t^{-1}}$ per $\mathrm{S\$M}$]; $C_R,X_E,G$ [$\mathrm{S\$M}$]; $f_D$ [$1$] the DCF bucket factor of S.0.1; $P_D(G)$ [$\mathrm{S\$M}$] from S.3a or S.3b; $s$ [$1$]; $\mathrm{LAC}_D(0)$ [$\mathrm{S\$\ t^{-1}}$].

**Where it comes from.** S.0.5. The DCF is the sum of dated flows, so a grant adds the GRANT line of S.3a or S.4a. Dividing by $\mathcal{A}\,\alpha(t_\ell)$ is the same as multiplying by $\mathrm{CRF}(t_\ell)/\mathcal{A}$, so the same $\kappa$ applies.

**Derivation.** $\Delta\mathrm{NPV}(s)=\Delta\mathrm{NPV}(0)+\mathrm{PV}(\text{GRANT})$. Progressive: $\mathrm{PV}=-s(C_Rf_D+X_E)$ (S.4a). At close: $\mathrm{PV}=-[C_Rf_D-P_D(\min(G,C_R))+\max(0,G-C_R)]$.

**Final implementable formula.**
$$\mathrm{LAC}_D(s)=\mathrm{LAC}_D(0)-\kappa\times\begin{cases}s\,(C_R f_D+X_E)&\text{progressive}\\ C_R f_D-P_D(\min(G,C_R))+\max(0,\,G-C_R)&\text{at close}\end{cases}$$

**Numbers.** BOAK 25 y, $\mathrm{LAC}_D(0)=190.0588$, in $\mathrm{S\$\ t^{-1}}$:

| $s$ | A progressive | A at close | B progressive | B at close |
|---|---|---|---|---|
| 0.25 | 153.9401 | 147.9264 | 146.9769 | 138.8790 |
| 0.50 | 117.8214 | 110.2526 | 103.8949 | 94.0087 |
| 0.75 | 81.7028 | 76.3338 | 60.8130 | 54.7874 |

**Feeds into.** S.7, S.8.

Rust implementation: `Model::subsidy_eval_at` (field `dcf` of `SubsidyEval`), which takes the net cash flow from `Model::dcf_at`

Validation test: `eq_s_6b_dcf_cost_follows_the_closed_form_for_each_timing` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.7a: when the cost is affine in the share (analytical)

**What it calculates.** The exact conditions under which $\mathrm{LAC}(s)$ is a straight line in $s$, and its slope.

**Variables.** $M_5=\kappa(C_Rf_{IDC}+X_E)$ and $M_D=\kappa(C_Rf_D+X_E)$ [$\mathrm{S\$\ t^{-1}}$ per unit share].

**Where it comes from.** S.6a and S.6b.

**Derivation.** $\mathrm{LAC}$ is a fixed part plus $\kappa$ times a present value of private flows. If every private flow is a fixed number times $(1-s)$ (or does not depend on $s$ at all), the present value is a straight line in $s$. This holds exactly under the following conditions.

1. The eligible set, the flags $e_j$ and the item capitals $K_j$ do not depend on $s$.
2. The share $s$ is paid on every eligible item at the dates of the spend that it funds, so the private flow of item $j$ at each date is $(1-se_j)$ times the gross flow (progressive timing).
3. $\mathcal{A}$, $Y$, $i$, $t_c$ and $t_\ell$ do not depend on $s$.
4. Interest is computed on the private balance at the single rate $i$, with no floor, cap or tax effect.
5. Replacement units and unused-life credits are outside the grant.
6. There is no budget cap on $G$ and no minimum private share.

If conditions 1 to 6 hold, then $\mathrm{LAC}_5$ and $\mathrm{LAC}_D$ are exactly affine in $s$ with the slopes above. Affinity fails when condition 2 fails. The grant-first rule at close makes the private flow $\min(b_k,\max(0,S_k-G))$, which has kinks in the DCF, and gives the exponential $h(u)$ in the ledger. Affinity also fails if a cap on $G$ or a floor on $\mathrm{IDC}$ binds, or if $E$ depends on $s$. The idle-cash bound of S.3d is affine, but it breaks the physical limits, so the model leaves it out.

**Final implementable formula.**
$$\mathrm{LAC}_k(s)=\mathrm{LAC}_k(0)-s\,M_k,\qquad M_5=\kappa\,(C_Rf_{IDC}+X_E),\quad M_D=\kappa\,(C_Rf_D+X_E).$$

**Numbers.** BOAK 25 y: $M_5=144.16543$ and $M_D=144.47475\ \mathrm{S\$\ t^{-1}}$ for A. For B add $\kappa X_E=27.853051$ to each. The scan in the script shows second differences of zero (to $10^{-25}$) for the progressive case. For the at-close grant the second difference with step 0.25 at $s=0.25,0.5,0.75$ is $3.79\ \mathrm{S\$\ t^{-1}}$, so the line bends.

**Feeds into.** S.7b.

Rust implementation: the slopes are the denominators of the private closed forms `Curve::ledger` and `Curve::dcf` in `engine/plant/src/subsidy.rs`

Validation test: `eq_s_7a_progressive_cost_is_affine_and_the_cost_at_close_bends` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.7b: break-even share for a progressive grant (analytical)

**What it calculates.** The infimum share $s^{*}$ at which the cost equals the target.

**Variables.** $\mathrm{LAC}_k(0)$ [$\mathrm{S\$\ t^{-1}}$]; $T$ [$\mathrm{S\$\ t^{-1}}$]; $M_k$ [$\mathrm{S\$\ t^{-1}}$]; $s^{*}_k$ [$1$]; $\Delta C_5(0)$ [$\mathrm{S\$M\ y^{-1}}$].

**Where it comes from.** Solve $\mathrm{LAC}_k(s)=T$ using S.7a.

**Derivation.**

1. $\mathrm{LAC}_k(0)-sM_k=T$.
2. $s^{*}_k=(\mathrm{LAC}_k(0)-T)/M_k$.
3. Because $M_k>0$ the cost is below $T$ exactly when $s>s^{*}_k$ (decision D3). The value $s^{*}_k$ itself gives cost equal to $T$ and fails the strict test.
4. Using $\mathrm{LAC}_5(0)=10^{6}\Delta C_5(0)/\mathcal{A}$ and $M_5=10^{6}\mathrm{CRF}(t_\ell)(C_Rf+X_E)/\mathcal{A}$, the abatement cancels in the numerator and denominator except through the allowed annual cost $T\mathcal{A}\times10^{-6}$:
$$s^{*}_5=\frac{\Delta C_5(0)-T\,\mathcal{A}\times10^{-6}}{\mathrm{CRF}(t_\ell)\,(C_Rf_{IDC}+X_E)} .$$
5. The overall verdict needs both methods, so the break-even for the scenario is $\max(s^{*}_5,s^{*}_D)$.

**Final implementable formula.** $s^{*}_k=(\mathrm{LAC}_k(0)-T)/M_k$, or the allowance form above for $k=5$ (and the same with $\Delta\mathrm{NPV}(0)-T\mathcal{A}\alpha(t_\ell)\times10^{-6}$ over $C_Rf_D+X_E$ for $k=D$).

**Numbers (progressive, A, legacy $\mathcal{A}$).** BOAK 25 y: $(189.7495-100)/144.16543=0.62255$ (L5) and $(190.0588-100)/144.47475=0.62335$ (DCF). Allowance form: $(109.685005-57.805161)/83.33506=0.62255$. The Phase 1 audit estimated 0.62; the values for the other points are in the table of S.7d.

**Feeds into.** S.7e.

Rust implementation: `Curve::ledger` and `Curve::dcf` (progressive branch), called by `Model::breakeven_at`

Validation test: `eq_s_7b_break_even_for_a_progressive_grant` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.7c: break-even share at close, ledger form (analytical)

**What it calculates.** The exact $s^{*}$ for the at-close grant in the ledger, where $\mathrm{LAC}_5(s)$ is not a straight line.

**Variables.** $D_0=(\mathrm{LAC}_5(0)-T)/\kappa$ [$\mathrm{S\$M}$] the reduction in year-0 value that the grant must deliver; $H^{*}$ [$1$]; $u^{*}$ [$1$]; $G^{*}$ [$\mathrm{S\$M}$]; $E$ [$\mathrm{S\$M}$].

**Where it comes from.** Set $\mathrm{LAC}_5(s)=T$ in S.6a and solve in two regimes.

**Derivation.**

1. Reactor regime ($G\le C_R$): $C_R[f-h(u^{*})]=D_0$, so $h(u^{*})=H^{*}$ with $H^{*}=f-D_0/C_R$.
2. If $H^{*}\ge0$ the equation $\big[(1+i)^{t_c(1-u^{*})}-1\big]/(it_c)=H^{*}$ gives $(1+i)^{t_c(1-u^{*})}=1+it_cH^{*}$, so $u^{*}=1-\dfrac{\ln(1+it_cH^{*})}{t_c\ln(1+i)}$, and $s^{*}=u^{*}C_R/E$.
3. If $H^{*}<0$ the reactor alone cannot deliver $D_0$. In the second regime $C_Rf+(G^{*}-C_R)=D_0$, so $G^{*}=D_0-C_Rf+C_R$ and $s^{*}=G^{*}/E$. The two regimes meet at $H^{*}=0$, where $u^{*}=1$ and $G^{*}=C_R$.
4. If $s^{*}>1$ the target cannot be reached on this eligible set. If $H^{*}\ge f$ (that is $\mathrm{LAC}_5(0)\le T$) the formula returns $u^{*}\le0$ and the target is met without a grant.
5. The script confirms symbolically that $h(u^{*})=H^{*}$ and numerically that the formula equals a bisection root to $10^{-30}$.

**Final implementable formula.**
$$u^{*}=1-\frac{\ln\!\big(1+i\,t_c\,H^{*}\big)}{t_c\ln(1+i)},\quad H^{*}=f-\frac{D_0}{C_R},\quad s^{*}=\frac{u^{*}C_R}{E}\ \ (H^{*}\ge0);\qquad s^{*}=\frac{D_0-C_Rf+C_R}{E}\ \ (H^{*}<0).$$

**Numbers.** BOAK 25 y, A: $D_0=89.7495/0.16205954=553.81$, $H^{*}=1.19766-553.81/742.767=0.452063$, $u^{*}=1-\ln(1.198907)/0.423286=0.57142$, and $s^{*}=u^{*}$ because $E=C_R$ (L5). For B the same $u^{*}$ applies, because the reactor value that must be removed does not change, and $s^{*}=u^{*}C_R/E=0.57142\times0.81209=0.46405$.

**Feeds into.** S.7e.

Rust implementation: `Curve::ledger`, at-close branch, called by `Model::breakeven_at`

Validation test: `eq_s_7c_break_even_at_close_in_both_regimes_of_the_ledger` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.7d: break-even share at close, DCF, and the numerical fallback (numerical)

**What it calculates.** $s^{*}_D$ at close, and a general root finder for any case where the cost is not affine.

**Variables.** $s_m=S_m/E$ [$1$] the shares at which the grant exactly clears bucket $m$; $\mathrm{LAC}_D(s)$ [$\mathrm{S\$\ t^{-1}}$]; $\epsilon$ [$1$] tolerance.

**Where it comes from.** S.6b. With the grant-first rule $\mathrm{LAC}_D(s)$ is continuous, strictly decreasing and piecewise linear in $s$, with kinks at the shares $s_m=S_m/E$ at which the grant just clears a bucket. Its slope magnitude is $\kappa E(1+i)^{-t_m}$ while bucket $m$ is being paid, which shrinks from $1.469\,\kappa E$ at year $-5$ to $\kappa E$ at year 0.

**Derivation.**

1. Exact route. Evaluate $\mathrm{LAC}_D$ at every knot $s_m\le1$, including the end points $s=0$ and $s=1$. Find the first knot $s_b$ with $\mathrm{LAC}_D(s_b)<T$ and its predecessor $s_a$.
2. On $[s_a,s_b]$ the function is a straight line, so $s^{*}=s_a+\dfrac{\mathrm{LAC}_D(s_a)-T}{\mathrm{LAC}_D(s_a)-\mathrm{LAC}_D(s_b)}\,(s_b-s_a)$.
3. Fallback for any monotone case. Because $\mathrm{LAC}(0)>T>\mathrm{LAC}(1)$ brackets the root, bisect on $[0,1]$ until $s_{hi}-s_{lo}\le\epsilon$ and return $s_{hi}$. The upper end always satisfies the strict inequality, so a returned share always passes.
4. If $\mathrm{LAC}(1)\ge T$ report "unreachable". If $\mathrm{LAC}(0)<T$ report $s^{*}=0$ with "met without a grant".

**Final implementable formula.** Linear interpolation of step 2, or bisection of step 3 with $\epsilon=10^{-12}$.

**Numbers.** The script confirms that the exact route and bisection agree to $10^{-25}$ for all 8 points and both definitions. Break-even shares with the legacy $\mathcal{A}$ (L5 / DCF):

| Stage | $t_\ell$ | A progressive | A at close | B progressive | B at close | idle-cash bound (A, L5) |
|---|---|---|---|---|---|---|
| FOAK | 25 | 0.77138 / 0.77187 | 0.73126 / 0.73301 | 0.68382 / 0.68442 | 0.63403 / 0.63555 | 0.60502 |
| FOAK | 60 | 0.71223 / 0.71317 | 0.66643 / 0.66778 | 0.63138 / 0.63238 | 0.57782 / 0.57899 | 0.55863 |
| BOAK | 25 | 0.62255 / 0.62335 | 0.57142 / 0.57302 | 0.52174 / 0.52260 | 0.46405 / 0.46535 | 0.48829 |
| BOAK | 60 | 0.52790 / 0.52944 | 0.47513 / 0.47638 | 0.44242 / 0.44387 | 0.38585 / 0.38687 | 0.41405 |
| 10OAK_P46 | 25 | 0.45622 / 0.45738 | 0.40473 / 0.40611 | 0.36036 / 0.36144 | 0.30694 / 0.30799 | 0.35783 |
| 10OAK_P46 | 60 | 0.32191 / 0.32411 | 0.27821 / 0.27913 | 0.25427 / 0.25612 | 0.21099 / 0.21169 | 0.25248 |
| 10OAK_P47 | 25 | 0.66743 / 0.66814 | 0.61849 / 0.61911 | 0.56872 / 0.56951 | 0.51206 / 0.51257 | 0.52349 |
| 10OAK_P47 | 60 | 0.58348 / 0.58484 | 0.53120 / 0.53340 | 0.49719 / 0.49851 | 0.43979 / 0.44161 | 0.45765 |

In every row the progressive share exceeds the at-close share, which exceeds the idle-cash bound. These are worked examples with the legacy abatement, not results.

**Feeds into.** S.7e.

Rust implementation: `Curve::dcf`, at-close branch (exact piecewise-linear route with `Curve::dcf_close`), `subsidy::bisect_share` and the residual check of `Model::solve_share`; all called by `Model::breakeven_at`

Validation test: `eq_s_7d_the_exact_route_and_the_bisection_agree_and_the_orderings_hold` in `engine/plant/tests/m10_subsidy.rs`; `subsidy::tests::eq_s_7d_a_closed_form_that_fails_the_residual_check_falls_back_to_bisection` in `engine/plant/src/subsidy.rs`

### Equation S.7e: cases of $s^{*}$ and the target flag (analytical)

**What it calculates.** What to report for each value of $s^{*}$, and the pass or fail flag for a given share.

**Variables.** $s^{*}_5,s^{*}_D$ [$1$]; $s^{*}=\max(s^{*}_5,s^{*}_D)$ [$1$]; flags [$1$].

**Where it comes from.** Decision D3, and the monotone decrease of S.9a.

**Derivation.** $\mathrm{LAC}$ decreases with $s$, so the set of passing shares is the open interval $(s^{*},1]$ when $\mathrm{LAC}(1)<T$.

| Case | Meaning | Report |
|---|---|---|
| $s^{*}<0$ | $\mathrm{LAC}(0)<T$ | met without a grant, $s^{*}$ shown as 0, every grid share passes |
| $s^{*}=0$ | $\mathrm{LAC}(0)=T$ exactly | the strict test fails at $s=0$ and passes at every $s>0$ |
| $0<s^{*}\le0.75$ | inside the grid range | grid shares above $s^{*}$ pass, the others fail |
| $0.75<s^{*}<1$ | beyond the grid | all four grid shares fail, a larger share passes |
| $s^{*}\ge1$ | $\mathrm{LAC}(1)\ge T$ | unreachable on this eligible set (a share of 1 itself gives $\mathrm{LAC}(1)\ge T$), report $\mathrm{LAC}(1)$ |

**Final implementable formula.**
$$\text{met}_5=\big[\mathrm{LAC}_5(s)<T\big],\quad \text{met}_D=\big[\mathrm{LAC}_D(s)<T\big],\quad \text{met}=\text{met}_5\wedge\text{met}_D .$$
The comparison uses the unrounded values. Present $s^{*}$ rounded up in the last digit shown, and check that the rounded share passes.

**Numbers.** The script rescales the placeholder abatement to show every case (progressive, A, L5): BOAK 25 y with twice the abatement gives $s^{*}=-0.0711$ and all grid shares pass; BOAK 25 y as it stands gives $0.6225$ and only the 75 percent share passes; FOAK 25 y gives $0.7714$ and no grid share passes; BOAK 25 y with 0.4 times the abatement gives $1.0387$ (unreachable). Scaling the abatement so that $\mathrm{LAC}_5(0)=100$ exactly gives $s^{*}=0$ and the grid result fail, pass, pass, pass.

**Feeds into.** S.8.

Rust implementation: `subsidy::BreakEven`, `subsidy::combine`, `subsidy::round_up_share` and the strict flag `MethodEval::target_met`

Validation test: `eq_s_7e_cases_of_the_break_even_share_and_the_strict_target_flag` and `eq_s_7e_a_break_even_set_is_a_pure_function_of_its_inputs` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.8: reported quantities (analytical)

**What it calculates.** The quantities printed for every scenario, with their formulas.

**Variables.** Each quantity of the table below with the unit given there; the symbols ($K_{tot}$, $E$, $G$, $K^{priv}$, $\mathrm{IDC}^{(2)}$, $\Delta C_5$, $\mathcal{A}$, $\mathrm{LAC}_k$, $s^{*}$) are defined in the Reference Frame.

**Where it comes from.** S.1 to S.7.

**Derivation.**

1. Capital rows. The first-unit capitals of S.0.5 add to $K_{tot}$. Replacement units and decommissioning are left out, and so is the reformer island, which cancels between the two plants (S.1). Reactor interest is $C_R(f_{IDC}-1)$ in the ledger (L.21) and $C_R(f_D-1)$ in the DCF (S.0.1).
2. Grant rows. $E$ comes from S.1 and $G=sE$ from S.2. The private capital is $K_{tot}-G$, and the private interest is $\mathrm{IDC}^{(2)}$ of S.5a.
3. Annual cost rows. $\Delta C_5(0)$ is the sum of the ledger lines. Because $\mathrm{LAC}_5=10^{6}\Delta C_5/\mathcal{A}$ (S.0.5) and $\mathcal{A}$ does not depend on $s$, $\Delta C_5(s)=\mathrm{LAC}_5(s)\,\mathcal{A}\,10^{-6}$ with $\mathrm{LAC}_5(s)$ from S.6a.
4. Cost per tonne, break-even and flag rows are S.6, S.7 and S.7e in turn.
5. State-side present value. A progressive grant is paid on the dates of the spend it funds, so its value at year 0 is the magnitude of the GRANT line of S.4a, $s(C_Rf_D+X_E)$. A grant at close is paid $t_c$ before year 0 and is valued at the same rate $i$, which gives $G(1+i)^{t_c}$ (S.3d). The excess over the private benefit is the carry on idle cash.

**Final implementable formulas.**

| Quantity | Formula | Units |
|---|---|---|
| Total project capital (overnight) | $K_{tot}=C_R+K_{cap}+K_{ihx}+K_{lic}+K_{pi}+K_{rec}$ (first units; excludes interest, decommissioning and the reformer island) | $\mathrm{S\$M}$ |
| Gross interest, ledger and DCF basis | $C_R(f_{IDC}-1)$ and $C_R(f_D-1)$ | $\mathrm{S\$M}$ |
| Total project capital with interest | $K_{tot}+C_R(f_{IDC}-1)$ | $\mathrm{S\$M}$ |
| Eligible capital | $E$ (S.1) | $\mathrm{S\$M}$ |
| Grant | $G=sE$ | $\mathrm{S\$M}$ |
| Private capital | $K^{priv}=K_{tot}-G$, and with interest $K^{priv}+\mathrm{IDC}^{(2)}$ | $\mathrm{S\$M}$ |
| Annual incremental cost, gross and net | $\Delta C_5(0)$ and $\Delta C_5(s)=\mathrm{LAC}_5(s)\,\mathcal{A}\,10^{-6}$ | $\mathrm{S\$M\ y^{-1}}$ |
| Annual net abatement | $\mathcal{A}$ (same at every $s$) | $\mathrm{t\ y^{-1}}$ |
| Cost per tonne, gross and net of grant | $\mathrm{LAC}_k(0)$ and $\mathrm{LAC}_k(s)$ for $k=5,D$ | $\mathrm{S\$\ t^{-1}}$ |
| Break-even share | $s^{*}_5$, $s^{*}_D$, $s^{*}$ (S.7) | $1$ |
| Target flag | $\text{met}$ (S.7e) | boolean |
| State-side present value (information only) | progressive $s(C_Rf_D+X_E)$; at close $G(1+i)^{t_c}$ | $\mathrm{S\$M}$ |

"Gross" means with the grant ignored. "Net of grant" is the cost to the private owner. The cost to society also includes what the state pays. At close the state-side present value exceeds the private benefit by the carry on idle cash: BOAK 25 y, $s=0.5$, A: $567.092$ against $492.450$, a difference of $74.642\ \mathrm{S\$M}$.

**Numbers.** BOAK 25 y, $s=0.5$, legacy $\mathcal{A}$: $K_{tot}=1062.227$ and the total with interest is $1209.043\ \mathrm{S\$M}$. A: $E=742.767$, $G=371.384$, $K^{priv}=690.844$; B: $E=914.637$, $G=457.318$, $K^{priv}=604.909$. Gross annual cost $109.685\ \mathrm{S\$M\ y^{-1}}$; net annual cost, A progressive $68.017$, A at close $63.625$, B progressive $59.967$, B at close $54.285\ \mathrm{S\$M\ y^{-1}}$. Net $\mathrm{LAC}_5$ and $\mathrm{LAC}_D$ are in S.6. The 50 percent share passes both tests only for definition B at close (93.91 and 94.01).

**Feeds into.** The CSV table of Phase 5 and the interface of Phase 6.

Rust implementation: `Model::subsidy_eval_at` (types `SubsidyEval`, `MethodEval` and `GrantPlan`) and the exporter function `export::subsidy_scenarios`, which writes `results/Subsidy_Scenarios.csv`

Validation test: `eq_s_8_reported_quantities_are_consistent`, `eq_s_8_a_grant_leaves_the_rx_mult_lever_alone`, `eq_s_8_the_committed_table_matches_the_engine` and `eq_s_8_the_committed_table_reproduces_the_economic_ledger_at_a_zero_share` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.9a: monotonicity and convexity (analytical)

**What it calculates.** The shape of $\mathrm{LAC}(s)$, which justifies a unique root and the bracketing in S.7d.

**Variables.** $\mathrm{LAC}'(s)$ [$\mathrm{S\$\ t^{-1}}$ per unit share]; $t_m$ [$\mathrm{y}$] date of the bucket being covered.

**Where it comes from.** Differentiate S.6a and S.6b.

**Derivation.**

1. Progressive: $\mathrm{LAC}'=-M<0$ and $\mathrm{LAC}''=0$.
2. Ledger at close, reactor regime: with $u=sE/C_R$, $\mathrm{LAC}_5'(s)=\kappa E\,h'(u)=-\kappa E\ln(1+i)(1+i)^{t_c(1-u)}/i<0$, and $\mathrm{LAC}_5''=\kappa E^{2}h''(u)/C_R>0$ (S.3c).
3. Ledger at close, beyond the reactor: the slope is $-\kappa E$. At $u=1$ the slope jumps from $-\kappa E\ln(1+i)/i=-0.96201\,\kappa E$ to $-\kappa E$. This is a small concave kink, present only for definition B and only for $s>C_R/E$, which is 0.812 for BOAK at 25 y. The function is still strictly decreasing, so the root is unique.
4. DCF at close: $\mathrm{LAC}_D'(s)=-\kappa E(1+i)^{-t_m}$, where $t_m$ is the date of the bucket being covered. The slope magnitude falls with $m$ and equals $\kappa E$ beyond the reactor, so the function is piecewise linear and convex, and it decreases strictly.

**Final formula.** $\mathrm{LAC}_k'(s)<0$ for all $s\in[0,1]$ in every case.

**Numbers.** A 201-point scan over all 8 points, each in $2\times2\times2$ scenario variants, finds strictly decreasing values everywhere. It finds convexity for every case except the ledger definition B beyond $s=C_R/E$, where the measured slope ratio at the kink is 0.96201301, equal to $\ln(1+i)/i$.

**Feeds into.** S.7d, S.9b.

Rust implementation: no separate function: the shape follows from the forms of S.6a and S.6b

Validation test: `eq_s_9a_the_cost_falls_strictly_with_the_share_and_bends_one_way` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.9b: limiting cases (analytical)

**What it calculates.** What the model must return at the ends of the range.

**Variables.** $s$ [$1$]; $G$, $C_R$, $X_E$ [$\mathrm{S\$M}$]; $u$ [$1$]; $h(u)$ [$1$]; $P_D$ [$\mathrm{S\$M}$]; $\kappa$ [$\mathrm{S\$\ t^{-1}}$ per $\mathrm{S\$M}$]; $f_{IDC}$, $f_D$ [$1$].

**Where it comes from.** S.6a and S.6b at $s=0$ and $s=1$.

**Derivation.**

1. $s=0$. Then $G=0$, $u=0$, $h(0)=f_{IDC}(t_c)$, $p_k=b_k$ and $\mathrm{PV}(\text{GRANT})=0$. Both formulas return $\mathrm{LAC}(0)$ of S.0.5. The engine must skip the grant block, so the result is bit-identical.
2. $s=1$, definition A, progressive: the reductions are $\kappa C_Rf_{IDC}$ and $\kappa C_Rf_D$, which are exactly the REACTOR_CAP plus IDC lines of the ledger and the whole reactor present value of the DCF.
3. $s=1$, definition A, at close: $G=C_R$, $u=1$, $h(1)=0$ and $P_D=0$, so the same reductions result.
4. $s=1$, definition B: the reactor goes as above and the other eligible items drop out of the first installation (progressive: $\kappa X_E$; at close: the excess $G-C_R=X_E$). The IHX replacement units stay private.
5. The idle-cash bound of S.3d fails step 2 at $s=1$.

**Final formula.** $\mathrm{LAC}(0)$ equals the S.0.5 value; $\mathrm{LAC}_5(1)=\mathrm{LAC}_5(0)-\kappa C_Rf_{IDC}$ for A in both timings.

**Numbers.** The script reads the REACTOR_CAP and IDC lines straight from the CSV and confirms that the reduction at $s=1$ equals $(\text{REACTOR\_CAP}+\text{IDC})10^{6}/\mathcal{A}$ for all 8 points and both timings, to within the CSV rounding. BOAK 25 y: $83.335061\times10^{6}/578{,}051.6=144.1654\ \mathrm{S\$\ t^{-1}}$, which is $M_5$.

**Feeds into.** The Phase 4 limit tests.

Rust implementation: the zero-share skip in `Model::ledger` and `Model::dcf_at` (a share of zero leaves `Ledger::net` as `None` and adds no GRANT line)

Validation test: `eq_s_9b_a_zero_share_changes_nothing_bit_for_bit`, `eq_s_9b_zero_subsidy_reproduces_the_committed_ledger_and_dcf` and `eq_s_9b_limiting_cases_at_zero_and_full_share` in `engine/plant/tests/m10_subsidy.rs`

### Equation S.9c: independent closed-form check of the grant effect (analytical, with three further routes)

**What it calculates.** A check of the grant effect that does not use the engine's code path and does not rely on the two methods agreeing with each other.

**Variables.** $C_R$ [$\mathrm{S\$M}$] from the registry (L.5, L.6); $\mathrm{LAC}_k(0)$ [$\mathrm{S\$\ t^{-1}}$]; $\kappa$; $i$, $t_c$, $t_\ell$, $\mathcal{A}$.

**Where it comes from.** The closed forms of S.6 use only $C_R$ (computed from `OCC_CENTRAL`, the reference size, the dollar factor and the rating, with no engine call), $i$, $t_c$, $t_\ell$, $\mathcal{A}$ and the zero-subsidy cost. The engine, when it is built, can be compared with them. The script also builds the answer in three other ways.

**Derivation of the routes.**

1. Route 1, closed form: S.6a and S.6b.
2. Route 2, ledger lines: rebuild each ledger line from private capital ($K^{priv}_R\,\mathrm{CRF}$, interest on the private window, credits annualised with $\mathrm{CRF}(t_\ell)$) and add them.
3. Route 3, dated cash flows: list the gross flows, add the GRANT flows at their dates, discount each, and divide by $\mathcal{A}\,\alpha(t_\ell)$.
4. Route 4, funded balance: run the year-by-year recursion of S.3a with explicit cash accounts and rebuild the cost from the balance at year 0.
5. Gap law. For a progressive grant, S.0.6 and S.4a predict $\mathrm{gap}(s)=\mathrm{gap}(0)-s\,C_R\,\mathrm{CRF}(t_\ell)\,\Delta f\,10^{6}/\mathcal{A}$, so the reactor term scales by $(1-s)$ exactly. At close, the gap stays between zero and its zero-subsidy value for every $s$ on a 101-point grid.

**Final formula for the independent check (BOAK, 25 y, A).**
$$\mathrm{LAC}_5(0.5)=189.7495-0.5\times144.16543=117.6668,\qquad \mathrm{LAC}_5^{close}(0.5)=189.7495-120.37256\,(1.19766-0.53570)=110.0672 .$$

**Numbers.** The four routes agree to $3\times10^{-38}$ (the script works to 40 digits) over 224 combinations of stage, life, definition, timing and share. The DCF values for A at $s=0.5$ are $117.8214$ (progressive) and $110.2526$ (at close), and the closed forms are $190.0588-0.5\times144.47475=117.8214$ and $190.0588-0.16206\times492.4501=110.2526$. The independent evidence therefore consists of the closed form, the line rebuild, the dated flows, the balance recursion and the CSV reduction at $s=1$ (S.9b). The agreement between the ledger and the DCF is only one part of it.

**Feeds into.** The Phase 4 validation design.

Rust implementation: `SubsidyEval` fields `gap_s_per_t` and `gap_rel` with `SubsidyEval::gap_within`, and the GRANT term in `DcfResult::gap_terms`

Validation test: `eq_s_9c_the_grant_effect_by_four_routes_that_share_no_code_with_the_engine`, `eq_s_9c_the_gap_between_the_methods_equals_the_prediction_and_stays_below_the_limit` and `eq_s_9c_gap_terms_still_sum_to_the_gap_with_a_grant` in `engine/plant/tests/m10_subsidy.rs`

---

# New parameters

Provenance classes are those of `parameters.toml`: published, assumed, calculated, user_decision. The abatement $\mathcal{A}$ is not new; it comes from the carbon rebuild.

| Id (proposed) | Symbol | Unit | Proposed value | Provenance | Needs |
|---|---|---|---|---|---|
| `SUBSIDY_SHARE_GRID` | $s$ | $1$ | 0, 0.25, 0.5, 0.75 | user_decision | none |
| `SUBSIDY_DEFINITION` | | enum | A (reactor only) | user_decision | the user to confirm A as default |
| `SUBSIDY_TIMING` | | enum | progressive default; at close printed beside it | user_decision | the user to confirm the default |
| `SUBSIDY_ELIGIBLE_B_ITEMS` | $e_j$ | flags | reactor, IHX, licence, power island on; capture, recycle off | assumed | review of the licence overlap and of the power island proxy |
| `SUBSIDY_GRANT_CASH_RATE` | $\rho_g$ | $\mathrm{y^{-1}}$ | 0 | assumed | only 0 is implemented; $i$ is the bound of S.3d |
| `SUBSIDY_IDC_BASIS` | | enum | private balance | assumed | none |
| `SUBSIDY_SHARE_MAX` | | $1$ | 1 | calculated | a share cannot exceed 1 |
| `SUBSIDY_SOLVER_TOL` | $\epsilon$ | $1$ | $10^{-12}$ | assumed | numerical setting only |
| `TARGET_COST` (existing) | $T$ | $\mathrm{S\$\ t^{-1}}$ | 100 | user_decision | reused unchanged, with the strict test of D3 |

The financial close date is a convention ($-t_c$), not a parameter.

---

# Hand-over to Phase 3

- Add the grant inputs to `CostQuery` and the eligible and granted amounts to `CostItems`. Leave `rx_mult` alone.
- Keep `Ledger::lines` (16 entries) and the L0 to L5 rungs gross. Add the net-of-grant block GRANT_CREDIT and IDC_EFFECT after L5.
- In `Model::dcf` add one GRANT line with negative flows (progressive: $-sb_k$ and $-sK_j$; at close: $-g_k$ and $-G_X$), and extend `gap_terms` with a GRANT term that pairs with GRANT_CREDIT plus IDC_EFFECT, so that `gap_terms_sum_to_the_gap` stays exact. All dates stay whole years.
- Predictions that tests can assert tightly (the 10 percent limit cannot): the progressive reactor gap term scales by $(1-s)$; at close $0\le\mathrm{gap}(s)\le\mathrm{gap}(0)$; $P_D(G)$ equals the balance recursion; the ledger at close equals $C_Rh(u)$; $\mathrm{LAC}(s^{*}+\delta)<T<\mathrm{LAC}(s^{*}-\delta)$.
- Zero subsidy must reproduce `results/` bit for bit.

# Open points and unverified items

- The Rust engine was not built or run here (no cargo). All numbers are reconstructions from `results/Economic_Ledger.csv`, which could be stale. The reconstruction matches it to the 6-decimal rounding.
- The cited source documents were not opened here, so every source status is copied from `parameters.toml`. The statement that a Total Capital Requirement includes interest during construction rests on the comment at the top of `dcf.rs` and on the registry text, and is unverified against the IEAGHG table.
- Definition B is an engineering judgement. The licence line may overlap the licensing share inside the INL overnight cost, and the power island is partly a proxy for the tail-gas turbine. Both are flagged for the user.
- The idle-cash rule of convention 3 drives the at-close result. The bound of S.3d shows the size of the alternative.
- Items other than the reactor are bought at year 0 in both methods and carry no interest. If a construction profile is later added for them, S.1 and S.5a apply to them unchanged, but zero-subsidy results would change and need a separate decision.
- Not modelled: tax, clawback, a programme budget cap, cost overruns, a separate financing rate for interest during construction, and the High and Low cost levels (the DCF runs at the central level only).
- The break-even tables use the legacy abatement as a placeholder. They will change when the carbon rebuild supplies $\mathcal{A}$.

# Verification record

Command, run from the repository root with a Python 3.10 interpreter that has SymPy 1.14 and mpmath 1.3 (no other package is needed):

`python -I math/verify/sympy_subsidy.py`

Result on 2026-11-06: `120 checks, 120 passed, 0 failed`, exit status 0. The 120 checks are 22 symbolic identities (part A), 69 checks that rebuild the zero-subsidy ledger and DCF from the CSV (part B) and 29 checks of the grant model (part C). The script exits with status 1 if any check fails. It reads the CSV and the registry by relative path, so it does not depend on the working directory.
