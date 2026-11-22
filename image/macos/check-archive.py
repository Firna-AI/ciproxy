#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the macos.tar writer and checker offline against the agent's rules.

Needs zstd. The large-member test writes a sparse archive of about 9 GiB
logical size, which takes almost no disk space.
"""

import contextlib
import importlib.util
import io
import pathlib
import tarfile
import tempfile
import unittest

HERE = pathlib.Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("macos_archive", HERE / "macos_archive.py")
archive = importlib.util.module_from_spec(spec)
spec.loader.exec_module(archive)
DISK_BYTES = 64 * 1024 * 1024


def members_folder(root, members):
    for member in members:
        if member in (archive.DISK_ARCHIVE, archive.CONTENT_SUMS):
            continue
        path = root / member
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(member.encode() * 3)
    with open(root / archive.DISK, "wb") as disk:
        disk.truncate(DISK_BYTES)
        disk.seek(5 * 1024 * 1024)
        disk.write(b"guest data" * 1000)


def plain_archive(root, path, members, extra=None):
    with tarfile.open(path, "w", format=tarfile.PAX_FORMAT) as tar:
        for member in members:
            data = member.encode()
            info = tarfile.TarInfo(member)
            info.size = len(data)
            if extra:
                extra(info)
            tar.addfile(info, io.BytesIO(data))
    return path


class Archive(unittest.TestCase):
    def setUp(self):
        self.root = pathlib.Path(tempfile.mkdtemp())
        self.members = archive.agent_members()
        members_folder(self.root, self.members)

    def quiet(self, function, *args, **options):
        with contextlib.redirect_stdout(io.StringIO()):
            return function(*args, **options)

    def test_the_agent_list_has_the_bundle_and_the_sums(self):
        self.assertEqual(len(self.members), 8)
        self.assertEqual(self.members[0], archive.DISK_ARCHIVE)
        self.assertEqual(self.members[-1], archive.CONTENT_SUMS)

    def test_packed_archive_streams_the_disk_and_writes_the_sums(self):
        path = self.root / "macos.tar"
        self.quiet(archive.pack, self.root, path, self.members, 3)
        self.quiet(archive.check, path, self.members, verify=True)
        with tarfile.open(path) as tar:
            self.assertEqual(tar.getnames(), self.members)
            self.assertTrue(all(member.isreg() for member in tar.getmembers()))
            sums = tar.extractfile(archive.CONTENT_SUMS).read().decode()
        names = [line[66:] for line in sums.splitlines()]
        self.assertEqual(sorted(names), sorted([archive.DISK, *self.members[:-1]]))
        self.assertIn(f"{archive.sha256(self.root / archive.DISK)}  {archive.DISK}", sums)

    def test_a_changed_disk_member_fails_the_verify(self):
        path = self.root / "macos.tar"
        self.quiet(archive.pack, self.root, path, self.members, 3)
        sums = self.root / archive.CONTENT_SUMS
        data = path.read_bytes().replace(sums.read_bytes()[:64], b"0" * 64)
        path.write_bytes(data)
        with self.assertRaises(SystemExit):
            self.quiet(archive.check, path, self.members, verify=True)

    def test_member_over_8_gib_uses_the_pax_size(self):
        path = self.root / "large.tar"
        large = 9 * 1024**3 + 7
        with open(path, "wb") as out:
            for member in self.members:
                size = large if member == archive.DISK_ARCHIVE else 3
                info = archive.member_info(member, size, 0)
                out.write(info.tobuf(tarfile.PAX_FORMAT, "utf-8", "surrogateescape"))
                if size == large:
                    out.seek(-(-size // 512) * 512, 1)
                else:
                    out.write(b"abc".ljust(512, b"\0"))
            out.write(bytes(1024))
        self.quiet(archive.check, path, self.members)
        sizes = {name: size for name, _, _, size in archive.headers(path)}
        self.assertEqual(sizes[archive.DISK_ARCHIVE], large)

    def test_sparse_member_fails(self):
        def sparse(info):
            if info.name == "bundle/aux.img":
                info.name = "bundle/GNUSparseFile.0/aux.img"
                info.pax_headers = {"GNU.sparse.major": "1", "GNU.sparse.minor": "0",
                                    "GNU.sparse.name": "bundle/aux.img"}
        path = plain_archive(self.root, self.root / "sparse.tar", self.members, sparse)
        with self.assertRaises(SystemExit) as raised:
            archive.check(path, self.members)
        self.assertIn("GNU.sparse", str(raised.exception))

    def test_other_path_link_and_missing_member_fail(self):
        cases = {"extra": self.members + ["._manifest.json"], "missing": self.members[:-1]}
        for case, members in cases.items():
            path = plain_archive(self.root, self.root / f"{case}.tar", members)
            with self.subTest(case), self.assertRaises(SystemExit):
                archive.check(path, self.members)
        path = plain_archive(self.root, self.root / "link.tar", self.members)
        with tarfile.open(path, "a") as tar:
            link = tarfile.TarInfo("bundle/link")
            link.type, link.linkname = tarfile.SYMTYPE, "manifest.json"
            tar.addfile(link)
        with self.subTest("link"), self.assertRaises(SystemExit):
            archive.check(path, self.members)


if __name__ == "__main__":
    unittest.main()
