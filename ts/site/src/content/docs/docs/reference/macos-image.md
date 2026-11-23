---
title: The macOS image
description: >-
  What the macOS image holds, and how a macOS job on ciproxy differs from a
  job on GitHub macos-latest.
---

Each job of a [macOS label](/docs/reference/labels/#macos-labels) runs in a
fresh macOS VM on one of your Macs. The VM starts from the macOS image.
ciproxy builds the image from the GitHub runner image
[`macos-26-arm64/20261117.0516`][github-image], which `macos-latest` named
on 17 November 2026.

## What the image holds

The current image is
`macos26-arm64-20261117.0516-runner-2.338.2-xcode-26.6-standard-v2`. The
report script of the GitHub runner images lists its software. Of the 92
entries of the GitHub report, the image has 91. Only the Android SDK is
missing. The tests of the GitHub image pass in the VM: 374 passed, 0
failed, and 10 skipped.

- **System:** macOS 26.7.3 (25G342) on arm64. System Integrity Protection
  (SIP) is off, as on the GitHub image. The user is `runner`, with the home
  folder `/Users/runner`, as on GitHub.
- **Runner:** the GitHub Actions runner 2.338.2, in `/opt/actions-runner`.
- **Xcode:** Xcode 26.6 (17F113) only, at `/Applications/Xcode_26.6.app`
  and `/Applications/Xcode.app`. It has the SDKs of macOS, iOS, tvOS,
  watchOS, and visionOS 26.5, and DriverKit 25.5.
- **Simulators:** the iOS 26.5 runtime, with iPhone 17, iPhone 17 Pro,
  iPhone 17 Pro Max, iPhone 17e, iPhone Air, iPad (A16), iPad Air 11-inch
  and 13-inch (M4), iPad mini (A17 Pro), and iPad Pro 11-inch and 13-inch
  (M5).
- **Languages:** .NET SDK 8, 9, 10, and 11. Node.js 24, with 22 and 24 in
  the tool cache. Python 3.15, with 3.11 to 3.15 in the tool cache. Ruby
  3.4, with 3.2 to 4.0 in the tool cache. Go 1.25 to 1.27 in the tool
  cache. Java 11, 17, 21, and 25, with 21 as the default. Kotlin, Rust,
  Perl, GCC 13 to 15 with Fortran, Clang 21, and Homebrew LLVM 20.
- **Package and build tools:** Homebrew, CocoaPods, Carthage, Bundler, npm,
  Yarn, pip, pipx, vcpkg, Maven, Gradle, Ant, CMake, Ninja, Bazel, and
  Bazelisk.
- **Other tools:** GitHub CLI, Git and Git LFS, AWS CLI and SAM CLI, Azure
  CLI, Bicep, the CodeQL bundle, Fastlane, SwiftFormat, xcbeautify, xcodes,
  Packer, jq, yq, zstd, 7-Zip, OpenSSL 4 as the default and OpenSSL 3, and
  PowerShell 7.6 with Az, Pester, and PSScriptAnalyzer.
- **Browsers:** Safari, Chrome and Chrome for Testing with ChromeDriver,
  Edge with its WebDriver, Firefox with geckodriver, and Selenium server.

## ciproxy compared with GitHub macos-latest

| Item                   | ciproxy                                                                          | GitHub `macos-latest`                                                    |
| ---------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------------------------------ |
| macOS                  | 26.7.3 (25G342)                                                                  | 26.7.2                                                                   |
| Xcode                  | 26.6 only                                                                        | 26.0.1, 26.1.1, 26.2, 26.3, 26.4.1, 26.5, and 26.6. The default is 26.6. |
| Simulator runtimes     | iOS 26.5 only                                                                    | iOS, tvOS, watchOS, and visionOS, for several versions                   |
| Android SDK            | Not in the image. `ANDROID_HOME` is set, but no SDK is there.                    | In the image                                                             |
| Safari                 | 26.6.2, the Safari of macOS                                                      | 27.2                                                                     |
| Other tools            | The same tools. About 24 have a newer patch release, such as Chrome 158 for 157. | As GitHub lists them                                                     |
| SIP                    | Off                                                                              | Off                                                                      |
| Free disk of a job     | About 21.8 GB                                                                    | As GitHub states it                                                      |
| VMs at a time          | At most 2 on each Mac, as Apple allows                                           | As many as GitHub starts                                                 |
| Start of a job         | Cold: the VM boots macOS for each job                                            | As GitHub starts it                                                      |
| Cache                  | The GitHub cache only. A Mac has no machine cache.                               | The GitHub cache                                                         |
| When all Macs are busy | The job waits for a free Mac. It never runs on Boat.                             | Not applicable                                                           |
| Image download         | About 46.2 GB for each new image                                                 | None                                                                     |

The toolset of GitHub pins Node.js, Python, Ruby, Go, Java, .NET, and
Xcode, so the image has the same versions. Homebrew and some installers
take the newest release when the image is built, so other tools can have a
newer patch release.

## How the image affects your workflows

- A workflow that selects an Xcode other than 26.6 fails, for example with
  `xcode-select`, `DEVELOPER_DIR`, or `maxim-lobanov/setup-xcode`. Move it
  to Xcode 26.6.
- A workflow that needs a tvOS, watchOS, or visionOS simulator, or the
  Android SDK, cannot run on ciproxy yet. Keep it on GitHub-hosted runners.
- A workflow that writes more than about 21.8 GB fails when the disk is
  full.
- Each job boots macOS, so a short job takes longer than on a warm GitHub
  runner.
- A cache step uses the GitHub cache. Its speed depends on the link of the
  Mac to GitHub.

## The image on the Mac

- The first image is a download of about 46.2 GB. Each new image is a new
  download of the same size.
- The unpacked image takes about 77 GB on the disk of the Mac.
- While a new image unpacks, the Mac also keeps its download. So a Mac
  needs about 150 GB of free disk, as
  [Add a Mac](/docs/guides/add-a-machine/#add-a-mac) says.
- The download comes from Cloudflare R2, at
  `785233830ca9dfa2eb7c3561dd52f675.r2.cloudflarestorage.com`.
- A Mac takes no job until its first image is ready.

[github-image]: https://github.com/actions/runner-images/releases/tag/macos-26-arm64%2F20261117.0516
