extern crate chrono;
extern crate hyper;
extern crate libc;
extern crate multirust;
extern crate rust_install;

extern crate rust_bisect;

use std::process;

use chrono::NaiveDate;
use hyper::client::Client;
use rust_install::dist::ToolchainDesc;

use rust_bisect::cli;
use rust_bisect::{Result, least_satisfying, nightly};

fn list_available_nightlies(dist_root: &str,
                            from: NaiveDate,
                            to: NaiveDate)
                            -> Result<Vec<NaiveDate>> {
    assert!(from < to, "`from` must be less than `to`");
    println!("finding available nightlies between {} and {}", from, to);
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
    println!("found {} nightlies", nightlies.len());
    Ok(nightlies)
}

fn run_rust_bisect() -> Result<i32> {
    let matches = cli::app().get_matches();

    let cfg = try!(multirust::Cfg::from_env(rust_install::notify::SharedNotifyHandler::none()));

    let good = matches.value_of("good").expect("clap didn't respect required arg `good`");
    let good = ToolchainDesc::from_str(good).expect("clap validator misbehaved for `good`");

    let bad = matches.value_of("bad").expect("clap didn't respect required arg `bad`");
    let bad = ToolchainDesc::from_str(bad).expect("clap validator misbehaved for `bad`");

    let good_date: NaiveDate = try!(good.date
                                        .expect("clap validator misbehaved for `good`")
                                        .parse());
    let bad_date: NaiveDate = try!(bad.date
                                      .expect("clap validator misbehaved for `bad`")
                                      .parse());

    let cmd = matches.value_of_os("COMMAND")
                     .expect("clap didn't respect required arg `COMMAND`");
    let args: Vec<_> = matches.values_of_os("ARGS")
                              .map(|args| args.collect())
                              .unwrap_or(Vec::new());

    let nightlies = try!(list_available_nightlies(&*cfg.dist_root_url, good_date, bad_date));
    println!("bisecting across {} nightlies (about {} steps)",
             nightlies.len(),
             nightlies.len().next_power_of_two().trailing_zeros());

    let idx = least_satisfying(&nightlies[..], |date| {
        let version = nightly(*date);
        println!("testing with {}", version);

        let toolchain = cfg.get_toolchain(&version, false).expect("could not get toolchain");
        toolchain.install_from_dist_if_not_installed().expect("could not install toolchain");

        let mut cmd = toolchain.create_command(cmd).expect("could not create command");
        cmd.args(&args);
        let res = cmd.status().expect("could not run command").success();

        println!("command {} with {}",
                 if res {
                     "succeeded"
                 } else {
                     "failed"
                 },
                 version);
        !res
    });

    println!("{} is the first failing nightly", nightly(nightlies[idx]));
    Ok(libc::EXIT_SUCCESS)
}

fn main() {
    process::exit(run_rust_bisect().expect("something went wrong"));
}
