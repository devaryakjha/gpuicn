# CI runners

gpuicn's Linux and macOS jobs can run on Karya's self-hosted machines, the
same ones Matty uses: an Ubuntu box at OVH for Linux and a Mac mini for macOS
(Matty's `docs/self-hosted-runners.md` covers setting them up).

Two repository variables decide where the jobs run. Set them at
<https://github.com/karyalabs/gpuicn/settings/variables/actions>:

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
