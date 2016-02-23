extern crate multirust;
extern crate rust_bisect;
extern crate rust_install;

use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

///! This example is equivalent to running
///!     rust-bisect --good=nightly-2015-10-27 --bad=nightly-2015-11-30 cargo build
///! in the `examples/rust-issue-30123` directory.
fn main() {
    let manifest_dir = PathBuf::from(&env::var("CARGO_MANIFEST_DIR").unwrap());
    env::set_current_dir(manifest_dir.join("examples/rust-issue-30123")).unwrap();

    let cmd = OsStr::new("cargo");
    let args = vec![OsStr::new("build")];

    let cfg = rust_bisect::Cfg {
        good: "nightly-2015-10-27".parse().unwrap(),
        bad: "nightly-2015-11-30".parse().unwrap(),
        cmd: cmd,
        args: args,
    };

    let mr_cfg = multirust::Cfg::from_env(rust_bisect::cli::notify_handler())
                     .expect("multirust config");

    rust_bisect::run(&cfg, &mr_cfg).unwrap();
}
