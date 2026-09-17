#!/usr/bin/env python3
"""Build and package the experimental Linux workspace app."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import shutil
import subprocess
import tarfile
import tempfile
import tomllib


def run(*args):
    subprocess.run(args, check=True)


TARGETS = {
    "x86_64-unknown-linux-gnu": ("x86_64", 62, "strip"),
    "aarch64-unknown-linux-gnu": ("aarch64", 183, "aarch64-linux-gnu-strip"),
}


def validate_elf(path, expected_machine):
    with path.open("rb") as file:
        header = file.read(20)
    if len(header) < 20 or header[:4] != b"\x7fELF" or header[4] != 2:
        raise ValueError(f"{path} is not a 64-bit ELF executable")
    byteorder = {1: "little", 2: "big"}.get(header[5])
    if not byteorder:
        raise ValueError(f"{path} has an unknown ELF byte order")
    machine = int.from_bytes(header[18:20], byteorder)
    if machine != expected_machine:
        raise ValueError(
            f"{path} has ELF machine {machine}, expected {expected_machine}"
        )


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path("target/linux-dist"))
    parser.add_argument("--skip-build", action="store_true")
    parser.add_argument("--target", choices=TARGETS)
    args = parser.parse_args()
    if platform.system() != "Linux":
        parser.error("Linux is required to build this archive")
    target = args.target
    if target:
        architecture, elf_machine, strip_name = TARGETS[target]
    else:
        architecture = platform.machine()
        if architecture != "x86_64":
            parser.error("use --target aarch64-unknown-linux-gnu on ARM64")
        elf_machine, strip_name = 62, "strip"

    root = Path(__file__).resolve().parent.parent
    os.chdir(root)
    if not args.skip_build:
        command = [
            "cargo", "build", "--locked", "--release", "-j", "4",
            "-p", "gpuicn-showcase",
            "--features", "gpui_platform/x11,gpui_platform/wayland",
        ]
        if target:
            command += ["--target", target]
        run(*command)

    target_dir = Path(os.environ.get("CARGO_TARGET_DIR", "target"))
    if not target_dir.is_absolute():
        target_dir = root / target_dir
    source = (
        target_dir / target / "release/showcase"
        if target
        else target_dir / "release/showcase"
    )
    if not source.is_file():
        parser.error(f"missing executable: {source}")
    try:
        validate_elf(source, elf_machine)
    except ValueError as error:
        parser.error(str(error))
    strip = shutil.which(strip_name)
    if not strip:
        parser.error(f"{strip_name} is required")

    version = tomllib.loads((root / "site/Cargo.toml").read_text())["package"]["version"]
    revision = subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip()
    dirty = bool(subprocess.check_output(["git", "status", "--porcelain"], text=True).strip())
    if dirty:
        parser.error("commit source changes before packaging a release")
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    filename = f"gpuicn-workspace-linux-{architecture}.tar.gz"
    archive = output / filename

    with tempfile.TemporaryDirectory(prefix="gpuicn-linux-package-") as folder:
        bundle = Path(folder) / f"gpuicn-workspace-linux-{architecture}"
        bundle.mkdir()
        executable = bundle / "gpuicn-workspace"
        shutil.copy2(source, executable)
        run(strip, str(executable))
        shutil.copy2(root / "LICENSE", bundle / "LICENSE")
        shutil.copytree(root / "LICENSES", bundle / "LICENSES")
        runtime_note = (
            "The ARM64 runtime has not been tested.\n\n"
            if architecture == "aarch64"
            else ""
        )
        (bundle / "README.txt").write_text(
            "gpuicn Workspace for Linux (experimental)\n\n"
            "Run: ./gpuicn-workspace\n\n"
            f"This {architecture} build targets Ubuntu 24.04 and glibc 2.39. It "
            "includes X11 and Wayland support, but still needs wider desktop "
            f"testing.\n\n{runtime_note}"
            "Ubuntu runtime packages:\n"
            "  sudo apt install libxcb1 libxkbcommon0 libxkbcommon-x11-0 libfontconfig1 libwayland-client0 libwayland-cursor0 libvulkan1\n"
            "A working Vulkan driver is also required (for example mesa-vulkan-drivers).\n"
        )
        with tarfile.open(archive, "w:gz") as tar:
            tar.add(bundle, arcname=bundle.name)

    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    (output / f"{filename}.sha256").write_text(f"{digest}  {filename}\n")
    (output / f"showcase-linux-{architecture}.json").write_text(json.dumps({
        "url": f"/downloads/{filename}",
        "architecture": architecture,
        "bytes": archive.stat().st_size,
        "sha256": digest,
        "build": revision,
        "dirty": dirty,
        "experimental": True,
        "requires": f"Ubuntu 24.04 (glibc 2.39 or newer) {architecture} desktop",
        "version": version,
    }, indent=2) + "\n")
    print(f"Packaged {archive} ({revision[:12]}); experimental Linux build")


if __name__ == "__main__":
    main()
