# AGENTS.md

Instructions for AI agents working in this repository.

## Prime directive

This is a [CodeCrafters](https://codecrafters.io) "Build your own Shell" challenge.
The challenge is solved **by the human, not by agents**.

- **Never** write, modify, refactor, or fix code under `src/`.
- **Never** add runtime `[dependencies]` to `Cargo.toml`.
- **Never** edit `stage_descriptions/`, `your_program.sh`, `.codecrafters/`, or `codecrafters.yml`.
- **Never** suggest an implementation approach, algorithm, or crate for a stage, in code
  or in prose, unless the human explicitly asks.

Agents own exactly one thing: the **integration test harness** under `tests/`, plus
`test.sh`, `AGENTS.md`, and `[dev-dependencies]` in `Cargo.toml`.

If a test fails, the correct agent response is to report the failure and the observed
output. Do not fix `src/`. A red test is the human's next task, not a bug.

## What the tests are

Black-box integration tests. They spawn the compiled shell binary and interact with it
the way CodeCrafters does: send input, observe stdout/stderr, observe side effects on the
filesystem and process table. No unit tests, no `#[cfg(test)]` inside `src/`, no
knowledge of internal types.

The binary under test is resolved via `env!("CARGO_BIN_EXE_codecrafters-shell")`, so
`cargo test` always rebuilds and tests current `src/`. Tests never invoke
`your_program.sh`.

## Why a PTY

Later stages (tab completion, job control, signals, history navigation) only behave
correctly when stdin is a terminal — readline implementations disable line editing and
completion on a pipe. Every test therefore runs the shell on a pseudo-terminal, from
stage 1 onward, so no test needs rewriting when those stages arrive.

## Layout

```
tests/
  harness/
    mod.rs         re-exports
    session.rs     PTY spawn, send_line, expect_*, read_until, timeouts
    sandbox.rs     temp cwd/HOME, generated PATH with fake executables
  stage_01_base.rs
  stage_02_navigation.rs
  ...
  stage_12_parameter_expansion.rs
```

One file per stage group. One module per step, named after the step's slug. One `#[test]`
per behaviour.

```rust
// tests/stage_01_base.rs
mod base_06_ez5 {
    // --- spec ---
    #[test] fn type_reports_echo_as_builtin() {}
    #[test] fn type_reports_unknown_command_as_not_found() {}

    // --- additional ---
    #[test] fn shell_stays_alive_after_type_sequence() {}
}
```

`// --- spec ---` marks assertions the CodeCrafters tester will actually make.
`// --- additional ---` marks agent-authored hardening. Keep the sections separate so the
human can tell at a glance what is required versus what is extra.

## Writing test cases

Go **beyond** the stage description. The description is a floor, not a ceiling. Cover
edge cases the human would plausibly get wrong: empty input, extra whitespace, repeated
invocations, ordering, the shell surviving the sequence.

The hard constraint: **every assertion must be derivable from the stage description or
from POSIX/bash behaviour the stage explicitly references.** Never assert on behaviour the
stage leaves unspecified. If bash does X and the stage is silent about X, that is not a
test — that is an assumption, and it will produce a red test for work the human is not
required to do. When in doubt, leave it out and mention it to the human instead.

Tests for stages not yet reached are expected to fail. Do not `#[ignore]` them; the red
count is the progress indicator.

Other rules:

- **Exact matching.** Compare output byte-for-byte, as CodeCrafters does. Failure
  messages must show expected vs. actual clearly enough to debug without rerunning.
- **Hard timeout on every read.** A hung shell must fail fast, never block the suite.
- **Full isolation.** Each test gets a fresh temp cwd, `HOME`, `PATH`, and history file.
  Tests must pass in any order and in parallel, and must not touch the real environment.
- **Fake executables are generated at runtime** as `#!/bin/sh` scripts in the sandbox
  `PATH`. Never commit binaries or fixture executables.
- **Harness code stays in `tests/harness/`.** No copy-pasted PTY plumbing in stage files;
  a stage file should read as a list of behaviours.

## Running

```sh
./test.sh          # all stages
./test.sh ez5      # one step (slug filter)
./test.sh base     # one stage group
```

`test.sh` uses `cargo nextest` when available — process-per-test isolation, which PTY
tests want — and falls back to `cargo test` otherwise.

## Commits

Agents may commit autonomously, but **only** test-harness changes.

- Scope every commit `test:`.
- Commit only your own changes; the working tree may contain the human's in-progress
  challenge work. Never `git add -A`. Never commit `src/`.
- Never `git push`, never amend or rewrite the human's commits.
- Attribute with a trailer:

```
test: add PTY session harness

Agent: <agent> <version> (<model>)
```
