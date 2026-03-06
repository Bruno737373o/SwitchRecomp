# Research Directions and Current Findings

Last updated: 2026-03-05.

This document records the external findings that currently drive implementation priorities. It is no longer a generic wishlist; it is the working evidence base for the next milestones.

## Current Verified Repo State
- `nix develop --impure -c cargo test` passes.
- `recomp-cli run` emits the minimal sample, and the emitted crate now builds and runs from repo root.
- `recomp-cli automate --config samples/automation.toml` is implemented and exercised by tests with a real pipeline/build/run loop.
- Homebrew intake and lift work against the checked-in synthetic fixture.
- The first runnable milestone is public homebrew, not retail XCI. Retail title work remains scaffold-only in the repo.

## Confirmed Findings

### 1) Public Homebrew Is The Right First Executable Milestone
- The current best first target is Bernardo Giordano's Switch `2048` homebrew:
  - Makefile: https://raw.githubusercontent.com/BernardoGiordano/2048/master/Makefile
  - Main loop: https://raw.githubusercontent.com/BernardoGiordano/2048/master/source/main.cpp
- The current published Makefile links only `-lnx`, which keeps the dependency surface small.
- The main loop pattern is a good fit for the current pipeline direction: applet loop, pad polling, framebuffer presentation, and sdmc-backed save data.
- Conclusion: the first end-to-end title profile should target `2048`, not a private retail title.

### 2) Homebrew ABI Coverage Is Broader Than The Runtime Currently Models
- Switchbrew's Homebrew ABI page remains the primary reference:
  - https://switchbrew.org/wiki/Homebrew_ABI
- The loader configuration key space extends beyond the minimal keys already modeled. Current docs include keys `14` through `17`:
  - `RandomSeed`
  - `UserIdStorage`
  - `HosVersion`
  - `SyscallAvailableHint2`
- Conclusion: the runtime spec must pin a firmware baseline and distinguish:
  - keys required to boot the first public title
  - optional keys that must be surfaced or rejected explicitly

### 3) HID Service Details Drift Across Firmware Versions
- Switchbrew's HID service documentation is version-sensitive:
  - https://switchbrew.org/wiki/HID_services
- Command IDs and surface details vary across HOS revisions.
- Conclusion: the service spec and the first-title plan must name a pinned HOS baseline instead of treating HID command IDs as timeless.

### 4) Retail Intake Must Be Pinned To A Private Tool Compatibility Matrix
- `hactool` remains the clearest documented extractor surface for XCI/NCA/NSO work:
  - README: https://github.com/SciresM/hactool
  - Releases: https://github.com/SciresM/hactool/releases
- The latest tagged release is still `1.4.0` from 2020-06-02.
- Conclusion: real retail intake must be treated as a private-user workflow with pinned tool compatibility, not as a repo-level completed milestone.

### 5) Cloud Scale Should Be Built Around Deterministic Artifacts, Not Around Agents
- AWS references that materially affect the design:
  - Step Functions Distributed Map: https://docs.aws.amazon.com/step-functions/latest/dg/state-map-distributed.html
  - Step Functions to AWS Batch `.sync`: https://docs.aws.amazon.com/step-functions/latest/dg/connect-batch.html
  - S3 performance guidance: https://docs.aws.amazon.com/AmazonS3/latest/userguide/optimizing-performance.html
  - S3 Object Lock: https://docs.aws.amazon.com/AmazonS3/latest/userguide/object-lock.html#object-lock-overview
  - EC2 Mac instances: https://docs.aws.amazon.com/AWSEC2/latest/UserGuide/ec2-mac-instances.html
- S3 is a good fit for hash-addressed immutable artifacts.
- Step Functions Standard is the right control plane for stageful runs; Distributed Map should be reserved for explicit batch fan-out, not every single run.
- AWS Batch `.sync` is the clean fit for long-running worker stages.
- EC2 Mac remains a constrained validation lane because it still requires Dedicated Hosts with a 24-hour minimum allocation.
- Conclusion: default cloud workers should be Linux. macOS belongs only in a narrow parity lane when Metal-specific validation is required.

## Implementation Consequences
- Prioritize a runnable public-homebrew path over retail bring-up.
- Treat `run-manifest.json` as the stable local/cloud stage contract.
- Keep retail intake private-user only until external-tool compatibility is pinned.
- Keep agents out of artifact mutation until stage contracts are stable.

## Seed Resources Still In Use
- Jamulator write-up on static recompilation pitfalls and concurrency: https://andrewkelley.me/post/jamulator.html
- N64Recomp repository for pipeline patterns: https://github.com/N64Recomp/N64Recomp
- Dinosaur Planet recomp for asset-separation precedent: https://github.com/DinosaurPlanetRecomp/dino-recomp
- Tegra X1 whitepaper: https://www.nvidia.com/content/tegra/embedded-systems/pdf/tegra-x1-whitepaper.pdf
- Switch hardware overview: https://switchbrew.org/wiki/Hardware
- Switch NRO format: https://switchbrew.org/wiki/NRO
- Switch NSO format: https://switchbrew.org/wiki/NSO
- Switch NACP format: https://switchbrew.org/wiki/NACP
- Switch NCA format: https://switchbrew.org/wiki/NCA
- `hactoolnet`: https://github.com/Thealexbarney/hactoolnet
- `nstool`: https://github.com/jakcron/nstool
- FFmpeg docs: https://ffmpeg.org/ffmpeg.html
- FFmpeg filters: https://ffmpeg.org/ffmpeg-filters.html
- Ghidra SLEIGH reference: https://github.com/NationalSecurityAgency/ghidra/blob/master/GhidraDocs/languages/html/sleigh.html
- `sleigh` library: https://github.com/lifting-bits/sleigh
- Resurrecting Crimsonland (2026-02-01): https://banteg.xyz/posts/crimsonland/
- EBU R 128: https://tech.ebu.ch/publications/r128

## Open Research Questions
- Which minimal libnx/Horizon service subset is enough to run `2048` without title-specific hacks?
- What is the right deterministic homebrew validation gate: framebuffer hashes, save-state hashes, or both?
- Which GPU path is sufficient for the first public title without overcommitting to a full renderer?
- Which exact `hactool` and `hactoolnet` revisions should be pinned for private retail intake?
- What artifact schema and retention policy should be enforced first in the cloud path?
