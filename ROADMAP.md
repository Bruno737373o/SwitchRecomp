# Roadmap

This roadmap reflects the current implementation state as of 2026-03-05. The sequencing is intentionally homebrew-first, retail-second, cloud-third.

## Current State
- The local operator path is now real for the public samples:
  - pipeline tests pass
  - repo-root emitted-crate build/run is verified
  - homebrew intake and lift are verified
  - the automation CLI is wired and tested
- The retail title track remains scaffold-only.
- The cloud path exists only as design and contract work.

## Phase 0: Truth-Reset And Local Operator Path
- Verify the documented local commands against the actual workspace.
- Make emitted crates build from repo root.
- Keep status docs honest about what is implemented versus what is scaffold.
- Stabilize `module.json`, `manifest.json`, and `run-manifest.json` as deterministic stage outputs.

Exit criteria:
- Local sample pipeline emits, builds, and runs from repo root.
- Homebrew intake and lift work against checked-in non-proprietary fixtures.
- `cargo test` passes without relying on stale docs or placeholder commands.

## Phase 1: Public Homebrew Runnable Milestone
- Target a legally distributable homebrew title, starting with `2048`.
- Preserve real segment metadata through intake, lift, and translation.
- Expand the shared CPU state/codegen contract enough for multi-function control flow and memory.
- Implement the minimal libnx/Horizon service surface for the chosen title:
  - `sm:`
  - `sm:m`
  - applet lifecycle
  - `hid`
  - `vi` plus the framebuffer-related `nv` path
  - sdmc-backed filesystem

Exit criteria:
- A public homebrew title recompiles and reaches a stable local loop.
- Two runs with the same inputs produce matching deterministic validation outputs.
- Unsupported services fail explicitly instead of silently stubbing past breakage.

## Phase 2: Homebrew Validation Gate
- Move the first executable gate to deterministic input replay plus framebuffer/save-state hashes.
- Keep video validation available, but do not require it for the first public homebrew milestone.
- Improve run-manifest schema and artifact indexing so local and cloud stages share the same contract.

Exit criteria:
- The chosen homebrew title passes a deterministic validation run twice in succession.
- Validation artifacts are hash-addressed and referenced from a stable run manifest.

## Phase 3: Private Retail Intake And Bring-Up
- Keep retail inputs, keys, and reference media in a private user workspace.
- Pin and test `hactool` or `hactoolnet` compatibility.
- Reuse the same lifted-module, CPU-state, service, and validation contracts proven by the homebrew path.
- Keep title-a24b9e807b456252 in scaffold status until it uses the same real path.

Exit criteria:
- A private retail intake run produces deterministic extraction outputs with pinned external tooling.
- The first private title uses the same core contracts as the homebrew milestone instead of a parallel scaffold.

## Phase 4: Hybrid Cloud Scale
- Promote the local run manifest into the cloud stage contract.
- Store artifacts in S3 using content-addressed keys, Versioning, and Object Lock governance mode.
- Use Step Functions Standard as the orchestration layer.
- Use Distributed Map only for explicit batch fan-out.
- Use AWS Batch `.sync` for heavy worker stages.
- Split worker pools into:
  - `cpu-linux`
  - `video-linux`
  - `metal-macos`

Exit criteria:
- A single run can be executed locally or in AWS without changing stage contracts.
- Artifact lineage is immutable and auditable.
- macOS workers are limited to the cases that truly require Metal parity.

## Phase 5: Stabilization And Compatibility Reporting
- Harden diagnostics, error reporting, and replayability.
- Add a compatibility matrix for public homebrew titles and private-user retail experiments.
- Publish a contributor path that keeps policy, tests, and docs aligned.

Exit criteria:
- Regression results are stable and repeatable.
- The repo can distinguish clearly between:
  - verified runnable targets
  - scaffold profiles
  - unsupported experiments
