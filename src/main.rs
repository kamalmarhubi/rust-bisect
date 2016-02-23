extern crate chrono;
extern crate clap;
extern crate hyper;
extern crate libc;
extern crate multirust;
extern crate rust_install;

extern crate rustc_bisect;

use std::process;

use chrono::NaiveDate;
use clap::{App, AppSettings, Arg};
use hyper::client::Client;
use rust_install::dist::ToolchainDesc;

use rustc_bisect::{Result, least_satisfying};

const NIGHTLY: &'static str = "nightly";

fn nightly(date: NaiveDate) -> String {
    format!("{}-{}", NIGHTLY, date)
}

fn list_available_nightlies(dist_root: &str,
                            from: NaiveDate,
                            to: NaiveDate)
                            -> Result<Vec<NaiveDate>> {
    assert!(from < to, "`from` must be less than `to`");
    let client = Client::new();
    let mut nightlies = Vec::with_capacity((to - from).num_days() as usize);
    let mut date = from;
    while date < to {
        let desc = ToolchainDesc::from_str(&nightly(date)).expect("should always parse");
        let resp = try!(client.head(&desc.manifest_url(dist_root)).send());
        // TODO: ensure failures are 404
        if resp.status.is_success() {
            nightlies.push(date);
        }
        date = date.succ();
    }
    Ok(nightlies)
}

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

    let nightlies = try!(list_available_nightlies(&*cfg.dist_root_url, good_date, bad_date));

    let idx = least_satisfying(&nightlies[..], |date| {
        let version = nightly(*date);
        let toolchain = cfg.get_toolchain(&version, false).expect("could not get toolchain");

        toolchain.install_from_dist_if_not_installed().expect("could not install toolchain");

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
        !res
    });

    println!("first failing nightly: {}", nightly(nightlies[idx]));
    Ok(libc::EXIT_SUCCESS)
}

fn main() {
    process::exit(run_rust_bisect().expect("something went wrong"));
}
