# multirust-rs

[![Build Status](https://travis-ci.org/Diggsey/multirust-rs.svg)](https://travis-ci.org/Diggsey/multirust-rs)
[![Build status](https://ci.appveyor.com/api/projects/status/au79mlftfkhkpr0v/branch/master?svg=true)](https://ci.appveyor.com/project/Diggsey/multirust-rs/branch/master)

Multirust-rs is a reimplementation of multirust in rust. It provides both a command line interface, and a rust library, so it's trivial to integrate it with external tools.

## Library Installation

Add [multirust-rs](https://crates.io/crates/multirust-rs) or [rust-install](https://crates.io/crates/rust-install) as a standard cargo dependency to your project, depending on your requirements.

## Library Documentation

- [multirust](http://diggsey.github.io/multirust-rs/multirust/index.html)
- [rust-install](http://diggsey.github.io/multirust-rs/rust_install/index.html)


## Tool Installation

### Installing from binaries

- [Windows GNU 64-bit installer](https://github.com/Diggsey/multirust-rs-binaries/raw/master/x86_64-pc-windows-gnu/multirust-rs.exe)
- [Windows MSVC 64-bit installer](https://github.com/Diggsey/multirust-rs-binaries/raw/master/x86_64-pc-windows-msvc/multirust-rs.exe)
- [Windows GNU 32-bit installer](https://github.com/Diggsey/multirust-rs-binaries/raw/master/i686-pc-windows-gnu/multirust-rs.exe)
- [Windows MSVC 32-bit installer](https://github.com/Diggsey/multirust-rs-binaries/raw/master/i686-pc-windows-msvc/multirust-rs.exe)
- [Linux 64-bit installer](https://github.com/Diggsey/multirust-rs-binaries/raw/master/x86_64-unknown-linux-gnu/multirust-rs)
- [Linux 32-bit installer](https://github.com/Diggsey/multirust-rs-binaries/raw/master/i686-unknown-linux-gnu/multirust-rs)
- [Mac 64-bit installer](https://github.com/Diggsey/multirust-rs-binaries/raw/master/x86_64-apple-darwin/multirust-rs)
- [Mac 32-bit installer](https://github.com/Diggsey/multirust-rs-binaries/raw/master/i686-apple-darwin/multirust-rs)

Binaries for other platforms are not yet available. Follow the instructions below for installing from source.


### Installing from source

Run this command in a writable directory:
```
git clone --depth 1 https://github.com/Diggsey/multirust-rs.git multirust-rs && cd multirust-rs && cargo run --release self install [-a]
```

Passing `-a` will attempt to automatically add `~/.multirust/bin` to your PATH.

On linux, this is done by appending to `~/.profile`.
On windows, this is done by modifying the registry entry `HKCU\Environment\PATH`.

The changes to PATH will not take effect immediately within the same terminal.

The `multirust-rs` directory which is created is no longer required once installation has completed, but keeping it around will make future updates much faster:

```
cd multirust-rs && git pull && cargo run --release install
```


## Tool Documentation

### Usage

```
multirust 0.0.4
Diggory Blake
Port of multirust to rust

USAGE:
        multirust [FLAGS] [SUBCOMMAND]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information
    -v, --verbose    Enable verbose output

SUBCOMMANDS:
    ctl
    default             Set the default toolchain.
    delete-data         Delete all user metadata.
    doc                 Open the documentation for the current toolchain.
    help                Prints this message
    list-overrides      List all overrides.
    list-toolchains     List all installed toolchains.
    override            Set the toolchain override.
    proxy               Proxy a command.
    remove-override     Remove an override.
    remove-toolchain    Uninstall a toolchain.
    run                 Run a command.
    self                Commands for managing multirust itself.
    show-default        Show information about the current default.
    show-override       Show information about the current override.
    update              Install or update a given toolchain.
    upgrade-data        Upgrade the ~/.multirust directory.
    which               Report location of the currently active Rust tool.
```

### Toolchain names

Standard toolchain names have the following form:
```
[<arch>-][<os>-][<env>-]<channel>[-<date>]

<arch>		= i686|x86_64
<os>		= pc-windows|unknown-linux|apple-darwin
<env>		= gnu|msvc
<channel>	= stable|beta|nightly
<date>		= YYYY-MM-DD
```

Any combination of optional parts are acceptable.

Parts of the target triple which are omitted, will default to that of the host.
If the date is omitted, the toolchain will track the most recent version.

### Envionment variables

The following environment variables can be used to customize the behaviour of
multirust-rs:

- `MULTIRUST_TOOLCHAIN` (default: none)
	If set, will override the toolchain used for all rust tool invocations. A toolchain
	with this name should be installed, or invocations will fail.

- `MULTIRUST_DIST_ROOT` (default: `https://static.rust-lang.org/dist`)
	Sets the root URL for downloading packages. You can change this to instead use
	a local mirror, or to test the binaries from the staging directory.

- `MULTIRUST_HOME` (default: `~/.multirust` or `%LOCALAPPDATA%/.multirust`)
	Sets the root multirust folder, used for storing installed toolchains and configuration
	options.

- `MULTIRUST_GPG_KEY` (default: none)
	Sets the GPG key used to verify the signatures of downloaded files.
	WARNING: GPG signature verification is not yet implemented.


### Example usage

- Set the default toolchain to the latest nightly:
	`multirust default nightly`

- For the current directory, use the most recent stable build using the MSVC linker:
	`multirust override msvc-stable`

- For the current directory, use a 32-bit beta build instead:
	`multirust override i686-beta`

- For the current directory, use a nightly from a specific date:
	`multirust override nightly-2015-04-01`

- Combine these:
	`multirust override i686-msvc-nightly-2015-04-01`

- Install a custom toolchain using an installer (windows):
	`multirust override my_custom_toolchain --install "C:\RustInstaller.msi"`

- Install a custom toolchain using an installer (linux):
	`multirust override my_custom_toolchain --install "/home/user/RustInstaller.tar.gz"`

- Install a custom toolchain using an installer from the internet (linux):
	`multirust override my_custom_toolchain --install "http://domain.tld/installer.tar.gz"`

- Install a custom toolchain by symlinking an existing installation:
	`multirust override my_custom_toolchain --link-local "C:\RustInstallation"`

- Install a custom toolchain by copying an existing installation:
	`multirust override my_custom_toolchain --copy-local "C:\RustInstallation"`

- Switch back to the default toolchain for the current directory:
	`multirust remove-override`

- See which toolchain will be used in the current directory:
	`multirust show-override`


## License

    Licensed under either of

     * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
     * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

    at your option.


## Contributing

1. Fork it!
2. Create your feature branch: `git checkout -b my-new-feature`
3. Commit your changes: `git commit -am 'Add some feature'`
4. Push to the branch: `git push origin my-new-feature`
5. Submit a pull request :D

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
