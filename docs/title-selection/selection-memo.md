# Title Selection Memo

## Selected Title (Provisional)
- Bernardo Giordano's `2048` homebrew.

## Rationale
- Preservation-safe profile with open-source code and minimal assets.
- Exercises core input, simple presentation, applet lifecycle, and sdmc-backed save data without a retail-scale service surface.
- Small binary size enables fast iteration and validation.
- The current published build links only `-lnx`, which keeps the early runtime surface tractable.

## Checklist Coverage
- Legal/preservation rationale: open-source codebase; requires license verification and asset redistribution confirmation.
- Minimal service dependency map: `sm:`, `sm:m`, applet loop, `hid`, `vi`/framebuffer-related `nv`, and filesystem access for save/settings.
- Instruction coverage estimate: primarily integer + branch operations, modest SIMD usage.
- GPU feature usage: basic 2D/texture rendering, minimal shader complexity.
- Asset separation plan: assets are either embedded as open-source data or supplied separately by the user.

## Follow-ups
- Verify candidate license and asset redistribution terms.
- Capture a concrete service map and framebuffer path for the current public source build.
- Convert this memo into a checked-in `2048` title profile once the minimal homebrew runtime surface is ready.
