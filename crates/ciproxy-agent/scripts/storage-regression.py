#!/usr/bin/env python3
"""Apache-2.0. Run the production sizing/write regression on disposable ext4/XFS."""

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile


def main():
    evidence = Path('.context/ciproxy-mvp').resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    for tool in ['cargo', 'mkfs.ext4', 'mkfs.xfs', 'mount', 'umount', 'losetup', 'xfs_growfs', 'findmnt', 'cp', 'filefrag']:
        if shutil.which(tool) is None:
            print(f'SKIP: {tool} is required', file=sys.stderr)
            return 77
    if os.geteuid() != 0:
        print('SKIP: root and mount permission are required', file=sys.stderr)
        return 77
    root = Path(tempfile.mkdtemp(prefix='storage-regression-', dir=evidence))
    host = root / 'host'
    host.mkdir()
    outer = root / 'host.ext4'
    status = 1
    mounted = False
    try:
        with outer.open('xb') as output:
            output.truncate(2 * 1024 * 1024 * 1024)
        subprocess.run(['mkfs.ext4', '-q', '-F', '-m', '0', '--', str(outer)], check=True)
        probe = subprocess.run(['mount', '-t', 'ext4', '-o', 'loop', '--', str(outer), str(host)], capture_output=True, text=True)
        if probe.returncode:
            print('SKIP: loop mounts are unavailable: ' + probe.stderr.strip(), file=sys.stderr)
            return 77
        mounted = True
        probe_file = host / 'probe.xfs'
        probe_mount = host / 'probe'
        probe_mount.mkdir()
        with probe_file.open('xb') as output:
            output.truncate(320 * 1024 * 1024)
        formatted = subprocess.run(['mkfs.xfs', '-q', '-m', 'reflink=1', '--', str(probe_file)], capture_output=True, text=True)
        probe = subprocess.run(['mount', '-t', 'xfs', '-o', 'loop,nosuid', '--', str(probe_file), str(probe_mount)], capture_output=True, text=True) if formatted.returncode == 0 else formatted
        if probe.returncode:
            print('SKIP: XFS reflink mounts are unavailable: ' + probe.stderr.strip(), file=sys.stderr)
            return 77
        subprocess.run(['umount', '--', str(probe_mount)], check=True)
        probe_file.unlink()
        probe_mount.rmdir()
        env = dict(os.environ, CIPROXY_REGRESSION_ROOT=str(root), LC_ALL='C')
        result = subprocess.run(['cargo', 'test', '--locked', '--package', 'ciproxy-agent', '--features', 'filesystem-regression', '--test', 'storage-regression'], env=env, capture_output=True, text=True)
        (evidence / 'budget-filesystem-regression.log').write_text(result.stdout + result.stderr)
        print(result.stdout, end='')
        print(result.stderr, file=sys.stderr, end='')
        status = result.returncode
    finally:
        area = host / 'area'
        if area.exists() and subprocess.run(['findmnt', '--mountpoint', str(area)], capture_output=True).returncode == 0:
            if subprocess.run(['umount', '--', str(area)]).returncode:
                raise RuntimeError('The busy area is preserved at ' + str(root))
        if mounted and subprocess.run(['umount', '--', str(host)]).returncode:
            raise RuntimeError('The busy host is preserved at ' + str(root))
        shutil.rmtree(root)
    print(json.dumps({'exit_code': status, 'temporary_mounts_and_files_removed': True}))
    return status


if __name__ == '__main__':
    sys.exit(main())
