# SPEC-200: title-a24b9e807b456252 First-Level Milestone (macOS/aarch64)

## Status
Scaffold v0.3

## Rationale
- Added non-proprietary scaffolding for title config, patch placeholders, and run instructions.
- The repo does not currently contain a runnable first-level implementation for this title.
- This spec now records the private retail scaffold and prerequisites instead of implying that the milestone is already on track or complete.

## Purpose
Define the private retail scaffold for title-a24b9e807b456252 on macOS/aarch64. This spec does not claim that the title is currently runnable; it records the requirements that must be met after the public homebrew path is proven.

## Goals
- Keep the title-specific config, provenance, and validation placeholders organized.
- Define the private-user prerequisites for later retail bring-up.
- Reuse the shared pipeline/runtime contracts proven by the public homebrew milestone.
- Keep all proprietary assets external and user-supplied.

## Non-Goals
- Completing or validating the full game.
- Achieving exact performance parity with Switch hardware.
- Distributing assets, keys, or copyrighted footage.

## Background
- A retail XCI and a long-form gameplay reference video are available locally.
- The project already defines pipeline, runtime, and validation scaffolding for homebrew.
- The current repository does not yet provide the minimal service, graphics, and validation surface required to run this title.

## Requirements
- Intake must extract Program NCA, ExeFS, and NSO segments from the XCI using user keys.
- Recompiled output must build on macOS/aarch64 and link against the runtime ABI.
- Runtime must implement enough OS services, GPU translation, audio, and input to reach the first level.
- RomFS assets must be loaded from an external, user-managed path.
- Validation must compare the first level segment against the reference video and record results.
- Validation artifacts must be tracked externally (artifact index + report output paths).
- This title must not be promoted beyond scaffold status until it uses the same lifted-module, CPU-state, service, and validation contracts proven by the public homebrew path.

## Operator Inputs
- title-a24b9e807b456252 validation depends on external reference and capture artifacts.
- Absolute paths and timeline confirmations are tracked in `docs/title-a24b9e807b456252-validation-prereqs.md`.

## Interfaces and Data
- `title.toml` for title-a24b9e807b456252 configuration (stubbed services, patches, asset paths).
- `provenance.toml` for XCI and reference video inputs.
- `validation-report.json` for the first-level comparison results.
- `artifacts.json` (external) for linking intake manifests, captures, and reports.
- `docs/title-a24b9e807b456252-first-level.md` for the scaffolding walkthrough.

## Deliverables
- Title-specific configuration and patch placeholders.
- A private-user runbook that documents required external inputs.
- A validation prerequisites checklist and artifact-index template.

## Scaffolding
- `samples/title-a24b9e807b456252/title.toml` with external asset and key paths.
- `samples/title-a24b9e807b456252/provenance.toml` with placeholder inputs and metadata.
- `samples/title-a24b9e807b456252/patches/patches.toml` with patch placeholders.
- `samples/title-a24b9e807b456252/module.json` placeholder for lifted output wiring.

## Open Questions
- Which services are the minimum set required to reach the first level?
- What frame rate and resolution should be treated as the baseline for comparison?
- Which shader or GPU features are required for the first level?

## Acceptance Criteria
- The title scaffold clearly documents the external prerequisites, private inputs, and unresolved implementation gaps.
- No proprietary assets are committed to the repo or build outputs.
- The spec can be advanced only after the title runs on the shared post-homebrew core.

## Risks
- GPU translation gaps may block rendering.
- Missing OS services may cause boot failure.
- Validation thresholds may need tuning for frame pacing variance.

## References
- `docs/title-a24b9e807b456252-first-level.md`
- `docs/xci-intake.md`
- `docs/validation-artifacts.md`
- `docs/validation-traces.md`
- `docs/validation-video.md`
