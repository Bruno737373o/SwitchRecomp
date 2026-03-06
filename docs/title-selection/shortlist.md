# Title Shortlist

This shortlist prioritizes preservation-safe, open-source homebrew titles with redistributable assets and minimal service dependencies. Each candidate requires license verification before selection.

## Candidate A: Bernardo Giordano's `2048`
Pros:
- Small codebase and minimal assets.
- Limited service usage relative to a retail title.
- Exercises the exact surfaces the current plan needs first: applet loop, pad input, simple presentation, and sdmc-backed save data.
- Good fit for a public first milestone.

Cons:
- Does not exercise the full audio or network surface.
- Still requires enough `vi`/`nv` and filesystem work to avoid fake success.

Service dependencies:
- `sm:`, `sm:m`, applet lifecycle, `hid`, `vi` plus framebuffer-related `nv`, and sdmc-backed filesystem.

Instruction coverage estimate:
- Integer, branch, and memory heavy; minimal SIMD usage expected.

## Candidate B: Snake-style Homebrew (e.g., SnakeNX)
Pros:
- Simple gameplay loop with consistent input + timing.
- Modest graphics requirements.

Cons:
- Minimal OS service coverage.
- Often limited to 2D rendering path.

Service dependencies:
- `hid`, `vi`/`nv`, optional `applet`.

Instruction coverage estimate:
- Integer + branch; small memory footprint.

## Candidate C: Libnx Graphics Demo (triangle/texture sample)
Pros:
- Explicit GPU usage for early graphics validation.
- Small asset footprint.

Cons:
- Not a full game loop; limited gameplay or input coverage.

Service dependencies:
- `vi`/`nv`, minimal `applet`.

Instruction coverage estimate:
- Graphics setup + minimal logic; low CPU instruction variety.
