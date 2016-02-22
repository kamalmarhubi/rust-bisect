extern crate chrono;
extern crate clap;
extern crate libc;
extern crate multirust;
extern crate rust_install;

extern crate rustc_bisect;

use std::process;

use chrono::{Datelike, NaiveDate};
use clap::{App, AppSettings, Arg};

use rustc_bisect::{Cmd, Error, Result, ToolchainSpec, bisect};

fn run_rust_bisect() -> Result<i32> {
    let matches = App::new("rustc-bisect")
                      .author("Kamal Marhubi <kamal@marhubi.com>")
                      .setting(AppSettings::TrailingVarArg)
                      .arg(Arg::with_name("good")
                               .long("good")
                               .takes_value(true)
                               .value_name("TOOLCHAIN")
                               .required(true))
                      .arg(Arg::with_name("bad")
                               .long("bad")
                               .takes_value(true)
                               .value_name("TOOLCHAIN")
                               .required(true))
                      .arg(Arg::with_name("COMMAND")
                               .multiple(true)
                               .required(true))
                      .get_matches();


    let cfg = try!(multirust::Cfg::from_env(rust_install::notify::SharedNotifyHandler::none()));

    use std::str::FromStr;

    let good = matches.value_of("good").expect("good");
    let good_spec = try!(ToolchainSpec::from_str(good));

    let bad = matches.value_of("bad").expect("bad");
    let bad_spec = try!(ToolchainSpec::from_str(bad));

    fn get_nightly_date(spec: ToolchainSpec) -> Result<NaiveDate> {
        match spec {
            ToolchainSpec::Nightly(date) => Ok(date),
            _ => Err(Error::from("only nightlies for now")),
        }
    }

    let good_date = try!(get_nightly_date(good_spec));
    let bad_date = try!(get_nightly_date(bad_spec));

    let cmd: Vec<_> = matches.values_of_os("COMMAND").expect("COMMAND").collect();
    let cmd = Cmd::from(&cmd[..]);

    let range = good_date.num_days_from_ce()..bad_date.num_days_from_ce();

    let res = bisect(range, |num_days| {
        let spec = ToolchainSpec::Nightly(NaiveDate::from_num_days_from_ce(num_days));
        let toolchain = cfg.get_toolchain(&spec.to_string(), false).expect("get_toolchain");

        if toolchain.install_from_dist_if_not_installed().is_err() {
            return None;
        }

        let res = cmd.succeeds_with(&toolchain).expect("run command");

        println!("command {} at {}", if res { "succeeded" } else { "failed" }, spec);
        Some(!res)
    });

    if let Some(num_days) = res {
        println!("first failing nightly: {}",
                 ToolchainSpec::Nightly(NaiveDate::from_num_days_from_ce(num_days)));
        Ok(libc::EXIT_SUCCESS)
    } else {
        println!("bisect failed");
        Ok(libc::EXIT_FAILURE)
    }
}

fn main() {
    process::exit(run_rust_bisect().expect("something went wrong"));
}
