# NUS-H2 completion package

This folder is the persistent instruction set for a future Claude Code session that finishes the NUS-H2 engineering project and its browser simulator. The repository is the source of truth. A session that reads this folder needs no other briefing.

## Reading order

1. `MASTER_PROMPT.md`: mission, settled decisions, hard rules, definition of done.
2. `CURRENT_STATE_AND_BASELINE.md`: the exact Git state, what exists, what is missing.
3. `AUTONOMOUS_EXECUTION.md`: what the session may do without asking, how to use agents.
4. `COMMIT_AND_DEPLOYMENT.md`: milestones, commit and push policy, the final pull request.
5. The six workstream files: `ENGINE_REFERENCE_PLANT.md`, `CARBON_AND_SUBSIDY.md`, `ECONOMIC_ASSUMPTIONS.md`, `FRONTEND_AND_VISUAL_DESIGN.md`, `DOCUMENTATION_AND_DERIVATIONS.md`, `TESTING_AND_VERIFICATION.md`.
6. `GITHUB_REFERENCES.md`: outside tools to study, with licence notes.
7. `PROGRESS.md`: the live record. Resume from it. Update it at every milestone.

`reference/` holds the verified IEAGHG reference-plant specification and its critique, written before this package. They are inputs, not instructions to repeat. Where the critique and the specification disagree, the critique wins.

## Principles

- Scientific traceability first. Every number is published, calculated or assumed, and says which.
- The Rust engine calculates. The exporter writes results. Python only processes exported data. JavaScript only displays the engine reply.
- Never invent a value, a citation or a coordinate. Mark gaps as unavailable and continue with other work.
- Never weaken a tolerance or edit a golden file to make a test pass.
- Existing work is valuable. Never reset, force-push or delete history.
- Do not merge into `main`. The final pull request stays open.

## What this package does not contain

No engineering change. The implementation is the job of the future session.

## raystorage

`RAYSTORAGE_INTEGRATION.md` and `RAYSTORAGE_FILE_INDEX.md` map the public archive weewzr/raystorage (pinned commit `3a25c4a7`) to the tasks. Read the privacy findings in section 3 of the integration file before opening any archived file.
