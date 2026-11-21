#!/usr/bin/env python3
"""Build a .deb from release files using only Python's standard library.

cargo-deb would need an extra install (and network) in every build
environment; a Debian binary package is just an ar archive of three tarballs,
so we can emit it directly. Paths inside data.tar.gz are root-relative.
"""

import argparse
import gzip
import io
import pathlib
import re
import subprocess
import tarfile


def tar_add(tar, source, name, mode=0o644):
    info = tarfile.TarInfo(name)
    info.size = source.stat().st_size
    info.mode = mode
    info.mtime = 0
    info.uid = info.gid = 0
    info.uname = info.gname = "root"
    with source.open("rb") as handle:
        tar.addfile(info, handle)


def gzip_tar(entries):
    """entries: list of (path, archive name, mode)."""
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w") as tar:
        for source, name, mode in entries:
            tar_add(tar, source, name, mode)
    return gzip.compress(raw.getvalue(), mtime=0)


def ar_archive(members):
    """members: list of (name, bytes) in dpkg order."""
    out = bytearray(b"!<arch>\n")
    for name, data in members:
        header = (
            f"{name + '/':<16}"
            f"{'0':<12}"
            f"{'0':<6}"
            f"{'0':<6}"
            f"{'100644':<8}"
            f"{len(data):<10}"
            "`\n"
        ).encode("ascii")
        assert len(header) == 60, len(header)
        out += header + data
        if len(data) % 2:
            out += b"\n"
    return bytes(out)


def glibc_requirement(binary):
    """Pin the minimum glibc the binary actually needs (with `objdump`)."""
    try:
        symbols = subprocess.check_output(["objdump", "-T", str(binary)], text=True)
    except (OSError, subprocess.CalledProcessError):
        return None
    versions = set(re.findall(r"GLIBC_(\d+\.\d+)", symbols))
    if not versions:
        return None
    newest = max(versions, key=lambda v: tuple(map(int, v.split("."))))
    return f"libc6 (>= {newest})"


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--version", required=True)
    parser.add_argument("--binary", type=pathlib.Path, required=True)
    parser.add_argument("--icon", type=pathlib.Path, required=True)
    parser.add_argument("--desktop", type=pathlib.Path, required=True)
    parser.add_argument("--out", type=pathlib.Path, required=True)
    args = parser.parse_args()

    depends = [
        glibc_requirement(args.binary),
        "libgcc-s1",
        "libgtk-3-0",     # file dialogs (rfd/gtk3) and GTK accessibles
        "libxkbcommon0",
        "libwayland-client0",
        "libxcb1",
    ]
    depends = [d for d in depends if d]

    control = (
        "Package: pixesh\n"
        f"Version: {args.version}\n"
        "Section: graphics\n"
        "Priority: optional\n"
        "Architecture: amd64\n"
        f"Depends: {', '.join(depends)}\n"
        "Maintainer: EnotCoder <https://github.com/EnotCoder>\n"
        "Homepage: https://github.com/EnotCoder/pixesh\n"
        "Description: minimal pixel art editor\n"
        " Pixesh is a small pixel art editor built with Rust and egui.\n"
        " It supports layers, animation, effects and PNG export.\n"
    ).encode("utf-8")

    data = gzip_tar(
        [
            (args.binary, "usr/bin/pixesh", 0o755),
            (args.icon, "usr/share/icons/hicolor/256x256/apps/pixesh.png", 0o644),
            (args.desktop, "usr/share/applications/pixesh.desktop", 0o644),
        ]
    )
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w") as tar:
        info = tarfile.TarInfo("control")
        info.size = len(control)
        info.mtime = 0
        info.mode = 0o644
        info.uid = info.gid = 0
        tar.addfile(info, io.BytesIO(control))
    control_tar = gzip.compress(raw.getvalue(), mtime=0)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(
        ar_archive(
            [
                ("debian-binary", b"2.0\n"),
                ("control.tar.gz", control_tar),
                ("data.tar.gz", data),
            ]
        )
    )
    print(f"wrote {args.out} ({args.out.stat().st_size} bytes)")


if __name__ == "__main__":
    main()
