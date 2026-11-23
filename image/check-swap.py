#!/usr/bin/env python3
# SPDX-License-Identifier: Apache-2.0
"""Check boot-time zram swap with private proc and sysfs fixtures."""

import pathlib
import shutil
import subprocess
import sys
import tempfile
import unittest


SCRIPT = pathlib.Path(__file__).with_name("ciproxy-swap.sh")


class SwapCheck(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory(prefix="ciproxy-swap-")
        self.addCleanup(temporary.cleanup)
        root = pathlib.Path(temporary.name)
        self.proc = root / "proc"
        self.vm = self.proc / "sys/vm"
        self.vm.mkdir(parents=True)
        (self.vm / "page-cluster").write_text("3\n")
        (self.vm / "swappiness").write_text("60\n")
        self.set_memory(4_000_000)
        self.swaps = self.proc / "swaps"
        self.swaps.write_text("Filename Type Size Used Priority\n/swapfile file 1048576 0 -2\n")
        sys_root = root / "sys"
        self.zram_control = sys_root / "class/zram-control"
        self.zram_control.mkdir(parents=True)

        tools = root / "tools"
        tools.mkdir()
        self.calls = root / "calls"
        self.calls.touch()
        shim = tools / "swap-tool"
        shim.write_text(f"#!{sys.executable}\n" + '''
import os
import pathlib
import sys

tool = pathlib.Path(sys.argv[0]).name
args = sys.argv[1:]
with open(os.environ['FIXTURE_CALLS'], 'a') as log:
    log.write(' '.join([tool, *args]) + '\\n')
if tool == os.environ.get('FIXTURE_FAIL'):
    sys.exit(1)
if tool == 'modprobe':
    if not os.environ.get('FIXTURE_LOAD_ZRAM'):
        sys.exit(1)
    (pathlib.Path(os.environ['CIPROXY_SYS_ROOT']) / 'class/zram-control').mkdir(parents=True)
if tool == 'zramctl' and '--find' in args:
    if '--algorithm' in args and os.environ.get('FIXTURE_NO_ZSTD'):
        sys.exit(1)
    print('/dev/zram0')
''')
        shim.chmod(0o755)
        for name in ["modprobe", "zramctl", "mkswap", "swapon", "swapoff"]:
            (tools / name).symlink_to(shim.name)
        (tools / "awk").symlink_to(shutil.which("awk"))
        self.environment = {"PATH": str(tools), "LC_ALL": "C", "TZ": "UTC",
                            "CIPROXY_PROC_ROOT": str(self.proc), "CIPROXY_SYS_ROOT": str(sys_root),
                            "FIXTURE_CALLS": str(self.calls)}

    def set_memory(self, total_kib):
        (self.proc / "meminfo").write_text(f"MemTotal: {total_kib} kB\nMemFree: 100000 kB\n")

    def run_swap(self, **environment):
        result = subprocess.run([shutil.which("bash"), str(SCRIPT)], capture_output=True, text=True,
                                env={**self.environment, **environment})
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(result.stdout, "")
        return result.stderr, self.calls.read_text().splitlines()

    def test_sets_zstd_before_size_and_activates_swap(self):
        stderr, calls = self.run_swap()
        self.assertEqual(stderr, "")
        self.assertEqual(calls, [
            "zramctl --find --algorithm zstd --size 4096000000",
            "mkswap /dev/zram0",
            "swapon --priority 100 /dev/zram0",
        ])
        self.assertEqual((self.vm / "page-cluster").read_text(), "0\n")
        self.assertEqual((self.vm / "swappiness").read_text(), "60\n")

    def test_caps_swap_at_eight_gib(self):
        self.set_memory(16_000_000)
        _, calls = self.run_swap()
        self.assertEqual(calls[:1], ["zramctl --find --algorithm zstd --size 8589934592"])

    def test_warns_and_uses_kernel_default_when_zstd_is_unavailable(self):
        stderr, calls = self.run_swap(FIXTURE_NO_ZSTD="1")
        self.assertEqual(calls, [
            "zramctl --find --algorithm zstd --size 4096000000",
            "zramctl --find --size 4096000000",
            "mkswap /dev/zram0",
            "swapon --priority 100 /dev/zram0",
        ])
        self.assertIn("Warning: Could not configure zram with zstd.", stderr)

    def test_loads_zram_module_before_setting_up_swap(self):
        self.zram_control.rmdir()
        stderr, calls = self.run_swap(FIXTURE_LOAD_ZRAM="1")
        self.assertEqual(stderr, "")
        self.assertEqual(calls[:2], ["modprobe zram", "zramctl --find --algorithm zstd --size 4096000000"])

    def test_leaves_active_zram_swap_alone(self):
        with self.swaps.open("a") as swaps:
            swaps.write("/dev/zram7 partition 8192 0 100\n")
        stderr, calls = self.run_swap()
        self.assertEqual(stderr, "")
        self.assertEqual(calls, [])
        self.assertEqual((self.vm / "page-cluster").read_text(), "3\n")

    def test_warns_and_continues_when_zram_is_missing(self):
        self.zram_control.rmdir()
        stderr, calls = self.run_swap()
        self.assertEqual(calls, ["modprobe zram"])
        self.assertIn("Warning: zram is unavailable.", stderr)

    def test_keeps_swap_when_page_cluster_cannot_be_set(self):
        page_cluster = self.vm / "page-cluster"
        page_cluster.unlink()
        page_cluster.mkdir()
        stderr, calls = self.run_swap()
        self.assertEqual(calls[-1], "swapon --priority 100 /dev/zram0")
        self.assertEqual(stderr, "Warning: Could not set vm.page-cluster to 0.\n")

    def test_releases_device_when_mkswap_fails(self):
        stderr, calls = self.run_swap(FIXTURE_FAIL="mkswap")
        self.assertEqual(calls[-2:], ["mkswap /dev/zram0", "zramctl --reset /dev/zram0"])
        self.assertIn("Warning: Could not format /dev/zram0 as swap.", stderr)

    def test_releases_device_when_swapon_fails(self):
        stderr, calls = self.run_swap(FIXTURE_FAIL="swapon")
        self.assertEqual(calls[-3:], [
            "mkswap /dev/zram0", "swapon --priority 100 /dev/zram0", "zramctl --reset /dev/zram0",
        ])
        self.assertIn("Warning: Could not enable swap on /dev/zram0.", stderr)


if __name__ == "__main__":
    unittest.main(verbosity=2)
