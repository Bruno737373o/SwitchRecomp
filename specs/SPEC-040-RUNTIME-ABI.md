# SPEC-040: Runtime and ABI Contract

## Status
Draft v0.5

## Rationale
- Added a minimal runtime ABI stub crate with versioning for early linkage tests.
- Added runtime-facing scaffolds for services, timing, and graphics.
- Added shared `CpuState` register, flag, and PC/SP handling across emitted functions.

## Purpose
Define the runtime library and ABI contract between recompiled code and host platform.

## Goals
- Provide a stable ABI for recompiled code.
- Encapsulate platform services and hardware behaviors in the runtime.

## Non-Goals
- Full hardware-level emulation.
- Exposing a public SDK for game development.

## ABI Requirements
- Shared register and flag state representation (`CpuState`).
- Memory model, alignment, and endianness assumptions.
- Atomic and synchronization primitives.

## Runtime Services
- Memory allocation and mapping.
- System services and IPC stubs.
- Graphics and audio dispatch layers.
- Timing, interrupts, and event scheduling.

## Deliverables
- A runtime library with a stable ABI.
- ABI documentation and versioning rules.

## Open Questions
- Which services must be implemented for a first milestone?
- How should ABI changes be versioned and validated?

## Acceptance Criteria
- A unit test that links and executes a minimal recompiled program using shared CPU state.
- A defined ABI version embedded in produced outputs.

## References
- https://andrewkelley.me/post/jamulator.html
