# Boat runner check

```sh
sudo python3 image/check-boat-runner.py
```

The check runs the real NVM guard on a fixture profile. It also runs the
real Boat runner files with a fixture listener. The
[build steps](boat-build-steps.md#the-runner-stderr-of-a-boat-job) say
what these files do. The check needs no real registration, Boat account,
or network. The [checks guide](checks.md) lists the other image checks.

Each case that runs an entry or the helper needs root, `chroot`, and
`ldd`. Such a case runs as an unprivileged user in a private root with
Bash, Python, the real Boat runner files, and a fixture journal writer.
The check removes its fixtures under `.context/image-checks/boat-runner`
at the end.

- The NVM guard: a fixture profile with the same lookup line ends a
  `bash -e` shell before the guard. After the guard, the profile goes on
  with the system `PATH` and no stderr. The eager `nvm.sh` fallback and a
  selected version path still work. A second run changes nothing. A
  missing or unknown profile gives a warning and stays unchanged.
- The stderr helper reads all of its input. The journal gets at most
  16 KiB and one truncation notice, under the identifier
  `ciproxy-boat-runner-stderr`. The journal writer gets no registration
  value. The kept text starts with the profile error and the constant
  missing-registration line. It holds no registration value and no value
  decoded from it. It also holds no bearer, cookie, or password line, no
  long opaque value, no oversized line, and no output past the limit. A
  URL line keeps its text before the first `://` and ends with
  `://[redacted]`, unless a private part redacts the whole line.
- The real Boat entry runs with the real helper, the real error hook, and
  a recording journal writer. A profile prints the decoded fixture
  credential and then fails without output. After a pipe shows that the
  helper has ended, the entry must have exit status 1 and no output. The
  journal must hold exactly `[redacted]` and then the hook record.
- The entry sends its stderr to the helper before `/etc/profile` runs.
  Neither a failed journal writer nor a missing helper file stops the
  runner. The check runs the entry once with each failure. In each run, a
  profile confirms that its stderr is a pipe. The listener gets no
  `BASH_ENV` and writes more stderr than a pipe buffer holds. Its stdout
  and exit status must pass through, and the original stderr of the entry
  must stay empty.
- The error hook: a silent profile failure keeps exit status 1, and stderr
  then holds only the file name, the line, and the status.
