# CI

gpuicn follows the tiers of the Karya Labs engineering process
([karyalabs/matty `docs/engineering-process.md`](https://github.com/karyalabs/matty/blob/master/docs/engineering-process.md)):

| Tier | When | What |
| --- | --- | --- |
| T1 | every pull request | macOS fmt and clippy, the registry and version checks, the package list, the WASM check, the website build and lint |
| T2 | every push to main, in `release.yml` before it releases | T1 plus the macOS workspace tests and the MSRV tests on Linux |
| Canary | push to main, nightly, or the `canary` label | `canary.yml` builds Matty's master against this tree; a failure on main opens an issue |
| Release | merging a version bump | `verify.yml`, crates.io, the showcase builds, the GitHub release, the site, then Matty's pin bump PR |

Add the `tests` label, then push, to run T2 on a pull request. Setup steps
come from the shared actions in
[karyalabs/.github](https://github.com/karyalabs/.github/tree/main/actions);
the Rust toolchain is read from `release-pins.toml`.

## Runners

gpuicn's Linux and macOS jobs can run on Karya's self-hosted machines, the
same ones Matty uses: an Ubuntu box at OVH for Linux and a Mac mini for macOS
(Matty's `docs/self-hosted-runners.md` covers setting them up).

The organization variables `KARYA_LINUX_RUNNER` and `KARYA_MACOS_RUNNER`
decide where the CI jobs run, for every Karya Labs repository. Until they
are set, the repository variables below
(<https://github.com/karyalabs/gpuicn/settings/variables/actions>) do:

| Variable | Value for the self-hosted runners | Unset |
| --- | --- | --- |
| `GPUICN_LINUX_RUNNER` | `["self-hosted","Linux","X64","matty"]` | `ubuntu-latest` |
| `GPUICN_MACOS_RUNNER` | `["self-hosted","macOS","ARM64","matty"]` | `macos-15` |

The value is a `runs-on` value written as JSON. Deleting a variable sends the
jobs back to GitHub's runners at once: do that when a machine is down, since a
job waits up to a day for an offline runner. Pull requests from forks always
get GitHub's runners, so their code never runs on these machines.

The runners must be available to this repository: register them at the
karyalabs organization level (Settings → Actions → Runners) in a runner group
that includes gpuicn, rather than on one repository.

What runs where once both are set:

| Workflow | Job | Runner |
| --- | --- | --- |
| CI | `rust` | Mac mini |
| CI | `linux`, `wasm` | OVH box |
| Verify release | `registry-install` | Mac mini |
| Verify release | `new-apps` | OVH box |
| Verify release | `wasm-size` | GitHub's Ubuntu 24.04 (Playwright needs that image) |
| Build release downloads | all | GitHub's runners (signing, and an arm64 Linux build) |

On a self-hosted runner the checkout keeps `target/` so builds stay warm, and
`scripts/ci/prepare-workspace` clears everything else a previous job left.
`scripts/ci/apt-install` installs GPUI's Linux libraries only when one is
missing, so the box needs them preinstalled (it has them for Matty).
