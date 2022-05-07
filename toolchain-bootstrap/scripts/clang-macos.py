#!/usr/bin/env python3

# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at https://mozilla.org/MPL/2.0/.

import gzip
import hashlib
import http
import multiprocessing
import os
import pathlib
import platform
import shutil
import stat
import subprocess
import sys
import tarfile
import tempfile
import urllib.request
import zstandard

HERE = pathlib.Path(os.path.abspath(os.path.dirname(__file__)))
ROOT = HERE.parent

# 2021-01-01T00:00:00
DEFAULT_MTIME = 1609488000

COMPRESSION_LEVEL = 18

DOWNLOADS = [
    {
        "name": "cmake",
        "url": "https://github.com/Kitware/CMake/releases/download/v3.23.1/cmake-3.23.1-macos-universal.tar.gz",
        "sha256": "f794ed92ccb4e9b6619a77328f313497d7decf8fb7e047ba35a348b838e0e1e2",
        "version": "3.23.1",
    },
    {
        "name": "ninja",
        "url": "https://github.com/ninja-build/ninja/releases/download/v1.10.2/ninja-mac.zip",
        "sha256": "6fa359f491fac7e5185273c6421a000eea6a2f0febf0ac03ac900bd4d80ed2a5",
        "version": "1.10.2",
    },
    {
        "name": "llvm",
        "url": "https://github.com/llvm/llvm-project/releases/download/llvmorg-14.0.3/llvm-project-14.0.3.src.tar.xz",
        "sha256": "44d3e7a784d5cf805e72853bb03f218bd1058d448c03ca883dabbebc99204e0c",
        "version": "14.0.3",
    }
]

SCCACHE_AARCH64 = {
    "name": "sccache",
    "url": "https://github.com/mozilla/sccache/releases/download/v0.3.0/sccache-v0.3.0-aarch64-apple-darwin.tar.gz",
    "sha256": "65d0a04fac51eaeeadd72d3f7eee3fdc27409aaf23b97945ea537e92bd0b0f0d",
    "version": "0.3.0",
}

SCCACHE_X86_64 = {
    "name": "sccache",
    "url": "https://github.com/mozilla/sccache/releases/download/v0.3.0/sccache-v0.3.0-x86_64-apple-darwin.tar.gz",
    "sha256": "61c16fd36e32cdc923b66e4f95cb367494702f60f6d90659af1af84c3efb11eb",
    "version": "0.3.0",
}

if platform.mac_ver()[2] == "arm64":
    ARCH = "aarch64"
    DOWNLOADS.append(SCCACHE_AARCH64)
else:
    ARCH = "x86_64"
    DOWNLOADS.append(SCCACHE_X86_64)


def hash_path(p: pathlib.Path):
    h = hashlib.sha256()

    with p.open("rb") as fh:
        while True:
            chunk = fh.read(65536)
            if not chunk:
                break

            h.update(chunk)

    return h.hexdigest()


class IntegrityError(Exception):
    """Represents an integrity error when downloading a URL."""


def secure_download_stream(url, sha256):
    """Securely download a URL to a stream of chunks.

    If the integrity of the download fails, an IntegrityError is
    raised.
    """
    h = hashlib.sha256()

    with urllib.request.urlopen(url) as fh:
        if not url.endswith(".gz") and fh.info().get("Content-Encoding") == "gzip":
            fh = gzip.GzipFile(fileobj=fh)

        while True:
            chunk = fh.read(65536)
            if not chunk:
                break

            h.update(chunk)

            yield chunk

    digest = h.hexdigest()

    if digest != sha256:
        raise IntegrityError(
            "integrity mismatch on %s: wanted sha256=%s; got sha256=%s"
            % (url, sha256, digest))


def download_to_path(url: str, path: pathlib.Path, sha256: str):
    """Download a URL to a filesystem path, possibly with verification."""

    # We download to a temporary file and rename at the end so there's
    # no chance of the final file being partially written or containing
    # bad data.
    print("downloading %s to %s" % (url, path))

    if path.exists():
        good = True

        if good:
            if hash_path(path) != sha256:
                print("existing file hash is wrong; removing")
                good = False

        if good:
            print("%s exists and passes integrity checks" % path)
            return

        path.unlink()

    tmp = path.with_name("%s.tmp" % path.name)

    for _ in range(5):
        try:
            try:
                with tmp.open("wb") as fh:
                    for chunk in secure_download_stream(url, sha256):
                        fh.write(chunk)

                break
            except IntegrityError:
                tmp.unlink()
                raise
        except http.client.HTTPException as e:
            print("HTTP exception; retrying: %s" % e)
        except urllib.error.URLError as e:
            print("urllib error; retrying: %s" % e)
    else:
        raise Exception("download failed after multiple retries")

    tmp.rename(path)
    print("successfully downloaded %s" % url)


def create_tar_from_directory(fh, base_path: pathlib.Path, path_prefix=None):
    def normalize_tarinfo(ti):
        ti.pax_headers = {}
        ti.mtime = DEFAULT_MTIME
        ti.uid = 0
        ti.uname = "root"
        ti.gid = 0
        ti.gname = "root"

        # Give user/group read/write on all entries.
        ti.mode |= stat.S_IRUSR | stat.S_IWUSR | stat.S_IRGRP | stat.S_IWGRP

        # If user executable, give to group as well.
        if ti.mode & stat.S_IXUSR:
            ti.mode |= stat.S_IXGRP

        return ti

    with tarfile.open(name="", mode="w", fileobj=fh) as tf:
        for root, dirs, files in os.walk(base_path):
            dirs.sort()

            for d in dirs:
                full = base_path / root / d
                rel = full.relative_to(base_path)
                if path_prefix:
                    rel = pathlib.Path(path_prefix) / rel

                ti = normalize_tarinfo(tf.gettarinfo(full, rel))
                tf.addfile(ti)

            for f in sorted(files):
                full = base_path / root / f
                rel = full.relative_to(base_path)
                if path_prefix:
                    rel = pathlib.Path(path_prefix) / rel

                ti = normalize_tarinfo(tf.gettarinfo(full, rel))

                if ti.isreg():
                    with full.open("rb") as fh:
                        tf.addfile(ti, fh)
                else:
                    tf.addfile(ti)


def main():
    build_path = pathlib.Path(os.path.abspath(sys.argv[1]))

    downloaded_paths = []

    for entry in DOWNLOADS:
        filename = entry["url"].rsplit("/")[-1]
        dest = build_path / filename

        download_to_path(entry["url"], dest, entry["sha256"])
        downloaded_paths.append(dest)

    with tempfile.TemporaryDirectory(prefix="toolchain-bootstrap-") as td:
        temp_dir = pathlib.Path(td)

        for path in downloaded_paths:
            shutil.copy(path, temp_dir / path.name)

        shutil.copy(ROOT / "scripts" / "clang-macos.sh", temp_dir / "clang-macos.sh")

        env = dict(os.environ)
        for entry in DOWNLOADS:
            env["%s_VERSION" % entry["name"].upper()] = entry["version"]

        cpu_count = multiprocessing.cpu_count()
        env["NUM_CPUS"] = "%d" % cpu_count
        env["NUM_JOBS_AGGRESSIVE"] = "%d" % max(cpu_count + 2, cpu_count * 2)
        env["MACOSX_DEPLOYMENT_TARGET"] = "11.0"

        subprocess.run([str(temp_dir / "clang-macos.sh")], cwd=temp_dir, env=env, check=True)

        dest_path = build_path / ("llvm-%s-apple-darwin.tar.zst" % ARCH)
        print("writing %s" % dest_path)

        cctx = zstandard.ZstdCompressor(level=COMPRESSION_LEVEL)

        with zstandard.open(dest_path, "wb", cctx=cctx) as fh:
            create_tar_from_directory(fh, temp_dir / "out" / "toolchain", "llvm")


if __name__ == "__main__":
    main()
