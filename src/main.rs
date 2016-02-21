#[macro_use(shared_ntfy)]
extern crate rust_install;

extern crate clap;
extern crate multirust;

extern crate rustc_bisect;

use clap::{App, AppSettings, Arg};

use multirust::{Cfg, Notification};

use rustc_bisect::{Cmd, Result, ToolchainSpec};

// Copied from multirust-rs.
fn set_globals() -> multirust::Result<Cfg> {
    // Base config
    let verbose = false;
    Cfg::from_env(shared_ntfy!(move |n: Notification| {
        use multirust::notify::NotificationLevel::*;
        match n.level() {
            Verbose => {
                if verbose {
                    println!("{}", n);
                }
            }
            _ => {
                println!("{}", n);
            }
        }
    }))
}

fn run_rust_bisect() -> Result<()> {
    let matches = App::new("rustc-bisect")
                      .author("Kamal Marhubi <kamal@marhubi.com>")
                      .setting(AppSettings::TrailingVarArg)
                      .arg(Arg::with_name("toolchain")
                               .long("tmp-toolchain")
                               .takes_value(true)
                               .value_name("TOOLCHAIN")
                               .required(true))
                      .arg(Arg::with_name("good")
                               .long("good")
                               .takes_value(true)
                               .value_name("TOOLCHAIN"))
                      .arg(Arg::with_name("bad")
                               .long("bad")
                               .takes_value(true)
                               .value_name("TOOLCHAIN"))
                      .arg(Arg::with_name("COMMAND")
                               .multiple(true)
                               .required(true))
                      .get_matches();

    let cfg = try!(set_globals());

    let cmd: Vec<_> = matches.values_of_os("COMMAND").expect("COMMAND").collect();
    let cmd = Cmd::from(&cmd[..]);

    let toolchain = try!(cfg.get_toolchain(matches.value_of("toolchain").expect("toolchain"),
                                           false));
    try!(toolchain.install_from_dist_if_not_installed());

    println!("{}", try!(cmd.succeeds_with(&toolchain)));

    Ok(())
}

fn main() {
    run_rust_bisect().expect("something went wrong");
}
