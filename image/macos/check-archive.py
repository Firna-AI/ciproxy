#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check the macos.tar writer and checker offline against the agent's rules."""

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


def members_folder(root, members):
    for member in members:
        path = root / member
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(member.encode() * 3)


class Archive(unittest.TestCase):
    def setUp(self):
        self.root = pathlib.Path(tempfile.mkdtemp())
        self.members = archive.agent_members()
        members_folder(self.root, self.members)

    def test_the_agent_list_has_the_bundle_and_the_sums(self):
        self.assertEqual(len(self.members), 8)
        self.assertIn("bundle/disk.img.zst", self.members)
        self.assertIn("bundle/aux.img", self.members)
        self.assertIn("macos-content.sha256", self.members)

    def test_packed_archive_passes_and_has_regular_members(self):
        path = self.root / "macos.tar"
        with contextlib.redirect_stdout(io.StringIO()):
            archive.pack(self.root, path, self.members)
            archive.check(path, self.members)
        with tarfile.open(path) as tar:
            self.assertEqual(tar.getnames(), self.members)
            self.assertTrue(all(member.isreg() for member in tar.getmembers()))

    def test_sparse_member_fails(self):
        path = self.root / "sparse.tar"
        with tarfile.open(path, "w", format=tarfile.PAX_FORMAT) as tar:
            for member in self.members:
                data = (self.root / member).read_bytes()
                info = tarfile.TarInfo(member)
                info.size = len(data)
                if member == "bundle/aux.img":
                    info.name = "bundle/GNUSparseFile.0/aux.img"
                    info.pax_headers = {"GNU.sparse.major": "1", "GNU.sparse.minor": "0",
                                        "GNU.sparse.name": member}
                tar.addfile(info, io.BytesIO(data))
        with self.assertRaises(SystemExit) as raised:
            archive.check(path, self.members)
        self.assertIn("GNU.sparse", str(raised.exception))

    def test_other_path_link_and_missing_member_fail(self):
        for case in ("extra", "link", "missing"):
            path = self.root / f"{case}.tar"
            with tarfile.open(path, "w", format=tarfile.PAX_FORMAT) as tar:
                for member in self.members[:-1] if case == "missing" else self.members:
                    data = (self.root / member).read_bytes()
                    info = tarfile.TarInfo(member)
                    info.size = len(data)
                    tar.addfile(info, io.BytesIO(data))
                if case == "extra":
                    tar.addfile(tarfile.TarInfo("._manifest.json"), io.BytesIO())
                if case == "link":
                    link = tarfile.TarInfo("bundle/link")
                    link.type, link.linkname = tarfile.SYMTYPE, "manifest.json"
                    tar.addfile(link)
            with self.subTest(case), self.assertRaises(SystemExit):
                archive.check(path, self.members)


if __name__ == "__main__":
    unittest.main()
