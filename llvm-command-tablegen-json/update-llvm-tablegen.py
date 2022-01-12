#!/usr/bin/env python3
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

"""Update LLVM tablegen definitions in this crate."""

import argparse
import json
import os
import pathlib
import subprocess


def normalize_json(data: bytes) -> str:
    """Normalize llvm-tblgen JSON output."""
    data = json.loads(data)

    # !instanceof key just adds noise we don't care about. Remove it.
    del data["!instanceof"]

    # Sort keys for determinism. Indent for human readability and better
    # diffing.
    return json.dumps(data, sort_keys=True, indent=2) + "\n"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--tblgen",
        help="Name of `llvm-tablgen` executable to use",
        default="llvm-tblgen-13",
    )
    parser.add_argument(
        "llvm_source_path", help="Directory of unified LLVM source checkout"
    )
    parser.add_argument("output_path", help="Output directory to write files")

    args = parser.parse_args()

    llvm_dir = pathlib.Path(args.llvm_source_path)
    tblgen_exe = args.tblgen
    out_dir = pathlib.Path(args.output_path)

    out_dir.mkdir(parents=True, exist_ok=True)

    for root, dirs, files in os.walk(llvm_dir):
        root = pathlib.Path(root)
        root_rel = root.relative_to(llvm_dir)

        # Ignore unittests/ directories.
        if "unittests" in root_rel.parts:
            dirs.clear()
            continue

        # Make deterministic.
        dirs.sort()
        files.sort()

        for f in files:
            if f not in ("Options.td", "Opts.td", "DarwinLdOptions.td"):
                continue

            full = pathlib.Path(root) / f
            rel = full.relative_to(llvm_dir)
            includes = ["llvm/include"]

            if root_rel == pathlib.Path("clang/include/clang/Driver"):
                out_name = "clang"
            elif root_rel.parts == ("lld", "lib", "Driver"):
                out_name = "lld-darwin-ld"
            elif root_rel.parts[0] == "lld":
                out_name = "lld-%s" % root_rel.parts[1].lower()
            # We ignore lldb for now.
            elif root_rel.parts[0] == "lldb":
                continue
            elif root_rel.parts[0:3] == ("llvm", "lib", "ToolDrivers"):
                out_name = root_rel.parts[3]
            elif root_rel.parts[0:3] == ("llvm", "tools", "dsymutil"):
                out_name = "dsymutil"
            elif root_rel.parts[0:2] == ("llvm", "tools"):
                out_name = root_rel.parts[2]
            else:
                print("unhandled td file: %s" % rel)
                continue

            out_filename = "%s.json" % out_name

            dest_path = out_dir / out_filename

            args = [tblgen_exe, "--dump-json"]
            for include in includes:
                args.append("-I")
                args.append(include)

            args.append(str(rel))

            print("running %s for %s -> %s" % (tblgen_exe, full, dest_path))
            res = subprocess.run(args, cwd=llvm_dir, capture_output=True)

            if res.returncode != 0:
                print("error: " % res.stderr.decode("utf-8", "replace"))
                continue

            data = normalize_json(res.stdout)
            with dest_path.open("w", encoding="utf-8") as fh:
                fh.write(data)


if __name__ == "__main__":
    main()
