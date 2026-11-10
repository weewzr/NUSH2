# Autonomous execution

The repository owner wants the session to manage routine development without asking the collaborator, who is overseas and not available to approve each action.

## 1. Pre-authorised, inside this repository and its development environment

Inspect all files, Git history and branches; create and edit project files; install ordinary project dependencies (project-local, never global); perform clean Rust rebuilds; run Python scripts; run verification and tests; build PDFs, figures and the WebAssembly module; run the browser self-test; create dedicated development branches; commit completed phases; push completed phase commits to the authorised completion branch; open pull requests; use independent subagents and integrate their output; continue through phases automatically.

These instructions express the owner's authorisation. They cannot grant GitHub credentials, override a protected branch, or change the permissions of the collaborator's account. Before leaving, the collaborator must set the real repository access, authentication and Claude Code permission rules (settings files or a permission mode that allows the commands above). Do not bypass approval controls, the sandbox, authentication or any security restriction. If an action needs an unavailable permission, record a BLOCKER in `PROGRESS.md` and continue with other work.

## 2. Do not interrupt the collaborator for

Routine edits, dependency installs, tests, builds, document generation, normal calculations, phase commits, pushes to the completion branch.

## 3. Not pre-authorised (never do these)

Merging into `main` or any protected branch; force-push; history rewrite; deleting branches or remote refs; deleting files outside the repository and its temp folders; touching other repositories; creating a new repository; reading, uploading or publishing the privacy-flagged local files (947 files) and the 140 local Office and PDF files without the owner's explicit permission for that exact action; changing authentication, credentials or settings; sending messages or posting outside GitHub pull requests of this repository.

## 4. Scientific decisions

IEAGHG reference direction and definition C are settled: implement them. For a new unresolved physical datum or consequential assumption: research authoritative evidence, record source and uncertainty, keep unsupported assumptions labelled, never invent evidence, and do not halt unrelated work. Do not alter the owner's accepted definitions without a documented technical reason. Defaults D21 to D23 stay as labelled defaults.

## 5. Agent roles (one coordinating agent, serial writers)

- Reference-plant engine; carbon and subsidy; economics and source verification; frontend and PFD; derivations and documents; independent testing and verification.
- Assign disjoint file sets. Writers that touch shared files (`parameters.toml`, `economics.rs`, `anchored.rs`, `export.rs`, `subsidy.rs`) run one at a time. Read-only audit agents may run in parallel (at most 4). One cargo job at a time. Verification agents do not write except temporary mutations they restore.
- The coordinating agent reviews every diff, reconciles interfaces, resolves conflicts, runs the full gates itself (never relying only on an agent's report) and verifies headline numbers independently.
- Agents return structured results and write long notes into a working folder outside the repository; only reviewed text enters the repository.

## 6. Phase loop

Complete one phase at a time. When a phase passes its acceptance criteria: run the relevant tests, review the diff, update documentation, update `PROGRESS.md`, commit, push the completion branch, continue to the next phase without asking. If a phase is blocked by missing source data, mark that item BLOCKED and continue with other work. Resume from `PROGRESS.md`; never repeat completed work.

## 6b. Usage limits

If a usage limit interrupts a long task, record the exact state in `PROGRESS.md`, commit finished work, and resume from the record. Do not start overlapping writers on resume; inspect `git status` and the last commits first.

## 7. Failure policy

A failing test is investigated, not silenced. Never use `--no-verify`, skip hooks, weaken a tolerance, edit a golden, or mark a feature complete while its acceptance tests fail.

## 8. Reading the archive

The session may read raystorage (public) and run reviewed scripts from it. It must not copy browser-profile data, personal information or credentials, must not edit raystorage, and must record any copied item in `PROGRESS.md`. Permissions follow the real Claude Code settings and GitHub credentials; none is bypassed.
