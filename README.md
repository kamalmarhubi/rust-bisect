# rust-bisect

rust-bisect helps track down when a change—usually a bug!—was introduced into
Rust. Rather than bisect directly on the Rust repository, it uses nightly
builds to speed up the process.

## Usage

```
rust-bisect
Kamal Marhubi <kamal@marhubi.com>
Find the Rust nightly that introduced a bug

USAGE:
	rust-bisect [FLAGS] --bad <VERSION> --good <VERSION> <COMMAND> [ARGS...]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
	--bad <VERSION>     A known bad nightly release
	--good <VERSION>    A known good nightly release

ARGS:
    COMMAND    The command to run
    ARGS...    Arguments for COMMAND

```

## Installation

rust-bisect uses the multirust-rs crate as a library, which currently requires
a nightly build to install. I suggest using multirust and cargo-install to
install rust-bisect:

```
$ multirust run nightly cargo install rust-bisect
```

**NB** This takes a *really* long time the first time! There are a few changes
I made in upstream multirust-rs that aren't in any release yet, so I use a git
dependency. Unfortunately, there is a submodule that has all binaries, and
Cargo will get the submodule and its entire history, which drastically slows
down the initial build as it downloads almost 700 MB of binaries.


## How it works

rust-bisect does binary search on nightly builds. This quickly narrows down
which nightly introduced a bug or changed some behavior. It trades compilation
time for download time when comparing against `git bisect`. This can be
followed up with a `git bisect` to find the exact commit from among a much
smaller set of commits.
