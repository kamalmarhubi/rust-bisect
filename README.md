# rustc-bisect

rustc-bisect helps track down when a change—usually a bug!—was introduced into
Rust. Rather than bisect directly on the Rust repository, it uses nightly
builds to speed up the process.

## Usage

```
rustc-bisect
Kamal Marhubi <kamal@marhubi.com>
Find the Rust nightly that introduced a bug

USAGE:
	rustc-bisect [FLAGS] --bad <VERSION> --good <VERSION> <COMMAND> [ARGS...]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
	--bad <VERSION>     A known bad nightly release
	--good <VERSION>    A known good nightly release

ARGS:
    COMMAND    The command to run
    ARGS...    Arguments for COMMAND

rustc-bisect [--good=TOOLCHAIN] [--bad=TOOLCHAIN] COMMAND [ARGS...]
```

## Installation

rustc-bisect uses the multirust-rs crate as a library, which curently requires
a nightly build to install. I suggest using multirust and cargo-install to
install rustc-bisect:

```
$ multirust run nightly cargo install rustc-bisect
```

## How it works

rustc-bisect does binary search on nightly builds. This quickly narrows down
which nightly introduced a bug or changed some behavior. It trades compilation
time for download time when comparing against `git bisect`. This can be
followed up with a `git bisect` to find the exact commit from among a much
smaller set of commits.
