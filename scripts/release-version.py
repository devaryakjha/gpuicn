#!/usr/bin/env python3
"""Check or bump every place the gpuicn release version is written.

    python3 scripts/release-version.py check        # CI: all references agree
    python3 scripts/release-version.py bump 0.8.0   # rewrite them, then update Cargo.lock
    python3 scripts/release-version.py current      # print the workspace version
"""
import argparse
from pathlib import Path
import re
import subprocess
import sys
import tomllib

ROOT = Path(__file__).resolve().parent.parent
V = r"(\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?)"

# (file, pattern): group 1 of each pattern is the version. Every pattern must
# match at least once, so a moved or reworded line fails loudly instead of
# silently shipping a stale version.
REFERENCES = [
    ("Cargo.toml", r'\[workspace\.package\][^\[]*?\nversion = "' + V + '"'),
    ("site/Cargo.toml", r'\[package\][^\[]*?\nversion = "' + V + '"'),
    ("README.md", r"cargo install gpuicn-cli --version " + V + " --locked"),
    ("README.md", r"gpuicn v" + V + " targets"),
    ("crates/ui/README.md", r"cargo install gpuicn-cli --version " + V + " --locked"),
    ("docs/registry.md", r"cargo install gpuicn-cli --version " + V + " --locked"),
    ("docs/registry.md", r"gpuicn v" + V + " supports"),
    ("docs/registry.md", r"devaryakjha/gpuicn/v" + V + "/"),
    ("web/src/lib/release.ts", r'export const release = "v' + V + '"'),
]


def current():
    data = tomllib.loads((ROOT / "Cargo.toml").read_text())
    return data["workspace"]["package"]["version"]


def check():
    version = current()
    problems = []
    for name, pattern in REFERENCES:
        text = (ROOT / name).read_text()
        found = [m.group(1) for m in re.finditer(pattern, text)]
        if not found:
            problems.append(f"{name}: no match for {pattern!r}")
        problems += [f"{name}: says {v}, expected {version}" for v in found if v != version]
    if not (ROOT / f"docs/releases/v{version}.md").is_file():
        problems.append(f"docs/releases/v{version}.md is missing")
    if not re.search(rf"^## {re.escape(version)}$", (ROOT / "CHANGELOG.md").read_text(), re.M):
        problems.append(f"CHANGELOG.md has no '## {version}' entry")
    if problems:
        print(f"Release version {version} is inconsistent:", *problems, sep="\n  ")
        print("Run: python3 scripts/release-version.py bump <version>")
        return 1
    print(f"All release references say {version}")
    return 0


def bump(version):
    if not re.fullmatch(V, version):
        sys.exit(f"not a version: {version}")
    for name in dict.fromkeys(name for name, _ in REFERENCES):
        path = ROOT / name
        text = path.read_text()
        for file, pattern in REFERENCES:
            if file == name:
                text = re.sub(pattern, lambda m: m.group(0).replace(m.group(1), version), text)
        path.write_text(text)
    subprocess.run(["cargo", "update", "--workspace"], cwd=ROOT, check=True)
    notes = ROOT / f"docs/releases/v{version}.md"
    if not notes.exists():
        notes.write_text(f"# gpuicn v{version}\n\nTODO: release notes.\n")
    print(f"Bumped to {version}. Still to do by hand:")
    print(f"  - write {notes.relative_to(ROOT)} and a '## {version}' CHANGELOG.md entry")
    print("  - cargo run -p gpuicn-cli --locked -- build   # regenerates registry URLs")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("check")
    sub.add_parser("current")
    sub.add_parser("bump").add_argument("version")
    args = parser.parse_args()
    if args.command == "current":
        print(current())
        return 0
    return check() if args.command == "check" else bump(args.version)


if __name__ == "__main__":
    sys.exit(main())
