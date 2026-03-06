# SPEC-140: Homebrew Runtime Surface

## Status
Draft v0.3

## Rationale
- Added a homebrew entry shim and loader-config builder in the runtime.
- Added deterministic time/input helpers and explicit failure behavior for unsupported calls.
- Confirmed that the Homebrew ABI key space is broader than the current runtime surface, so the first-title plan must pin a firmware baseline and implement only the minimum required subset.

## Purpose
Define the runtime ABI surface required to boot a recompiled homebrew title and satisfy the Switch homebrew ABI expectations.

## Goals
- Provide a minimal, deterministic runtime surface for homebrew startup.
- Map required loader configuration fields into the runtime environment.
- Establish a clear contract for unsupported services.

## Non-Goals
- Re-implementing the full Horizon OS service set.
- Supporting dynamically loaded NROs at runtime.

## Background
The Switch homebrew ABI defines how NRO entrypoints receive a loader configuration, which register arguments are passed at entry, and which loader config keys are expected at startup. The current public docs also include optional keys beyond the original minimum set, so the runtime contract must separate required keys from optional keys and pin a firmware baseline for the first public title.

## Requirements
- The runtime must provide an entrypoint shim that invokes the recompiled NRO entrypoint with:
  - X0 pointing to the loader configuration structure.
  - X1 set to `0xFFFF_FFFF_FFFF_FFFF` for NROs.
- The runtime must populate loader config entries for `EndOfList`, `MainThreadHandle`, and `AppletType` at a minimum.
- The runtime must name the pinned firmware baseline used for service and loader-config expectations.
- The runtime must surface optional loader config entries when present and fail with a clear error if a required field is missing.
- The runtime spec must explicitly track the broader documented key space, including `RandomSeed`, `UserIdStorage`, `HosVersion`, and `SyscallAvailableHint2`, even if the first public title does not require them.
- The runtime must provide a stable, deterministic time source and input event queue to minimize nondeterminism in validation.
- The runtime must document which Horizon OS services are stubbed and which are implemented, with a hard failure for unsupported calls.
- The first public title implementation target is a minimal libnx/Horizon subset, not a full Horizon surface.

## Interfaces and Data
- Runtime ABI struct definitions for loader config entries.
- A generated `runtime_manifest.json` describing:
  - Supported loader config keys.
  - Stubbed services and behavior.
  - Determinism knobs (time, input).

## Deliverables
- Entry shim implementation for the homebrew ABI.
- Loader config builder.
- Runtime service capability documentation.

## Open Questions
- Which HOS baseline should be the first pinned target for homebrew service work?
- Should libnx-facing behavior be expressed as a thin compatibility layer or by directly implementing a small Horizon subset?

## Acceptance Criteria
- A recompiled public homebrew binary boots and reaches its main loop with the runtime providing the required loader config keys for the pinned baseline.
- Unsupported services fail with an explicit, logged error that references the missing service.
- The runtime manifest enumerates which loader config keys were provided for a run.

## Risks
- Some homebrew titles may assume additional loader config keys not covered by the minimum set.
- Overly strict service stubs may block otherwise runnable titles.

## References
- https://switchbrew.org/wiki/Homebrew_ABI
- https://switchbrew.org/wiki/HID_services
