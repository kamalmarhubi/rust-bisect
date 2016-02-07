# rustc-bisect

rustc-bisect helps track down when a change—usually a bug!—was introduced into
Rust. Rather than bisect directly on the Rust repository, it uses nightly builds
to speed up the process.

## Usage

```
rustc-bisect [--good=TOOLCHAIN] [--bad=TOOLCHAIN] COMMAND [ARGS...]
```

TODO: document args

## Installation

rustc-bisect uses the multirust-rs crate as a library, which requires a nightly
build to install. I suggest using multirust and cargo-install to install
rustc-bisect:

```
$ multirust run nightly cargo install rustc-bisect
```

## How it works

rustc-bisect does binary search on nightly builds. To reduce the number of
nightlies that must be downloaded, it first searches by first-day-of-month, and
then within the month. This should result in more reuse of builds, saving
bandwidth and disk space.

If no `--good` build is specified, it will start at the current date, and expand
a window exponentially: first checking 1 month in the past, then 2, then 4, and
so on.
