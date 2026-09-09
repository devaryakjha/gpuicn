#!/usr/bin/env python3
"""Bundle the current native showcase and write its website download manifest."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import platform
import plistlib
import shutil
import subprocess
import tempfile
import tomllib


def run(*args):
    subprocess.run(args, check=True)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=Path("web/public/downloads"))
    parser.add_argument("--skip-build", action="store_true")
    args = parser.parse_args()
    if platform.system() != "Darwin":
        parser.error("macOS is required to build and validate this app bundle")
    root = Path(__file__).resolve().parent.parent
    os.chdir(root)
    version = tomllib.loads((root / "site/Cargo.toml").read_text())["package"]["version"]
    output = args.output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    if not args.skip_build:
        run("cargo", "build", "--locked", "--release", "-p", "gpuicn-showcase",
            "--features", "gpui_platform/runtime_shaders")
    architecture = platform.machine()
    if architecture not in ("arm64", "x86_64"):
        parser.error(f"unsupported architecture: {architecture}")
    revision = subprocess.check_output(["git", "rev-parse", "--short", "HEAD"], text=True).strip()
    dirty = bool(subprocess.check_output(["git", "status", "--porcelain"], text=True).strip())
    build = revision + ("-local" if dirty else "")
    identity = os.environ.get("SHOWCASE_SIGN_IDENTITY", "-")
    profile = os.environ.get("SHOWCASE_NOTARY_PROFILE")
    if profile and identity == "-":
        parser.error("notarization requires SHOWCASE_SIGN_IDENTITY")
    filename = f"gpuicn-workspace-macos-{architecture}.zip"
    archive = output / filename
    with tempfile.TemporaryDirectory(prefix="gpuicn-package-") as folder:
        app = Path(folder) / "gpuicn Workspace.app"
        contents = app / "Contents"
        (contents / "MacOS").mkdir(parents=True)
        (contents / "Resources").mkdir()
        shutil.copy2(root / "target/release/showcase", contents / "MacOS/gpuicn-workspace")
        iconset = Path(folder) / "Showcase.iconset"
        iconset.mkdir()
        for size in (16, 32, 128, 256, 512):
            for scale in (1, 2):
                suffix = "@2x" if scale == 2 else ""
                run("sips", "-z", str(size * scale), str(size * scale),
                    str(root / "web/public/brand/gpuicn-panels.png"), "--out",
                    str(iconset / f"icon_{size}x{size}{suffix}.png"))
        run("iconutil", "-c", "icns", str(iconset), "-o", str(contents / "Resources/Showcase.icns"))
        with (contents / "Info.plist").open("wb") as file:
            plistlib.dump({
                "CFBundleName": "gpuicn Workspace", "CFBundleDisplayName": "gpuicn Workspace",
                "CFBundleIdentifier": "com.imajha.gpuicn.workspace",
                "CFBundleExecutable": "gpuicn-workspace", "CFBundlePackageType": "APPL",
                "CFBundleShortVersionString": version.split("-")[0], "CFBundleVersion": "1",
                "CFBundleIconFile": "Showcase.icns", "NSHighResolutionCapable": True,
                "LSMinimumSystemVersion": "12.0", "NSPrincipalClass": "NSApplication",
                "GPUICNBuild": build,
                "GPUICNVersion": version,
            }, file)
        signing = ["codesign", "--force", "--sign", identity]
        if identity != "-":
            signing += ["--options", "runtime", "--timestamp"]
        run(*signing, str(app))
        run("codesign", "--verify", "--deep", "--strict", str(app))
        run("ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(app), str(archive))
        if profile:
            run("xcrun", "notarytool", "submit", str(archive), "--keychain-profile", profile, "--wait")
            run("xcrun", "stapler", "staple", str(app))
            run("xcrun", "stapler", "validate", str(app))
            run("spctl", "--assess", "--type", "execute", str(app))
            archive.unlink()
            run("ditto", "-c", "-k", "--sequesterRsrc", "--keepParent", str(app), str(archive))
        # Keep the same verified app available for local review.
        local = root / "target/showcase/gpuicn Workspace.app"
        local.parent.mkdir(parents=True, exist_ok=True)
        if local.exists():
            shutil.rmtree(local)
        shutil.copytree(app, local)
    digest = hashlib.sha256(archive.read_bytes()).hexdigest()
    (output / f"{filename}.sha256").write_text(f"{digest}  {filename}\n")
    (output / "showcase.json").write_text(json.dumps({
        "url": f"/downloads/{filename}", "architecture": architecture,
        "bytes": archive.stat().st_size, "sha256": digest, "build": build,
        "notarized": bool(profile), "version": version,
    }, indent=2) + "\n")
    print(f"Packaged {archive} ({build}); notarized: {bool(profile)}")


if __name__ == "__main__":
    main()
