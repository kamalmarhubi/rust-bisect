# rust-bisect

rust-bisect helps track down when a change—usually a bug!—was introduced into
Rust. Rather than bisect directly on the Rust repository, it uses nightly
builds to speed up the process.


## Usage

```
rust-bisect
Kamal Marhubi <kamal@marhubi.com>
Find the Rust nightly that that changed some behavior

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

rust-bisect uses the [multirust-rs] crate as a library, which currently
requires a nightly build to install. I suggest using multirust and
cargo-install to install rust-bisect:

```
$ multirust run nightly cargo install rust-bisect
```

Note that as rust-bisect uses [multirust-rs] to handle finding, downloading,
and installing the nightly builds, it requires an install of either
[multirust-rs] or [multirust].

[multirust]: https://github.com/brson/multirust
[multirust-rs]: https://github.com/Diggsey/multirust-rs/

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

Since rust-bisect uses multirust-rs, all nightlies that are installed to test
against will be installed in your multirust root directory. At present they are
not cleaned up, or in any way distinguished from toolchains you installed
directly through multirust or multirust-rs.


## Example

This example is based on a real Rust issue, [#30123][issue-30123]. This issue
was reported on 2015-11-30, and referred to behavior differing from 1.4.0,
which was built from a commit dated 2015-10-27:

```
$ multirust run 1.4.0 rustc -vV
rustc 1.4.0 (8ab8581f6 2015-10-27)
binary: rustc
commit-hash: 8ab8581f6921bc7a8e3fa4defffd2814372dcb15
commit-date: 2015-10-27
host: x86_64-unknown-linux-gnu
release: 1.4.0
```

We can run rust-bisect on the example crate in `examples/rust-issue-30123`,
which I extracted from the test added for the issue. The command we use is
`cargo build`, since the issue manifested as a compilation failure.

```
$ rust-bisect --good nightly-2015-10-27 --bad nightly-2015-11-30 cargo build
finding available nightlies between 2015-10-27 and 2015-11-30
found 23 nightlies
bisecting across 23 nightlies (about 5 steps)
testing with nightly-2015-11-13
   Compiling aux v0.1.0 (file:///home/kamal/projects/rust-bisect/examples/rust-issue-30123)
[...]
command succeeded with nightly-2015-11-26
nightly-2015-11-27 is the first failing nightly
```

In the discussion on [#30123][issue-30123], the commit that changed the
behavior [was identified][identified] as [f5fbefa][commit]. That commit was part of pull
request [#30043][pr], which was [merged on 2015-11-26][merged]. It looks like
we found the right nightly! We can check further by finding looking at the rust
repository's history between `nightly-2015-11-26` and `nightly-2015-11-27`:

```
rust$ multirust run nightly-2015-11-26 rustc -V
rustc 1.6.0-nightly (1805bba39 2015-11-26)
rust$ multirust run nightly-2015-11-27 rustc -V
rustc 1.6.0-nightly (1727dee16 2015-11-26)
rust$ git log --oneline 1805bba39..1727dee16 | grep f5fbefa
f5fbefa remove csearch from resolve and typeck
```

There it is!

After using rust-bisect, we could have used `git bisect` to narrow
it down to the exact commit. In this case, that would be testing over just 30
commits, which would require about 5 steps to bisect:

```
rust$ git log --oneline 1805bba39..1727dee16 | wc -l
30
```

Compare that against the number of commits if we had to `git bisect` across the whole range:

```
rust$ multirust run nightly-2015-10-27 rustc -V
rustc 1.5.0-nightly (95fb8d1c8 2015-10-27)
rust$ multirust run nightly-2015-11-30 rustc -V
rustc 1.6.0-nightly (52d95e644 2015-11-30)
rust$ git log --oneline 95fb8d1c8..52d95e644 | wc -l
881
```

Many more commits! Of course, because of binary search and logarithms, it would
only be double the number of bisect steps. But that's still double the number
of compiles of the Rust codebase!

[issue-30123]: https://github.com/rust-lang/rust/issues/30123
[identified]: https://github.com/rust-lang/rust/issues/30123#issuecomment-172980819
[commit]: https://github.com/rust-lang/rust/commit/f5fbefa3af48ed44b002a7423d6cbd74e4018c9c
[pr]: https://github.com/rust-lang/rust/pull/30043
[merged]: https://github.com/rust-lang/rust/pull/30043#event-475858549
