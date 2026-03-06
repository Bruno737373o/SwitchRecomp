# SwitchRecomp

This repository contains a preservation-focused Nintendo Switch static recompilation research project. The docs define scope, requirements, and legal boundaries, and the Rust workspace now implements a verified local operator path for intake, lift, translation, emitted-crate build, and automation without storing proprietary assets.

## Contents
- `specs/` contains the numbered specification series.
- `crates/` holds the current exploratory pipeline, runtime, validation, and CLI tooling.
- `skills/` provides the Codex skill set used for validation workflows.
- `ROADMAP.md` provides phased milestones and exit criteria.
- `RESEARCH.md` records current external findings and source links.
- `docs/` contains development notes.
- `docs/LEGAL-POLICY.md` defines legal use and asset separation rules.
- `docs/static-recomp-skills.md` documents the Codex skill set and project-level validation templates.

## How to Use the Specs
- Read `specs/README.md` for ordering.
- Each spec starts as Draft and should be updated as research is completed.
- Open questions are explicitly listed at the end of each spec.

## Contribution Notes
- Do not add or link to proprietary assets, keys, or copyrighted binaries.
- Keep inputs and outputs strictly separated.
- When updating a spec, add a short rationale and keep acceptance criteria testable.

Legal and provenance policy:
- Follow `docs/LEGAL-POLICY.md` for legal acquisition, asset separation, and prohibited content.
- All builds require a validated `provenance.toml` describing lawful inputs.

## Development
- The dev environment is managed with Nix + devenv.
- See `docs/DEVELOPMENT.md` for commands and sample usage.
- Verified local smoke path as of 2026-03-05:
  - `cargo run -p recomp-cli -- run --module samples/minimal/module.json --config samples/minimal/title.toml --provenance samples/minimal/provenance.toml --out-dir out/minimal`
  - `cargo build --manifest-path out/minimal/Cargo.toml`
  - `cargo run --manifest-path out/minimal/Cargo.toml`
- Verified homebrew sample path as of 2026-03-05:
  - `cargo run -p recomp-cli -- homebrew-intake --module samples/homebrew-intake/inputs/homebrew.nro --nso samples/homebrew-intake/inputs/overlay.nso --provenance samples/homebrew-intake/provenance.toml --out-dir out/homebrew-intake`
  - `cargo run -p recomp-cli -- homebrew-lift --module-json out/homebrew-intake/module.json --out-dir out/homebrew-lift --mode stub`
- Verified automation entry point as of 2026-03-05:
  - `cargo run -p recomp-cli -- automate --config samples/automation.toml`

## Samples and Flow Docs
- `samples/memory-image/` shows the memory image initialization flow (segment blob + lifted module).
- `samples/validation/` contains non-proprietary validation artifact index templates.
- `docs/static-recompilation-flow.md` outlines a hypothetical macOS static recompilation flow and verification pipeline (see the Real XCI intake section for external-tool usage).
- `docs/xci-intake.md` documents the XCI intake workflow and mock extractor format.
- `docs/validation-artifacts.md` defines validation artifact indexing, workflows, and dependencies.
- `docs/validation-video.md` describes the hash-based video validation workflow.
- `docs/title-hash-ingest.md` documents hash-based title/data ingest and local decoder-ring files.
- `scripts/capture-validation.sh`, `scripts/capture-video-macos.sh`, `scripts/capture_video.sh`, and `scripts/validate_artifacts.sh` provide capture and validation helpers.
- `scripts/ingest_hashed_title.sh` hashes title/data paths for private local workspaces.
- `scripts/rewrite_title_refs_in_history.sh` rewrites commit-message title references to hash ids.

## Back Pressure Hooks
These hooks add fast, consistent feedback to keep the repo autonomous and reduce review churn. Hooks are defined in `.pre-commit-config.yaml` and can be run with `prek` (preferred) or `pre-commit`.

- Install hooks: `prek install` or `pre-commit install`.
- Run on demand: `prek run --all-files` or `pre-commit run --all-files`.
- macOS note: the Nix dev shell ships `prek` only (to avoid Swift/.NET builds); install `pre-commit` separately if you need it.

Configured hooks:
- Pre-commit: `trailing-whitespace`, `end-of-file-fixer`, `check-merge-conflict`, `check-yaml`, `check-toml`, `check-json`, `check-added-large-files`, `detect-private-key`, `check-executables-have-shebangs`, `check-symlinks`, `check-case-conflict`, `cargo fmt --check`.
- Pre-push: `cargo clippy --workspace --all-targets --all-features -D warnings`, `cargo test --workspace`.

## Specs Update Log
- 2026-01-29: v0.2 pass started; added hardware baseline details, input/binary format research, OS/services surface notes, and timing/interrupts refinements across specs.
- 2026-01-29: Added `SPEC-TEMPLATE.md` and expanded `RESEARCH.md` with seed sources.
- 2026-01-29: Added exploratory pipeline/runtime crates, a minimal sample, and dev environment scaffolding.
- 2026-01-29: Added ISA, services, graphics, and timing scaffolds plus build manifest output.
- 2026-03-05: Verified repo-root emitted-crate build/run, added `recomp-cli automate`, extended `title.toml` parsing, preserved lifted segment metadata, and moved the documented first milestone to public homebrew before retail.

## Status
Exploratory implementation with a working local operator path. The first executable milestone is a public homebrew title; retail title work and cloud orchestration remain in scaffold/planning stages.
