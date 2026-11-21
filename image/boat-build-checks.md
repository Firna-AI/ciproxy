# Boat build check

```sh
python3 image/check-boat-build.py \
  --work-dir .context/image-checks/boat-build
```

The check runs the real build executor, the real pinned renderer, and real
recipe scripts on fixture recipes and private file trees.
It needs no root, Boat account, or network.
Its package-family scenario needs `apt-get` and `apt-cache`, and its
Ansible scenario needs `pip`. Its locale scenario needs `localedef`,
`locale`, and the glibc sources of the `en_US` locale and the `UTF-8`
charmap. Each reports a skip when one of its tools or sources is absent.
The [checks guide](checks.md) lists the other image checks.

## Build and resume

- A failed provisioner must stop the build with its exit code, and each
  later provisioner must stay pending.
- A resume must run the failed provisioner again and then each later
  provisioner. It must reuse a passed provisioner only when that
  provisioner and every earlier one are unchanged.
- A changed or new earlier provisioner, also a skipped one, must make each
  later provisioner run again. A later change must never run an earlier
  provisioner again.
- A change of the first step must run each later step again. A resume with
  no change must reuse them.
- A copy step that runs again must also run its move step. The move merges
  the folder into its target and keeps the files there. No copied source
  folder stays.
- A version difference between the reference and the installed inventory
  must appear in the comparison and must not fail the build.
- The list of services to restart always holds Docker and containerd. It
  adds each running service whose program or unit file the build changed,
  and leaves out the unit of the build itself.
- The host report download records each report that it cannot read. No
  request to the sandbox carries the Boat key.
- A change of release data, such as a download count, a reaction count,
  the release text, or a new CMake release, keeps each step identity. A
  refresh then reuses each passed step.
- Answers from `--metadata-cache` reach the answer store of the bundle. A
  refresh replaces that store and removes an old cache from the recipe
  tree.
- Only `boat.py` has `prepare`. `boat-build.py` keeps `bundle`, `refresh`,
  `report`, `resume`, and `finalize`.

## Records and cleanup of Boat's base

- The build records the owners of `/usr`, `/etc`, `/usr/local`, and
  `/usr/local/bin` before its first step, also without the inventory.
  A resume keeps the first record.
- The changed owner test replaces only the upstream root test, and keeps
  every other byte of its file. The change list gives its reason. When the
  pinned test has another directory list, the change fails and leaves the
  file unchanged.
- The first step removes each listed command and Rust setting after the
  package index update and before a package install. Provider and agent
  commands, extra tools, link targets, tool homes, and other settings stay
  byte for byte. A repeat changes no file and keeps the first record.
- The first step removes only the first copy of Boat's Rust block from
  `/etc/bash.bashrc`. The other comments and empty lines of the file stay.
  When nothing else changed, the file then has the exact bytes of its
  package.
- A package-owned Rust profile script stays. When the bash file still
  differs from its package, such as with a second block or another change,
  the cleanup gives a warning and still succeeds.
- The first step purges a listed package group only when the simulated
  plan of apt removes no other package. The record gives the version of
  each matched package and the purge action.
- A group that has another package in its plan stays, and its warning
  names that package. A group with no installed package is recorded as
  absent. No other package changes.
- The purge also clears packages that keep only their configuration
  files, such as the .NET files that point to Boat's .NET.
- The installed inventory gives the owner record and the cleanup record,
  or `null` when a record does not exist.
- The Boat cleanup keeps `/tmp` and the logs. It still runs each other
  upstream cleanup step, such as the removal of the test helper link.
- The cleanup script removes only the build-only files of ciproxy. It
  keeps `imagedata.json`, the software reports, the upstream helpers and
  tests, `SoftwareReport`, the post-generation files, and the restart list.
- A second run of the cleanup script changes no file, owner, mode, inode,
  or time. The script needs no build folder. Finalization runs it and keeps
  the upstream files.

## The locale of Boat's controller

- The check runs the real [boat-provision.sh](boat-provision.sh) from a
  private definition folder. Safe substitutes take the place of the
  account, service, and runner installation commands. A wrapper runs the
  real locale script with the fixture root. So the check fails when
  `boat-provision.sh` no longer runs the locale script.
- The locale script writes the `en_US.UTF-8` data to
  `/usr/local/lib/ciproxy/locales/en_US.utf8` in the fixture root.
- The check sets `LOCPATH` to the fixture copies of the folders that the
  profile names. The fixture `/usr/lib/locale` is empty. With `LOCPATH`
  set, glibc reads no locale archive. Then `en_US.UTF-8` and `C.UTF-8`
  must each give the `UTF-8` charmap and an empty stderr.
- For `HOME=/home/user`, two reads of the profile both set `LOCPATH` to
  `/usr/local/lib/ciproxy/locales:/usr/lib/locale`. With an inherited
  `LOCPATH`, the profile keeps each inherited entry and adds no folder
  twice.
- For another home, such as `/home/runner`, the check starts with a known
  exported `LANG` and no `LC_ALL`. The exported environment must be the
  same before and after the profile, and stderr must stay empty.
- A unique test locale name first fails to load. A link then points that
  name at the generated data, and the lookup must return UTF-8 without
  stderr. Host locale folders cannot supply that test name.
- [boat-locale-smoke.py](boat-locale-smoke.py) passes only for exit code
  0, the `UTF-8` codeset, and an empty stderr. Stderr with only white space
  fails it, and so does a failed request. Its messages hold no text from
  the Boat reply.

## Pins and native installers

- The shared toolset selects the real `netcat-openbsd` package.
- A release lookup with an exact version and no pin reads the release
  pages in order and selects the release whose tag names that version. It
  also accepts a tag with a prefix, such as `release-v1.39.0`.
- A cached lookup makes no new request. A pinned repository uses its pin.
  `latest` or a pattern without a pin calls the upstream lookup with the
  original arguments.
- The lookup reads the current bundle answers before the answers of an
  earlier run. It saves no answer in the recipe helpers.
- `bundle` stores the CMake release pages for the exact version of the
  rendered toolset in its answer store, outside the recipe. The lookup in
  the sandbox then needs no network.
- The Java installer selects the pinned Temurin package also when a newer
  package exists.
- An APT pin with an epoch selects its exact version also when a higher
  release exists.
- A package that differs from its APT pin appears in both Boat reports and
  in the machine measurement, also when no APT preference file exists. A
  pin of a source package covers each of its binary packages. No build
  fails because of a difference.
- The toolset keeps the upstream Node.js default form, and the installer
  installs the exact pinned Node.js version.
- After a failed download, a resume of the PyPy installer replaces only its
  incomplete version folder. A later replay reuses the complete version.
- An archive that extracts over an existing folder keeps the owner and
  mode of that folder. A new folder gets the metadata of the archive.
- The writable permission step gives the target folder itself mode 777.
  Existing entries below it keep their modes and owners. New entries get
  mode 777.
