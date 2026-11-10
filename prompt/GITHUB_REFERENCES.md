# Outside tools to study

These are reference tools, not permission to replace the Rust engine. Adopt one only when it closes a named gap, and record the decision here (licence read, maintenance checked, dependency and bundle cost, external hosts needed, compatibility, maintenance cost). Do not install a library merely because it is listed. Keep `THIRD_PARTY_NOTICES.md` current for anything adopted. Licence and activity below were read from the GitHub API on 2026-11-10; read each LICENSE file yourself before relying on it.

| Repository | Licence (API) | Last push | Status note | Use |
|---|---|---|---|---|
| IDAES/idaes-pse | not asserted by API (read LICENSE) | 2026-11-09 | active | Study process-model structure, unit-model balances and degrees-of-freedom checks. Optional offline cross-check only. Python and Pyomo stack, heavy. |
| DanWBR/dwsim | GPL-3.0 | 2026-07-17 | archived | Read for method only. GPL-3.0 forbids copying code into this MIT repository. Never link or vendor it. |
| CoolProp/CoolProp | MIT | 2026-11-09 | active | Independent thermodynamic property check (steam, CO2, mixtures) in a Python verification script. Not a runtime dependency of the engine or the site. |
| HypothesisWorks/hypothesis | not asserted by API (read LICENSE) | 2026-11-08 | active | Property-based tests for Python tooling (`scripts/`), for example unit conversion and CSV comparison. Dev-only. |
| proptest-rs/proptest | Apache-2.0 | 2026-10-10 | active | Generated Rust tests: mass and element conservation, zero streams, monotone cost in grant share, share bounds. Dev-dependency only; check it adds no build-time surprise. |
| nextest-rs/nextest | Apache-2.0 | 2026-11-08 | active | Faster, controllable test runner (limits concurrency, helps the fork-error problem). Developer tool, not a dependency. |
| d3/d3 | ISC | 2026-11-03 | active | Study scales, axes and accessible chart patterns. Do not bundle the whole library; copy no code. A small ISC helper such as d3-scale could be considered if it replaces hand-written code. |
| observablehq/plot | ISC | 2026-11-05 | active | Study concise layered scenario charts. Likely needs a bundler and d3; prefer the existing SVG helpers unless a clear gain is shown. |
| microsoft/playwright | Apache-2.0 | 2026-11-10 | active | Candidate for cross-viewport screenshots and accessibility checks. The repository already has a Chrome-based self-test; adopt only if it removes a known weakness, and note that it downloads browsers from external hosts at install time. |
| tectonic-typesetting/tectonic | not asserted by API (read LICENSE) | 2026-08-01 | active | The PDF build tool for the derivation book. Fetches TeX bundle files from the network once; record the pinned version and bundle. |
| astral-sh/ruff | MIT | 2026-11-10 | active | Python lint and format for `scripts/` and `math/verify/`. Dev tool; configure to the repository's existing style. |

## Rules

- External-host guard: the shipped site must make no external request. Build-time downloads are acceptable only for development tools and must be documented.
- Prefer improvements that directly resolve known gaps: test concurrency (nextest), generated balance tests (proptest), reproducible PDF (Tectonic), Python quality (ruff).
- Record each decision (adopted, deferred, rejected, with the reason) in a short table appended to this file.

## raystorage

weewzr/raystorage is the owner's own public archive, not an outside tool. See `RAYSTORAGE_INTEGRATION.md`.
