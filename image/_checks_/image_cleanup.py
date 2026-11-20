# SPDX-License-Identifier: Apache-2.0
"""Keep upstream image files when ciproxy build files are removed."""

import pathlib
import subprocess
import tempfile


UPSTREAM = {
    "imagegeneration/imagedata.json": b'{"image":"fixture"}\n',
    "imagegeneration/software-report.json": b'{"software":"fixture"}\n',
    "imagegeneration/software-report.md": b"Fixture software report\n",
    "imagegeneration/helpers/install.sh": b"upstream helper\n",
    "imagegeneration/helpers/invoke-tests.sh": b"upstream test helper\n",
    "imagegeneration/tests/Tools.Tests.ps1": b"upstream tests\n",
    "imagegeneration/SoftwareReport/Generate.ps1": b"upstream report generator\n",
    "opt/post-generation/cleanup-logs.sh": b"upstream log cleanup\n",
    "etc/ciproxy/boat-restart-services": b"docker.service\n",
}
BUILD_FILES = [
    "imagegeneration/helpers/github-cache/answer.json",
    "imagegeneration/helpers/ciproxy-pins.json",
    "tmp/ciproxy-github-cache/answer.json",
    "etc/apt/apt.conf.d/90ciproxy-build",
    "etc/apt/preferences.d/ciproxy-software",
    "etc/ciproxy/provider-docker-gid",
]


def legacy_image(root, *, preferences=True):
    for relative, content in {**UPSTREAM, **dict.fromkeys(BUILD_FILES, b"ciproxy build data\n")}.items():
        if not preferences and relative == "etc/apt/preferences.d/ciproxy-software":
            continue
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content)


def remaining_files(root):
    return {str(path.relative_to(root)): path.read_bytes()
            for path in root.rglob("*") if path.is_file()}


def assert_clean_image(root):
    for relative in BUILD_FILES:
        assert not (root / relative).exists(), relative
    for relative, content in UPSTREAM.items():
        assert (root / relative).read_bytes() == content, relative


def check_image_cleanup(case, workspace):
    root = pathlib.Path(tempfile.mkdtemp(dir=workspace)).resolve()
    legacy_image(root)
    definition = pathlib.Path(__file__).resolve().parents[1]
    def clean():
        result = subprocess.run(["bash", str(definition / "boat-build-clean-image.sh"), str(root)],
                                capture_output=True, text=True)
        case.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        metadata = {str(path.relative_to(root)): (record.st_ino, record.st_mode, record.st_uid,
                                                 record.st_gid, record.st_mtime_ns)
                    for path in root.rglob("*") for record in [path.stat()]}
        return remaining_files(root), metadata

    first, first_metadata = clean()
    with case.subTest(run="first"):
        case.assertEqual(first, UPSTREAM)
    second, second_metadata = clean()
    with case.subTest(run="second"):
        case.assertEqual(second, UPSTREAM)
        case.assertEqual(second_metadata, first_metadata)
