# Plans

This file is the truth-reset implementation plan. Checked items are verified in the repository today. Unchecked items are the next milestones. A checked box does not mean the full title or retail milestone is done; it means there is code or test evidence for that specific slice.

## Current Verified State
- [x] `cargo test` passes in the dev shell.
- [x] `recomp-cli run` emits the minimal sample and the emitted crate builds and runs from repo root.
- [x] `recomp-cli homebrew-intake` and `recomp-cli homebrew-lift` work against checked-in non-proprietary fixtures.
- [x] `recomp-cli automate --config ...` is implemented and tested with a real pipeline/build/run loop.
- [x] `title.toml` now parses `inputs`, `assets`, `keys`, `patches`, `validation`, `gpu`, and runtime entry-address settings.
- [x] Lifted `module.json` preserves entry address and segment metadata for text/rodata/data/bss.
- [x] Generated code uses shared `CpuState` registers and NZCV flags instead of per-function architectural locals.
- [x] The retail title profile remains documented as scaffold-only instead of being implied complete.

## Milestone 1: Local Public Homebrew
Goal
- Run a real public homebrew title through the local static recompilation flow.

Work items
- [x] Truth-reset the operator path so the documented local commands actually work.
- [x] Preserve lifted segment metadata into translation and runtime memory initialization.
- [x] Carry runtime entry address and shared CPU state through codegen.
- [x] Fail explicitly on unresolved indirect branches in emitted output.
- [ ] Build a deterministic address-to-function map that can drive larger decoded homebrew modules.
- [ ] Implement the minimum libnx/Horizon surface for the first public title:
  - `sm:`
  - `sm:m`
  - applet lifecycle
  - `hid`
  - `vi` plus the framebuffer-related `nv` pieces
  - sdmc-backed filesystem
- [ ] Create a concrete `2048` title profile and run it twice with matching deterministic outputs.

Exit criteria
- A public homebrew title reaches a stable local loop.
- The title uses the shared pipeline/runtime contracts already in the repo.

## Milestone 2: Deterministic Homebrew Validation
Goal
- Use a validation gate that fits the first public title instead of forcing video-first validation too early.

Work items
- [x] Emit a deterministic `run-manifest.json` with input fingerprint, stage outputs, and artifact hashes.
- [x] Record platform profile and tool metadata in the run manifest.
- [ ] Add deterministic input replay to the first-title run path.
- [ ] Add framebuffer hash comparison across two runs.
- [ ] Add save-state hash comparison where the title supports persistent state.
- [ ] Keep video validation as a secondary retail-oriented validator rather than the first public gate.

Exit criteria
- Two identical local runs of the first public title produce matching replay-driven validation outputs.

## Milestone 3: Private Retail Intake
Goal
- Reuse the proven homebrew core for private retail experimentation without pretending the retail path is already solved.

Work items
- [x] Provide XCI intake CLI wiring, config plumbing, provenance enforcement, and deterministic mock coverage.
- [x] Keep keys and assets external to the repository.
- [ ] Pin and test `hactool` and `hactoolnet` compatibility in a private-user workspace.
- [ ] Validate explicit Program NCA selection against real private-user extracts.
- [ ] Reuse the same lifted-module, `CpuState`, service, and validation contracts proven by the homebrew path.
- [ ] Keep title-a24b9e807b456252 in scaffold status until it runs on that shared path.

Exit criteria
- A private retail intake run is deterministic and externally reproducible.
- Retail bring-up does not fork away from the homebrew-first architecture.

## Milestone 4: Hybrid Cloud Scale
Goal
- Scale the same deterministic contracts into AWS without changing the core run model.

Work items
- [x] Define the local/cloud artifact contract around `run-manifest.json`.
- [ ] Define a stable S3 key layout for content-addressed artifacts, manifests, and reports.
- [ ] Enforce S3 Versioning and Object Lock governance mode for manifests and validation reports.
- [ ] Model Step Functions Standard as the run control plane.
- [ ] Use Distributed Map only for explicit batch fan-out.
- [ ] Use AWS Batch `.sync` for heavy intake/lift/build/validation workers.
- [ ] Split workers into `cpu-linux`, `video-linux`, and `metal-macos`.
- [ ] Keep the macOS lane small and explicit because EC2 Mac remains a dedicated-host, 24-hour-minimum resource.
- [ ] Keep agents read-only for planning, triage, and summaries until stage contracts are stable.

Exit criteria
- The same run contract can execute locally or in AWS with immutable artifact lineage.

## Spec Reality Check
- SPEC-030 and SPEC-040 now have real implementation evidence, but they are still exploratory.
- SPEC-140 is partially implemented: entry shim, deterministic time/input, and explicit failure behavior exist; the first real libnx/Horizon service set does not.
- SPEC-180 is partially implemented: CLI and deterministic mock/private hooks exist; pinned external-tool compatibility does not.
- SPEC-190 exists as tooling, but it is not the first public milestone gate.
- SPEC-200 is scaffold-only.
- SPEC-250 is design work, not deployed infrastructure.
