#[macro_use(shared_ntfy)]
extern crate rust_install;

extern crate clap;
extern crate multirust;

use std::ffi::OsStr;
use std::error::Error;

use clap::{App, AppSettings, Arg};

use multirust::{Cfg, Notification, Toolchain};

type Result<T> = std::result::Result<T, Box<Error>>;

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

struct Cmd<'a> {
    program: &'a OsStr,
    args: &'a [&'a OsStr],
}

impl<'a> Cmd<'a> {
    fn from(command: &'a [&'a OsStr]) -> Cmd<'a> {
        let program = command[0];
        let args = &command[1..];

        Cmd {
            program: program,
            args: args,
        }
    }

    fn succeeds_with<'b>(&self, toolchain: &Toolchain<'b>) -> Result<bool> {
        let mut cmd = try!(toolchain.create_command(&self.program));
        cmd.args(self.args);
        let status = try!(cmd.status());
        Ok(status.success())
    }
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
