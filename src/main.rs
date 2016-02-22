extern crate chrono;
extern crate clap;
extern crate libc;
extern crate multirust;
extern crate rust_install;

extern crate rustc_bisect;

use std::process;

use chrono::{Datelike, NaiveDate};
use clap::{App, AppSettings, Arg};
use rust_install::dist::ToolchainDesc;

use rustc_bisect::{Result, bisect};

const NIGHTLY: &'static str = "nightly";

fn run_rust_bisect() -> Result<i32> {
    fn validate_version(s: String) -> std::result::Result<(), String> {
        let ret = ToolchainDesc::from_str(&s);
        match ret {
            Some(ref desc) if desc.channel == NIGHTLY && desc.date.is_some() => Ok(()),
            Some(_) => Err(String::from("can only bisect on dated nightlies")),
            None => Err(String::from(format!("invalid version: {}", s))),
        }
    }
    let matches = App::new("rustc-bisect")
                      .author("Kamal Marhubi <kamal@marhubi.com>")
                      .about("Find the Rust nightly that introduced a bug")
                      .setting(AppSettings::TrailingVarArg)
                      .usage("rustc-bisect [FLAGS] --bad <VERSION> --good <VERSION> <COMMAND> \
                              [ARGS...]")
                      .arg(Arg::with_name("good")
                               .long("good")
                               .takes_value(true)
                               .value_name("VERSION")
                               .help("A known good nightly release")
                               .validator(validate_version)
                               .required(true))
                      .arg(Arg::with_name("bad")
                               .long("bad")
                               .takes_value(true)
                               .value_name("VERSION")
                               .help("A known bad nightly release")
                               .validator(validate_version)
                               .required(true))
                      .arg(Arg::with_name("COMMAND")
                               .index(1)
                               .help("The command to run")
                               .required(true))
                      .arg(Arg::with_name("ARGS")
                               .index(2)
                               .multiple(true)
                               .help("Arguments for COMMAND"))
                      .get_matches();

    let cfg = try!(multirust::Cfg::from_env(rust_install::notify::SharedNotifyHandler::none()));

    let good = matches.value_of("good").expect("clap didn't respect required arg `good`");
    let good = ToolchainDesc::from_str(good).expect("clap validator misbehaved for `good`");

    let bad = matches.value_of("bad").expect("clap didn't respect required arg `bad`");
    let bad = ToolchainDesc::from_str(bad).expect("clap validator misbehaved for `bad`");

    let good_date: NaiveDate = try!(good.date
                                        .expect("clap validator misbheaved for `good`")
                                        .parse());
    let bad_date: NaiveDate = try!(bad.date
                                      .expect("clap validator misbheaved for `bad`")
                                      .parse());

    let cmd = matches.value_of_os("COMMAND")
                     .expect("clap didn't respect required arg `COMMAND`");
    let args: Vec<_> = matches.values_of_os("ARGS")
                              .map(|args| args.collect())
                              .unwrap_or(Vec::new());

    let range = good_date.num_days_from_ce()..bad_date.num_days_from_ce();

    fn version_string(num_days: i32) -> String {
        format!("{}-{}", NIGHTLY, NaiveDate::from_num_days_from_ce(num_days))
    }

    let res = bisect(range, |num_days| {
        let version = version_string(num_days);

        let toolchain = cfg.get_toolchain(&version, false).expect("get_toolchain");

        if toolchain.install_from_dist_if_not_installed().is_err() {
            // Assuming this is because the nightly wasn't found.
            // TODO: check the error, and have better reporting.
            return None;
        }

        let mut cmd = toolchain.create_command(cmd).expect("could not create command");
        cmd.args(&args);
        let res = cmd.status().expect("could not run command").success();

        println!("command {} at {}",
                 if res {
                     "succeeded"
                 } else {
                     "failed"
                 },
                 version);
        Some(!res)
    });

    if let Some(num_days) = res {
        println!("first failing nightly: {}", version_string(num_days));
        Ok(libc::EXIT_SUCCESS)
    } else {
        println!("bisect failed");
        Ok(libc::EXIT_FAILURE)
    }
}

fn main() {
    process::exit(run_rust_bisect().expect("something went wrong"));
}
